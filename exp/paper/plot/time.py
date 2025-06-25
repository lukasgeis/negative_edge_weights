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
    plt_savefig
)

sns_colors = setup_plt_sns()


parser = cli.ArgumentParser()
parser.add_argument("datafile")
parser.add_argument("-i", "--initial", required=True, type=str)
parser.add_argument("-o", "--output", required=True, type=str)
parser.add_argument("-g", "--graph", required=True, type=str)


args = parser.parse_args()

data = pd.read_csv(args.datafile)
data = data[(data.graph == args.graph) & (data.initial == args.initial)]
data = data.drop(columns=[
    "m",
    "num_acc_rounds",
    "average_weight",
    "tvd",
    "num_neg_edges",
    "num_ins_bf_acc",
    "num_ins_bf_rej",
    "num_ins_dk_acc",
    "num_ins_dk_rej",
    "num_ins_bd_acc",
    "num_ins_bd_rej",
    "num_pot_dk",
    "num_pot_bd"
])


fdata = {
    "round": [],
    "algo": [],
    "time": [],
    "degree": [],
}

for _, row in data.iterrows():
    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{BellmanFord}")
    fdata["time"].append(row["time_bf"])
    fdata["degree"].append(row["degree"])

    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{Dijkstra}")
    fdata["time"].append(row["time_dk"])
    fdata["degree"].append(row["degree"])

    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{BiDijkstra}")
    fdata["time"].append(row["time_bd"])
    fdata["degree"].append(row["degree"])

data = pd.DataFrame.from_dict(fdata)

if len(data) == 0:
    exit(0)


initials = ["Maximum", "Uniform", "Zero"]

order = [r"\textsc{BellmanFord}", r"\textsc{Dijkstra}", r"\textsc{BiDijkstra}"]

plt.clf()
plt.rcParams['figure.figsize'] = 6.4, 3.5

if args.graph == "File":
    plot = sns.lineplot(
        data=data,
        x="round",
        y="time",
        hue="algo",
        hue_order=order,
        linestyle="solid",
    )

    plot.set(xlabel=r"\textsc{MCMC Steps}")
    plot.set(ylabel=r"\textsc{Time in} $ns$")

    plot.get_legend().set_title(r"\textsc{Algorithm}")

    plt.xscale("log")
    plt.yscale("log")

    plt_savefig(args.output)
    exit(0)


plot = sns.lineplot(
    data=data[data.degree == 10],
    x="round",
    y="time",
    hue="algo",
    hue_order=order,
    linestyle="solid",
    legend=False
)

sns.lineplot(
    data=data[data.degree == 20],
    x="round",
    y="time",
    hue="algo",
    hue_order=order,
    linestyle="dashed",
    legend=False
)

sns.lineplot(
    data=data[data.degree == 50],
    x="round",
    y="time",
    hue="algo",
    hue_order=order,
    linestyle="dotted",
    legend=False
)

plot.set(xlabel=r"\textsc{MCMC Steps}")
plot.set(ylabel=r"\textsc{Time in} $ns$")

plt.xscale("log")
plt.yscale("log")

texts = [
    r"\textsc{Average Degree}",
    r"$10$",
    r"$20$",
    r"$50$",
    r"\textsc{Algorithm}",
    r"\textsc{BellmanFord}",
    r"\textsc{Dijkstra}",
    r"\textsc{BiDijkstra}"
]
colors = [
    "none",
    "black",
    "black",
    "black",
    "none",
    sns_colors[0],
    sns_colors[1],
    sns_colors[2]
]
linestyles = [
    None,
    "solid",
    "dashed",
    "dotted"
]

handles, labels = gen_handles_labels(texts, colors, linestyles)

legend = plt.legend(handles, labels, ncols=1, fontsize=13, loc="upper left")

shift_labels_left(legend, [texts[0], texts[4]])
 
plt_savefig(args.output)
