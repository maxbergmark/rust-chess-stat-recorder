import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict
import numpy_indexed as npi
from enum import Enum
import time
import os

class Result(Enum):
    WHITE_WIN = 1
    BLACK_WIN = 2
    DRAW = 3
    UNFINISHED = 4

class Termination(Enum):
    NORMAL = 1
    TIME_FORFEIT = 2
    ABANDONED = 3
    UNTERMINATED = 4
    RULES_INFRACTION = 5

class TimeControl(Enum):
    CORRESPONDENCE_GAME = 1
    CLASSICAL_GAME = 2
    STANDARD_GAME = 3
    RAPID_GAME = 4
    BLITZ_GAME = 5
    BULLET_GAME = 6
    ULTRABULLET_GAME = 7

    CORRESPONDENCE_TOURNAMENT = 10
    CLASSICAL_TOURNAMENT = 11
    STANDARD_TOURNAMENT = 12
    RAPID_TOURNAMENT = 13
    BLITZ_TOURNAMENT = 14
    BULLET_TOURNAMENT = 15
    ULTRABULLET_TOURNAMENT = 16

    def format(self):
        return self.name.lower().split("_")[0]

game_player_data = np.dtype([
    ('name', "S20"),
    ('elo', np.int16),
    ('missed_mates', np.int16),
    ('missed_wins', np.int16),
    ('en_passant_mates', np.uint8),
    ('missed_en_passant_mates', np.uint8),
    ('en_passants', np.uint8),
    ('declined_en_passants', np.uint8),
])

game_data = np.dtype([
    ('white_player_data', game_player_data),
    ('black_player_data', game_player_data),
    ('start_time', np.uint32),
    ('game_link', "S8"),
    ('time_control', np.uint8),
    ('result', np.uint8),
    ('termination', np.uint8),
    ('padding', np.uint8)
])

def timed(f):
    def timed_f(*args, **kw):
        t0 = time.perf_counter()
        f(*args, **kw)
        t1 = time.perf_counter()
        print(f"{f.__name__}: {t1-t0:.3f}")
    return timed_f

class AggregatedSet:

    def __init__(self, x, y, counts, num_games, title):
        self.x = x
        self.y = y
        self.counts = counts
        self.num_games = num_games
        self.title = title

    @property
    def sizes(self):
#         print(self.counts)
        return 10 * self.counts / self.counts.max()

    def __add__(self, other):
        self_dict = defaultdict(int, {x: y for x, y in zip(self.x, self.y)})
        other_dict = defaultdict(int, {x: y for x, y in zip(other.x, other.y)})
        self_counts = defaultdict(float, {x: c for x, c in zip(self.x, self.counts)})
        other_counts = defaultdict(float, {x: c for x, c in zip(other.x, other.counts)})

        keys = sorted(set(self.x) | set(other.x))
#         print(keys)
        values = [(self_dict[k] * self_counts[k] + other_dict[k] * other_counts[k]) / (self_counts[k] + other_counts[k]) for k in keys]
        counts = np.array([self_counts[k] + other_counts[k] for k in keys])
#         print(len(keys), len(values))
        assert len(keys) == len(values)
        return AggregatedSet(keys, values, counts, self.num_games + other.num_games, self.title)



class Visualizer:

#     @timed
    def __init__(self, filename):
        self.filename = filename
        self.data_sets = {}
        self.num_games = 0
        if filename:
            arr = np.fromfile(filename, dtype=game_data)
            arr = arr[arr["termination"] != Termination.UNTERMINATED.value]
            self.arr = arr
            self.num_games = len(self.arr)
#             self.get_time_control_stats()
            self.parse()

    def __add__(self, other):
        ret = Visualizer(None)
        for key in set(self.data_sets.keys()) | set(other.data_sets.keys()):
            if key in self.data_sets:
                ret.data_sets[key] = self.data_sets[key]
            if key in other.data_sets:
                if key in ret.data_sets:
                    ret.data_sets[key] += other.data_sets[key]
                else:
                    ret.data_sets[key] = other.data_sets[key]
        ret.num_games = self.num_games + other.num_games
        return ret

    @property
    def time_controls(self):
        return [
               TimeControl.CORRESPONDENCE_GAME,
               TimeControl.CLASSICAL_GAME,
               TimeControl.STANDARD_GAME,
#                TimeControl.RAPID_GAME,
               TimeControl.BLITZ_GAME,
               TimeControl.BULLET_GAME,
               TimeControl.ULTRABULLET_GAME
           ]

    @property
    def checks(self):
        return [
            self.get_missed_wins,
            self.get_en_passant_rate
        ]

    def get_en_passant_rate(self, time_control):
        arr = self.all_player_data[self.time_controls_np == time_control.value]
        en_passant_opportunities = arr["en_passants"] + arr["declined_en_passants"]
        arr = arr[en_passant_opportunities > 0]
        en_passant_opportunities = en_passant_opportunities[en_passant_opportunities > 0]
        group = npi.group_by(arr["elo"])
        key, accepted = group.sum(arr["en_passants"])
        key, declined = group.sum(arr["declined_en_passants"])
        value = accepted / (accepted + declined)
        counts = group.count.astype(np.float64)
#         if counts.size > 0:
#             counts *= (10 / counts.max())
        return AggregatedSet(key, value, counts, arr.size, f"En passant rate, {time_control.format()}")

    def get_missed_wins(self, time_control):
        arr = self.all_player_data[self.time_controls_np == time_control.value]
        group = npi.group_by(arr["elo"])
        key, value = group.mean(arr["missed_wins"])
        counts = group.count.astype(np.float64)
