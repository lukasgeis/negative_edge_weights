#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![allow(clippy::too_many_arguments)]

use fxhash::FxHashMap;
use negative_edge_weights::{
    graph::{
        generators::{GraphType, Source},
        tarjan::extract_largest_scc,
        Graph,
    },
    logger::EmptyLogger,
    pot::{checks::is_feasible, johnson::Johnson, Potentials},
    search::bidijkstra::BiDijkstra,
    weight::InitialWeights,
};
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use smallvec::{smallvec, SmallVec};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
struct Parameters {
    /// Source for the graph, i.e. which generator or from file
    #[structopt(subcommand)]
    source: Source,

    #[structopt(short = "n", default_value = "2")]
    num_chains: usize,

    /// Number of independent MCMC rounds to mix up the edges;
    /// if not provided, rejection sampling (very slow) is used
    #[structopt(short = "p")]
    pre_rounds: Option<usize>,

    /// Maximum number of coupling rounds
    #[structopt(short = "r", default_value = "1")]
    max_rounds: usize,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Extract the largest SCC and run the MCMC on it
    #[structopt(long)]
    scc: bool,

    /// If `scc`, round `degree` to a multiple of `mult` for output
    #[structopt(long)]
    mult: Option<usize>,

    #[structopt(short = "i", long, default_value = "1")]
    iterations: usize,
}

use negative_edge_weights::pot::Graph as NegGraph;

