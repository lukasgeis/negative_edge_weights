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
parser.add_argument("-o", "--output", required=True, type=str)
parser.add_argument("-g", "--graph", required=True, type=str)

args = parser.parse_args()

data = pd.read_csv(args.datafile)
data = data[data.graph == args.graph]
data = data.drop(columns=[
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
])

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

if args.graph == "File":
    plot = sns.lineplot(
        data=data,
        x="round",
        y="rate",
        hue="initial",
        hue_order=order,
        linestyle="solid",
    )

    plot.set(xlabel=r"\textsc{MCMC Steps}")
    plot.set(ylabel=r"\textsc{Acceptance Rate}")
    plot.get_legend().set_title(r"\textsc{Initial Weights}")

    plt.xscale("log")

    plt_savefig(args.output)
    exit(0)

plot = sns.lineplot(
    data=data[data.degree == 10],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="solid",
    legend=False
)

sns.lineplot(
    data=data[data.degree == 20],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="dashed",
    legend=False
)

sns.lineplot(
    data=data[data.degree == 50],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="dotted",
    legend=False
)

sns.lineplot(
    data=data[data.degree == 500],
    x="round",
    y="rate",
    hue="initial",
    hue_order=order,
    linestyle="dashdot",
    legend=False
)

plot.set(xlabel=r"\textsc{MCMC Steps}")
plot.set(ylabel=r"\textsc{Acceptance Rate}")

plt.xscale("log")

texts = [
    r"\textsc{Average Degree}",
    r"$10$",
    r"$20$",
    r"$50$",
    r"$500$",
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
    "dotted",
    "dashdot"
]

handles, labels = gen_handles_labels(texts, colors, linestyles)

legend = plt.legend(handles, labels, ncols=2, fontsize=13, loc="upper center")

shift_labels_left(legend, [texts[0], texts[5]])

plt_savefig(args.output)
