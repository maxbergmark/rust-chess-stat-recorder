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

enriched_game_player_data = np.dtype(
    game_player_data.descr + [
    ('half_moves', np.uint16),
    ('result', np.uint8),
    ('termination', np.uint8),
    ('time_control', np.uint8),
])

game_data = np.dtype([
    ('white_player_data', game_player_data),
    ('black_player_data', game_player_data),
    ('start_time', np.uint32),
    ('game_link', "S8"),
    ('time_control', np.uint8),
    ('result', np.uint8),
    ('termination', np.uint8),
    ('padding', np.uint8),
    ('half_moves', np.uint16),
    ('padding2', np.uint16)
])

termination_data = np.dtype([
    (termination.name, np.int32) for termination in Termination
])

result_data = np.dtype([
    (result.name, np.int32) for result in Result
])

aggregation_data = np.dtype([
    ('elo', np.int32),
    ('time_control', np.int32),

    ('missed_mates_avg', np.float32),
    ('missed_mates_var', np.float32),

    ('missed_wins_avg', np.float32),
    ('missed_wins_var', np.float32),

    ('en_passants_avg', np.float32),
    ('en_passants_var', np.float32),

    ('declined_en_passants_avg', np.float32),
    ('declined_en_passants_var', np.float32),

    ('half_moves_avg', np.float32),
    ('half_moves_var', np.float32),

    ('en_passant_mates', np.int32),
    ('missed_en_passant_mates', np.int32),
    ('terminations', termination_data),
    ('results', result_data),
    ('count', np.int32) # important for aggregation
])

def get_combined_mean_and_variance_old(total, result, metric):
    mean_t = total[f"{metric}_avg"]
    var_t = total[f"{metric}_var"]
    n_t = total["count"]

    mean_r = result[f"{metric}_avg"]
    var_r = result[f"{metric}_var"]
    n_r = result["count"]

    # for empty groups we want to avoid dividing by zero
    n_c = np.maximum(1, n_t + n_r)
    mean_c = (n_t * mean_t + n_r * mean_r) / n_c
    # using the standard formula for combining variances of normal distributions
    var_c = (n_t * var_t + n_r * mean_r + n_t * (mean_t - mean_c)**2 + n_r * (mean_r - mean_c)**2) / n_c
    return mean_c, var_c

def get_combined_mean_and_variance(set_a, set_b, metric):
   # Using the Parallel algorithm found here:
   # https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
    mean_a = set_a[f"{metric}_avg"]
    n_a = set_a["count"]
    var_a = set_a[f"{metric}_var"]
    M2_a = var_a * (n_a - 1)

    mean_b = set_b[f"{metric}_avg"]
    n_b = set_b["count"]
    var_b = set_b[f"{metric}_var"]
    M2_b = var_b * (n_b - 1)

    n_ab = np.maximum(1, n_a + n_b)
    mean_ab = (n_a * mean_a + n_b * mean_b) / n_ab
    delta = mean_b - mean_a
    M2_ab = M2_a + M2_b + delta**2 * n_a * n_b / n_ab
    var_ab = M2_ab / np.maximum(1, n_ab - 1)

    return mean_ab, var_ab

def timed(f):
    def timed_f(*args, **kw):
        t0 = time.perf_counter()
        f(*args, **kw)
        t1 = time.perf_counter()
        print(f"{f.__name__}: {t1-t0:.3f}")
    return timed_f

def get_averaged_metrics():
    return [
        "missed_mates",
        "missed_wins",
        "en_passants",
        "declined_en_passants",
        "half_moves",
    ]

def get_counted_metrics():
    return [
        "en_passant_mates",
        "missed_en_passant_mates",
    ]