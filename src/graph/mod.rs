use std::io::Write;

use crate::weight::Weight;

pub mod generators;
pub mod neg_cycle;
pub mod tarjan;

/// Node of a Graph
pub type Node = usize;

/// A weighted directed edge
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge<W: Weight> {
    pub source: Node,
    pub target: Node,
    pub weight: W,
}

impl<W: Weight> Default for Edge<W> {
    fn default() -> Self {
        Self {
            source: 0,
            target: 0,
            weight: W::zero(),
        }
    }
}

unsafe impl<W: Weight> Send for Edge<W> {}
unsafe impl<W: Weight> Sync for Edge<W> {}

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

/// Graph representation for the MCMC supporting potentials and bidirectional edges
#[derive(Debug, Clone)]
pub struct Graph<W: Weight> {
    /// Edges sorted by source node
    pub edges: Vec<Edge<W>>,
    /// `limits[u]` is the first edge in `edges` with source node `u`
    limits: Vec<usize>,
    /// Edges sorted by target node
    rev_edges: Vec<Edge<W>>,
    /// `rev_limits[u]` is the first edge in `rev_edges` with target node `u`
    rev_limits: Vec<usize>,
    /// Potentials of all nodes
    potentials: Vec<W>,
}

impl<W: Weight> Graph<W> {
    /// Number of nodes
    #[inline]
    pub fn n(&self) -> usize {
        self.potentials.len()
    }

    /// Number of edges
    #[inline]
    pub fn m(&self) -> usize {
        self.edges.len()
    }

    /// Edge at index `idx`
    #[inline]
    pub fn edge(&self, idx: usize) -> Edge<W> {
        self.edges[idx]
    }

    /// Edge at index `idx` in reversed edges
    #[inline]
    pub fn rev_edge(&self, idx: usize) -> Edge<W> {
        self.rev_edges[idx]
    }

    /// Outgoging edges of node
    #[inline]
    pub fn out_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.edges[self.limits[u]..self.limits[u + 1]]
    }

    /// Incoming edges into node
    #[inline]
    pub fn in_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.rev_edges[self.rev_limits[u]..self.rev_limits[u + 1]]
    }

    /// Potential weight of an edge
    #[inline]
    pub fn pot_weight(&self, edge: Edge<W>) -> W {
        edge.weight + self.potentials[edge.target] - self.potentials[edge.source]
    }

    /// Update potential of a node
    #[inline]
    pub fn update_pot(&mut self, u: Node, delta: W) {
        self.potentials[u] += delta;
    }

    /// Update weight of an edge
    #[inline]
    pub fn update_weight(&mut self, idx: usize, weight: W) {
        let (u, v, w) = self.edges[idx].into();
        self.edges[idx].weight = weight;

        for i in self.rev_limits[v]..self.rev_limits[v + 1] {
            if self.rev_edges[i].source == u && self.rev_edges[i].weight == w {
                self.rev_edges[i].weight = weight;
                break;
            }
        }
    }

    /// Update weight of an edge with no known index
    #[inline]
    pub fn update_edge_weight(&mut self, u: Node, v: usize, weight: W) {
        for i in self.limits[u]..self.limits[u + 1] {
            if self.edges[i].target == v {
                self.edges[i].weight = weight;
                break;
            }
        }

        for i in self.rev_limits[v]..self.rev_limits[v + 1] {
            if self.rev_edges[i].source == u {
                self.rev_edges[i].weight = weight;
                break;
            }
        }
    }

    /// Total weight of edges
    #[inline]
    pub fn total_weight(&self) -> W {
        self.edges.iter().map(|e| e.weight).sum()
    }

    /// Average weight of edges
    #[inline]
    pub fn avg_weight(&self) -> f64 {
        self.total_weight().to_f64() / self.m() as f64
    }

    /// Average potential
    #[inline]
    pub fn avg_pot(&self) -> f64 {
        self.potentials.iter().copied().sum::<W>().to_f64() / self.n() as f64
    }

    /// Fraction of negative edges
    #[inline]
    pub fn frac_neg_edges(&self) -> f64 {
        self.edges.iter().filter(|e| e.weight < W::zero()).count() as f64 / self.m() as f64
    }

    /// Consume graph into edge vector
    #[inline]
    pub fn into_edges(self) -> Vec<Edge<W>> {
        self.edges
    }

    /// Create graph from vector of positive weighted edges
    pub fn from_pos_edges(n: usize, mut edges: Vec<Edge<W>>) -> Self {
        assert!(edges.len() > 1);

        edges.sort_unstable();

        let mut curr_edge: usize = 0;
        let limits: Vec<usize> = (0..n)
            .map(|i| {
                while curr_edge < edges.len() && edges[curr_edge].source < i {
                    curr_edge += 1;
                }
                curr_edge
            })
            .chain(std::iter::once(edges.len()))
            .collect();

        let (rev_edges, rev_limits) = {
            let mut rev_edges = edges.clone();
            rev_edges
                .sort_unstable_by(|e1, e2| (e1.target, e1.source).cmp(&(e2.target, e2.source)));

            curr_edge = 0;
            let rev_limits: Vec<usize> = (0..n)
                .map(|i| {
                    while curr_edge < rev_edges.len() && rev_edges[curr_edge].target < i {
                        curr_edge += 1;
                    }
                    curr_edge
                })
                .chain(std::iter::once(rev_edges.len()))
                .collect();

            (rev_edges, rev_limits)
        };

        Self {
            edges,
            limits,
            rev_edges,
            rev_limits,
            potentials: vec![W::zero(); n],
        }
    }

    /// Write the graph to an output
    #[inline]
    pub fn store<WB: Write>(&self, writer: &mut WB) -> std::io::Result<()> {
        writeln!(writer, "p {} {}", self.n(), self.m())?;
        for edge in &self.edges {
            writeln!(writer, "{},{},{}", edge.source, edge.target, edge.weight)?
        }
        Ok(())
    }
}
