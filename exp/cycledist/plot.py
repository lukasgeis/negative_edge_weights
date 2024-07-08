import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import seaborn as sns
import pandas as pd

import sys

if len(sys.argv) > 1:
    plot_dir = "res/test"
else:
    plot_dir = "res"

data_path = "../../data/cycledist"

data = pd.read_csv(f"{data_path}/data.out")

def round_label(row):
    factor = int(row["round"])
    if factor == 0:
        label = r"$\frac{1}{2}n$"
    elif factor == 1:
        label = r"$n$"
    else:
        label = r"${{{fac}}}n$".format(fac=str(factor))
    return label

data["factor"] = data.apply(round_label, axis=1)

initial = {
    "m": r"\textsc{Maximum}",
    "z": r"\textsc{Zero}"
}

data["initial"].replace(initial, inplace=True)


sns.set_theme(style="darkgrid")
sns.set_palette("colorblind")
sns.set(font_scale=1.3)
plt.rcParams["text.usetex"] = True

plot = sns.violinplot(
    data=data,
    x="factor",
    y="weight",
    inner="quart",
    hue="initial",
    split=True,
)

plot.set(xlabel=r"\textsc{Number of Rounds}")
plot.set(ylabel=r"\textsc{EdgeWeights}")
plot.legend(loc='lower left',title=r"\textsc{InitialWeights}")

plt.savefig(
    f"{plot_dir}/cycledist.pdf",
    format="pdf",
    bbox_inches="tight"
)

