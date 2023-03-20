import numpy as np
import numpy_indexed as npi
import os
import time
import multiprocessing

from common import game_data, game_player_data, aggregation_data, get_result_metrics, Termination, Result
from num_games_dict import num_games_dict

# base_dir = "./resources"
base_dir = "/home/max/storage/chess"

def insert_group_sum(data, group, result, key):
    (elo, time_control), values = group.sum(data[key])
    np.add.at(result[key], (elo, time_control), values)

def parse_chunk(data):
    n = data.size
    all_player_data = np.empty(2*n, dtype=game_player_data)
    all_player_data[:n] = data["white_player_data"]
    all_player_data[n:] = data["black_player_data"]


    time_controls_np = np.empty(2*n, dtype=np.uint8)
    time_controls_np[:n] = data["time_control"]
    time_controls_np[n:] = data["time_control"]
    group = npi.group_by((all_player_data["elo"], time_controls_np))
    ret = np.zeros((4000, 20), dtype=aggregation_data)

    for metric in get_result_metrics():
        insert_group_sum(all_player_data, group, ret, metric)

    (elo, time_control), time_control_values = group.unique, group.count
    np.add.at(ret["count"], (elo, time_control), time_control_values)

    half_moves_np = np.empty(2*n, dtype=np.uint16)
    half_moves_np[:n] = data["half_moves"]
    half_moves_np[n:] = data["half_moves"]
    (elo, time_control), half_move_values = group.sum(half_moves_np)
    np.add.at(ret["half_moves"], (elo, time_control), half_move_values)

    terminations_np = np.empty(2*n, dtype=np.uint8)
    terminations_np[:n] = data["termination"]
    terminations_np[n:] = data["termination"]
    for termination in Termination:
        (elo, time_control), termination_values = group.sum(terminations_np == termination.value)
        np.add.at(ret["terminations"][termination.name], (elo, time_control), termination_values)

    results_np = np.empty(2*n, dtype=np.uint8)
    results_np[:n] = data["result"]
    results_np[n:] = data["result"]
    for result in Result:
        (elo, time_control), result_values = group.sum(results_np == result.value)
        np.add.at(ret["results"][result.name], (elo, time_control), result_values)

    return ret

def get_period_from_filename(filename):
    part = filename.split("_")[-1].replace(".bin", "")
    return tuple(map(int, part.split("-")))

def parse_file(filename):
    t0 = time.perf_counter()
    result_filename = filename.replace(".bin", ".result")
    if os.path.isfile(result_filename):
        return 0
    chunk_size = 1000000
    size = os.path.getsize(filename)
    total = np.zeros((4000, 20), dtype=aggregation_data)
    offset = 0
    while offset < size:
        batch_size = min(chunk_size, (size - offset) // game_data.itemsize)
        data = np.fromfile(filename, offset=offset, count=batch_size, dtype=game_data)
        result = parse_chunk(data)
        for metric in get_result_metrics():
            total[metric] += result[metric]

        total["half_moves"] += result["half_moves"]
        for termination in Termination:
            total["terminations"][termination.name] += result["terminations"][termination.name]
        for res in Result:
            total["results"][res.name] += result["results"][res.name]

        total["count"] += result["count"]
        offset += game_data.itemsize * batch_size
        if batch_size == 0:
            print(f"File {filename} is broken")
            break

    elos = np.tile(np.arange(4000).reshape(4000, 1), (1, 20))
    time_controls = np.tile(np.arange(20), (4000, 1))
    total["elo"] = elos
    total["time_control"] = time_controls
    total.tofile(result_filename)

    t1 = time.perf_counter()
    elapsed = t1-t0
    n = size // game_data.itemsize
    year, month = get_period_from_filename(filename)
    p = n / num_games_dict[(year, month)]
    print(f"{filename}: {n:.2e} games ({100*p:.2f}%) in {elapsed:.2f} seconds ({n/elapsed:.2e}/s)")
    return n

def parse_bin_files():
    filenames = sorted(filter(lambda s: s.endswith(".bin"), os.listdir(base_dir)), reverse=True)
    full_filenames = list(map(lambda s: f"{base_dir}/{s}", filenames))
    t0 = time.perf_counter()

#     process single-threaded
#     for filename in filenames:
#         parse_file(f"{base_dir}/{filename}")
    with multiprocessing.Pool(6) as pool:
        num_games = pool.map(parse_file, full_filenames, chunksize=1)

    t1 = time.perf_counter()
    elapsed = t1-t0
    n = sum(num_games)
    print(f"Total: {n} games parsed in {elapsed:.2f} seconds ({n/elapsed:.2e}/s)")

def check_missing_files():
    filenames = sorted(filter(lambda s: s.endswith(".pgn.zst"), os.listdir(base_dir)))
    full_filenames = list(map(lambda s: f"{base_dir}/{s}", filenames))
    print("Missing files:")
    for filename in full_filenames:
        bin_filename = filename.replace(".pgn.zst", ".bin")
        result_filename = filename.replace(".pgn.zst", ".result")
        if not os.path.isfile(bin_filename):
            print(f"Missing file: {bin_filename}")
        if not os.path.isfile(result_filename):
            print(f"Missing file: {result_filename}")

if __name__ == "__main__":
    parse_bin_files()
    check_missing_files()
