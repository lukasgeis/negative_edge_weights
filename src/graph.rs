use std::io::Write;

use rand::Rng;
use rand_distr::Geometric;

pub type Node = usize;
pub type Weight = f64;
pub type Edge = (Node, Node, Weight);

#[derive(Debug, Clone)]
pub struct Graph {
    /// List of all edges sorted by source node
    edges: Vec<Edge>,
    /// `limits[u]` is the index of the first edge with souce `u` in `edges`
    limits: Vec<usize>,
    /// List of node potentials
    potentials: Vec<Weight>,
}

impl Graph {
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
    pub fn neighbors(&self, u: Node) -> &[Edge] {
        &self.edges[self.limits[u]..self.limits[u + 1]]
    }

    /// Returns a mutable reference of a node potential
    #[inline]
    pub fn potential_mut(&mut self, u: Node) -> &mut Weight {
        &mut self.potentials[u]
    }

    /// Returns the potential weight of an edge `(u,v,w)`, i.e. `w - p[u] + p[v]`
    #[inline]
    pub fn potential_weight(&self, edge: Edge) -> Weight {
        edge.2 + self.potentials[edge.1] - self.potentials[edge.0]
    }

    /// Returns a uniform random edge from the graph and its index in `edges`
    #[inline]
    pub fn random_edge(&self, rng: &mut impl Rng) -> (usize, Edge) {
        let idx = rng.gen_range(0..self.m());
        (idx, self.edges[idx])
    }

    /// Updates the weight of the edge at index `idx` in `edges`
    #[inline]
    pub fn update_weight(&mut self, idx: usize, weight: Weight) {
        self.edges[idx].2 = weight;
    }

    /// Creates a graph using an edge list and the number of nodes. Since we need `edges` to be
    /// sorted, we can specify whether it already is to skip another sort
    pub fn from_edge_list(n: usize, mut edges: Vec<Edge>, sorted: bool) -> Self {
        assert!(edges.len() > 1);

        if !sorted {
            edges.sort_unstable_by(|(u1, v1, _), (u2, v2, _)| (u1, v1).cmp(&(u2, v2)));
        }

        let mut curr_edge: usize = 0;
        let limits: Vec<usize> = (0..n)
            .map(|i| {
                while edges[curr_edge].0 < i {
                    curr_edge += 1;
                }
                curr_edge
            })
            .chain(std::iter::once(edges.len()))
            .collect();

        // TODO: If graph with negative edges is provided, run BF to generate potentials instead
        Self {
            edges,
            limits,
            potentials: vec![0 as Weight; n],
        }
    }

    /// Check whether the given potentials are feasible, i.e. the graph has no negative cycle
    #[allow(unused)]
    #[inline]
    pub fn is_feasible(&self) -> bool {
        self.edges
            .iter()
            .all(|e| self.potential_weight(*e) >= -1e-8)
    }

    /// Generate a GNP graph with specified default_weight for every edge
    pub fn gen_gnp(rng: &mut impl Rng, n: usize, p: f64, default_weight: Weight) -> Self {
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

    /// Returns the average weight in the graph
    #[inline]
    pub fn avg_weight(&self) -> Weight {
        self.edges.iter().map(|(_, _, w)| *w).sum::<Weight>() / self.m() as Weight
    }

    /// Returns the fraction of negative edges in the graph
    #[inline]
    pub fn frac_negative_edges(&self) -> f64 {
        self.edges.iter().filter(|(_, _, w)| *w < 0.0).count() as f64 / self.m() as f64
    }

    /// Write the graph into an output
    #[inline]
    pub fn store_graph<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for (u, v, w) in &self.edges {
            writeln!(writer, "{u},{v},{w}")?
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
    pub(crate) const EDGES: [Edge; 10] = [
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
    pub(crate) const GOOD_WEIGHTS: [[Weight; 10]; 3] = [
        [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 3.0, 1.0, 0.0, 3.0],
        [0.0; 10],
        [1.0; 10],
    ];

    /// Distance matrices for each `GOOD_WEIGHTS` graph
    pub(crate) const DISTANCES: [[[Weight; 5]; 5]; 3] = [
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
    pub(crate) const BAD_WEIGHTS: [[Weight; 10]; 2] = [
        [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 3.0, 1.0, 0.0, 2.0],
        [-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0],
    ];
}
