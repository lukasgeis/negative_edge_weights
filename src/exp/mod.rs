//! # Experiments
//!
//! This module is a simple copy of the main code/version of the MCMC to allow for experiments
//! using feature flags without making the original code unreadable

use std::time::Instant;

use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;

use crate::{
    bidijkstra::Graph as Graph2,
    dijkstra::Graph as Graph1,
    exp::apsp::mean_max_paths,
    graph::{bellman_ford::Graph as Graph3, tarjan::num_sccs, *},
    weight::Weight,
    Algorithm, Parameters,
};

use self::{bellmanford::BellmanFord, bidijkstra::BiDijkstra, dijkstra::Dijkstra};

pub mod apsp;
pub mod bellmanford;
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
                        "{},{},{},{},d",
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
        let mut timer = Instant::now();

        #[cfg(feature = "acceptance")]
        let mut num_accepted_rounds = 0usize;

        #[cfg(feature = "acceptance")]
        let mut avg_accepted_rounds = 0.0f64;

        for i in 0..num_rounds {
            #[cfg(feature = "intervals")]
            {
                if (i + 1) % 10000 == 0 {
                    println!(
                        "{},{},{},{},bd",
                        (i + 1) / 10000,
                        self.avg_weight(),
                        self.frac_negative_edges(),
                        timer.elapsed().as_millis()
                    );
                    timer = Instant::now();
                }
            }
            #[cfg(feature = "apsp")]
            {
                if (i + 1) % self.m() as u64 == 0 {
                    let (mean, max) = mean_max_paths(self);
                    println!("{},{},{}", (i + 1) / self.m() as u64, mean, max);
                }
            }

            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            let potential_weight = self.potential_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                #[cfg(feature = "acceptance")]
                {
                    num_accepted_rounds += 1;
                }

                self.update_weight(idx, weight);

                if params.bftest {
                    assert!(
                        bf_tester.run(self, edge.target, edge.source, -weight),
                        "[FAIL] BF found a negative weight cycle when BiDijkstra accepted directly"
                    );
                }
            } else if let Some(((df, db), shortest_path_tree)) =
                bidijkstra.run(self, edge.target, edge.source, -potential_weight)
            {
                #[cfg(feature = "acceptance")]
                {
                    num_accepted_rounds += 1;
                }

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

            #[cfg(feature = "acceptance")]
            {
                avg_accepted_rounds += (num_accepted_rounds as f64 / (i + 1) as f64);
                if (i + 1) % 1000 == 0 {
                    println!(
                        "{},{},{},{}",
                        i + 1,
                        avg_accepted_rounds / 1000.0,
                        params.initial_weights.to_char(),
                        params.source.degree()
                    );
                    avg_accepted_rounds = 0.0;
                }
            }
        }
    }
}

impl<W> ExpNegWeightMCMC<W> for Graph3<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_exp_mcmc<R: Rng>(&mut self, rng: &mut R, params: &Parameters) {
        let num_rounds = (self.m() as f64 * params.rounds_per_edge).ceil() as u64;
        let mut bellman_ford = BellmanFord::new(self.n());
        let weight_sampler = Uniform::new_inclusive(
            W::from_f64(params.min_weight),
            W::from_f64(params.max_weight),
        );
        let edge_sampler = Uniform::new(0, self.m());

        for _ in 0..num_rounds {
            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            if weight >= edge.weight || bellman_ford.run(self, edge.target, edge.source, -weight) {
                self.update_weight(idx, weight);
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
    #[cfg(feature = "acceptance")]
    {
        run_with_graph::<W, Graph2<W>>(params);
        return;
    }

    #[cfg(feature = "cycle")]
    {
        run_cycle_exp::<W>(params);
        return;
    }

    match params.algorithm {
        Algorithm::Dijkstra => run_with_graph::<W, Graph1<W>>(params),
        Algorithm::BiDijkstra => run_with_graph::<W, Graph2<W>>(params),
        Algorithm::BellmanFord => run_with_graph::<W, Graph3<W>>(params),
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

    let max_weight = W::from_f64(params.max_weight);
    let mut graph: G = G::from_source(&params.source, &mut rng, params.initial_weights, max_weight);

    graph.run_exp_mcmc(&mut rng, &params);
}

#[cfg(feature = "cycle")]
pub fn run_cycle_exp<W: Weight>(params: Parameters) {
    use core::panic;

    use crate::Source;

    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let min_weight = W::from_f64(params.min_weight);
    let max_weight = W::from_f64(params.max_weight);

    let n = if let Source::Cycle { nodes } = params.source {
        nodes
    } else {
        panic!("The Cycle-Experiment needs the Cycle-Source!");
    };

    let edge_sampler = Uniform::new(0, n);
    let weight_sampler = Uniform::new_inclusive(min_weight, max_weight);

    let logging_points = [n / 2, n, 2 * n, 5 * n, 10 * n];

    let mut sum = W::zero();
    let mut weights: Vec<W> = (0..n)
        .map(|_| {
            let w = params.initial_weights.generate_weight(&mut rng, max_weight);
            sum += w;
            w
        })
        .collect();

    for i in 0..=*logging_points.last().unwrap() {
        let edge = edge_sampler.sample(&mut rng);
        let weight = weight_sampler.sample(&mut rng);

        let delta = weight - weights[edge];
        if sum + delta >= W::zero() {
            sum += delta;
            weights[edge] = weight;
        }

        if logging_points.contains(&i) {
            weights
                .iter()
                .for_each(|w| println!("{},{w},{}", (i as f64) / (n as f64), params.initial_weights.to_char()));
        }
    }
}
