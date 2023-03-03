import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict
import numpy_indexed as npi
from enum import Enum

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
        return self.name.lower().replace("_", " ")

game_player_data = np.dtype([
    ('elo', np.int16),
    ('missed_mates', np.int16),
    ('missed_wins', np.int16),
    ('en_passant_mates', np.uint8),
    ('missed_en_passant_mates', np.uint8),
    ('en_passants', np.uint8),
    ('declined_en_passants', np.uint8),
])

game_player_data = np.dtype([
    ('white_player_data', game_player_data),
    ('black_player_data', game_player_data),
    ('time_control', np.uint8),
    ('result', np.uint8),
    ('termination', np.uint8),
    ('padding', np.uint8)
])

def get_en_passant_rate(arr, time_controls, time_control):
    arr = arr[time_controls == time_control.value]
    en_passant_opportunities = arr["en_passants"] + arr["declined_en_passants"]
    arr = arr[en_passant_opportunities > 0]
    en_passant_opportunities = en_passant_opportunities[en_passant_opportunities > 0]
    group = npi.group_by(arr["elo"])
    key, accepted = group.sum(arr["en_passants"])
    key, declined = group.sum(arr["declined_en_passants"])
    value = accepted / (accepted + declined)
    counts = group.count.astype(np.float64)
    if counts.size > 0:
        counts *= (10 / counts.max())
    return key, value, counts, arr.size, f"en passant acceptance rate for {time_control.format()}"

def get_missed_mates(arr, time_controls, time_control):
    arr = arr[time_controls == time_control.value]
    group = npi.group_by(arr["elo"])
#     print(group)
    key, value = group.mean(arr["missed_mates"])
#     print(key, value)
    counts = group.count.astype(np.float64)
    if counts.size > 0:
        counts *= (10 / counts.max())
    return key, value, counts, arr.size, f"Missed mates in one per game for {time_control.format()}"

def get_time_control_stats(arr):
    group = npi.group_by(arr["time_control"])
    key = group.unique
    value = group.count
    for k, v in zip(key, value):
        print(TimeControl(k), v)

arr = np.fromfile("resources/lichess_db_standard_rated_2023-01.bin", dtype=game_player_data)
# arr["white_player_data"]["elo"] //= 10
# arr["white_player_data"]["elo"] *= 10

all_player_data = np.array([arr["white_player_data"], arr["black_player_data"]])
all_player_data["elo"] //= 10
all_player_data["elo"] *= 10
time_controls_np = np.array([arr["time_control"], arr["time_control"]])

time_controls = [
#     TimeControl.CORRESPONDENCE_GAME,
#     TimeControl.CLASSICAL_GAME,
#     TimeControl.STANDARD_GAME,
    TimeControl.RAPID_GAME,
    TimeControl.BLITZ_GAME,
    TimeControl.BULLET_GAME,
    TimeControl.ULTRABULLET_GAME
]

checks = [
    get_missed_mates,
    get_en_passant_rate
]
get_time_control_stats(arr)

fig, axes = plt.subplots(len(checks), len(time_controls))
fig.suptitle(f"Chess stats for January 2023 ({len(arr)} games)")
for check, ax_row in zip(checks, axes):
    for time_control, ax in zip(time_controls, ax_row):
        x, y, s, num_games, title = check(all_player_data, time_controls_np, time_control)
        if x.size == 0:
            continue
        ax.scatter(x, y, s=s)

        ax.set_title(title, wrap=True)
        ax.set_ylabel(f"Average {check.__name__}")
        ax.set_xlim([500, 3000])
        ax.set_ylim([0, 1])
        ax.set_xlabel("Player ELO")

plt.show()
