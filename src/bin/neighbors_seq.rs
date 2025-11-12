#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

use std::{fs::File, path::PathBuf, time::Instant};

use csv::Writer;
use negative_edge_weights::{
    graph::{
        generators::{GraphType, Source},
        neg_cycle::has_negative_cycle,
        tarjan::extract_largest_scc,
        Graph,
    },
    logger::NeighborSeqLogger,
    search::neighbors::MultiDijkstra,
    utils::ReusableVec,
    weight::{InitialWeights, Weight, WeightType},
};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;
use structopt::StructOpt;

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

    /// Carry out m * rounds_per_edge MCMC update steps
    #[structopt(short = "r", long, default_value = "10000")]
    rounds: usize,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Initial starting weights
    #[structopt(short = "i", default_value = "max")]
    initial_weights: InitialWeights,

    /// Optional output path for the resulting weighted graph
    #[structopt(short = "l", long, parse(from_os_str))]
    log: PathBuf,

    /// Extract the largest SCC and run the MCMC on it
    #[structopt(long)]
    scc: bool,

    /// If `scc`, round `degree` to a multiple of `mult` for output
    #[structopt(long)]
    mult: Option<usize>,
}

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);

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

    let logger = NeighborSeqLogger::new(
        graph.m(),
        degree,
        GraphType::from_source(&params.source),
        params.initial_weights,
    );

    let weight_sampler = Uniform::new_inclusive(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );
    run_mcmc(
        &mut graph,
        &mut rng,
        weight_sampler,
        params.rounds,
        Writer::from_path(params.log).expect("Could not open log-file!"),
        logger,
    );

    assert!(
        !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
        "[FAIL] Resulting Graph has negative weight cycle"
    );
}

fn run_mcmc<W: Weight, R: Rng, D: Distribution<W>>(
    graph: &mut Graph<W>,
    rng: &mut R,
    weight_sampler: D,
    num_rounds: usize,
    mut writer: Writer<File>,
    mut logger: NeighborSeqLogger,
) where
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut log_step = 10usize;

    let deg = (0..graph.n())
        .map(|u| graph.in_neighbors(u).len())
        .max()
        .unwrap_or(0);

    let mut dijkstra = MultiDijkstra::new(graph.n(), deg);
    let node_sampler = Uniform::new(0usize, graph.n());

    let mut direct_updates: ReusableVec<(usize, W), 8> = ReusableVec::with_capacity(deg);

    let mut timer = Instant::now();

    for _ in 0..num_rounds {
        logger.new_round();

        let mut num_acc = 0usize;

        let node = node_sampler.sample(rng);

        let mut max_distance = W::zero();

        let edges: Vec<(usize, W, W)> = graph
            .in_neighbors(node)
            .iter()
            .filter_map(|edge| {
                let weight = weight_sampler.sample(rng);
                let pot_weight = graph.pot_weight((edge.source, edge.target, weight).into());

                if pot_weight >= W::zero() {
                    direct_updates.push((edge.source, weight));
                    None
                } else if edge.source == node {
                    None
                } else {
                    if -pot_weight > max_distance {
                        max_distance = -pot_weight;
                    }

                    Some((edge.source, weight, -pot_weight))
                }
            })
            .collect();

        direct_updates.iter().for_each(|(source, weight)| {
            graph.update_edge_weight(*source, node, *weight);
            num_acc += 1;
        });
        direct_updates.clear();

        let (acc_iter, sp_iter) = dijkstra.run(graph, node, &edges, max_distance, &mut logger);

        let mut accepted_some = false;

        for u in acc_iter {
            graph.update_edge_weight(edges[u].0, node, edges[u].1);
            num_acc += 1;
            accepted_some = true;
        }

        if accepted_some {
            for (node, dist) in sp_iter {
                graph.update_pot(node, max_distance - dist);
                logger.add_pot_update();
            }
        }

        logger.end_round(num_acc, graph.in_neighbors(node).len() - num_acc);

        if logger.round.is_multiple_of(log_step) {
            logger.add_runtime(timer.elapsed().as_nanos());
            writer
                .serialize(logger.get_data(log_step))
                .expect("Could not serialize Neighbor-Data!");

            timer = Instant::now();

            if logger.round.is_multiple_of(100 * log_step) {
                log_step *= 10;
            }
        }
    }
}
