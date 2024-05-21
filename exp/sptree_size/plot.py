import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

n_values = [100, 1000]
ab_values = [(1, 1), (2, 5), (3, 10)]

data = {
    "n": [],
    "limits": [],
    "datatype": [],
    "algo": [],
    "nodes_visited": [],
    "nodes_queued": [],
    "edges_traversed": [],
}

for n in n_values:
    for (a, b) in ab_values:
        for t in ["float", "integer"]:
            path = "data/sptree_size/{}_{}_{}_{}.out".format(n, a, b, t[0])
            with open(path, "r") as datafile:
                lines = datafile.read().split("\n")[:-1]

                last_values = (0, 0, 0)
                for line in lines:
                    linedata = line.split(",")

                    data["n"].append(n)
                    data["limits"].append("({},{})".format(-a, b))
                    data["datatype"].append(t)
                    data["algo"].append(linedata[3] + " - " + linedata[4])
                    data["nodes_visited"].append(int(linedata[0]))
                    data["nodes_queued"].append(int(linedata[1]))
                    data["edges_traversed"].append(int(linedata[2]))

                    if linedata[3] == "bidijkstra":
                        if linedata[4] == "forward":
                            last_values = (int(linedata[0]), int(linedata[1]), int(linedata[2]))
                        else:
                            data["n"].append(n)
                            data["limits"].append("({},{})".format(-a, b))
                            data["datatype"].append(t)
                            data["algo"].append("bidijktra - total")
                            data["nodes_visited"].append(int(linedata[0]) + last_values[0])
                            data["nodes_queued"].append(int(linedata[1]) + last_values[1])
                            data["edges_traversed"].append(int(linedata[2]) + last_values[2])

                            last_values = (0, 0, 0)

                                


data = pd.DataFrame.from_dict(data)

sns.set_theme(style="darkgrid")
plt.rcParams["text.usetex"] = True
plt.rcParams["figure.figsize"] = 24, 10

fig, ax = plt.subplots(2, 3)

for i in range(2):
    for j in range(3):
        plot = sns.violinplot(
            ax=ax[i, j],
            data=data[
                (data.n == n_values[i]) &
                (data.limits == "({},{})".format(-ab_values[j][0],ab_values[j][1]))
            ],
            x="algo",
            y="edges_traversed",
            split=True,
            hue="datatype"
        )

        plot.set(xlabel=None)
        plot.set(ylabel=None)

ax[0][0].set_ylabel(r'$n = 100$', fontsize=14)
ax[1][0].set_ylabel(r'$n = 1000$', fontsize=14)

ax[0][0].set_title(r'$(a, b) = (-1, 1)$')
ax[0][1].set_title(r'$(a, b) = (-2, 5)$')
ax[0][2].set_title(r'$(a, b) = (-3, 10)$')

plt.savefig("exp/sptree_size/results.pdf", format="pdf", bbox_inches="tight")

plt.show()
