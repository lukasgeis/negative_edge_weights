#!/usr/bin/env python3
import seaborn as sns
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np

sns.set_theme(style="darkgrid")
sns.color_palette("colorblind")
plt.rcParams["text.usetex"] = True

data = pd.read_json("result.json", lines=True)
data["opt_conv"] = data["completion_run"]
data.loc[data["steps"] != 0, "opt_conv"] = None

data["opt_conv_mean"] = data.groupby("nodes")["opt_conv"].transform(lambda x: x.mean())
data["frac_coverage"] = data.groupby(["nodes", "steps"])["completion_run"].transform(lambda x: (1.0 - np.mean(x.isna())))

data["normalized"] = data["completion_run"] / data["opt_conv_mean"]

data["steps_per_node"] = data["steps"] / data["nodes"]
nlabel=r"\sc Nodes"
data[nlabel] = data["nodes"]

fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(6, 3), sharex=True, height_ratios=[3,2])

sns.lineplot(data=data[data.steps > 0], x="steps_per_node", y="normalized", hue=nlabel, style=nlabel, ax=ax1)
bottom = sns.lineplot(data=data[data.steps > 0], x="steps_per_node", y="frac_coverage", hue=nlabel, style=nlabel, ax=ax2)
bottom.legend_.remove()

plt.xlabel(r"\textsc{MCMC steps per node}")

ax1.set_ylabel(r"\begin{minipage}{10em}\begin{center}\sc Normalized  \\[-0.3em] runs until    \\[-0.3em] coverage \\\end{center}\end{minipage}")
ax2.set_ylabel(r"\begin{minipage}{10em}\begin{center}\sc Fraction of \\[-0.3em] runs reaching \\[-0.3em] coverage \\\end{center}\end{minipage}",
               labelpad=13)

ax1.set_xlim(0.25, 7)

plt.savefig(
    f"cycle_cover.pdf",
    format="pdf",
    bbox_inches="tight"
)

