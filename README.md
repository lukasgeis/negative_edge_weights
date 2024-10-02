# Random Negative Edge Weights

## Building the binary

Make sure that Rust is installed. The default way to acchieve this by running
the [one-liner from hell](https://www.rust-lang.org/tools/install).
You most certainly do not want to use your system's package manager to install Rust.

You also need the unstable nightly channel to run this application:
```bash
rustup toolchain install nightly
rustup default nightly
```

After that navigate to this directory and run:

```bash
cargo build --release
```
The binary then can be found under `target/release/random_negative_weights`.


## Using the tool

A help page can be accessed by running `random_negative_weights --help`.
At time of writing it looks as follows:

```text
USAGE:
    random_negative_weights [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -W <max-weight>             Minimum Weight [Default: 1]
    -w <min-weight>             Maximum Weight [Default: -1]
    -o <output>                 Optional Output Path
    -r <rounds-per-edge>        Carry out m * rounds_per_edge MCMC update steps [default: 1]
    -t <type>                   The primitive type of edge weights: can be any signed integer or float [default: f64]
    -s <seed>                   Optional starting seed for the RNG
    -i                          InitialWeights: Maximum (m), Uniform (u), zero (z) [Default: m]
    -a <algorithm>              Specify the algorithm used (BiDijktra: bd, Dijkstra: d, BellmanFord: bf) [Default: bd]
    --scc                       Extract the largest SCC and run the MCMC on it instead
    --check                     Run additional NegativeCycleDetector checks before and after the MCMC 


SUBCOMMANDS:
    gnp    
    dsf
    rhg
    complete
    cycle
    file
    help    Prints this message or the help of the given subcommand(s)
```

The last part of each command specified the data:

`random_negative_weights -s 1234 gnp -n 1000 -d 10` will produce a graph with 1000 nodes and an expected average
degree of 10 (i.e. we compute as `p=d/(n-1)`) using a fixed seed of `1234`. Since the seed is fixed, multiple runs
of the same binary should produce the same graph; drop `-s 1234` to seed from the system's entropy.

In front of the subcommand (`gnp` in the above example), you can specify the edge update routine. We will run a Markov Chain process known to converge to a uniform distribution over all legal edge
  weights between `-w` and `-W`. Though, nothing is known about the convergence time.

If you specify `-o test.gr`, the output will be written into the file `test.gr`; if the option is omitted, the graph is
dumped to stderr.

```bash
# Produce a graph of 100 nodes and average degree 4.
random_negative_weights -s 1234 -w=-10 gnp -n 100 -d 4

# Produce a graph of 100 nodes and average degree 4, run MCMC for 100*m steps and randomly assign weights in the interval [-3, 10] 
random_negative_weights -s 1234 -w=-3 -W 10 -r 100 gnp -n 100 -d 4
```

When specifying negative values please use `-w=-10` instead of `-w -10` to allow the command line argument parser to distinguish it from other input parameters.



## Experiments
All experiments and plots are implemented as features and can be accessed in the `exp` folder.
There are the following experiments:
- `acceptance`: Log the AcceptanceRate of the MCMC over time for different graph models
- `insertions`: Log the number of Queue-Insertions of different algorithms for different graph models
- `intervals`: Log average runtime, average weight and fraction of negative edges over time for different graph models
- `cycledist`: Log the weight distribution on the simply cycle over time

An experiment can be executed via
```bash
make run -C [NAME]
```
and plotted via
```bash
make plot -C [NAME]
```
If you want a small test sample (and plot) on your machine, use
```bash
make test -C [NAME]
```

All experiments can instead also be executed in succession via
```bash
bash exp/run.sh
```
or tested via
```bash
bash exp/test.sh 
```

Note that before plotting you need to setup a Python-Environment via
```bash
bash exp/pyprep.sh
```

### Warning
All experiments and plotting might take a few days depending on your machine (some but not all are parallelized)

#### Note
The code base will be restructured for the final version of the paper.
