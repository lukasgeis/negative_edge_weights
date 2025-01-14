use std::{
    convert::Infallible,
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind},
    path::PathBuf,
    str::FromStr,
};

use crate::weight::InitialWeights;

use super::*;

mod dsf;
mod gnp;
mod rhg;

pub use dsf::*;
pub use gnp::*;
use rand::Rng;
pub use rhg::*;
use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;

/// A base trait for all graph generators
pub trait GraphGenerator {
    fn generate(&mut self, rng: &mut impl Rng) -> Vec<(Node, Node)>;
}

/// Generator for complete graphs with/without self-loops
pub struct Complete {
    /// Number of nodes
    n: usize,
    /// Are self-loops allowed?
    loops: bool,
}

impl Complete {
    /// Creates the generator with given parameters
    #[inline]
    pub fn new(n: usize, loops: bool) -> Self {
        Self { n, loops }
    }
}

impl GraphGenerator for Complete {
    #[inline]
    fn generate(&mut self, _: &mut impl Rng) -> Vec<(Node, Node)> {
        (0..(self.n * self.n))
            .filter_map(|x| {
                let u = (x / self.n) as Node;
                let v = (x % self.n) as Node;

                if u != v || self.loops {
                    Some((u, v))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// A generator for a simple cycle
pub struct Cycle {
    /// Number of nodes
    n: usize,
}

impl Cycle {
    /// Creates the generator with given parameters
    #[inline]
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl GraphGenerator for Cycle {
    #[inline]
    fn generate(&mut self, _: &mut impl Rng) -> Vec<(Node, Node)> {
        (0..self.n)
            .map(|u| (u as Node, ((u + 1) % self.n) as Node))
            .collect()
    }
}

#[derive(StructOpt, Debug, Clone)]
pub enum Source {
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

impl<W: Weight> Graph<W> {
    pub fn from_source<R: Rng>(
        source: &Source,
        rng: &mut R,
        default_weight: InitialWeights,
        max_weight: W,
    ) -> Self {
        let (n, edges) = match *source {
            Source::Gnp { nodes, avg_deg } => {
                assert!(nodes > 1 && avg_deg > 0.0);
                let prob = avg_deg / (nodes as f64);
                (nodes, Gnp::new(nodes, prob).generate(rng))
            }
            Source::Dsf {
                nodes,
                alpha,
                beta,
                gamma,
                avg_deg,
                delta_out,
                delta_in,
            } => {
                let (alpha, beta) = compute_dsf_params(alpha, beta, gamma, avg_deg);

                (
                    nodes,
                    DirectedScaleFree::new(nodes, alpha, beta, delta_out, delta_in).generate(rng),
                )
            }
            Source::Rhg {
                nodes,
                alpha,
                radius,
                avg_deg,
                num_bands,
                prob,
            } => (
                nodes,
                Hyperbolic::new(nodes, alpha, radius, avg_deg, num_bands, prob).generate(rng),
            ),
            Source::Complete { nodes, loops } => (nodes, Complete::new(nodes, loops).generate(rng)),
            Source::Cycle { nodes } => (nodes, Cycle::new(nodes).generate(rng)),
            Source::File {
                ref path,
                undirected,
            } => {
                let file = File::open(path).expect("Could not open file!");
                let reader = BufReader::new(file);
                read_graph_from_file(reader, undirected).unwrap()
            }
        };

        Self::from_pos_edges(
            n,
            edges
                .into_iter()
                .map(|(u, v)| (u, v, default_weight.generate_weight(rng, max_weight)).into())
                .collect(),
        )
    }
}

// Returns an IO-Error with a custom error message.
#[inline]
fn io_error<O>(msg: &str) -> Result<O, Error> {
    Err(Error::new(ErrorKind::Other, msg))
}

/// Reads a graph from file
fn read_graph_from_file<R: BufRead>(
    reader: R,
    undirected: bool,
) -> Result<(usize, Vec<(Node, Node)>), Error> {
    let mut lines = reader.lines().filter_map(|x| -> Option<String> {
        if let Ok(line) = x {
            if !line.starts_with('%') {
                return Some(line);
            }
        }
        None
    });

    let (n, m) = parse_header(&mut lines)?;

    let cap = m * (undirected as usize + 1);
    let mut edges = Vec::with_capacity(cap);

    for (line, content) in lines.enumerate() {
        if line >= m {
            return io_error("Too many edges given");
        }

        let edge: Vec<_> = content.trim().split(',').collect();
        if edge.len() != 2 {
            return io_error(
                format!(
                    "Line {}: An edge should consist of exactly 2 nodes!",
                    line + 1
                )
                .as_str(),
            );
        }

        let u: Node = match edge[0].parse::<Node>() {
            Ok(u) => u,
            Err(_) => {
                return io_error(format!("Line {}: Cannot parse first node!", line + 1).as_str())
            }
        };

        let v: Node = match edge[1].parse::<Node>() {
            Ok(v) => v,
            Err(_) => {
                return io_error(format!("Line {}: Cannot parse second node!", line + 1).as_str())
            }
        };

        if u >= n as Node || v >= n as Node {
            return io_error(
                format!("Line {}: Node {u} in edge is bigger than n={n}!", line + 1).as_str(),
            );
        }

        edges.push((u, v));
        if undirected {
            edges.push((v, u));
        }
    }

    Ok((n, edges))
}

/// Parses the header of a graph file and returns (name, n, m) or an IO-Error.
#[inline]
fn parse_header<I: Iterator<Item = String>>(lines: &mut I) -> Result<(usize, usize), Error> {
    if let Some(header) = lines.next() {
        let fields: Vec<_> = header.split(' ').collect();
        if fields.len() < 3 {
            return io_error("Expected at least 3 header fields");
        }

        let n: usize = match fields[1].parse() {
            Ok(n) => n,
            Err(_) => return io_error("Cannot parse number of nodes"),
        };

        let m: usize = match fields[2].parse() {
            Ok(m) => m,
            Err(_) => return io_error("Cannot parse number of edges"),
        };

        Ok((n, m))
    } else {
        io_error("Cannot read header")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphType {
    Gnp = 0,
    Rhg = 1,
    Dsf = 2,
    Complete = 3,
    Cycle = 4,
    File = 5,
}

impl GraphType {
    pub const ALL: [GraphType; 6] = [
        GraphType::Gnp,
        GraphType::Rhg,
        GraphType::Dsf,
        GraphType::Complete,
        GraphType::Cycle,
        GraphType::File,
    ];

    #[inline]
    pub fn from_source(source: &Source) -> Self {
        match source {
            Source::Gnp { .. } => Self::Gnp,
            Source::Rhg { .. } => Self::Rhg,
            Source::Dsf { .. } => Self::Dsf,
            Source::Complete { .. } => Self::Complete,
            Source::Cycle { .. } => Self::Cycle,
            Source::File { .. } => Self::File,
        }
    }

    #[inline]
    pub fn to_string(&self) -> &str {
        match self {
            Self::Gnp => "gnp",
            Self::Rhg => "rhg",
            Self::Dsf => "dsf",
            Self::Complete => "complete",
            Self::Cycle => "cycle",
            Self::File => "file",
        }
    }
}

impl FromStr for GraphType {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.starts_with('g') {
            Self::Gnp
        } else if s.starts_with('r') {
            Self::Rhg
        } else if s.starts_with('d') {
            Self::Dsf
        } else if s.starts_with('f') {
            Self::File
        } else if s.contains('y') {
            Self::Cycle
        } else {
            Self::Complete
        })
    }
}
