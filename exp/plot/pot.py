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

data = data[data.pot > 0]

if len(data) == 0:
    exit(0)

max_pot = data["pot"].max()

graphs = ["Gnp", "Rhg", "Dsf"]
initials = ["Maximum", "Uniform", "Zero"]
degrees = [10, 20, 50]


order = ["DK", "BD"]

plt.clf()
if args.graph is None:
    plt.rcParams['figure.figsize'] = 16, 3

    fig, ax = plt.subplots(1, 3, sharex=True, sharey=True)
    plt.yscale("log")

    for i in range(3):
        print(f"i: {i}")

        graph = graphs[i]

        sdata = data[data.graph == graph]

        sdata = repeat_observations(sdata, "num")

        plot = sns.boxenplot(
            ax=ax[i],
            data=sdata,
            x="round",
            y="pot",
            hue="algo",
            hue_order=order,
            legend=False,
            gap=0.1,
        )

        plot.axhline(2, 0, 1, linestyle="dashed", linewidth=1, color=sns_colors[3])

        if i == 0:
            xticks = plot.get_xticks()
            xlabels = plot.get_xticklabels()
            for k in range(len(xlabels)):
                exp = int(math.log10(int(xlabels[k].get_text())))
                xlabels[k].set_text(
                    r"$\mathdefault{{10^{{{exp}}}}}$".format(exp=str(exp))
                )
            plot.set_xticks(xticks)
            plot.set_xticklabels(xlabels)

            plot.set_yticks([1.e+00, 2.e+00, 1.e+01] + [
                x for x in list(plt.yticks()[0]) if x > 1.e+01 and x < max_pot * 10
            ])

            ylabels = plot.get_yticklabels()
            ylabels[1].set_text(r'$\mathdefault{2}$')
            ylabels[1].set_size(10)
            plot.set_yticklabels(ylabels)

        plot.set(xlabel="")
        plot.set(ylabel="")

    texts = [
        r"\textsc{Algorithm}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}",
    ]
    colors = [
        "none",
        sns_colors[0],
        sns_colors[1],
    ]
    linestyles = [] 

    handles, labels = gen_handles_labels(texts, colors, linestyles)

    legend = plt.legend(handles, labels, ncols=4, fontsize=13, bbox_to_anchor=(0.10, -0.25))
     
    fig.text(0.520,-0.08, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.085, 0.50, r'\textsc{PotentialUpdates}', va="center", rotation="vertical", fontsize=17)

    fig.text(0.227, 0.90, r'$\mathcal{GNP}$', ha="center", fontsize=15)
    fig.text(0.510, 0.90, r'$\mathcal{RHG}$', ha="center", fontsize=15)
    fig.text(0.785, 0.90, r'$\mathcal{DSF}$', ha="center", fontsize=15)
else:
    plt.rcParams['figure.figsize'] = 16, 9

    fig, ax = plt.subplots(3, 3, sharex=True, sharey=True)
    plt.yscale("log")

    for i in range(3):
        for j in range(3):
            print(f"i: {i}, j: {j}")

            degree = degrees[i]
            initial = initials[j]

            sdata = data[(data.initial == initial) & (data.degree == degree)]

            sdata = repeat_observations(sdata, "num")

            plot = sns.boxenplot(
                ax=ax[i, j],
                data=sdata,
                x="round",
                y="pot",
                hue="algo",
                hue_order=order,
                legend=False,
                gap=0.1,
            )

            plot.axhline(2, 0, 1, linestyle="dashed", linewidth=1, color=sns_colors[3])

            if i == 2 and j == 0:
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
                plot.set_yticks([1.e+00, 2.e+00, 1.e+01] + [
                    x for x in list(plt.yticks()[0]) if x > 1.e+01 and x < max_pot * 10
                ])

                ylabels = plot.get_yticklabels()
                ylabels[1].set_text(r'$\mathdefault{2}$')
                ylabels[1].set_size(10)
                ylabels[1].set_color(sns_colors[3])
                plot.set_yticklabels(ylabels)

            plot.set(xlabel="")
            plot.set(ylabel="")

    texts = [
        r"\textsc{Algorithm}",
        r"\textsc{SamplerPot}",
        r"\textsc{BiSamplerPot}",
    ]
    colors = [
        "none",
        sns_colors[0],
        sns_colors[1],
    ]
    linestyles = [] 

    handles, labels = gen_handles_labels(texts, colors, linestyles)

    legend = plt.legend(handles, labels, ncols=4, fontsize=13, bbox_to_anchor=(0.10, -0.3))

    fig.text(0.520, 0.05, r'\textsc{Rounds}', ha="center", fontsize=20)
    fig.text(0.052, 0.50, r'\textsc{PotentialUpdates}', va="center", rotation="vertical", fontsize=17)
    
    fig.text(0.227, 0.90, r'$w_{max}$', ha="center", fontsize=18)
    fig.text(0.510, 0.90, r'$w_{unif}$', ha="center", fontsize=18)
    fig.text(0.785, 0.90, r'$w_{zero}$', ha="center", fontsize=18)

    fig.text(0.085, 0.23, r'$\overline{d} = 10$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.50, r'$\overline{d} = 20$', va="center", rotation="vertical", fontsize=15)
    fig.text(0.085, 0.77, r'$\overline{d} = 50$', va="center", rotation="vertical", fontsize=15)

plt_savefig(args.output)
