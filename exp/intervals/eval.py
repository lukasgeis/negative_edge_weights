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

sns.set_theme(style="darkgrid")
plt.rcParams["text.usetex"] = True
plt.rcParams["figure.figsize"] = 10, 10

fig, ax = plt.subplots(4, 1)


def correct_labels(plot):
    plot.set(xlabel=None)
    plot.set(ylabel=None)
    plot.get_legend().set_title(r"\textsc{Generator}")
    plot.get_legend().get_texts()[0].set_text(r"\textsc{Gnp}")
    plot.get_legend().get_texts()[1].set_text(r"\textsc{Rhg}")
    plot.get_legend().get_texts()[2].set_text(r"\textsc{Dsf}")


# Average Weight
plot = sns.lineplot(
    ax=ax[0],
    data=data,
    x="round",
    y="avg",
    hue="gen",
)
correct_labels(plot)

# Fraction Negative Edges
plot = sns.lineplot(
    ax=ax[1],
    data=data,
    x="round",
    y="frac",
    hue="gen",
)
correct_labels(plot)

# Runtime Dikjstra
plot = sns.lineplot(
    ax=ax[2],
    data=data[data.algo == "onedir"],
    x="round",
    y="time",
    hue="gen",
)
correct_labels(plot)

# Runtime BiDikjstra
plot = sns.lineplot(
    ax=ax[3],
    data=data[data.algo == "twodir"],
    x="round",
    y="time",
    hue="gen",
)
correct_labels(plot)


ax[3].set_xlabel(r"\textsc{Number of} $10000$ \textsc{Rounds}")

ax[0].set_ylabel(
    r"\textsc{Average}"
    "\n"
    r"\textsc{Weight}"
)
ax[1].set_ylabel(
    r"\textsc{Fraction of}"
    "\n"
    r"\textsc{Negative Edges}"
)
ax[2].set_ylabel(
    r"\textsc{Dikjstra}"
    "\n"
    r"\textsc{Time per}"
    "\n"
    r"$10000$ \textsc{Rounds}"
    "\n"
    r"\textsc{\small in} $ms$"
)
ax[3].set_ylabel(
    r"\textsc{BiDikjstra}"
    "\n"
    r"\textsc{Time per}"
    "\n"
    r"$10000$ \textsc{Rounds}"
    "\n"
    r"\textsc{\small in} $ms$"
)


fig.suptitle(
    "Average Weight / Fraction of Negative Edges / Runtime" "\n"
    r"over time for $n = 1e5$ and $m \approx 1e6$"
)

plt.savefig("exp/intervals/results.pdf", format="pdf", bbox_inches="tight")
plt.show()