#         if counts.size > 0:
#             counts *= (10 / counts.max())
        return AggregatedSet(key, value, counts, arr.size, f"Missed wins, {time_control.format()}")

    def get_time_control_stats(self):
        group = npi.group_by(self.arr["time_control"])
        key = group.unique
        value = group.count
        for k, v in zip(key, value):
            print(TimeControl(k), v)

    def find_outliers(self):
        arr = self.arr
        is_correspondence = (arr["time_control"] == TimeControl.CORRESPONDENCE_GAME.value) | (arr["time_control"] == TimeControl.CLASSICAL_GAME.value)
        is_normal_termination = arr["termination"] == Termination.NORMAL.value
        is_decisive_game = (arr["result"] == Result.WHITE_WIN.value) | (arr["result"] == Result.BLACK_WIN.value)
        is_high_elo = (arr["white_player_data"]["elo"] > 1800) | (arr["black_player_data"]["elo"] > 1800)
        is_low_elo = (arr["white_player_data"]["elo"] < 1200) | (arr["black_player_data"]["elo"] < 1200)
        has_missed_wins = (arr["white_player_data"]["missed_wins"] > 5) | (arr["black_player_data"]["missed_wins"] > 10)
        has_en_passant_mate = (arr["white_player_data"]["en_passant_mates"] > 0) | (arr["black_player_data"]["en_passant_mates"] > 0)
        has_missed_en_passant_mate = (arr["white_player_data"]["missed_en_passant_mates"] > 0) | (arr["black_player_data"]["missed_en_passant_mates"] > 0)
        check = is_correspondence & is_decisive_game & is_high_elo & is_normal_termination & has_missed_wins
#         check = has_missed_en_passant_mate & is_high_elo
        print(arr[check])

#     @timed
    def parse(self):
        n = self.arr.size
        print(n)
        self.all_player_data = np.empty(2*n, dtype=game_player_data)
        self.all_player_data[:n] = self.arr["white_player_data"]
        self.all_player_data[n:] = self.arr["black_player_data"]
        self.all_player_data["elo"] //= 10
        self.all_player_data["elo"] *= 10

        self.time_controls_np = np.empty(2*n, dtype=np.uint8)
        self.time_controls_np[:n] = self.arr["time_control"]
        self.time_controls_np[n:] = self.arr["time_control"]

        self.data_sets = {}
        for check in self.checks:
            for time_control in self.time_controls:
                data = check(time_control)
                self.data_sets[check.__name__, time_control] = data


    def plot(self):

        fig, axes = plt.subplots(len(self.checks), len(self.time_controls))
        fig.suptitle(f"Chess stats for January 2018 ({self.num_games} games)")
        limits = {
            self.get_missed_wins.__name__: [0, 0.3],
            self.get_en_passant_rate.__name__: [0, 1]
        }

        for check, ax_row in zip(self.checks, axes):
            for time_control, ax in zip(self.time_controls, ax_row):
                data = self.data_sets[check.__name__, time_control]
                x, y, s = data.x, data.y, data.sizes
                num_games, title = data.num_games, data.title

                if len(x) == 0:
                    continue
                ax.scatter(x, y, s=s)
                print(min(x), max(x))

                ax.set_title(title, wrap=True)
                ax.set_ylabel(f"Average {check.__name__}")
                ax.set_xlim([500, 3000])
                ax.set_ylim(limits[check.__name__])
                ax.set_xlabel("Player ELO")

        plt.show()

def parse_bin_files():
    sum_visualizer = Visualizer(None)
    for filename in sorted(filter(lambda s: s.endswith(".remote.bin"), os.listdir("resources"))):
        print(filename)
        visualizer = Visualizer(f"resources/{filename}")
#         visualizer.find_outliers()
        sum_visualizer += visualizer

    sum_visualizer.plot()
#         visualizer.plot()

def check_piece_moves(all_moves):
    for piece in "KQRNB":
        for file in "abcdefgh":
            for rank in "12345678":
                for checks in ("#", "+", ""):
                    for capture in ("x", ""):
                        move = f"{piece}{capture}{file}{rank}{checks}"
                        if move not in all_moves:
                            print(move, "has not been played")

def possible_pawn_moves(file, rank):
    yield f"{file}{rank}"
    if file < "h":
        yield f"{chr(ord(file)+1)}x{file}{rank}"
    if file > "a":
        yield f"{chr(ord(file)-1)}x{file}{rank}"

def check_pawn_moves(all_moves):
    moves = []
    for file in "abcdefgh":
        for rank in "12345678":
            for checks in ("#", "+", ""):
                for move in possible_pawn_moves(file, rank):
                    if rank in "18":
                        moves.append(f"{move}=Q{checks}")
                        moves.append(f"{move}=R{checks}")
                        moves.append(f"{move}=N{checks}")
                        moves.append(f"{move}=B{checks}")
                    else:
                        moves.append(f"{move}{checks}")

    for move in moves:
        if move not in all_moves:
            print(move, "has not been played")

def check_all_moves(all_moves):
    check_piece_moves(all_moves)
    check_pawn_moves(all_moves)

def parse_move_files():
    all_moves = defaultdict(int)
    for filename in sorted(filter(lambda s: s.endswith(".remote.moves"), os.listdir("resources"))):
        for line in open(f"resources/{filename}", "r"):
            key, value = line.strip().split(": ")
            all_moves[key] += int(value)

    print(f"Total moves: {sum(all_moves.values()):.2e}")

    print(len(all_moves))
    check_all_moves(all_moves)
#     for k, v in all_moves.items():
#         if v < 2:
#             print(k, v)




if __name__ == "__main__":
    parse_move_files()
#     parse_bin_files()