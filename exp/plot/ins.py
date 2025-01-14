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

import math

sns_colors = setup_plt_sns()


parser = cli.ArgumentParser()
parser.add_argument("datafile")
parser.add_argument("-g", "--graph", type=str)
parser.add_argument("-i", "--initial", type=str)
parser.add_argument("-d", "--degree", type=int)
parser.add_argument("-o", "--output", required=True, type=str)

args = parser.parse_args()

if args.graph is None:
    if args.initial is None or args.degree is None:
        print("Either provide Graph or Initial+Degree")
        exit(0)

data = pd.read_csv(args.datafile)

if args.graph is not None:
    data = data[data.graph == args.graph]
else:
    data = data[(data.initial == args.initial) & (data.degree == args.degree)]

data = data[(data.ins > 0) & (data.num > 0)]

if len(data) == 0:
    exit(0)

factor = 100
max_round = int(math.log10(data["round"].max()))

factors = {}
for alg in ["BF", "DK", "BD"]:
    afactors = {}
    for acc in [True, False]:
        acfactors = {}
        for rounds in range(max_round + 1):
            mv = data[
                (data.algo == alg) &
                (data.acc == acc) &
                (data["round"] == int(10**rounds))
            ]["num"].max()

            npt = min(factor, 10 * (mv // 10))
            if npt == 0:
                npt = 1

            acfactors[int(10**rounds)] = mv / npt
        afactors[acc] = acfactors
    factors[alg] = afactors


def scale_num(row):
    row["num"] = row["num"] // factors[row["algo"]][row["acc"]][row["round"]]
    if row["num"] == 0:
        row["num"] = 1
    return row


data = data.apply(scale_num, axis=1)

data = data.astype({"num": "int"})


data = data.reindex(data.index.repeat(data["num"]))
data.reset_index(drop=True, inplace=True)

max_ins = data["ins"].max()

graphs = ["Gnp", "Rhg", "Dsf"]
initials = ["Maximum", "Uniform", "Zero"]
degrees = [10, 20, 50]
accepted = [True, False]

order = ["BF", "DK", "BD"]

plt.clf()
if args.graph is None:
    plt.rcParams['figure.figsize'] = 16, 6

    fig, ax = plt.subplots(2, 3, sharex=True, sharey=True)
    plt.yscale("log")

    for i in range(2):
        for j in range(3):
            print(f"i: {i}, j: {j}")

            acc = accepted[i]
            graph = graphs[j]

            sdata = data[(data.graph == graph) & (data.acc == acc)]

            plot = sns.boxenplot(
                ax=ax[i, j],
                data=sdata,
                x="round",
                y="ins",
                hue="algo",
                hue_order=order,
                legend=False,
                gap=0.1,
            )

            plot.axhline(2, 0, 1, linestyle="dashed", linewidth=1, color=sns_colors[3])

            if i == 1 and j == 0:
                xticks = plot.get_xticks()
                xlabels = plot.get_xticklabels()
                for k in range(len(xlabels)):
                    exp = int(math.log10(int(xlabels[k].get_text())))
                    xlabels[k].set_text(
                        r"$\mathdefault{{10^{{{exp}}}}}$".format(exp=str(exp))
                    )
                plot.set_xticks(xticks)
                plot.set_xticklabels(xlabels)

            if i == 0 and j == 0:
                plot.set_yticks([1.e+00, 2.1e+00, 1.e+01] + [
                    x for x in list(plt.yticks()[0]) if x > 1.e+01 and x < max_ins * 10
                ])

                ylabels = plot.get_yticklabels()
                ylabels[1].set_text(r'$\mathdefault{2}$')
                ylabels[1].set_size(8)
                plot.set_yticklabels(ylabels)

            plot.set(xlabel="")
            plot.set(ylabel="")

    texts = [
        r"\textsc{Algorithm}",
        r"\textsc{Sampler}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}",
    ]
    colors = [
        "none",
        sns_colors[0],
        sns_colors[1],
        sns_colors[2],
    ]
    linestyles = [] 

    handles, labels = gen_handles_labels(texts, colors, linestyles)

    legend = plt.legend(handles, labels, ncols=4, fontsize=13, bbox_to_anchor=(0.25, -0.25))
     
    fig.text(0.520, 0.03, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.085, 0.50, r'\textsc{QueueInsertions}', va="center", rotation="vertical", fontsize=17)

    fig.text(0.227, 0.90, r'$\mathcal{GNP}$', ha="center", fontsize=15)
    fig.text(0.510, 0.90, r'$\mathcal{RHG}$', ha="center", fontsize=15)
    fig.text(0.785, 0.90, r'$\mathcal{DSF}$', ha="center", fontsize=15)

    fig.text(0.91, 0.70, r'\textsc{Accepted}', va="center", rotation=270, fontsize=15)
    fig.text(0.91, 0.30, r'\textsc{Rejected}', va="center", rotation=270, fontsize=15)
else:
    plt.rcParams['figure.figsize'] = 16, 18

    fig, ax = plt.subplots(6, 3, sharex=True, sharey=True)
    plt.yscale("log")

    for i in range(3):
        for j in range(3):
            for l in range(2):
                print(f"i: {i}, j: {j}, l: {l}")

                degree = degrees[i]
                initial = initials[j]
                acc = accepted[l]

                ri = i * 2 + l

                sdata = data[(data.initial == initial) & (data.degree == degree) & (data.acc == acc)]

                plot = sns.boxenplot(
                    ax=ax[ri, j],
                    data=sdata,
                    x="round",
                    y="ins",
                    hue="algo",
                    hue_order=order,
                    legend=False,
                    gap=0.1,
                )

                plot.axhline(2, 0, 1, linestyle="dashed", linewidth=1, color=sns_colors[3])

                if ri == 5 and j == 0:
                    xticks = plot.get_xticks()
                    xlabels = plot.get_xticklabels()
                    for k in range(len(xlabels)):
                        exp = int(math.log10(int(xlabels[k].get_text())))
                        xlabels[k].set_text(
                            r"$\mathdefault{{10^{{{exp}}}}}$".format(exp=str(exp))
                        )
                    plot.set_xticks(xticks)
                    plot.set_xticklabels(xlabels)

                if i == 0 and j == 0:
                    plot.set_yticks([1.e+00, 2.1e+00, 1.e+01] + [
                        x for x in list(plt.yticks()[0]) if x > 1.e+01 and x < max_ins * 10
                    ])

                    ylabels = plot.get_yticklabels()
                    ylabels[1].set_text(r'$\mathdefault{2}$')
                    ylabels[1].set_size(8)
                    plot.set_yticklabels(ylabels)

                plot.set(xlabel="")
                plot.set(ylabel="")

    texts = [
        r"\textsc{Algorithm}",
        r"\textsc{Sampler}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}",
    ]
    colors = [
        "none",
        sns_colors[0],
        sns_colors[1],
        sns_colors[2],
    ]
    linestyles = []

    handles, labels = gen_handles_labels(texts, colors, linestyles)

    legend = plt.legend(handles, labels, ncols=4, fontsize=13, bbox_to_anchor=(0.22, -0.35))

    fig.text(0.520, 0.08, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.052, 0.50, r'\textsc{QueueInsertions}', va="center", rotation="vertical", fontsize=17)

    fig.text(0.227, 0.89, r'$w_{max}$', ha="center", fontsize=18)
    fig.text(0.510, 0.89, r'$w_{unif}$', ha="center", fontsize=18)
    fig.text(0.785, 0.89, r'$w_{zero}$', ha="center", fontsize=18)

    fig.text(0.085, 0.23, r'$\overline{d} = 50$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.50, r'$\overline{d} = 20$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.77, r'$\overline{d} = 10$', va="center", rotation="vertical", fontsize=15)

    fig.text(0.91, 0.83, r'\textsc{Accepted}', va="center", rotation=270, fontsize=15)
    fig.text(0.91, 0.69, r'\textsc{Rejected}', va="center", rotation=270, fontsize=15)
    fig.text(0.91, 0.56, r'\textsc{Accepted}', va="center", rotation=270, fontsize=15)
    fig.text(0.91, 0.43, r'\textsc{Rejected}', va="center", rotation=270, fontsize=15)
    fig.text(0.91, 0.30, r'\textsc{Accepted}', va="center", rotation=270, fontsize=15)
    fig.text(0.91, 0.17, r'\textsc{Rejected}', va="center", rotation=270, fontsize=15)
plt_savefig(args.output)
