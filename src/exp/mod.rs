//! # Experiments
//!
//! This module is a simple copy of the main code/version of the MCMC to allow for experiments
//! using feature flags without making the original code unreadable

#[cfg(feature = "intervals")]
use std::time::Instant;

use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;

use crate::{
    bidijkstra::Graph as Graph2, dijkstra::Graph as Graph1, graph::*, weight::Weight, Algorithm,
    Parameters,
};

use self::{bellman_ford::BellmanFord, bidijkstra::BiDijkstra, dijkstra::Dijkstra};

pub mod bidijkstra;
pub mod dijkstra;

pub trait ExpNegWeightMCMC<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_exp_mcmc<R: Rng>(&mut self, rng: &mut R, params: &Parameters);
}

impl<W> ExpNegWeightMCMC<W> for Graph1<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_exp_mcmc<R: Rng>(&mut self, rng: &mut R, params: &Parameters) {
        let num_rounds = (self.m() as f64 * params.rounds_per_edge).ceil() as u64;
        let mut dijkstra = Dijkstra::new(self.n());
        let weight_sampler = Uniform::new_inclusive(
            W::from_f64(params.min_weight),
            W::from_f64(params.max_weight),
        );
        let edge_sampler = Uniform::new(0usize, self.m());

        let mut bf_tester = BellmanFord::new(self.n());

        #[cfg(feature = "intervals")]
        let mut round = 0usize;

        #[cfg(feature = "intervals")]
        let mut timer = Instant::now();

        for _ in 0..num_rounds {
            #[cfg(feature = "intervals")]
            {
                round += 1;
                if round % 10000 == 0 {
                    println!(
                        "{},{},{},{},onedir",
                        round / 10000,
                        self.avg_weight(),
                        self.frac_negative_edges(),
                        timer.elapsed().as_millis()
                    );
                    timer = Instant::now();
                }
            }

            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            let potential_weight = self.potential_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                self.update_weight(idx, weight);

                if params.bftest {
                    assert!(
                        bf_tester.run(self, edge.target, edge.source, -weight),
                        "[FAIL] BF found a negative weight cycle when Dijkstra accepted directly"
                    );
                }
                continue;
            }

            if let Some(shortest_path_tree) =
                dijkstra.run(self, edge.target, edge.source, -potential_weight)
            {
                self.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    self.update_potential(node, -potential_weight - dist);
                }

                if params.bftest {
                    assert!(
                        bf_tester.run(self, edge.target, edge.source, -weight),
                        "[FAIL] BF found a negative weight cycle when Dijkstra accepted"
                    );
                }
            } else if params.bftest {
                assert!(
                    !bf_tester.run(self, edge.target, edge.source, -weight),
                    "[FAIL] BF found no negative weight cycle when Dijkstra rejected"
                );
            }
        }
    }
}

impl<W> ExpNegWeightMCMC<W> for Graph2<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_exp_mcmc<R: Rng>(&mut self, rng: &mut R, params: &Parameters) {
        let num_rounds = (self.m() as f64 * params.rounds_per_edge).ceil() as u64;
        let mut bidijkstra = BiDijkstra::new(self.n());
        let weight_sampler = Uniform::new_inclusive(
            W::from_f64(params.min_weight),
            W::from_f64(params.max_weight),
        );
        let edge_sampler = Uniform::new(0usize, self.m());

        let mut bf_tester = BellmanFord::new(self.n());

        #[cfg(feature = "intervals")]
        let mut round = 0usize;

        #[cfg(feature = "intervals")]
        let mut timer = Instant::now();

        for _ in 0..num_rounds {
            #[cfg(feature = "intervals")]
            {
                round += 1;
                if round % 10000 == 0 {
                    println!(
                        "{},{},{},{},twodir",
                        round / 10000,
                        self.avg_weight(),
                        self.frac_negative_edges(),
                        timer.elapsed().as_millis()
                    );
                    timer = Instant::now();
                }
            }

            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            let potential_weight = self.potential_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                self.update_weight(idx, weight);

                if params.bftest {
                    assert!(
                        bf_tester.run(self, edge.target, edge.source, -weight),
                        "[FAIL] BF found a negative weight cycle when BiDijkstra accepted directly"
                    );
                }
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

                if params.bftest {
                    assert!(
                        bf_tester.run(self, edge.target, edge.source, -weight),
                        "[FAIL] BF found a negative weight cycle when BiDijkstra accepted"
                    );
                }
            } else if params.bftest {
                assert!(
                    !bf_tester.run(self, edge.target, edge.source, -weight),
                    "[FAIL] BF found no negative weight cycle when BiDijkstra rejected"
                );
            }
        }
    }
}

#[inline]
pub fn run<W>(params: Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    match params.algorithm {
        Algorithm::Dijkstra => run_with_graph::<W, Graph1<W>>(params),
        Algorithm::BiDijkstra => run_with_graph::<W, Graph2<W>>(params),
        Algorithm::BellmanFord => (),
    };
}

#[inline]
fn run_with_graph<W, G>(params: Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
    G: GraphStats + GraphEdgeList<W> + GraphFromSource<W> + GraphNeigbors<W> + ExpNegWeightMCMC<W>,
{
    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let default_weight = W::from_f64(params.max_weight);
    let mut graph: G = G::from_source(&params.source, &mut rng, default_weight);

    graph.run_exp_mcmc(&mut rng, &params);
}
