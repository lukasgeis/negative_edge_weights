use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Write},
};

use rand::Rng;

use crate::{weight::Weight, InitialWeights, Source};

pub mod bellman_ford;
mod generators;
pub mod tarjan;

pub use generators::*;

/// Node of a graph
pub type Node = usize;

/// A weighted directed edge consists of a `source`, `target`, and `weight`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge<W: Weight> {
    pub source: Node,
    pub target: Node,
    pub weight: W,
}

impl<W: Weight> Eq for Edge<W> {}

impl<W: Weight> PartialOrd for Edge<W> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<W: Weight> Ord for Edge<W> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.source, self.target).cmp(&(other.source, other.target))
    }
}

impl<W: Weight> From<(Node, Node, W)> for Edge<W> {
    #[inline]
    fn from(value: (Node, Node, W)) -> Self {
        Self {
            source: value.0,
            target: value.1,
            weight: value.2,
        }
    }
}

impl<W: Weight> From<Edge<W>> for (Node, Node, W) {
    #[inline]
    fn from(value: Edge<W>) -> Self {
        (value.source, value.target, value.weight)
    }
}

pub trait GraphEdgeList<W: Weight> {
    fn from_edges(n: usize, edges: Vec<Edge<W>>) -> Self;

    fn into_edges(self) -> Vec<Edge<W>>;
}

pub trait GraphFromSource<W: Weight> {
    fn from_source<R: Rng>(
        source: &Source,
        rng: &mut R,
        default_weight: InitialWeights,
        max_weight: W,
    ) -> Self;
}

impl<W: Weight, G: GraphEdgeList<W>> GraphFromSource<W> for G {
    fn from_source<R: Rng>(
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

        Self::from_edges(
            n,
            edges
                .into_iter()
                .map(|(u, v)| (u, v, default_weight.generate_weight(rng, max_weight)).into())
                .collect(),
        )
    }
}

/// Write the graph into an output
#[inline]
pub fn store_graph<W: Weight, G: GraphEdgeList<W>, WB: Write>(
    graph: G,
    writer: &mut WB,
) -> std::io::Result<()> {
    for edge in graph.into_edges() {
        writeln!(writer, "{},{},{}", edge.source, edge.target, edge.weight)?
    }
    Ok(())
}

pub trait GraphStats {
    fn n(&self) -> usize;

    fn m(&self) -> usize;

    fn avg_weight(&self) -> f64;

    fn frac_negative_edges(&self) -> f64;
}

pub trait GraphNeigbors<W: Weight> {
    fn out_neighbors(&self, u: Node) -> &[Edge<W>];
}

macro_rules! impl_debug_graph {
    ($id:ident) => {
        impl<W: Weight> Debug for $id<W> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "\n<== Graph with {} nodes and {} edges ==>\n\nEdge = (source, target, weight, potential weight)", self.n(), self.m())?;
                for u in 0..self.n() {
                    write!(f, "Outgoing edges from {u} => ")?;
                    for edge in self.out_neighbors(u) {
                        write!(
                            f,
                            "  ({u}, {}, {}, {})",
                            edge.target,
                            edge.weight,
                            self.potential_weight(*edge)
                        )?;
                    }
                    writeln!(f)?;
                }
                Ok(())
            }
        }
    };
}

pub(crate) use impl_debug_graph;

/// Returns an IO-Error with a custom error message.
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

        let edge: Vec<_> = content.trim().split(' ').collect();
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
            Ok(u) => u - 1,
            Err(_) => {
                return io_error(format!("Line {}: Cannot parse first node!", line + 1).as_str())
            }
        };

        let v: Node = match edge[1].parse::<Node>() {
            Ok(v) => v - 1,
            Err(_) => {
                return io_error(format!("Line {}: Cannot parse second node!", line + 1).as_str())
            }
        };

        if u >= n as Node || v >= n as Node {
            return io_error(format!("Line {}: Node in edge is bigger than n!", line + 1).as_str());
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

#[cfg(test)]
pub(crate) mod test_graph_data {
    use super::*;

    /// A graph with `5` nodes and `10` edges
    ///
    /// Image: `https://dreampuf.github.io/GraphvizOnline/#digraph%20G%20%7Bv0%20-%3E%20%7Bv1%2C%20v2%7D%3Bv1%20-%3E%20%7Bv3%2C%20v4%7D%3Bv2%20-%3E%20%7Bv1%2C%20v3%7D%3Bv3%20-%3E%20%7Bv0%2C%20v1%2C%20v4%7D%3Bv4%20-%3E%20%7Bv0%7D%3B%7D`
    pub(crate) const EDGES: [(Node, Node, f64); 10] = [
        (0, 1, 1.0),
        (0, 2, 1.0),
        (1, 3, 1.0),
        (1, 4, 1.0),
        (2, 1, 1.0),
        (2, 3, 1.0),
        (3, 0, 1.0),
        (3, 1, 1.0),
        (3, 4, 1.0),
        (4, 0, 1.0),
    ];

    /// Weights for `EDGES` that **do not** introduce a negative cycle
    pub(crate) const GOOD_WEIGHTS: [[f64; 10]; 3] = [
        [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 3.0, 1.0, 0.0, 3.0],
        [0.0; 10],
        [1.0; 10],
    ];

    /// Distance matrices for each `GOOD_WEIGHTS` graph
    pub(crate) const DISTANCES: [[[f64; 5]; 5]; 3] = [
        [
            [0.0, -2.0, -1.0, -3.0, -3.0],
            [2.0, 0.0, 1.0, -1.0, -1.0],
            [1.0, -1.0, 0.0, -2.0, -2.0],
            [3.0, 1.0, 2.0, 0.0, 0.0],
            [3.0, 1.0, 2.0, 0.0, 0.0],
        ],
        [[0.0; 5]; 5],
        [
            [0.0, 1.0, 1.0, 2.0, 2.0],
            [2.0, 0.0, 3.0, 1.0, 1.0],
            [2.0, 1.0, 0.0, 1.0, 2.0],
            [1.0, 1.0, 2.0, 0.0, 1.0],
            [1.0, 2.0, 2.0, 3.0, 0.0],
        ],
    ];

    /// Weights for `EDGES` that **do** introduce a negative cycle
    pub(crate) const BAD_WEIGHTS: [[f64; 10]; 2] = [
        [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 3.0, 1.0, 0.0, 2.0],
        [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0],
    ];
}
