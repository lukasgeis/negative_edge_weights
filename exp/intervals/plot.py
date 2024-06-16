import matplotlib.pyplot as plt
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

gnp["gen"] = r"\textsc{Gnp}"
rhg["gen"] = r"\textsc{Rhg}"
dsf["gen"] = r"\textsc{Dsf}"

data = pd.concat([gnp, rhg, dsf])

sns.set_theme(style="darkgrid")
sns.color_palette("colorblind")
plt.rcParams["text.usetex"] = True


def plot_data(data: pd.DataFrame, y_col: str, y_label: str, file_name: str):
    plt.clf()
    plot = sns.lineplot(
        data=data,
        x="round",
        y=y_col,
        hue="gen",
    )

    plot.set(xlabel=r"\textsc{Number of} $10000$ \textsc{Rounds}")
    plot.set(ylabel=y_label)
    plot.get_legend().set_title(r"\textsc{Generator}")

    plt.savefig(
        f"{plot_dir}/{file_name}.pdf",
        format="pdf",
        bbox_inches="tight"
    )


plot_data(
    data,
    "avg",
    r"\textsc{Average Weight}",
    "avg_weight"
)

plot_data(
    data,
    "frac",
    r"\textsc{Fraction of}"
    "\n"
    r"\textsc{Negative Edges}",
    "frac_neg_edges"
)

plot_data(
    data[data.algo == "d"],
    "time",
    r"\textsc{Time per}"
    "\n"
    r"$10000$ \textsc{Rounds}"
    "\n"
    r"\textsc{\small in} $ms$",
    "dijkstra"
)

plot_data(
    data[data.algo == "bd"],
    "time",
    r"\textsc{Time per}"
    "\n"
    r"$10000$ \textsc{Rounds}"
    "\n"
    r"\textsc{\small in} $ms$",
    "bidijkstra"
)
