import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

import sys

if len(sys.argv) > 1:
    plot_dir = "res/test"
else:
    plot_dir = "res"

data_path = "../../data/insertions"

gnp = pd.read_csv(f"{data_path}/gnp.out")
rhg = pd.read_csv(f"{data_path}/rhg.out")
dsf = pd.read_csv(f"{data_path}/dsf.out")
roads = pd.read_csv(f"{data_path}/roads.out")

algorithms = {
    "bf": r"\textsc{BellmanFord}",
    "bd": r"\textsc{BiDijkstra}",
    "d": r"\textsc{Dijkstra}"
}

order = [algorithms["bf"], algorithms["d"], algorithms["bd"]]

acceptance = {
    "acc": r"\textsc{Accepted}",
    "rej": r"\textsc{Rejected}",
}


sns.set_theme(style="darkgrid")
sns.color_palette("colorblind")
plt.rcParams["text.usetex"] = True
plt.rcParams["figure.figsize"] = 6.4, 3


def prep_and_plot_data(data: pd.DataFrame, file_name: str):
    data["algo"].replace(algorithms, inplace=True)
    data["acc"].replace(acceptance, inplace=True)

    plt.clf()
    plot = sns.boxenplot(
        data,
        x="algo",
        y="insertions",
        hue="acc",
        order=order,
    )

    plot.set(xlabel=None)
    plot.set(ylabel=r"\textsc{Insertions}")

    plt.yscale("log")

    plot.get_legend().set_title("")

    plt.savefig(
        f"{plot_dir}/{file_name}.pdf",
        format="pdf",
        bbox_inches="tight"
    )


prep_and_plot_data(gnp, "gnp")
prep_and_plot_data(rhg, "rhg")
prep_and_plot_data(dsf, "dsf")
prep_and_plot_data(roads, "roads")
