# Random Negative Edge Weights

### ESA Details

Experiments and plotting tools used for the submission to ESA25 are found in `exp/esa`.
Other experiments in this repository can be disregarded for this submission.

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
cargo build --release --bin gen_unif_weights
```
The binary then can be found under `target/release/gen_unif_weights`.


## Using the tool

A help page can be accessed by running `gen_unif_weights --help`.
At time of writing it looks as follows:

```text
USAGE:
    gen_unif_weights [OPTIONS] <SUBCOMMAND>

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
    --scc                       Extract the largest SCC and run the MCMC on it instead


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

`gen_unif_weights -s 1234 gnp -n 1000 -d 10` will produce a graph with 1000 nodes and an expected average
degree of 10 (i.e. we compute as `p=d/(n-1)`) using a fixed seed of `1234`. Since the seed is fixed, multiple runs
of the same binary should produce the same graph; drop `-s 1234` to seed from the system's entropy.

In front of the subcommand (`gnp` in the above example), you can specify the edge update routine. We will run a Markov Chain process known to converge to a uniform distribution over all legal edge
  weights between `-w` and `-W`. Though, nothing is known about the convergence time.

If you specify `-o test.gr`, the output will be written into the file `test.gr`; if the option is omitted, the graph is
dumped to stderr.

```bash
# Produce a graph of 100 nodes and average degree 4.
gen_unif_weights -s 1234 -w=-10 gnp -n 100 -d 4

# Produce a graph of 100 nodes and average degree 4, run MCMC for 100*m steps and randomly assign weights in the interval [-3, 10] 
gen_unif_weights -s 1234 -w=-3 -W 10 -r 100 gnp -n 100 -d 4
```

When specifying negative values please use `-w=-10` instead of `-w -10` to allow the command line argument parser to distinguish it from other input parameters.



## Experiments
All experiments are available as a standalone binary in the `src/bin` folder and can be executed with
```bash
cargo run --release --bin [EXP-BINARY] -- [EXP-ARGS]
```

Execution of specific settings and experiments conducted can be found in `exp/run`.
This also serves as an reproducibility feature.

Plotting can be done via `python` and the scripts found in `exp/plot`.
