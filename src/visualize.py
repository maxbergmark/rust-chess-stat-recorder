import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict
import numpy_indexed as npi
from datetime import datetime
from enum import Enum
import time
import os

from common import (aggregation_data, get_counted_metrics, get_averaged_metrics, TimeControl,
                    Termination, Result, get_combined_mean_and_variance)

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

def get_termination_color(termination):
    return {
        Termination.NORMAL: "#000000",
        Termination.TIME_FORFEIT: "#222222",
        Termination.ABANDONED: "#444444",
        Termination.UNTERMINATED: "#666666",
        Termination.RULES_INFRACTION: "#888888",
    }[termination]

def get_result_color(result):
    return {
        Result.WHITE_WIN: "#ffffff",
        Result.BLACK_WIN: "#000000",
        Result.DRAW: "#888888",
        Result.UNFINISHED: "#ffaaaa",
    }[result]

def get_termination_order():
    return list(range(5))

def get_result_order():
    return [0, 2, 3, 1]

def get_enum_color(e):
    if type(e) == Termination:
        return get_termination_color(e)
    elif type(e) == Result:
        return get_result_color(e)

def get_enum_order(e):
    if e == Termination:
        return get_termination_order()
    elif e == Result:
        return get_result_order()

def get_average_missed_wins(data):
    return data["missed_wins_avg"], data["missed_wins_var"]

def get_en_passant_rate(data):
    return data["en_passants_avg"] / (data["en_passants_avg"] + data["declined_en_passants_avg"]), 0

def get_en_passant_mate_rate(data):
    # TODO: fix this
    return data["en_passant_mates"] / (data["en_passant_mates"] + data["missed_en_passant_mates"])

def get_en_passants(data):
    return data["en_passants_avg"], data["en_passants_var"]

def declined_en_passants(data):
    return data["declined_en_passants_avg"], data["declined_en_passants_var"]

def get_num_moves(data):
    # divide by two to get whole moves
    return data["half_moves_avg"] / 2, data["half_moves_var"] / 4

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
        get_en_passants,
#         get_en_passant_mate_rate,
        declined_en_passants,
        get_num_moves,
#         get_termination_stats,
#         get_result_stats,
    ]

def plot_average(result, ax, time_control, check):
    data = result[:,time_control.value]
    x = data["elo"]
    y, variance = check(data)
    std_dev = variance ** .5
    s = data["count"] / max(1, data["count"].max())

    if len(x) == 0:
        return
    ax.fill_between(x, y-std_dev, y+std_dev, alpha=1.0, color="#aaaaaa")
    ax.scatter(x, y, s=s, color="#000000")
    ax.text(0.98, 0.98, f"{data['count'].sum():.2e} players",
         horizontalalignment='right',
         verticalalignment='top',
         transform = ax.transAxes)

def plot_distribution(result, ax, time_control, check):
    data = result[:,time_control.value]
    x = data["elo"]
    ys, enum_values = check(data)
    order = get_enum_order(enum_values)
    ys_ordered = ys[:,order]
    enum_ordered = [list(enum_values)[o] for o in order]

    colors = list(map(get_enum_color, enum_ordered))
    ax.stackplot(x, *ys_ordered.T, labels=list(map(lambda e: e.name, enum_ordered)), colors=colors)
    ax.legend(loc="lower right")


def plot(result):

#     plt.plot(result[:,5]["elo"], result[:,5]["en_passants_avg"], "r")
#     plt.plot(result[:,5]["elo"], result[:,5]["en_passants_avg"] - result[:,5]["en_passants_var"], "b")
#     plt.show()
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
        get_average_missed_wins: [-1, 1.3],
        get_en_passant_rate: [-1, 2],
        get_en_passants: [-1, 1.05],
        declined_en_passants: [-1, 1.1],
        get_en_passant_mate_rate: [0, 1],
        get_num_moves: [0, 60],
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

        for metric in get_counted_metrics():
            total[metric] += result[metric]

        for metric in get_averaged_metrics():
            mean_c, var_c = get_combined_mean_and_variance(total, result, metric)
            total[f"{metric}_avg"] = mean_c
            total[f"{metric}_var"] = var_c

        for termination in Termination:
            total["terminations"][termination.name] += result["terminations"][termination.name]

        for res in Result:
            total["results"][res.name] += result["results"][res.name]

        total["count"] += result["count"]

    return total


if __name__ == "__main__":
    result = get_summed_result_files()
    plot(result)



