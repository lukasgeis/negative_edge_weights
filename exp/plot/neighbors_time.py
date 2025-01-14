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
parser.add_argument("logfile")
parser.add_argument("neighborfile")
parser.add_argument("-d", "--degree", type=int)
parser.add_argument("-o", "--output", required=True, type=str)

args = parser.parse_args()

data = {
    "graph": [],
    "degree": [],
    "initial": [],
    "round": [],
    "rate": [],
    "algo": [],
}


log = pd.read_csv(args.logfile)
for _, row in log.iterrows():
    data["graph"].append(row["graph"])
    data["degree"].append(row["degree"])
    data["initial"].append(row["initial"])
    data["round"].append(row["round"])
    data["rate"].append(
        row["time_dk"] / row["num_acc_rounds"]
    )
    data["algo"].append("DK")

    data["graph"].append(row["graph"])
    data["degree"].append(row["degree"])
    data["initial"].append(row["initial"])
    data["round"].append(row["round"])
    data["rate"].append(
        row["time_bd"] / row["num_acc_rounds"]
    )
    data["algo"].append("BD")

neighbors = pd.read_csv(args.neighborfile)
for _, row in neighbors.iterrows():
    data["graph"].append(row["graph"])
    data["degree"].append(row["degree"])
    data["initial"].append(row["initial"])
    data["round"].append(row["round"])
    data["rate"].append(
        row["runtime"] / row["num_acc"]
    )
    data["algo"].append("NS")

data = pd.DataFrame.from_dict(data)

if args.degree is not None:
    data = data[data.degree == args.degree]

if len(data) == 0:
    exit(0)


graphs = ["Gnp", "Rhg", "Dsf"]
degrees = [10, 20, 50]

order = ["DK", "BD", "NS"]

plt.clf()
if args.degree is not None:
    plt.rcParams['figure.figsize'] = 16, 3
    fig, ax = plt.subplots(1, 3, sharex=True, sharey=True)

    for i in range(3):
        print(f"i: {i}")

        graph = graphs[i]

        sdata = data[data.graph == graph]

        plot = sns.lineplot(
            ax=ax[i],
            data=sdata[sdata.initial == "Maximum"],
            x="round",
            y="rate",
            hue="algo",
            hue_order=order,
            linestyle="solid",
            legend=False
        )

        sns.lineplot(
            ax=ax[i],
            data=sdata[sdata.initial == "Uniform"],
            x="round",
            y="rate",
            hue="algo",
            hue_order=order,
            linestyle="dashed",
            legend=False
        )

        sns.lineplot(
            ax=ax[i],
            data=sdata[sdata.initial == "Zero"],
            x="round",
            y="rate",
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
        r"\textsc{Initial Weights}",
        r"$w_{max}$",
        r"$w_{unif}$",
        r"$w_{zero}$",
        r"\textsc{Algorithm}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}",
        r"\textsc{NodeSampler}"
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

    legend = plt.legend(handles, labels, ncols=8, fontsize=13, loc="lower center", bbox_to_anchor=(-0.8, -0.5))

    fig.text(0.515,-0.08, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.065, 0.50, r'\textsc{Time in} $ns$ \textsc{per}' "\n" r'\textsc{Weight Update}', va="center", rotation="vertical", fontsize=17)

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
            degree = degrees[j]

            sdata = data[(data.graph == graph) & (data.degree == degree)]

            plot = sns.lineplot(
                ax=ax[i, j],
                data=sdata[sdata.initial == "Maximum"],
                x="round",
                y="rate",
                hue="algo",
                hue_order=order,
                linestyle="solid",
                legend=False
            )

            sns.lineplot(
                ax=ax[i, j],
                data=sdata[sdata.initial == "Uniform"],
                x="round",
                y="rate",
                hue="algo",
                hue_order=order,
                linestyle="dashed",
                legend=False
            )

            sns.lineplot(
                ax=ax[i, j],
                data=sdata[sdata.initial == "Zero"],
                x="round",
                y="rate",
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
        r"\textsc{Initial Weights}",
        r"$w_{max}$",
        r"$w_{unif}$",
        r"$w_{zero}$",
        r"\textsc{Algorithm}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}",
        r"\textsc{NodeSampler}"
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

    legend = plt.legend(handles, labels, ncols=8, fontsize=13, loc="lower center", bbox_to_anchor=(-0.8, -0.5))

    fig.text(0.515, 0.06, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.053, 0.50, r'\textsc{Time in} $ns$ \textsc{per}' "\n" r'\textsc{Weight Update}', va="center", rotation="vertical", fontsize=17)
    
    fig.text(0.227, 0.90, r'$\overline{d} = 10$', ha="center", fontsize=18)
    fig.text(0.510, 0.90, r'$\overline{d} = 20$', ha="center", fontsize=18)
    fig.text(0.785, 0.90, r'$\overline{d} = 50$', ha="center", fontsize=18)

    fig.text(0.085, 0.23, r'$\mathcal{DSF}$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.50, r'$\mathcal{RHG}$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.77, r'$\mathcal{GNP}$', va="center", rotation="vertical", fontsize=15)


plt_savefig(args.output)
