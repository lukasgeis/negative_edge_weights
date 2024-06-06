#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![feature(float_next_up_down)]

use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use bidirectional::BiDijkstra;
use dijkstra::Dijkstra;
use graph::*;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;
use structopt::StructOpt;

#[cfg(test)]
pub(crate) use graph::test_graph_data as test_data;
use mcmc::*;
use weight::{Weight, WeightType};

mod bidirectional;
mod dijkstra;
mod graph;
mod mcmc;
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
        /// Number of nodes
        #[structopt(short = "n")]
        nodes: Node,

        /// Average degree
        #[structopt(short = "d")]
        avg_deg: f64,
    },
    Dsf {
        /// Number of nodes
        #[structopt(short = "n")]
        nodes: Node,

        /// Probability for adding a new node with an outgoing edge
        #[structopt(short = "a")]
        alpha: Option<f64>,

        /// Probability for adding a new edge between two existing nodes
        #[structopt(short = "b")]
        beta: Option<f64>,

        /// Probability for adding a new node with an incoming edge
        #[structopt(short = "g")]
        gamma: Option<f64>,

        /// Bias for choosing nodes from out-degree distrbution
        #[structopt(long = "do", default_value = "1")]
        delta_out: f64,

        /// Bias for choosing nodes from in-degree distrbution
        #[structopt(long = "di", default_value = "1")]
        delta_in: f64,
    },
    Rhg {
        /// Number of nodes
        #[structopt(short = "n")]
        nodes: Node,

        /// Radial dispersion
        #[structopt(short = "a", default_value = "1")]
        alpha: f64,

        /// Radius of hyperbolic disk
        #[structopt(short = "r")]
        radius: Option<f64>,

        /// Average degree
        #[structopt(short = "d")]
        avg_deg: Option<f64>,

        /// Number of bands
        #[structopt(short = "b")]
        num_bands: Option<usize>,

        /// Probability for including two directed edges instead of an undirected one
        #[structopt(short = "p", default_value = "1")]
        prob: f64,
    },
    Complete {
        /// Number of nodes
        #[structopt(short = "n")]
        nodes: Node,

        /// Are self-loops allowed?
        #[structopt(short = "l", long)]
        loops: bool,
    },
    Cycle {
        /// Number of nodes
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
        let mut rng = Pcg64::from_entropy();

        for (a, b) in [(-1.0, 1.0), (-2.0, 5.0), (-3.0, 10.0)] {
            let params = Parameters {
                source: Source::Gnp {
                    nodes: 100,
                    avg_deg: 5.0,
                },
                min_weight: a,
                max_weight: b,
                weight_type: WeightType::F64,
                rounds_per_edge: 5.0,
                seed: None,
                output: None,
                check: true,
                bftest: true,
                bidir: true,
            };

            let default_weight = i64::from_f64(params.max_weight);
            let mut graph: TwoDirGraph<i64> =
                TwoDirGraph::from_source(&params.source, &mut rng, default_weight);
            run_mcmc(&mut rng, &mut graph, &params);
            run_mcmc_bidirectional(&mut rng, &mut graph, &params);

            let default_weight = f64::from_f64(params.max_weight);
            let mut graph: TwoDirGraph<f64> =
                TwoDirGraph::from_source(&params.source, &mut rng, default_weight);
            run_mcmc(&mut rng, &mut graph, &params);
            run_mcmc_bidirectional(&mut rng, &mut graph, &params)
        }
    }
}
