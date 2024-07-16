import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

data_path = "../../data/acceptance/roads.out"

data = pd.read_csv(data_path)

initials = {
    "m": r"\textsc{Maximum}",
    "z": r"\textsc{Zero}",
    "u": r"\textsc{Uniform}"
}

data["initial"].replace(initials, inplace=True)


sns.set_theme(style="darkgrid")
sns.set_palette("colorblind")
sns.set(font_scale=1.3)
plt.rcParams["text.usetex"] = True

plt.clf()
plot = sns.lineplot(
    data=data,
    x="round",
    y="rate",
    hue="initial"
)

plot.set(xlabel=r"\textsc{MCMC Steps}")
plot.set(ylabel=r"\textsc{Acceptance Rate}")
plot.get_legend().set_title(r"\textsc{InitialWeights}")

plt.xscale("log")

plt.savefig(
    "res/roads.pdf",
    format="pdf",
    bbox_inches="tight"
)
