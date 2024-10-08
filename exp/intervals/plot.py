import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import seaborn as sns
import pandas as pd

import sys

if len(sys.argv) > 1:
    plot_dir = "res/test"
else:
    plot_dir = "res"

data_path = "../../data/intervals"

gnp = pd.read_csv(f"{data_path}/gnp.out")
rhg = pd.read_csv(f"{data_path}/rhg.out")
dsf = pd.read_csv(f"{data_path}/dsf.out")
roads = pd.read_csv(f"{data_path}/roads.out")

gnp["gen"] = r"$\mathcal{GNP}$"
rhg["gen"] = r"$\mathcal{RHG}$"
dsf["gen"] = r"$\mathcal{DSF}$"
roads["gen"] = r"$\mathcal{ROAD}$"

data = pd.concat([gnp, rhg, dsf, roads])
data["round"] = data["round"] * 10000
data["time"] = data["time"] / 10

data = data[
    data.groupby(
        ["round", "gen", "algo"]
    )[
        ["round", "gen", "algo"]
    ].transform('size') > 9
]

sns.set_theme(style="darkgrid")
sns.set_palette("colorblind")
sns.set(font_scale=1.3)
plt.rcParams["text.usetex"] = True
plt.rcParams["figure.figsize"] = 6.4, 3.7

# Plot Average Weight and Fraction Negative Edges
plt.clf()
plot1 = sns.lineplot(
    data=data,
    x="round",
    y="avg",
    hue="gen"
)
ax2 = plt.twinx()
plot2 = sns.lineplot(
    data=data,
    x="round",
    y="frac",
    hue="gen",
    linestyle="dashed",
    ax=ax2
)

plt.xscale("log")

plot1.set(xlabel=r"\textsc{Number of MCMC Steps}")
plot1.set(ylabel=r"\textsc{Average Weight}")
plot2.set(ylabel=r"\textsc{Fraction of}" "\n" r"\textsc{Negative Edges}")

handles, labels = plot1.get_legend_handles_labels()

type_labels = [
    r"\textsc{Weight}",
    r"\textsc{NegEdges}"
]

type_lines = [
    Line2D([], [], color="black", linestyle="solid", label=type_labels[0]),
    Line2D([], [], color="black", linestyle="dashed", label=type_labels[1])
]

title_labels = [
    r"\textsc{Metric}",
    r"\textsc{Generator}"
]

title_lines = [
    Line2D([], [], color="none", label=title_labels[0]),
    Line2D([], [], color="none", label=title_labels[1])
]

handles = [title_lines[0]] + type_lines + [title_lines[1]] + handles
labels = [title_labels[0]] + type_labels + [title_labels[1]] + labels

plot1.get_legend().remove()
plot2.get_legend().remove()
leg = plt.legend(handles, labels, fontsize=13)

for item, label in zip(leg.legend_handles, leg.texts):
    if label._text in title_labels:
        width = item.get_window_extent(
            leg.figure.canvas.get_renderer()
        ).width
        label.set_ha('left')
        label.set_position((-1.4 * width, 0))

plt.savefig(
    f"{plot_dir}/avg_frac.pdf",
    format="pdf",
    bbox_inches="tight"
)

dummy_label = ""
dummy_line = Line2D([], [], color="none", label=dummy_label)

# Plot Time
plt.clf()
sns.lineplot(
    data=data[data.algo == "d"],
    x="round",
    y="time",
    hue="gen"
)
plot = sns.lineplot(
    data=data[data.algo == "bd"],
    x="round",
    y="time",
    hue="gen",
    linestyle="dashed"
)

plot.set(xlabel=r"\textsc{Number of MCMC Steps}")
plot.set(ylabel=r"\textsc{Time per}" "\n" r"\textsc{MCMC Step in} $\mu s$")

plt.xscale("log")
plt.yscale("log")

handles, labels = plot.get_legend_handles_labels()

algo_labels = [
    r"\textsc{Dijkstra}",
    r"\textsc{BiDijkstra}"
]

algo_lines = [
    Line2D([], [], color="black", linestyle="solid", label=algo_labels[0]),
    Line2D([], [], color="black", linestyle="dashed", label=algo_labels[1])
]

title_labels = [
    r"\textsc{Algorithm}",
    r"\textsc{Generator}"
]

title_lines = [
    Line2D([], [], color="none", label=title_labels[0]),
    Line2D([], [], color="none", label=title_labels[1])
]

handles = [title_lines[0]] + algo_lines + [title_lines[1]] + handles[:4]
labels = [title_labels[0]] + algo_labels + [title_labels[1]] + labels[:4]

plot.get_legend().remove()
leg = plt.legend(handles, labels, fontsize=13)

for item, label in zip(leg.legend_handles, leg.texts):
    if label._text in title_labels:
        width = item.get_window_extent(
            leg.figure.canvas.get_renderer()
        ).width
        label.set_ha('left')
        label.set_position((-1.4 * width, 0))

plt.savefig(
    f"{plot_dir}/time.pdf",
    format="pdf",
    bbox_inches="tight"
)
