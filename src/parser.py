import numpy as np
import numpy_indexed as npi
import os
import time
import multiprocessing

from common import game_data, game_player_data, result_data, get_result_metrics

base_dir = "./resources"
# base_dir = "/home/max/storage/chess"

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
    result = np.zeros((4000, 20), dtype=result_data)

    for metric in get_result_metrics():
        insert_group_sum(all_player_data, group, result, metric)

    (elo, time_control), values = group.unique, group.count
    np.add.at(result["count"], (elo, time_control), values)

    return result

def parse_file(filename):
    t0 = time.perf_counter()
    result_filename = filename.replace(".bin", ".result")
#     if os.path.isfile(result_filename):
#         return
    chunk_size = 1000000
    size = os.path.getsize(filename)
    total = np.zeros((4000, 20), dtype=result_data)
    offset = 0
    while offset < size:
        batch_size = min(chunk_size, (size - offset) // game_data.itemsize)
        data = np.fromfile(filename, offset=offset, count=batch_size, dtype=game_data)
        result = parse_chunk(data)
        for metric in get_result_metrics():
            total[metric] += result[metric]
        total["count"] += result["count"]
        offset += game_data.itemsize * batch_size

    elos = np.tile(np.arange(4000).reshape(4000, 1), (1, 20))
    time_controls = np.tile(np.arange(20), (4000, 1))
    total["elo"] = elos
    total["time_control"] = time_controls
    total.tofile(result_filename)

    t1 = time.perf_counter()
    elapsed = t1-t0
    n = size // game_data.itemsize
    print(f"{filename}: {n:.2e} games in {elapsed:.2f} seconds ({n/elapsed:.2e}/s)")

def parse_bin_files():
    filenames = sorted(filter(lambda s: s.endswith(".remote.bin"), os.listdir(base_dir)))
    with multiprocessing.Pool(8) as pool:
        pool.map(parse_file, map(lambda s: f"{base_dir}/{s}", filenames))


if __name__ == "__main__":
    parse_bin_files()