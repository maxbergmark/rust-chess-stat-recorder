import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict
import numpy_indexed as npi
from datetime import datetime
from enum import Enum
import time
import os

from common import aggregation_data, get_result_metrics, TimeControl, Termination, Result

base_dir = "./resources"
# base_dir = "/home/max/storage/chess"

def get_time_controls():
    return [
           TimeControl.CORRESPONDENCE_GAME,
           TimeControl.CLASSICAL_GAME,
           TimeControl.STANDARD_GAME,
           TimeControl.RAPID_GAME,
           TimeControl.BLITZ_GAME,
           TimeControl.BULLET_GAME,
           TimeControl.ULTRABULLET_GAME
       ]

def get_average_missed_wins(data):
    return data["missed_wins"] / np.maximum(data["count"], 1)

def get_en_passant_rate(data):
    return data["en_passants"] / (data["en_passants"] + data["declined_en_passants"])

def get_en_passant_mate_rate(data):
    return data["en_passant_mates"] / (data["en_passant_mates"] + data["missed_en_passant_mates"])

def get_en_passants(data):
    return data["en_passants"] / np.maximum(data["count"], 1)

def declined_en_passants(data):
    return data["declined_en_passants"] / np.maximum(data["count"], 1)

def get_num_moves(data):
    return data["half_moves"] / np.maximum(data["count"], 1)

def get_termination_stats(data):
    ret = np.zeros((data.size, len(Termination)), dtype=np.float64)
    for i, termination in enumerate(Termination):
        ret[:,i] = data["terminations"][termination.name]
    return ret / np.maximum(ret.sum(axis=1), 1)[:,np.newaxis], Termination

def get_result_stats(data):
    ret = np.zeros((data.size, len(Result)), dtype=np.float64)
    for i, result in enumerate(Result):
        ret[:,i] = data["results"][result.name]
    return ret / np.maximum(ret.sum(axis=1), 1)[:,np.newaxis], Result

def get_checks():
    return [
        get_average_missed_wins,
#         get_en_passant_rate,
#         get_en_passants,
#         get_en_passant_mate_rate,
#         declined_en_passants,
        get_num_moves,
        get_termination_stats,
        get_result_stats,
    ]

def plot_average(result, ax, time_control, check):
    data = result[:,time_control.value]
    x = data["elo"]
    y = check(data)
    s = data["count"] / max(1, data["count"].max())

    if len(x) == 0:
        return
    ax.scatter(x, y, s=s)
    ax.text(0.98, 0.98, f"{data['count'].sum():.2e} players",
         horizontalalignment='right',
         verticalalignment='top',
         transform = ax.transAxes)

def plot_distribution(result, ax, time_control, check):
    data = result[:,time_control.value]
    x = data["elo"]
    ys, enum_values = check(data)
#     print(ys)

    offset = (0*x).astype(np.float64)
    ax.stackplot(x, *ys.T, labels=list(map(lambda e: e.name, enum_values)))
    ax.legend()
#     for y in ys.T:
#         print(time_control, y.shape)
#         ax.bar(x, y, bottom=offset)
#         offset += y

def plot(result):

    fig, axes = plt.subplots(len(get_checks()), len(get_time_controls()))
    fig.subplots_adjust(
        left  = 0.05,  # the left side of the subplots of the figure
        right = 0.95,    # the right side of the subplots of the figure
        bottom = 0.1,   # the bottom of the subplots of the figure
        top = 0.9,      # the top of the subplots of the figure
        wspace = 0.25,   # the amount of width reserved for blank space between subplots
        hspace = 0.25   # the amount of height reserved for white space between subplots
    )
    fig.suptitle(f"Chess stats for ({result['count'].sum():.2e} players)")
    limits = {
        get_average_missed_wins: [0, .3],
        get_en_passant_rate: [0, 1],
        get_en_passants: [0, .05],
        declined_en_passants: [0, .1],
        get_en_passant_mate_rate: [0, 1],
        get_num_moves: [0, 1],
        get_termination_stats: [0, 1],
        get_result_stats: [0, 1],
    }
    plot_types = {
        get_average_missed_wins: plot_average,
        get_en_passant_rate: plot_average,
        get_en_passants: plot_average,
        declined_en_passants: plot_average,
        get_en_passant_mate_rate: plot_average,
        get_num_moves: plot_average,
        get_termination_stats: plot_distribution,
        get_result_stats: plot_distribution,
    }

    for i, (check, ax_row) in enumerate(zip(get_checks(), axes)):
        ax_row[0].set_ylabel(f"{check.__name__}")
        for time_control, ax in zip(get_time_controls(), ax_row):

            plot_function = plot_types[check]
            plot_function(result, ax, time_control, check)


            title = f"{time_control.format()}"
            if i == 0:
                ax.set_title(title, wrap=True)
            ax.set_xlim([0, 4000])
            ax.set_ylim(limits[check])
            if i == len(axes)-1:
                ax.set_xlabel("Player ELO")

    plt.show()


def get_summed_result_files():
    total = np.zeros((4000, 20), dtype=aggregation_data)
    for filename in sorted(filter(lambda s: s.endswith(".result"), os.listdir(base_dir))):
        result = np.fromfile(f"{base_dir}/{filename}", dtype=aggregation_data)

        result.shape = (4000, 20)
        total["elo"] = result["elo"]
        total["time_control"] = result["time_control"]
        for metric in get_result_metrics():
            total[metric] += result[metric]
        for termination in Termination:
            total["terminations"][termination.name] += result["terminations"][termination.name]
        for res in Result:
            total["results"][res.name] += result["results"][res.name]

        total["count"] += result["count"]

    return total


if __name__ == "__main__":
    result = get_summed_result_files()
    plot(result)



