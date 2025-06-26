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
parser.add_argument("-o", "--output", required=True, type=str)
parser.add_argument("-g", "--graph", required=True, type=str)

args = parser.parse_args()

drop_columns = [
    "m",
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
    "num_pot_bd",
    "time_bf",
    "time_dk",
    "time_bd"
]

weight_ints = [
    r"$[-100, 100]$",
    r"$[-200, 100]$",
    r"$[-100, 200]$"
]

sym_data = pd.read_csv(args.full_sym)
sym_data = sym_data[sym_data.graph == args.graph]
sym_data = sym_data[sym_data.degree == 10]
sym_data = sym_data.drop(columns=drop_columns)
sym_data["weights"] = weight_ints[0]

asym_a_data = pd.read_csv(args.asym_a)
asym_a_data = asym_a_data[asym_a_data.graph == args.graph]
asym_a_data = asym_a_data.drop(columns=drop_columns)
asym_a_data["weights"] = weight_ints[1]

asym_b_data = pd.read_csv(args.asym_b)
asym_b_data = asym_b_data[asym_b_data.graph == args.graph]
asym_b_data = asym_b_data.drop(columns=drop_columns)
asym_b_data["weights"] = weight_ints[2]

data = pd.concat([sym_data, asym_a_data, asym_b_data])

if len(data) == 0:
    exit(0)


initials = {
    "Maximum": r"$w_{max}$",
    "Zero": r"$w_{zero}$",
    "Uniform": r"$w_{unif}$"
}


data.replace({"initial": initials}, inplace=True)
data["rate"] = data["num_acc_rounds"] / data["round"]

order = [initials["Maximum"], initials["Uniform"], initials["Zero"]]

plt.clf()
plt.rcParams['figure.figsize'] = 6.4, 3.5

plot = sns.lineplot(
    data=data[data.weights == weight_ints[0]],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="solid",
    legend=False
)

sns.lineplot(
    data=data[data.weights == weight_ints[1]],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="dashed",
    legend=False
)

sns.lineplot(
    data=data[data.weights == weight_ints[2]],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="dotted",
    legend=False
)

plot.set(xlabel=r"\textsc{MCMC Steps}")
plot.set(ylabel=r"\textsc{Acceptance Rate}")

plt.xscale("log")

texts = [
    r"\textsc{Weights} $\mathcal{W}$",
    weight_ints[0],
    weight_ints[1],
    weight_ints[2],
    r"\textsc{Initial Weights}",
    r"$w_{max}$",
    r"$w_{unif}$",
    r"$w_{zero}$"
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
