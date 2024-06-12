use rand::Rng;
use rand_distr::{Distribution, Uniform};
use std::fmt::Debug;

use crate::{graph::*, mcmc::NegWeightMCMC, weight::Weight};

use self::search::Dijkstra;

pub mod search;

/// Graph representation for the normal dijkstra search
pub struct Graph<W: Weight> {
    /// Potentials of each node
    potentials: Vec<W>,
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the first edge in `edges` with source node `u`
    limits: Vec<usize>,
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

        Self {
            edges,
            limits,
            potentials: vec![W::zero(); n],
        }
    }

    #[inline]
    fn into_edges(self) -> Vec<Edge<W>> {
        self.edges
    }
}

impl<W: Weight> Graph<W> {
    #[inline]
    pub fn edge(&self, idx: usize) -> Edge<W> {
        self.edges[idx]
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
        self.edges[idx].weight = weight;
    }
}

impl<W> NegWeightMCMC<W> for Graph<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_mcmc<R: Rng, D: Distribution<W>>(
        &mut self,
        rng: &mut R,
        weight_sampler: D,
        rounds_factor: f64,
    ) {
        let num_rounds = (self.m() as f64 * rounds_factor).ceil() as u64;
        let mut dijkstra = Dijkstra::new(self.n());
        let edge_sampler = Uniform::new(0, self.m());

        for _ in 0..num_rounds {
            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            let potential_weight = self.potential_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                self.update_weight(idx, weight);
                continue;
            }

            if let Some(shortest_path_tree) =
                dijkstra.run(self, edge.target, edge.source, -potential_weight)
            {
                self.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    self.update_potential(node, -potential_weight - dist);
                }
            }
        }
    }
}