fn main() {
    let params = Parameters::from_args();

    (1..=params.iterations).into_par_iter().for_each(|iter| {
        let rng = if let Some(seed) = params.seed {
            &mut Pcg64Mcg::seed_from_u64(seed ^ iter as u64)
        } else {
            &mut Pcg64Mcg::from_entropy()
        };

        let base_graph: Graph<i32> = {
            let graph = Graph::from_source(&params.source, rng, InitialWeights::Uniform, 1);

            if params.scc {
                extract_largest_scc(graph)
            } else {
                graph
            }
        };

        let edge_sampler = Uniform::new(0, base_graph.m());
        let weight_sampler = Uniform::new_inclusive(-1, 1);

        let degree = if params.scc {
            let mut degree = (base_graph.m() as f64 / base_graph.n() as f64).round();
            if let Some(m) = params.mult {
                assert!(m > 0);
                let d = degree as usize;
                if !d.is_multiple_of(m) {
                    let next_smallest_multiple = (d / m) * m;
                    let next_biggest_multiple = next_smallest_multiple + m;

                    if d - next_smallest_multiple >= next_biggest_multiple - d {
                        degree = next_biggest_multiple as f64;
                    } else {
                        degree = next_smallest_multiple as f64;
                    }
                }
            }
            degree
        } else {
            params.source.degree()
        };

        let mut graphs: Vec<Graph<i32>> = if params.pre_rounds.is_some() {
            (0..params.num_chains)
                .map(|_| {
                    Graph::from_pos_edges(
                        base_graph.n(),
                        base_graph
                            .edges()
                            .iter()
                            .copied()
                            .map(|mut e| {
                                e.weight = InitialWeights::Uniform.generate_weight(rng, 1);
                                e
                            })
                            .collect(),
                    )
                })
                .collect()
        } else {
            (0..params.num_chains)
                .into_par_iter()
                .map(|piter| {
                    let mut graph = NegGraph::from_edges(
                        base_graph.n(),
                        base_graph
                            .edges()
                            .iter()
                            .map(|e| (e.source, e.target, e.weight))
                            .collect(),
                    );

                    let rng = &mut Pcg64Mcg::seed_from_u64((iter as u64) ^ ((piter as u64) << 10));

                    let mut i = 0;
                    loop {
                        i += 1;
                        graph.update_weights(|| -> i32 { weight_sampler.sample(rng) });

                        if let Ok(pot) = Johnson::compute_pot(&graph) {
                            assert!(is_feasible(&graph, &pot, true));
                            eprintln!("Sampled uniform graph in round {i}");
                            unsafe {
                                return Graph::from_neg_graph(graph, pot);
                            }
                        }
                    }
                })
                .collect()
        };

        let mut hashes: Vec<u64> = (0..params.num_chains)
            .into_par_iter()
            .map(|i| {
                let mut hash = 0u64;
                for j in 0..graphs[i].m() {
                    hash_mcmc(&mut hash, j, 0, graphs[i].edge(j).weight);
                }
                hash
            })
            .collect();

        let mut bds: Vec<BiDijkstra<i32>> = (0..params.num_chains)
            .map(|_| BiDijkstra::new(base_graph.n()))
            .collect();

        // Pre-Rounds
        if let Some(pre_rounds) = params.pre_rounds {
            graphs
                .par_iter_mut()
                .zip(bds.par_iter_mut())
                .zip(hashes.par_iter_mut())
                .enumerate()
                .for_each(|(idx, ((graph, bd), hash))| {
                    let rng = &mut Pcg64Mcg::seed_from_u64(((iter as u64) << 20) ^ (idx as u64));

                    for _ in 0..pre_rounds {
                        let edge_idx = edge_sampler.sample(rng);
                        let weight = weight_sampler.sample(rng);

                        let (prev, curr) = mcmc_step(graph, edge_idx, weight, bd);
                        hash_mcmc(hash, edge_idx, prev, curr);
                    }
                });
        }

        let mut hash_collisions: FxHashMap<u64, SmallVec<[usize; 4]>> = Default::default();
        let mut collisions = Vec::with_capacity(params.num_chains - 1);

        let mut collided = Vec::with_capacity(params.num_chains);
        for step in 1..=params.max_rounds {
            let edge_idx = edge_sampler.sample(rng);
            let weight = weight_sampler.sample(rng);

            graphs
                .par_iter_mut()
                .zip(bds.par_iter_mut())
                .zip(hashes.par_iter_mut())
                .for_each(|((graph, bd), hash)| {
                    let (prev, curr) = mcmc_step(graph, edge_idx, weight, bd);
                    hash_mcmc(hash, edge_idx, prev, curr);
                });

            hash_collisions.clear();
            for i in (0..hashes.len()).rev() {
                hash_collisions
                    .entry(hashes[i])
                    .and_modify(|c| {
                        if c.iter().any(|j| graphs[i].edges() == graphs[*j].edges()) {
                            collided.push(i);
                        } else {
                            c.push(i);
                        }
                    })
                    .or_insert(smallvec![i]);
            }

            for col in collided.drain(..) {
                graphs.swap_remove(col);
                hashes.swap_remove(col);
                // In theory, one could just pop the last element here
                bds.swap_remove(col);

                collisions.push(format!("{step}"));
                eprintln!("Collision at Step {step}");
            }
        }

        while collisions.len() + 1 < params.num_chains {
            collisions.push("INF".to_owned());
        }

        println!(
            "{:?},{},{},{},{:?},{},{:?}",
            GraphType::from_source(&params.source),
            degree,
            base_graph.n(),
            base_graph.m(),
            params.pre_rounds,
            params.max_rounds,
            collisions
        );
    });
}

/// Do one step in the MCMC
fn mcmc_step(
    graph: &mut Graph<i32>,
    edge_idx: usize,
    weight: i32,
    bd: &mut BiDijkstra<i32>,
) -> (i32, i32) {
    let edge = graph.edge(edge_idx);

    let pot_weight = graph.pot_weight((edge.source, edge.target, weight).into());
    if pot_weight >= 0 {
        graph.update_weight(edge_idx, weight);
    } else if let Some(((df, db), spt)) = bd.run(
        graph,
        edge.target,
        edge.source,
        -pot_weight,
        &mut EmptyLogger,
    ) {
        graph.update_weight(edge_idx, weight);
        for (node, dist) in spt {
            if node < graph.n() {
                graph.update_pot(node, df - dist);
            } else {
                graph.update_pot(node - graph.n(), dist - db);
            }
        }
    }

    (edge.weight, graph.edge(edge_idx).weight)
}

/// Mutate the current hash based on an updated edge weight
const fn hash_mcmc(hash: &mut u64, edge: usize, prev: i32, curr: i32) {
    *hash ^= (prev as u64 + 2) * edge as u64;
    *hash ^= (curr as u64 + 2) * edge as u64;
}
