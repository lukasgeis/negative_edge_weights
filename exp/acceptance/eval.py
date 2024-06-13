import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

gnp = pd.read_csv("data/acceptance/gnp.out")
rhg = pd.read_csv("data/acceptance/rhg.out")
dsf = pd.read_csv("data/acceptance/dsf.out")

sns.set_theme(style="darkgrid")
sns.color_palette("colorblind")
plt.rcParams["text.usetex"] = True

sns.lineplot(
    data=gnp,
    x="round",
    y="rate",
    hue="initial"
)

plt.savefig("exp/acceptance/gnp.pdf", format="pdf", bbox_inches="tight")
plt.clf()

sns.lineplot(
    data=rhg,
    x="round",
    y="rate",
    hue="initial"
)

plt.savefig("exp/acceptance/rhg.pdf", format="pdf", bbox_inches="tight")
plt.clf()

sns.lineplot(
    data=dsf,
    x="round",
    y="rate",
    hue="initial"
)

plt.savefig("exp/acceptance/dsf.pdf", format="pdf", bbox_inches="tight")
plt.clf()
