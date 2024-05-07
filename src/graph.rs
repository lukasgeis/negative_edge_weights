use std::{fmt::Debug, io::Write};

use rand::Rng;
use rand_distr::Geometric;

use crate::weight::Weight;

pub type Node = usize;
pub type Edge<W> = (Node, Node, W);

#[derive(Clone)]
pub struct Graph<W: Weight> {
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the index of the first edge with souce `u` in `edges`
    limits: Vec<usize>,
    /// List of node potentials
    potentials: Vec<W>,

    #[cfg(feature = "bidir")]
    rev_edges: Vec<Edge<W>>,

    #[cfg(feature = "bidir")]
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

    #[cfg(feature = "bidir")]
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
    #[inline]
    pub fn update_weight(&mut self, idx: usize, weight: W) {
        self.edges[idx].2 = weight;

        #[cfg(feature = "bidir")]
        {
            let (u, v, _) = self.edges[idx];
            for i in self.rev_limits[v]..self.rev_limits[v + 1] {
                if self.rev_edges[i].0 == u {
                    self.rev_edges[i].2 = weight;
                    break;
                } 
            }
        }
    }

    /// Gets the weight of edge at index `idx`
    #[allow(unused)]
    #[inline]
    pub fn weight(&self, idx: usize) -> W {
        self.edges[idx].2
    }

    /// Creates a graph using an edge list and the number of nodes. Since we need `edges` to be
    /// sorted, we can specify whether it already is to skip another sort
    pub fn from_edge_list(n: usize, mut edges: Vec<Edge<W>>, sorted: bool) -> Self {
        assert!(edges.len() > 1);

        if !sorted {
            edges.sort_unstable_by(|(u1, v1, _), (u2, v2, _)| (u1, v1).cmp(&(u2, v2)));
        }

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

        #[cfg(feature = "bidir")]
        let (rev_edges, rev_limits) = {
            let mut rev_edges = edges.clone();
            edges.sort_unstable_by(|(u1, v1, _), (u2, v2, _)| (v1, u1).cmp(&(v2, u2)));

            let rev_limits: Vec<usize> = (0..n)
            .map(|i| {
                while curr_edge < edges.len() && edges[curr_edge].1 < i {
                    curr_edge += 1;
                }
                curr_edge
            })
            .chain(std::iter::once(edges.len()))
            .collect();

            (rev_edges, rev_limits)
        };

        // TODO: If graph with negative edges is provided, run BF to generate potentials instead
        Self {
            edges,
            limits,
            potentials: vec![W::zero(); n],
            #[cfg(feature = "bidir")]
            rev_edges,
            #[cfg(feature = "bidir")]
            rev_limits,
        }
    }

    /// Check whether the given potentials are feasible, i.e. the graph has no negative cycle
    #[allow(unused)]
    #[inline]
    pub fn is_feasible(&self) -> bool {
        self.edges
            .iter()
            .all(|e| self.potential_weight(*e) >= -W::ZERO_THRESHOLD)
    }

    /// Generate a GNP graph with specified default_weight for every edge
    pub fn gen_gnp(rng: &mut impl Rng, n: usize, p: f64, default_weight: W) -> Self {
        // TODO: flip probability for `p > 0.5` and generate complement graph
        assert!((0.0..=1.0).contains(&p));

        let mut edges = Vec::new();

        let geom_distr = Geometric::new(p).unwrap();
        let mut cur = 0u64;
        let end = (n * n) as u64;

        loop {
            let skip = rng.sample(geom_distr);
            cur = match (cur + 1).checked_add(skip) {
                Some(x) => x,
                None => break,
            };

            if cur > end {
                break;
            }

            let u = ((cur - 1) / n as u64) as Node;
            let v = ((cur - 1) % n as u64) as Node;

            edges.push((u, v, default_weight));
        }

        Self::from_edge_list(n, edges, true)
    }

    /// Generates a complete graph with `n` nodes and possible self loops
    #[inline]
    pub fn gen_complete(n: usize, self_loops: bool, default_weight: W) -> Self {
        let edges = (0..n)
            .flat_map(|u| {
                (0..n).filter_map(move |v| {
                    if self_loops || u != v {
                        Some((u, v, default_weight))
                    } else {
                        None
                    }
                })
            })
            .collect();
        Self::from_edge_list(n, edges, true)
    }

    /// Generates a cycle of `n` nodes
    #[inline]
    pub fn gen_cycle(n: usize, default_weight: W) -> Self {
        let edges = (0..n).map(|u| (u, (u + 1) % n, default_weight)).collect();
        Self::from_edge_list(n, edges, true)
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
