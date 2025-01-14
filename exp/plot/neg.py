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

args = parser.parse_args()

data = pd.read_csv(args.datafile)
data = data.drop(columns=[
    "average_weight",
    "tvd",
    "num_acc_rounds",
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
data["rate"] = data["num_neg_edges"] / data["m"]

graphs = ["Gnp", "Rhg", "Dsf"]

order = [initials["Maximum"], initials["Uniform"], initials["Zero"]]

plt.clf()
plt.rcParams['figure.figsize'] = 16, 3
fig, ax = plt.subplots(1, 3, sharex=True, sharey=True)

for i in range(3):
    print(f"i: {i}")

    graph = graphs[i]

    sdata = data[data.graph == graph]

    plot = sns.lineplot(
        ax=ax[i],
        data=sdata[sdata.degree == 10],
        x="round",
        y="rate",
        hue="initial",
        hue_order=order,
        linestyle="solid",
        legend=False
    )

    sns.lineplot(
        ax=ax[i],
        data=sdata[sdata.degree == 20],
        x="round",
        y="rate",
        hue="initial",
        hue_order=order,
        linestyle="dashed",
        legend=False
    )

    sns.lineplot(
        ax=ax[i],
        data=sdata[sdata.degree == 50],
        x="round",
        y="rate",
        hue="initial",
        hue_order=order,
        linestyle="dotted",
        legend=False
    )

    plot.set(xlabel="")
    plot.set(ylabel="")

plt.xscale("log")

texts = [
    r"\textsc{Average Degree}",
    r"$10$",
    r"$20$",
    r"$50$",
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

legend = plt.legend(handles, labels, ncols=8, fontsize=13, bbox_to_anchor=(0.85, -0.25))

fig.text(0.515,-0.08, r'\textsc{Rounds}', ha="center", fontsize=20)
fig.text(0.065, 0.50, r"\textsc{Fraction of}" "\n" r"\textsc{Negative Edges}", va="center", rotation="vertical", fontsize=17)

fig.text(0.227, 0.90, r'$\mathcal{GNP}$', ha="center", fontsize=15)
fig.text(0.510, 0.90, r'$\mathcal{RHG}$', ha="center", fontsize=15)
fig.text(0.785, 0.90, r'$\mathcal{DSF}$', ha="center", fontsize=15)

plt_savefig(args.output)
