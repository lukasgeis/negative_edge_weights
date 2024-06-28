use rand_distr::{Distribution, Uniform};

use crate::{graph::*, mcmc::NegWeightMCMC, weight::Weight};
use std::fmt::Debug;

use self::search::BiDijkstra;

pub mod search;

/// Graph representation for the bidirectional search
pub struct Graph<W: Weight> {
    /// Potentials of all nodes
    potentials: Vec<W>,
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the first edge in `edges` with source node `u`
    limits: Vec<usize>,
    /// List of all edges sorted by target node
    rev_edges: Vec<Edge<W>>,
    /// `rev_limits[u]` is the first edge in `rev_edges` with target node `u`
    rev_limits: Vec<usize>,
}

impl_debug_graph!(Graph);

impl<W: Weight> GraphStats for Graph<W> {
    #[inline]
    fn n(&self) -> usize {
        self.potentials.len()
    }

    #[inline]
    fn m(&self) -> usize {
        self.edges.len()
    }

    #[inline]
    fn avg_weight(&self) -> f64 {
        self.edges.iter().map(|e| e.weight).sum::<W>().to_f64() / self.m() as f64
    }

    #[inline]
    fn frac_negative_edges(&self) -> f64 {
        self.edges.iter().filter(|e| e.weight < W::zero()).count() as f64 / self.m() as f64
    }
}

impl<W: Weight> GraphNeigbors<W> for Graph<W> {
    fn out_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.edges[self.limits[u]..self.limits[u + 1]]
    }
}

impl<W: Weight> GraphEdgeList<W> for Graph<W> {
    fn from_edges(n: usize, mut edges: Vec<Edge<W>>) -> Self {
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
            potentials: vec![W::zero(); n],
            rev_edges,
            rev_limits,
        }
    }

    #[inline]
    fn into_edges(self) -> Vec<Edge<W>> {
        self.edges
    }
}

impl<W: Weight> Graph<W> {
    #[inline]
    pub fn in_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.rev_edges[self.rev_limits[u]..self.rev_limits[u + 1]]
    }

    #[inline]
    pub fn edge(&self, idx: usize) -> Edge<W> {
        self.edges[idx]
    }

    #[allow(unused)]
    #[inline]
    pub fn potential(&self, u: Node) -> W {
        self.potentials[u]
    }

    #[inline]
    pub fn potential_weight(&self, edge: Edge<W>) -> W {
        edge.weight + self.potentials[edge.target] - self.potentials[edge.source]
    }

    #[inline]
    pub fn update_potential(&mut self, u: Node, delta: W) {
        self.potentials[u] += delta;
    }

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
}

impl<W> NegWeightMCMC<W> for Graph<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_mcmc<R: rand::prelude::Rng, D: rand::prelude::Distribution<W>>(
        &mut self,
        rng: &mut R,
        weight_sampler: D,
        rounds_factor: f64,
    ) {
        let num_rounds = (self.m() as f64 * rounds_factor).ceil() as u64;
        let mut bidijkstra = BiDijkstra::new(self.n());
        let edge_sampler = Uniform::new(0usize, self.m());
        for _ in 0..num_rounds {
            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            let potential_weight = self.potential_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                self.update_weight(idx, weight);
                continue;
            }

            if let Some(((df, db), shortest_path_tree)) =
                bidijkstra.run(self, edge.target, edge.source, -potential_weight)
            {
                self.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    if node < self.n() {
                        self.update_potential(node, df - dist);
                    } else {
                        self.update_potential(node - self.n(), dist - db);
                    }
                }
            }
        }
    }
}
