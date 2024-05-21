import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

n_values = [100, 1000]
ab_values = [(1, 1), (2, 5), (3, 10)]

data = {
    "n": [],
    "limits": [],
    "hops": [],
}

for n in n_values:
    for (a, b) in ab_values:
        path = f"data/complete_hops/{n}_{a}_{b}.out"
        with open(path, "r") as datafile:
            lines = datafile.read().split("\n")[:-1]
            for line in lines:
                data["n"].append(n)
                data["limits"].append("({},{})".format(-a, b))
                data["hops"].append(int(line))

data = pd.DataFrame.from_dict(data)

sns.set_theme(style="darkgrid")
plt.rcParams["text.usetex"] = True
plt.rcParams['figure.figsize'] = 10, 6

fig, ax = plt.subplots(1, 1, sharex="row", sharey=True)

for i in range(2):
    for j in range(3):
        if i > 0 or j > 0:
            continue
        plot = sns.histplot(
            ax=ax[i, j],
            data=data[
                (data.n == n_values[i])
                & (data.limits == "({},{})".format(
                    -ab_values[j][0], ab_values[j][1]
                ))
            ],
            x="hops",
            discrete=True,
            stat="probability",
        )

        plot.set(xlabel=None)
        plot.set(ylabel=None)

ax[0][0].set_ylabel(r'$n = 100$', fontsize=14)
ax[1][0].set_ylabel(r'$n = 1000$', fontsize=14)

ax[0][0].set_title(r'$(a, b) = (-1, 1)$')
ax[0][1].set_title(r'$(a, b) = (-2, 5)$')
ax[0][2].set_title(r'$(a, b) = (-3, 10)$')

fig.suptitle("Number of Hops in Negative Weight Cycles", fontsize=20)
fig.text(
    0.5,
    0.025,
    "Number of Hops",
    ha="center",
    fontsize=15
)
fig.text(
    0.05,
    0.5,
    "Portion of all Rejections",
    va="center",
    rotation="vertical",
    fontsize=15
)

plt.savefig("exp/complete_hops/results.pdf", format="pdf", bbox_inches="tight")

plt.show()
