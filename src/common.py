import numpy as np
from enum import Enum

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

result_data = np.dtype([
    ('elo', np.int32),
    ('time_control', np.int32),
    ('missed_mates', np.int32),
    ('missed_wins', np.int32),
    ('en_passant_mates', np.int32),
    ('missed_en_passant_mates', np.int32),
    ('en_passants', np.int32),
    ('declined_en_passants', np.int32),
    ('count', np.int32)
])

def timed(f):
    def timed_f(*args, **kw):
        t0 = time.perf_counter()
        f(*args, **kw)
        t1 = time.perf_counter()
        print(f"{f.__name__}: {t1-t0:.3f}")
    return timed_f

def get_result_metrics():
    return [
        "missed_mates",
        "missed_wins",
        "en_passant_mates",
        "missed_en_passant_mates",
        "en_passants",
        "declined_en_passants"
    ]