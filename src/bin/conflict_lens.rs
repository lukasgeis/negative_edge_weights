#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

use std::{fs::File, path::PathBuf};

use csv::Writer;
use negative_edge_weights::{
    graph::{
        generators::{GraphType, Source},
        tarjan::extract_largest_scc,
        Graph,
    },
    logger::ConflictLogger,
    search::conflicts_dk_bd::*,
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
    #[structopt(short = "r", default_value = "1000")]
    num_rounds: usize,

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
    let graph: Graph<W> = {
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
            if d % m != 0 {
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

    let logger = ConflictLogger::new(
        GraphType::from_source(&params.source),
        graph.n(),
        degree,
        params.initial_weights,
    );

    let weight_sampler = Uniform::new_inclusive(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );
    run_mcmc(
        graph,
        &mut rng,
        weight_sampler,
        params.num_rounds,
        Writer::from_path(params.log).expect("Could not open log-file!"),
        logger,
    );
}

fn run_mcmc<W: Weight, R: Rng, D: Distribution<W>>(
    graph: Graph<W>,
    rng: &mut R,
    weight_sampler: D,
    num_rounds: usize,
    mut writer: Writer<File>,
    mut logger: ConflictLogger,
) where
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut log_step = 1000usize;

    let edge_sampler = Uniform::new(0usize, graph.m());

    let mut graph_dk = graph.clone();
    let mut graph_bd = graph;

    let mut dk = Dijkstra::new(graph_dk.n());
    let mut bd = BiDijkstra::new(graph_dk.n());

    for _ in 0..num_rounds {
        let idx = edge_sampler.sample(rng);
        let edge = graph_dk.edge(idx);
        let weight = weight_sampler.sample(rng);

        logger.new_round(edge.source, edge.target);

        // Dijkstra
        {
            let potential_weight = graph_dk.pot_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                graph_dk.update_weight(idx, weight);
            } else if let Some(shortest_path_tree) = dk.run(
                &graph_dk,
                edge.target,
                edge.source,
                -potential_weight,
                &mut logger,
            ) {
                graph_dk.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    graph_dk.update_pot(node, -potential_weight - dist);
                }
            }
        }

        // BiDijkstra
        {
            let potential_weight = graph_bd.pot_weight((edge.source, edge.target, weight).into());
            if potential_weight >= W::zero() {
                graph_bd.update_weight(idx, weight);
            } else if let Some(((df, db), shortest_path_tree)) = bd.run(
                &graph_bd,
                edge.target,
                edge.source,
                -potential_weight,
                &mut logger,
            ) {
                graph_bd.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    if node < graph_bd.n() {
                        graph_bd.update_pot(node, df - dist);
                    } else {
                        graph_bd.update_pot(node - graph_bd.n(), dist - db);
                    }
                }
            }
        }

        logger.end_round();

        if logger.round % (log_step) == 0 {
            log_step *= 10;

            logger.get_data().for_each(|d| {
                writer
                    .serialize(d)
                    .expect("Could not serialize Conflict-Data!");
            });
        }
    }
}
