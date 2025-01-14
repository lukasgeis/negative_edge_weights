import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import seaborn as sns
import sys

outdir = sys.argv[1]


sns.set_theme(style="darkgrid")
sns.set_palette("colorblind")
plt.rcParams["text.usetex"] = True

sns_colors = sns.color_palette()

def plot_legend_and_save(legend, path):
    plt.gca().set_axis_off()

    frame = legend.get_frame()
    frame.set_linewidth(3)
    frame.set_edgecolor('0.75')
    frame.set_facecolor('0.90')


    fig = legend.figure
    bbox  = legend.get_window_extent().transformed(fig.dpi_scale_trans.inverted())
    fig.savefig(f"{outdir}/{path}", format="pdf", dpi="figure", bbox_inches=bbox)


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
        if i < len(linestyles) and linestyles[i] != None:
            handles.append(
                Line2D([], [], color=colors[i], linestyle=linestyles[i], label=texts[i])
            )
        else:
            handles.append(
                Line2D([], [], color=colors[i], label=texts[i])
            )

    return handles, labels


# (Average Degree) x (Initial Weights)

texts = [
    r"\textsc{Average Degree}",
    r"$10$",
    r"$20$",
    r"$50$",
    r"\textsc{Initial Weights}",
    r"$w_{max}$",
    r"$w_{unif}$",
    r"$w_{zero}$"
]
colors = [
    "none",
    "black",
    "black",
    "black",
    "none",
    sns_colors[0],
    sns_colors[1],
    sns_colors[2]
]
linestyles = [
    None,
    "solid",
    "dashed",
    "dotted"
]

handles, labels = gen_handles_labels(texts, colors, linestyles)

legend = plt.legend(handles, labels, ncols=1, fontsize=13)
shift_labels_left(legend, [texts[0], texts[4]])
plot_legend_and_save(legend, "deg_init.pdf")


# Algorithms[2]

texts = [
    r"\textsc{SamplerPot}",
    r"\textsc{BiSamplerPot}"
]
colors = sns_colors[:2]

handles, labels = gen_handles_labels(texts, colors, [])

legend = plt.legend(handles, labels, title=r"\textsc{Algorithm}", ncol=3)
plot_legend_and_save(legend, "algorithms2.pdf")


# Algorithms[3]

texts = [
    r"\textsc{Sampler}",
    r"\textsc{SamplerPot}",
    r"\textsc{BiSamplerPot}"
]
colors = sns_colors[:3]

handles, labels = gen_handles_labels(texts, colors, [])

legend = plt.legend(handles, labels, title=r"\textsc{Algorithm}", ncol=3)
plot_legend_and_save(legend, "algorithms3.pdf")


# (Average Degree) x (Algorithms[3])

texts = [
    r"\textsc{Average Degree}",
    r"$10$",
    r"$20$",
    r"$50$",
    r"\textsc{Algorithm}",
    r"\textsc{Sampler}",
    r"\textsc{SamplerPot}",
    r"\textsc{BiSamplerPot}"
]
colors = [
    "none",
    "black",
    "black",
    "black",
    "none",
    sns_colors[0],
    sns_colors[1],
    sns_colors[2]
]
linestyles = [
    None,
    "solid",
    "dashed",
    "dotted"
]

handles, labels = gen_handles_labels(texts, colors, linestyles)

legend = plt.legend(handles, labels, ncols=1, fontsize=13)
shift_labels_left(legend, [texts[0], texts[4]])
plot_legend_and_save(legend, "deg_algo.pdf")


# Initial Weights

texts = [
    r"$w_{max}$",
    r"$w_{unif}$",
    r"$w_{zero}$"
]
colors = sns_colors[:3]

handles, labels = gen_handles_labels(texts, colors, [])

legend = plt.legend(handles, labels, title=r"\textsc{Initial Weights}", ncol=3)
plot_legend_and_save(legend, "initial.pdf")


# Average Degree

texts = [
    r"$10$",
    r"$20$",
    r"$50$",]
colors = [
    "black",
    "black",
    "black"
]
linestyles = [
    "solid",
    "dashed",
    "dotted"
]

handles, labels = gen_handles_labels(texts, colors, linestyles)

legend = plt.legend(handles, labels, title=r"\textsc{Average Degree}", ncols=3, fontsize=13)
plot_legend_and_save(legend, "degree.pdf")
