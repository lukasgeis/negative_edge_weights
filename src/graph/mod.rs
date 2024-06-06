use std::{fmt::Debug, io::Write};

use rand::Rng;

use crate::{weight::Weight, Source};

mod bellman_ford;
mod generators;
mod repr;

pub use bellman_ford::*;
pub use generators::*;
pub use repr::*;

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

pub trait Graph<W: Weight>: Sized {
    /// Gets the number of nodes
    fn n(&self) -> usize;

    /// Gets the number of edges
    fn m(&self) -> usize;

    /// Gets the potential of a node `u`
    fn potential(&self, u: Node) -> W;

    /// Gets the potential weight of an edge
    #[inline]
    fn potential_weight(&self, edge: Edge<W>) -> W {
        edge.weight + self.potential(edge.target) - self.potential(edge.source)
    }

    /// Adds `delta` to `potential[u]`
    fn update_potential(&mut self, u: Node, delta: W);

    /// Gets the edge at index `idx`
    fn edge(&self, idx: usize) -> Edge<W>;

    /// Updates the weight of the edge at index `idx` from `old_weight` to `new_weight`
    fn update_weight(&mut self, idx: usize, new_weight: W);

    /// Returns a slice over all outgoing edges of `u`
    fn out_neighbors(&self, u: Node) -> &[Edge<W>];

    /// Returns a slive over all incoming edges of `u`
    fn in_neighbors(&self, u: Node) -> &[Edge<W>];

    /// Returns a slice over all edges
    fn edges(&self) -> &[Edge<W>];

    /// Returns *true* if the graph is feasible, i.e. if all potential non-negative
    #[allow(unused)]
    #[inline]
    fn is_feasible(&self) -> bool {
        self.edges()
            .iter()
            .all(|e| self.potential_weight(*e) >= W::zero())
    }

    /// Returns the average weight in the graph
    #[inline]
    fn avg_weight(&self) -> f64 {
        self.edges().iter().map(|e| e.weight).sum::<W>().to_f64() / self.m() as f64
    }

    /// Returns the fraction of negative edges in the graph
    #[inline]
    fn frac_negative_edges(&self) -> f64 {
        self.edges().iter().filter(|e| e.weight < W::zero()).count() as f64 / self.m() as f64
    }

    /// Write the graph into an output
    #[inline]
    fn store_graph<WB: Write>(&self, writer: &mut WB) -> std::io::Result<()> {
        for edge in self.edges() {
            writeln!(writer, "{},{},{}", edge.source, edge.target, edge.weight)?
        }
        Ok(())
    }

    /// Creates the graph from a given number of nodes and a list of edges
    fn from_edges(n: usize, edges: Vec<Edge<W>>) -> Self;

    /// Creates the graph from a given `Source`
    #[inline]
    fn from_source(source: &Source, rng: &mut impl Rng, default_weight: W) -> Self {
        let (n, edges) = match *source {
            Source::Gnp { nodes, avg_deg } => {
                assert!(nodes > 1 && avg_deg > 0.0);
                let prob = avg_deg / (nodes as f64);
                (nodes, Gnp::new(nodes, prob).generate(rng, default_weight))
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
                    DirectedScaleFree::new(nodes, alpha, beta, delta_out, delta_in)
                        .generate(rng, default_weight),
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
                Hyperbolic::new(nodes, alpha, radius, avg_deg, num_bands, prob)
                    .generate(rng, default_weight),
            ),
            Source::Complete { nodes, loops } => (
                nodes,
                Complete::new(nodes, loops).generate(rng, default_weight),
            ),
            Source::Cycle { nodes } => (nodes, Cycle::new(nodes).generate(rng, default_weight)),
        };

        Self::from_edges(n, edges)
    }
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

impl_debug_graph!(OneDirGraph);
impl_debug_graph!(TwoDirGraph);

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
