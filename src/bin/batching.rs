#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

use std::time::Instant;

use negative_edge_weights::{
    graph::{
        generators::{GraphType, Source},
        tarjan::extract_largest_scc,
        Edge, Graph,
    },
    logger::EmptyLogger,
    search::{batching::BatchedBiDijkstra, bidijkstra::BiDijkstra},
    utils::CappedVec,
    weight::{InitialWeights, Weight, WeightType},
};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;
use structopt::StructOpt;

const MAX_THREADS: usize = 64;

#[derive(StructOpt, Debug, Clone)]
struct Parameters {
    /// Source for the graph, i.e. which generator or from file
    #[structopt(subcommand)]
    source: Source,

    /// Minimum weight for an edge
    #[structopt(short = "w", default_value = "-1")]
    min_weight: f64,

    /// Maximum weight for an edge
    #[structopt(short = "W", default_value = "1")]
    max_weight: f64,

    /// Primitive Type used as Weight
    #[structopt(short = "t", default_value = "f64")]
    weight_type: WeightType,

    /// Carry out this many rounds_per_edge MCMC update steps
    #[structopt(short = "r", default_value = "1000")]
    rounds: usize,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Initial starting weights
    #[structopt(short = "i", default_value = "max")]
    initial_weights: InitialWeights,

    /// Maximum Number of Threads/Parallel Runs
    #[structopt(long, default_value = "64")]
    threads: usize,

    /// Extract the largest SCC and run the MCMC on it
    #[structopt(long)]
    scc: bool,

    /// If `scc`, round `degree` to a multiple of `mult` for output
    #[structopt(long)]
    mult: Option<usize>,

    #[structopt(long)]
    seq: bool,
}

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);
    assert!(params.threads <= MAX_THREADS);

    match params.weight_type {
        WeightType::F32 => run::<f32>(params),
        WeightType::F64 => run::<f64>(params),
        WeightType::I8 => run::<i8>(params),
        WeightType::I16 => run::<i16>(params),
        WeightType::I32 => run::<i32>(params),
        WeightType::I64 => run::<i64>(params),
    };
}

/// Private specified helper for `run`
#[inline]
fn run<W: Weight>(params: Parameters)
where
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let max_weight = W::from_f64(params.max_weight);
    let mut graph: Graph<W> = {
        let graph =
            Graph::from_source(&params.source, &mut rng, params.initial_weights, max_weight);

        if params.scc {
            extract_largest_scc(graph)
        } else {
            graph
        }
    };

    let degree = if params.scc {
        let mut degree = (graph.m() as f64 / graph.n() as f64).round();
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

    let weight_sampler = Uniform::new_inclusive(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );

    let timer = Instant::now();
    if params.seq {
        run_mcmc_seq(&mut graph, &mut rng, weight_sampler, params.rounds);
    } else {
        run_mcmc_par(
            &mut graph,
            &mut rng,
            weight_sampler,
            params.rounds,
            params.threads,
        );
    }

    let elapsed_ms = timer.elapsed().as_millis();
    // graph,n,deg,initial,rounds,seq,time
    println!(
        "{:?},{},{},{:?},{},{},{}",
        GraphType::from_source(&params.source),
        graph.n(),
        degree,
        params.initial_weights,
        params.rounds,
        params.seq,
        elapsed_ms
    );
}

fn run_mcmc_par<W: Weight, R: Rng, D: Distribution<W>>(
    graph: &mut Graph<W>,
    rng: &mut R,
    weight_sampler: D,
    mut num_rounds: usize,
    max_threads: usize,
) where
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut batched_bidijkstra = BatchedBiDijkstra::<W, MAX_THREADS>::new(graph.n());

    let edge_sampler = Uniform::new(0usize, graph.m());

    let mut batch_edges = CappedVec::<Edge<W>, MAX_THREADS>::default();
    let mut update_edges = CappedVec::<(usize, W), MAX_THREADS>::default();

    let mut batch_size = 1usize;

    while num_rounds > 0 {
        while batch_edges.size() < MAX_THREADS.min(num_rounds) {
            let idx = edge_sampler.sample(rng);
            let edge = graph.edge(idx);
            let weight = weight_sampler.sample(rng);

            let potential_weight = graph.pot_weight((edge.source, edge.target, weight).into());
            update_edges.push((idx, weight));
            batch_edges.push(Edge {
                source: edge.target,
                target: edge.source,
                weight: -potential_weight,
            });
            num_rounds -= 1;
        }

        let (edge_results, pot_iter) =
            batched_bidijkstra.run(graph, batch_edges, batch_size.min(batch_edges.size()));

        if edge_results.len() < batch_size {
            batch_size = 1.max(batch_size.next_power_of_two() / 2);
        } else {
            batch_size = (batch_size * 2).min(max_threads);
        }

        edge_results
            .iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .for_each(|(i, _)| {
                graph.update_weight(update_edges.get(i).0, update_edges.get(i).1);
            });
        pot_iter.for_each(|(n, w)| {
            graph.update_pot(n, w);
        });

        batch_edges.remove_prefix(edge_results.size());
        update_edges.remove_prefix(edge_results.size());
        for i in 0..batch_edges.size() {
            let edge = *batch_edges.get(i);
            batch_edges.get_mut(i).weight =
                -graph.pot_weight((edge.target, edge.source, update_edges.get(i).1).into());
        }
    }
}

fn run_mcmc_seq<W: Weight, R: Rng, D: Distribution<W>>(
    graph: &mut Graph<W>,
    rng: &mut R,
    weight_sampler: D,
    num_rounds: usize,
) where
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut no_logging = EmptyLogger;
    let mut bidijkstra = BiDijkstra::new(graph.n());
    let edge_sampler = Uniform::new(0usize, graph.m());
    for _ in 0..num_rounds {
        let idx = edge_sampler.sample(rng);
        let edge = graph.edge(idx);
        let weight = weight_sampler.sample(rng);

        let potential_weight = graph.pot_weight((edge.source, edge.target, weight).into());
        if potential_weight >= W::zero() {
            graph.update_weight(idx, weight);
            continue;
        }

        if let Some(((df, db), shortest_path_tree)) = bidijkstra.run(
            graph,
            edge.target,
            edge.source,
            -potential_weight,
            &mut no_logging,
        ) {
            graph.update_weight(idx, weight);
            for (node, dist) in shortest_path_tree {
                if node < graph.n() {
                    graph.update_pot(node, df - dist);
                } else {
                    graph.update_pot(node - graph.n(), dist - db);
                }
            }
        }
    }
}
