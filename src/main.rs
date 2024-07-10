#![allow(unused)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![feature(float_next_up_down)]

use std::{convert::Infallible, num::ParseFloatError, path::PathBuf, str::FromStr};

use graph::Node;
use rand::Rng;
use structopt::StructOpt;

#[cfg(test)]
pub(crate) use graph::test_graph_data as test_data;
use weight::{Weight, WeightType};

#[cfg(not(feature = "exp"))]
use crate::mcmc::run;

#[cfg(feature = "exp")]
use crate::exp::run;

mod bidijkstra;
mod dijkstra;
#[cfg(feature = "exp")]
mod exp;
mod graph;
mod mcmc;
mod utils;
mod weight;

#[derive(StructOpt, Debug, Clone)]
struct Parameters {
    /// Source for the graph, i.e. which generator or from file
    #[structopt(subcommand)]
    source: Source,

    /// Minimum weight for an edge
    #[structopt(short = "w", default_value = "-1")]
    min_weight: f64,

    /// Maximum weight for an edge
    #[structopt(short = "W", default_value = "1")]
    max_weight: f64,

    /// Primitive Type used as Weight
    #[structopt(short = "t", default_value = "f64")]
    weight_type: WeightType,

    /// Carry out m * rounds_per_edge MCMC update steps
    #[structopt(short = "r", default_value = "1")]
    rounds_per_edge: f64,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Initial starting weights
    #[structopt(short = "i", default_value = "max")]
    initial_weights: InitialWeights,

    /// Optional output path for the resulting weighted graph
    #[structopt(short = "o")]
    output: Option<PathBuf>,

    /// Check if the generated graphs have negative weight cycles
    #[structopt(long)]
    check: bool,

    /// Cross-Reference decisions with a naive BF check
    #[cfg(feature = "exp")]
    #[structopt(long)]
    bftest: bool,

    /// Enable bidiretional search
    #[structopt(short = "a", long, default_value = "bd")]
    algorithm: Algorithm,

    /// Extract the largest SCC and run the MCMC on it
    #[structopt(long)]
    scc: bool,
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

        /// Average degree
        #[structopt(short = "d")]
        avg_deg: Option<f64>,

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
    File {
        /// Path to file
        #[structopt(short = "p", parse(from_os_str))]
        path: PathBuf,

        /// Are the edges in the graph file undirected?
        #[structopt(short = "u", long)]
        undirected: bool,
    },
}

#[cfg(feature = "exp")]
impl Source {
    /// Returns the average degree of a given source: `0.0` if not specified
    #[inline]
    pub fn degree(&self) -> f64 {
        match self {
            Self::Gnp { avg_deg, .. } => *avg_deg,
            Self::Dsf { avg_deg, .. } => avg_deg.unwrap_or(0.0),
            Self::Rhg { avg_deg, .. } => avg_deg.unwrap_or(0.0),
            Self::Cycle { .. } => 1.0,
            Self::Complete { nodes, loops } => *nodes as f64 - 1.0 + (*loops as usize) as f64,
            Self::File { .. } => 0.0,
        }
    }
}

/// Which algorithm to use for the MCMC
#[derive(Debug, Copy, Clone)]
pub enum Algorithm {
    /// The onedirectional dijkstra search
    Dijkstra,
    /// The bidirectional dijkstra search
    BiDijkstra,
    /// The naive version using bellman ford
    BellmanFord,
}

impl FromStr for Algorithm {
    // We should always use an algorithm - default to BiDijkstra
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('d') {
            Ok(Algorithm::Dijkstra)
        } else if s.contains('f') {
            Ok(Algorithm::BellmanFord)
        } else {
            Ok(Algorithm::BiDijkstra)
        }
    }
}

/// Starting weights for edges
#[derive(Debug, Copy, Clone)]
pub enum InitialWeights {
    /// Start with `Parameters::max_weight`
    Maximum,
    /// Start with `0`
    Zero,
    /// Start with a uniform weight in `[0,Parameters::max_weight]`
    Uniform,
    /// Start with a spefified weight: will be clamped to `[0,Parameters::max_weight]`
    Value(f64),
}

impl InitialWeights {
    /// Generate a weight based on `self` and `max_weight`: `rng` needs to be provided for the
    /// `Uniform` case
    #[inline]
    pub fn generate_weight<R: Rng, W: Weight>(&self, rng: &mut R, max_weight: W) -> W {
        match self {
            Self::Maximum => max_weight,
            Self::Zero => W::zero(),
            Self::Uniform => rng.gen_range(W::zero()..=max_weight),
            Self::Value(v) => W::from_f64(v.clamp(0.0, max_weight.to_f64())),
        }
    }

    /// Return a char representing the initial weight type: used for logging in experiments
    #[inline]
    pub fn to_char(&self) -> char {
        match self {
            Self::Maximum => 'm',
            Self::Zero => 'z',
            Self::Uniform => 'u',
            Self::Value(_) => 'v',
        }
    }
}

impl FromStr for InitialWeights {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('m') {
            Ok(Self::Maximum)
        } else if s.starts_with('z') {
            Ok(Self::Zero)
        } else if s.starts_with('u') {
            Ok(Self::Uniform)
        } else {
            s.parse::<f64>().map(Self::Value)
        }
    }
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
