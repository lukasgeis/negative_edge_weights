import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import seaborn as sns
import pandas as pd
import argparse as cli
import sys

def setup_plt_sns():
    sns.set_theme(style="darkgrid")
    sns.set_palette("colorblind")
    plt.rcParams["text.usetex"] = True
    plt.rcParams["figure.figsize"] = 6.4, 3.5

    return sns.color_palette()

def shift_labels_left(legend, labels):
    for item, label in zip(legend.legend_handles, legend.texts):
        if label._text in labels:
            width = item.get_window_extent(
                legend.figure.canvas.get_renderer()
            ).width
            label.set_ha('left')
            label.set_position((-1.4 * width, 0))


def gen_handles_labels(texts, colors, linestyles):
    handles = []
    labels = texts

    for i in range(len(texts)):
        if i < len(linestyles) and linestyles[i] is not None:
            handles.append(
                Line2D([], [], color=colors[i], linestyle=linestyles[i], label=texts[i])
            )
        else:
            handles.append(
                Line2D([], [], color=colors[i], label=texts[i])
            )

    return handles, labels


def plt_savefig(path):
    plt.savefig(
        path,
        format="pdf",
        bbox_inches="tight"
    )


def repeat_observations(data, field, f=1000000):
    factor = data[field].max() / f
    data[field] = data[field] / factor
    data = data.astype({field: "int"})

    data = data.reindex(data.index.repeat(data[field]))
    data.reset_index(drop=True, inplace=True)

    return data
