#![allow(unused)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

#[cfg(all(feature = "bidir", feature = "dfs_size"))]
compile_error!("Features `bidir` and `hops`, `dfs_size` are mutually exclusive!");

use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use bidirectional::BiDijkstra;
use dijkstra::Dijkstra;
use graph::*;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;
use structopt::StructOpt;

use bellman_ford::has_negative_cycle;

#[cfg(test)]
pub(crate) use graph::test_graph_data as test_data;
use mcmc::*;
use weight::{Weight, WeightType};

mod bellman_ford;
mod bidirectional;
mod dijkstra;
mod graph;
mod mcmc;
mod radixheap;
mod utils;
mod weight;

#[derive(StructOpt, Debug, Clone)]
struct Parameters {
    #[structopt(subcommand)]
    source: Source,

    #[structopt(short = "w", default_value = "-1")]
    min_weight: f64,

    #[structopt(short = "W", default_value = "1")]
    max_weight: f64,

    #[structopt(short = "t", default_value = "f64")]
    weight_type: WeightType,

    /// Carry out m * rounds_per_edge MCMC update steps
    #[structopt(short = "r", default_value = "1")]
    rounds_per_edge: f64,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Optional output path for the resulting weighted graph
    #[structopt(short = "o")]
    output: Option<PathBuf>,

    /// Check if the generated graphs have negative weight cycles
    #[structopt(long)]
    check: bool,

    /// Cross-Reference decisions with a naive BF check
    #[structopt(long)]
    bftest: bool,

    /// Enable bidiretional search
    #[structopt(long)]
    bidir: bool,
}

#[derive(StructOpt, Debug, Clone)]
enum Source {
    Gnp {
        #[structopt(short = "n")]
        nodes: Node,

        #[structopt(short = "d")]
        avg_deg: f64,
    },
    Complete {
        #[structopt(short = "n")]
        nodes: Node,

        #[structopt(short = "l", long)]
        loops: bool,
    },
    Cycle {
        #[structopt(short = "n")]
        nodes: Node,
    },
}

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);
    assert!(params.rounds_per_edge > 0.0);

    match params.weight_type {
        WeightType::F32 => run::<f32>(params),
        WeightType::F64 => run::<f64>(params),
        WeightType::I8 => run::<i8>(params),
        WeightType::I16 => run::<i16>(params),
        WeightType::I32 => run::<i32>(params),
        WeightType::I64 => run::<i64>(params),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dijkstra_vs_bf() {
        for (a, b) in [(-1.0, 1.0), (-2.0, 5.0), (-3.0, 10.0)] {
            for bidir in [true, false] {
                let params = Parameters {
                    source: Source::Gnp {
                        nodes: 100,
                        avg_deg: 5.0,
                    },
                    min_weight: a,
                    max_weight: b,
                    weight_type: WeightType::F64,
                    rounds_per_edge: 5.0,
                    seed: Some(1234),
                    output: None,
                    check: true,
                    bftest: true,
                    bidir,
                };

                run::<i64>(params.clone());
                run::<f64>(params);
            }
        }
    }
}
