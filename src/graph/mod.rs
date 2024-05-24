use std::{fmt::Debug, io::Write};

use rand::Rng;

use crate::{weight::Weight, Source};

mod bellman_ford;
mod generators;

pub use bellman_ford::*;
pub use generators::*;

pub type Node = usize;
pub type Edge<W> = (Node, Node, W);

#[derive(Clone)]
pub struct Graph<W: Weight> {
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the index of the first edge with source `u` in `edges`
    limits: Vec<usize>,
    /// List of node potentials
    potentials: Vec<W>,
    /// List of all edges sorted by target node
    rev_edges: Vec<Edge<W>>,
    /// `rev_limits[u]` is the index of the first edge with target `u` in `rev_edges`
    rev_limits: Vec<usize>,
}

impl<W: Weight> Graph<W> {
    /// Get the number of nodes
    #[inline]
    pub fn n(&self) -> usize {
        self.limits.len() - 1
    }

    /// Get the number of edges
    #[inline]
    pub fn m(&self) -> usize {
        self.edges.len()
    }

    /// Returns a slice over all outgoing edges from source node `u`
    #[inline]
    pub fn neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.edges[self.limits[u]..self.limits[u + 1]]
    }

    /// Returns a slice over all incoming edges to target node `u`
    #[inline]
    pub fn in_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.rev_edges[self.rev_limits[u]..self.rev_limits[u + 1]]
    }

    /// Returns a mutable reference of a node potential
    #[inline]
    pub fn potential_mut(&mut self, u: Node) -> &mut W {
        &mut self.potentials[u]
    }

    /// Returns the potential weight of an edge `(u,v,w)`, i.e. `w - p[u] + p[v]`
    #[inline]
    pub fn potential_weight(&self, edge: Edge<W>) -> W {
        edge.2 + self.potentials[edge.1] - self.potentials[edge.0]
    }

    /// Returns a uniform random edge from the graph and its index in `edges`
    #[inline]
    pub fn random_edge(&self, rng: &mut impl Rng) -> (usize, Edge<W>) {
        let idx = rng.gen_range(0..self.m());
        (idx, self.edges[idx])
    }

    /// Updates the weight of the edge at index `idx` in `edges`
    ///
    /// TODO: find better update method
    #[inline]
    pub fn update_weight(&mut self, idx: usize, old_weight: W, weight: W) {
        self.edges[idx].2 = weight;

        let (u, v, _) = self.edges[idx];
        for i in self.rev_limits[v]..self.rev_limits[v + 1] {
            if self.rev_edges[i].0 == u && self.rev_edges[i].2 == old_weight {
                self.rev_edges[i].2 = weight;
                break;
            }
        }
    }

    /// Gets the weight of edge at index `idx`
    #[inline]
    pub fn weight(&self, idx: usize) -> W {
        self.edges[idx].2
    }

    /// Creates a graph using an edge list and the number of nodes. Since we need `edges` to be
    /// sorted, we can specify whether it already is to skip another sort
    pub fn from_edge_list(n: usize, mut edges: Vec<Edge<W>>) -> Self {
        assert!(edges.len() > 1);

        edges.sort_unstable_by(|(u1, v1, _), (u2, v2, _)| (u1, v1).cmp(&(u2, v2)));

        let mut curr_edge: usize = 0;
        let limits: Vec<usize> = (0..n)
            .map(|i| {
                while curr_edge < edges.len() && edges[curr_edge].0 < i {
                    curr_edge += 1;
                }
                curr_edge
            })
            .chain(std::iter::once(edges.len()))
            .collect();

        let (rev_edges, rev_limits) = {
            let mut rev_edges = edges.clone();
            rev_edges.sort_unstable_by(|(u1, v1, _), (u2, v2, _)| (v1, u1).cmp(&(v2, u2)));

            curr_edge = 0;
            let rev_limits: Vec<usize> = (0..n)
                .map(|i| {
                    while curr_edge < rev_edges.len() && rev_edges[curr_edge].1 < i {
                        curr_edge += 1;
                    }
                    curr_edge
                })
                .chain(std::iter::once(rev_edges.len()))
                .collect();

            (rev_edges, rev_limits)
        };

        // TODO: If graph with negative edges is provided, run BF to generate potentials instead
        Self {
            edges,
            limits,
            potentials: vec![W::zero(); n],
            rev_edges,
            rev_limits,
        }
    }

    /// Check whether the given potentials are feasible, i.e. the graph has no negative cycle#
    ///
    /// Note that this method is subsceptible to floating-point-errors
    #[inline]
    pub fn is_feasible(&self) -> bool {
        self.edges
            .iter()
            .all(|e| self.potential_weight(*e) >= W::zero())
    }

    /// Creates the graph according to the specified source
    #[inline]
    pub fn from_source(source: &Source, rng: &mut impl Rng, default_weight: W) -> Self {
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
                delta_out,
                delta_in,
            } => {
                let (alpha, beta) = if let Some(a) = alpha {
                    if let Some(b) = beta {
                        (a, b)
                    } else if let Some(g) = gamma {
                        (a, 1.0 - a - g)
                    } else {
                        (a, (1.0 - a) / 2.0)
                    }
                } else if let Some(b) = beta {
                    if let Some(g) = gamma {
                        (1.0 - b - g, b)
                    } else {
                        ((1.0 - b) / 2.0, b)
                    }
                } else if let Some(g) = gamma {
                    let t = (1.0 - g) / 2.0;
                    (t, t)
                } else {
                    (1.0 / 3.0, 1.0 / 3.0)
                };

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

        Graph::from_edge_list(n, edges)
    }

    /// Returns the average weight in the graph
    #[inline]
    pub fn avg_weight(&self) -> f64 {
        self.edges.iter().map(|(_, _, w)| *w).sum::<W>().to_f64() / self.m() as f64
    }

    /// Returns the fraction of negative edges in the graph
    #[inline]
    pub fn frac_negative_edges(&self) -> f64 {
        self.edges.iter().filter(|(_, _, w)| *w < W::zero()).count() as f64 / self.m() as f64
    }

    /// Write the graph into an output
    #[inline]
    pub fn store_graph<WB: Write>(&self, writer: &mut WB) -> std::io::Result<()> {
        for (u, v, w) in &self.edges {
            writeln!(writer, "{u},{v},{w}")?
        }
        Ok(())
    }
}

impl<W: Weight> Debug for Graph<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n<== Graph with {} nodes and {} edges ==>\n\nEdge = (source, target, weight, potential weight)", self.n(), self.m())?;
        for u in 0..self.n() {
            write!(f, "Outgoing edges from {u} => ")?;
            for (_, v, w) in self.neighbors(u) {
                write!(
                    f,
                    "  ({u}, {v}, {w}, {})",
                    self.potential_weight((u, *v, *w))
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test_graph_data {
    use super::*;

    /// A graph with `5` nodes and `10` edges
    ///
    /// Image: `https://dreampuf.github.io/GraphvizOnline/#digraph%20G%20%7Bv0%20-%3E%20%7Bv1%2C%20v2%7D%3Bv1%20-%3E%20%7Bv3%2C%20v4%7D%3Bv2%20-%3E%20%7Bv1%2C%20v3%7D%3Bv3%20-%3E%20%7Bv0%2C%20v1%2C%20v4%7D%3Bv4%20-%3E%20%7Bv0%7D%3B%7D`
    pub(crate) const EDGES: [Edge<f64>; 10] = [
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
