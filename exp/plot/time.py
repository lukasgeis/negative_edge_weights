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
parser.add_argument("-i", "--initial", type=str)
parser.add_argument("-o", "--output", required=True, type=str)

args = parser.parse_args()

data = pd.read_csv(args.datafile)
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

if args.initial is not None:
    data = data[data.initial == args.initial]

fdata = {
    "graph": [],
    "initial": [],
    "round": [],
    "algo": [],
    "time": [],
    "degree": [],
}

for _, row in data.iterrows():
    fdata["graph"].append(row["graph"])
    fdata["initial"].append(row["initial"])
    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{BellmanFord}")
    fdata["time"].append(row["time_bf"])
    fdata["degree"].append(row["degree"])

    fdata["graph"].append(row["graph"])
    fdata["initial"].append(row["initial"])
    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{Dijkstra}")
    fdata["time"].append(row["time_dk"])
    fdata["degree"].append(row["degree"])

    fdata["graph"].append(row["graph"])
    fdata["initial"].append(row["initial"])
    fdata["round"].append(row["round"])
    fdata["algo"].append(r"\textsc{BiDijkstra}")
    fdata["time"].append(row["time_bd"])
    fdata["degree"].append(row["degree"])

data = pd.DataFrame.from_dict(fdata)

if len(data) == 0:
    exit(0)


graphs = ["Gnp", "Rhg", "Dsf"]
initials = ["Maximum", "Uniform", "Zero"]

order = [r"\textsc{BellmanFord}", r"\textsc{Dijkstra}", r"\textsc{BiDijkstra}"]

plt.clf()
if args.initial is not None:
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
            y="time",
            hue="algo",
            hue_order=order,
            linestyle="solid",
            legend=False
        )

        sns.lineplot(
            ax=ax[i],
            data=sdata[sdata.degree == 20],
            x="round",
            y="time",
            hue="algo",
            hue_order=order,
            linestyle="dashed",
            legend=False
        )

        sns.lineplot(
            ax=ax[i],
            data=sdata[sdata.degree == 50],
            x="round",
            y="time",
            hue="algo",
            hue_order=order,
            linestyle="dotted",
            legend=False
        )

        plot.set(xlabel="")
        plot.set(ylabel="")

    plt.xscale("log")
    plt.yscale("log")

    texts = [
        r"\textsc{Average Degree}",
        r"$10$",
        r"$20$",
        r"$50$",
        r"\textsc{Algorithm}",
        r"\textsc{Sampler}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}"
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

    legend = plt.legend(handles, labels, ncols=8, fontsize=13, bbox_to_anchor=(1.07, -0.25))
     
    fig.text(0.520,-0.08, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.085, 0.50, r'\textsc{Time in} $ns$', va="center", rotation="vertical", fontsize=20)

    fig.text(0.227, 0.90, r'$\mathcal{GNP}$', ha="center", fontsize=15)
    fig.text(0.510, 0.90, r'$\mathcal{RHG}$', ha="center", fontsize=15)
    fig.text(0.785, 0.90, r'$\mathcal{DSF}$', ha="center", fontsize=15)
else:
    plt.rcParams['figure.figsize'] = 16, 9

    fig, ax = plt.subplots(3, 3, sharex=True, sharey=True)

    for i in range(3):
        for j in range(3):
            print(f"i: {i}, j: {j}")

            graph = graphs[i]
            initial = initials[j]

            sdata = data[(data.graph == graph) & (data.initial == initial)]

            plot = sns.lineplot(
                ax=ax[i, j],
                data=sdata[sdata.degree == 10],
                x="round",
                y="time",
                hue="algo",
                hue_order=order,
                linestyle="solid",
                legend=False
            )

            sns.lineplot(
                ax=ax[i, j],
                data=sdata[sdata.degree == 20],
                x="round",
                y="time",
                hue="algo",
                hue_order=order,
                linestyle="dashed",
                legend=False
            )

            sns.lineplot(
                ax=ax[i, j],
                data=sdata[sdata.degree == 50],
                x="round",
                y="time",
                hue="algo",
                hue_order=order,
                linestyle="dotted",
                legend=False
            )

            plot.set(xlabel="")
            plot.set(ylabel="")

    plt.xscale("log")
    plt.yscale("log")

    texts = [
        r"\textsc{Average Degree}",
        r"$10$",
        r"$20$",
        r"$50$",
        r"\textsc{Algorithm}",
        r"\textsc{Sampler}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}"
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

    legend = plt.legend(handles, labels, ncols=8, fontsize=13, bbox_to_anchor=(1.07, -0.3))

    fig.text(0.520, 0.05, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.052, 0.50, r'\textsc{Time in} $ns$', va="center", rotation="vertical", fontsize=20)

    fig.text(0.227, 0.90, r'$w_{max}$', ha="center", fontsize=18)
    fig.text(0.510, 0.90, r'$w_{unif}$', ha="center", fontsize=18)
    fig.text(0.785, 0.90, r'$w_{zero}$', ha="center", fontsize=18)

    fig.text(0.085, 0.23, r'$\mathcal{DSF}$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.50, r'$\mathcal{RHG}$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.77, r'$\mathcal{GNP}$', va="center", rotation="vertical", fontsize=15)

plt_savefig(args.output)
