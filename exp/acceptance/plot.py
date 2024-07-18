import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import seaborn as sns
import pandas as pd

import sys

if len(sys.argv) > 1:
    plot_dir = "res/test"
else:
    plot_dir = "res"

data_path = "../../data/acceptance"

gnp = pd.read_csv(f"{data_path}/gnp.out")
rhg = pd.read_csv(f"{data_path}/rhg.out")
dsf = pd.read_csv(f"{data_path}/dsf.out")
roads = pd.read_csv(f"{data_path}/roads.out")

wrong_dsf_degrees = {
    6: 10,
    14: 20,
    47: 50,
}

dsf["degree"].replace(wrong_dsf_degrees, inplace=True)

initials = {
    "m": r"\textsc{Maximum}",
    "z": r"\textsc{Zero}",
    "u": r"\textsc{Uniform}"
}


gnp["initial"].replace(initials, inplace=True)
rhg["initial"].replace(initials, inplace=True)
dsf["initial"].replace(initials, inplace=True)
roads["initial"].replace(initials, inplace=True)

sns.set_theme(style="darkgrid")
sns.set_palette("colorblind")
sns.set(font_scale=1.3)
plt.rcParams["text.usetex"] = True
plt.rcParams["figure.figsize"] = 6.4, 3.5


def prep_and_plot_data(data: pd.DataFrame, file_name: str):
    data = data[
        data.groupby(
            ["round", "initial", "degree"]
        )[
            ["round", "initial", "degree"]
        ].transform('size') > 9
    ]

    plt.clf()
    plot = sns.lineplot(
        data=data[data.degree == 10],
        x="round",
        y="rate",
        hue="initial",
        linestyle="solid"
    )

    sns.lineplot(
        data=data[data.degree == 20],
        x="round",
        y="rate",
        hue="initial",
        linestyle="dashed",
        legend=False
    )

    sns.lineplot(
        data=data[data.degree == 50],
        x="round",
        y="rate",
        hue="initial",
        linestyle="dotted",
        legend=False
    )

    plot.set(xlabel=r"\textsc{MCMC Steps}")
    plot.set(ylabel=r"\textsc{Acceptance Rate}")

    plt.xscale("log")

    handles, labels = plot.get_legend_handles_labels()

    degree_lines = [
        Line2D([], [], color="black", linestyle="solid", label=r"$10$"),
        Line2D([], [], color="black", linestyle="dashed", label=r"$20$"),
        Line2D([], [], color="black", linestyle="dotted", label=r"$50$")
    ]

    degree_labels = [
        r"$10$",
        r"$20$",
        r"$50$"
    ]

    degree_title = r"\textsc{Average Degree}"
    weight_title = r"\textsc{Initial Weights}"

    title_lines = [
        Line2D([], [], color="none", label=degree_title),
        Line2D([], [], color="none", label=weight_title)
    ]

    handles = [title_lines[0]] + degree_lines + [title_lines[1]] + handles
    labels = [degree_title] + degree_labels + [weight_title] + labels

    leg = plt.legend(handles, labels, ncols=2, fontsize=13)

    for item, label in zip(leg.legend_handles, leg.texts):
        if label._text in [degree_title, weight_title]:
            width = item.get_window_extent(
                leg.figure.canvas.get_renderer()
            ).width
            label.set_ha('left')
            label.set_position((-1.4 * width, 0))

    plt.savefig(
        f"{plot_dir}/{file_name}.pdf",
        format="pdf",
        bbox_inches="tight"
    )


prep_and_plot_data(gnp, "gnp")
prep_and_plot_data(rhg, "rhg")
prep_and_plot_data(dsf, "dsf")

# Roads
plt.clf()
plot = sns.lineplot(
    data=roads,
    x="round",
    y="rate",
    hue="initial"
)

plot.set(xlabel=r"\textsc{MCMC Steps}")
plot.set(ylabel=r"\textsc{Acceptance Rate}")
plot.get_legend().set_title(r"\textsc{Initial Weights}")

plt.xscale("log")

plt.savefig(
    f"{plot_dir}/roads.pdf",
    format="pdf",
    bbox_inches="tight"
)
