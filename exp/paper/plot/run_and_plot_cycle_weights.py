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
from random import randrange


sns_colors = setup_plt_sns()


parser = cli.ArgumentParser()
parser.add_argument("-n", required=True, type=int)
parser.add_argument("-a", required=True, type=int)
parser.add_argument("-b", required=True, type=int)
parser.add_argument("-o", "--output", required=True, type=str)

args = parser.parse_args()

if args.n <= 0 or args.a >= 0 or args.b <= 0:
    exit(0)

wmax = [args.b for i in range(args.n)]
wzero = [0 for i in range(args.n)]

sums = [args.b * args.n, 0]

data = {
    "round": [],
    "init": [],
    "weight": []
}


def new_interval(length: int, label: str):
    for i in range(length):
        e = randrange(args.n)
        w = randrange(args.a, args.b)

        dm = w - wmax[e]
        dz = w - wzero[e]

        if dm + sums[0] >= 0:
            wmax[e] = w
            sums[0] += dm

        if dz + sums[1] >= 0:
            wzero[e] = w
            sums[1] += dz

    for i in range(args.n):
        data["round"].append(label)
        data["init"].append(r"$w_{max}$")
        data["weight"].append(wmax[i])

        data["round"].append(label)
        data["init"].append(r"$w_{zero}$")
        data["weight"].append(wzero[i])


new_interval(args.n // 2, r"$\frac{1}{2}n$")
new_interval(args.n // 2, r"$n$")
new_interval(args.n, r"$2n$")
new_interval(args.n * 3, r"$5n$")
new_interval(args.n * 5, r"$10n$")

data = pd.DataFrame.from_dict(data)

plt.clf()
plt.rcParams['figure.figsize'] = 6.4, 3.5

plot = sns.violinplot(
    data=data,
    x="round",
    y="weight",
    inner="quart",
    hue="init",
    split=True,
)

plot.set(xlabel=r"\textsc{MCMC Steps}")
plot.set(ylabel=r"\textsc{EdgeWeights}")
plot.legend(loc='lower left', title=r"\textsc{Initial Weights}")

plt_savefig(args.output)
