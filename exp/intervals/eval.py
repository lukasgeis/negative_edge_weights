import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

gnp = pd.read_csv("data/intervals/gnp.out")
rhg = pd.read_csv("data/intervals/rhg.out")
dsf = pd.read_csv("data/intervals/dsf.out")

gnp["gen"] = "gnp"
rhg["gen"] = "rhg"
dsf["gen"] = "dsf"

data = pd.concat([gnp, rhg, dsf])

data_labels = [
    (
        "avg",
        r"\textsc{Average}" "\n" r"\textsc{Weight}"
    ),
    (
        "frac",
        r"\textsc{Fraction of}" "\n" r"\textsc{Negative Edges}"
    ),
    (
        "time",
        r"\textsc{Time per}" "\n" r"$10000$ \textsc{Rounds}" "\n" r"\textsc{\small in} $ns$"
    )
]

sns.set_theme(style="darkgrid")
plt.rcParams["text.usetex"] = True
plt.rcParams["figure.figsize"] = 10, 10

fig, ax = plt.subplots(3, 1)

for i in range(3):
    plot = sns.lineplot(
        ax=ax[i],
        data=data,
        x="round",
        y=data_labels[i][0],
        hue="gen",
    )

    plot.set(xlabel=None)
    plot.set(ylabel=None)
    plot.get_legend().set_title(r"\textsc{Generator}")
    plot.get_legend().get_texts()[0].set_text(r"\textsc{Gnp}")
    plot.get_legend().get_texts()[1].set_text(r"\textsc{Rhg}")
    plot.get_legend().get_texts()[2].set_text(r"\textsc{Dsf}")

    ax[i].set_ylabel(data_labels[i][1])

ax[2].set_xlabel(r"\textsc{Number of} $10000$ \textsc{Rounds}")

fig.suptitle(
    "Average Weight / Fraction of Negative Edges / Runtime" "\n"
    r"over time for $n = 1e5$ and $m \approx 1e6$"
)

plt.savefig("exp/intervals/results.pdf", format="pdf", bbox_inches="tight")
plt.show()
