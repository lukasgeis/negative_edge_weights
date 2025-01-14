import sys
import pandas as pd

print(r'\begin{table}[!htb]')
print(r'    \begin{center}')
print(r'        \begin{tabular}{c|c|c|c|c|c}')
print(r'            \multirow{2}{*}{Graph} & \multirow{2}{*}{\shortstack{Average \\ Degree}} & \multirow{2}{*}{\shortstack{Initial \\ Weights}} & \multirow{2}{*}{\shortstack{\algbp \\ time in $ms$}} & \multirow{2}{*}{\shortstack{\algbt \\ time in $ms$}} & \multirow{2}{*}{\shortstack{Average \\ Slowdown}} \\ & & & & & \\ \hline')


last_graph = None
last_deg = None
for name, group in pd.read_csv(sys.argv[1]).groupby(["graph","deg","initial"]):
    graph = name[0].upper()
    sgraph = fr'$\mathcal{{{graph}}}$'
    if last_graph == graph:
        sgraph = r''
    else:
        last_graph = graph

    deg = name[1]
    sdeg = fr'${deg}$'
    if last_deg == deg:
        sdeg = r''
    else:
        last_deg = deg

    init = name[2]

    smin = group[group.seq == True]["time"].min()
    smax = group[group.seq == True]["time"].max()
    pmin = group[group.seq == False]["time"].min()
    pmax = group[group.seq == False]["time"].max()

    speedup = group[group.seq == False]["time"].mean() / group[group.seq == True]["time"].mean()
    speedup = round(speedup, 3)

    print(fr'           {sgraph} & {sdeg} & \textsc{{{init}}} & ${smin} \sim {smax}$ & ${pmin} \sim {pmax}$ & ${speedup}$ \\')

print(r'        \end{tabular}')
print(r'    \end{center}')
print(r'')
print(r'    \caption{TODO}')
print(r'    \label{tab:TODO}')
print(r'\end{table}')
