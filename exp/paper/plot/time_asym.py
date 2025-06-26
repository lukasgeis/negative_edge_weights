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
parser.add_argument("full_sym")
parser.add_argument("asym_a")
parser.add_argument("asym_b")
parser.add_argument("-i", "--initial", required=True, type=str)
parser.add_argument("-o", "--output", required=True, type=str)
parser.add_argument("-g", "--graph", required=True, type=str)


args = parser.parse_args()

drop_columns = [
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
]

weight_ints = [
    r"$[-100, 100]$",
    r"$[-200, 100]$",
    r"$[-100, 200]$"
]

sym_data = pd.read_csv(args.full_sym)
sym_data = sym_data[sym_data.graph == args.graph]
sym_data = sym_data[sym_data.initial == args.initial]
sym_data = sym_data[sym_data.degree == 10]
sym_data = sym_data.drop(columns=drop_columns)
sym_data["weights"] = weight_ints[0]

asym_a_data = pd.read_csv(args.asym_a)
asym_a_data = asym_a_data[asym_a_data.graph == args.graph]
asym_a_data = asym_a_data[asym_a_data.initial == args.initial]
asym_a_data = asym_a_data.drop(columns=drop_columns)
asym_a_data["weights"] = weight_ints[1]

asym_b_data = pd.read_csv(args.asym_b)
asym_b_data = asym_b_data[asym_b_data.graph == args.graph]
asym_b_data = asym_b_data[asym_b_data.initial == args.initial]
asym_b_data = asym_b_data.drop(columns=drop_columns)
asym_b_data["weights"] = weight_ints[2]

data = pd.concat([sym_data, asym_a_data, asym_b_data])


fdata = {
    "round": [],
    "algo": [],
    "time": [],
    "weights": [],
}

for _, row in data.iterrows():
    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{BellmanFord}")
    fdata["time"].append(row["time_bf"])
    fdata["weights"].append(row["weights"])

    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{Dijkstra}")
    fdata["time"].append(row["time_dk"])
    fdata["weights"].append(row["weights"])

    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{BiDijkstra}")
    fdata["time"].append(row["time_bd"])
    fdata["weights"].append(row["weights"])

data = pd.DataFrame.from_dict(fdata)

if len(data) == 0:
    exit(0)


initials = ["Maximum", "Uniform", "Zero"]

order = [r"\textsc{BellmanFord}", r"\textsc{Dijkstra}", r"\textsc{BiDijkstra}"]

plt.clf()
plt.rcParams['figure.figsize'] = 6.4, 3.5


plot = sns.lineplot(
    data=data[data.weights == weight_ints[0]],
    x="round",
    y="time",
    hue="algo",
    hue_order=order,
    linestyle="solid",
    legend=False
)

sns.lineplot(
    data=data[data.weights == weight_ints[1]],
    x="round",
    y="time",
    hue="algo",
    hue_order=order,
    linestyle="dashed",
    legend=False
)

sns.lineplot(
    data=data[data.weights == weight_ints[2]],
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
    r"\textsc{Weights} $\mathcal{W}$",
    weight_ints[0],
    weight_ints[1],
    weight_ints[2],
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
