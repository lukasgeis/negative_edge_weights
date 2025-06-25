from common import (
    plt,
    Line2D,
    sns,
    pd,
    cli,
    sys,
    setup_plt_sns,
    shift_labels_left,
    gen_handles_labels,
    repeat_observations,
    plt_savefig
)

sns_colors = setup_plt_sns()


parser = cli.ArgumentParser()
parser.add_argument("datafile")
parser.add_argument("-g", "--graph", required=True, type=str)
parser.add_argument("-i", "--initial", required=True, type=str)
parser.add_argument("-d", "--degree", required=True, type=int)
parser.add_argument("-o", "--output", required=True, type=str)

args = parser.parse_args()

data = pd.read_csv(args.datafile)

data = data[
    (data.graph == args.graph) &
    (data.initial == args.initial) &
    (data.degree == args.degree) &
    (data.ins > 0) &
    (data.num > 0)
]

max_round = data["round"].max()
data = data[data["round"] == max_round]

if len(data) == 0:
    exit(0)


def scale_num(row):
    row["num"] = row["num"] // 100
    if row["num"] == 0:
        row["num"] = 1
    return row


data = data.apply(scale_num, axis=1)
data = data.astype({"num": "int"})

data = data.reindex(data.index.repeat(data["num"]))
data.reset_index(drop=True, inplace=True)

max_ins = data["ins"].max()

algorithms = {
    "BF": r"\textsc{BellmanFord}",
    "BD": r"\textsc{BiDijkstra}",
    "DK": r"\textsc{Dijkstra}"
}

order = [algorithms["BF"], algorithms["DK"], algorithms["BD"]]

acceptance = {
    True: r"\textsc{Accepted}",
    False: r"\textsc{Rejected}",
}

data.replace({"algo": algorithms, "acc": acceptance}, inplace=True)

plt.rcParams["figure.figsize"] = 6.4, 3

plt.clf()
plot = sns.boxenplot(
    data,
    x="algo",
    y="ins",
    hue="acc",
    order=order,
)

plot.set(xlabel=None)
plot.set(ylabel=r"\textsc{Insertions}")

plt.yscale("log")

plot.get_legend().set_title("")

plt_savefig(args.output)
