#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![allow(clippy::too_many_arguments)]

use std::{fs::File, path::PathBuf, time::Instant};

use csv::Writer;
use negative_edge_weights::{
    graph::{
        generators::{GraphType, Source},
        tarjan::extract_largest_scc,
        Graph,
    },
    logger::{DiscreteSeqLogger, Logger},
    search::{bellman_ford::BellmanFord, bidijkstra::BiDijkstra, dijkstra::Dijkstra},
    weight::{InitialWeights, Weight},
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

    /// Number of MCMC update steps
    #[structopt(short = "r", default_value = "1")]
    rounds: usize,

    /// Only execute BF every `bfskip` times with a minimum of 100 times in between `log_steps`
    #[structopt(long = "bf-skip")]
    bfskip: Option<usize>,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Initial starting weights
    #[structopt(short = "i", default_value = "max")]
    initial_weights: InitialWeights,

    /// Extract the largest SCC and run the MCMC on it
    #[structopt(long)]
    scc: bool,

    /// If `scc`, round `degree` to a multiple of `mult` for output
    #[structopt(long)]
    mult: Option<usize>,

    /// Output path for the resulting Log-Data
    #[structopt(parse(from_os_str))]
    out_log: PathBuf,

    /// Output path for the resulting Ins-Data
    #[structopt(parse(from_os_str))]
    out_ins: PathBuf,

    /// Output path for the resulting Pot-Data
    #[structopt(parse(from_os_str))]
    out_pot: PathBuf,

    /// Output path for the resulting Weight-Data
    #[structopt(parse(from_os_str))]
    out_weight: PathBuf,
}

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);

    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let mut timer = Instant::now();
    let (min_weight, max_weight) = (
        i64::from_f64(params.min_weight),
        i64::from_f64(params.max_weight),
    );
    let graph: Graph<i64> = {
        let graph =
            Graph::from_source(&params.source, &mut rng, params.initial_weights, max_weight);

        if params.scc {
            extract_largest_scc(graph)
        } else {
            graph
        }
    };

    println!(
        "[INFO] Loaded graph with {} nodes and {} edges in {}ms",
        graph.n(),
        graph.m(),
        timer.elapsed().as_millis(),
    );

    timer = Instant::now();
    let weight_sampler = Uniform::new_inclusive(min_weight, max_weight);

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

    let mut logger = DiscreteSeqLogger::init(
        graph.n(),
        graph.m(),
        degree,
        GraphType::from_source(&params.source),
        params.initial_weights,
    );

    logger
        .weight_distr
        .init((min_weight, max_weight), &graph.edges);

    let bf_skip = params.bfskip.unwrap_or(1);

    run_mcmc(
        graph,
        &mut rng,
        weight_sampler,
        params.rounds,
        bf_skip,
        logger,
        Writer::from_path(params.out_log).expect("Could not open acc-file!"),
        Writer::from_path(params.out_ins).expect("Could not open ins-file!"),
        Writer::from_path(params.out_pot).expect("Could not open pot-file!"),
        Writer::from_path(params.out_weight).expect("Could not open int-file!"),
    );

    println!("[INFO] MCMC run in {}ms", timer.elapsed().as_millis());
}

fn run_mcmc<R: Rng, D: Distribution<i64>>(
    graph: Graph<i64>,
    rng: &mut R,
    weight_sampler: D,
    rounds: usize,
    bfskip: usize,
    mut logger: DiscreteSeqLogger,
    mut out_log: Writer<File>,
    mut out_ins: Writer<File>,
    mut out_pot: Writer<File>,
    mut out_weight: Writer<File>,
) {
    let mut timer;

    let mut log_step = 10usize;
    let mut skip_bf = 1;

    let edge_sampler = Uniform::new(0usize, graph.m());

    let mut graph_dk = graph.clone();
    let mut graph_bd = graph;

    let mut bf = BellmanFord::<i64>::new(graph_dk.n());
    let mut dk = Dijkstra::new(graph_dk.n());
    let mut bd = BiDijkstra::new(graph_dk.n());

    let mut acc: bool;

    for _ in 0..rounds {
        acc = false;

        let idx = edge_sampler.sample(rng);
        let edge = graph_dk.edge(idx);
        let weight = weight_sampler.sample(rng);

        logger.new_round(edge.weight, weight);

        // BellmanFord
        timer = Instant::now();
        {
            // We need not update the weight as we use a reference of the Dijkstra-Graph which will
            // be updated (if accepted) later
            if weight < edge.weight && logger.round % skip_bf == 0 {
                bf.run(&graph_dk, edge.target, edge.source, -weight, &mut logger);
            }
        }
        logger.set_runtime_bf(timer.elapsed().as_nanos());

        // Dijkstra
        timer = Instant::now();
        {
            let potential_weight = graph_dk.pot_weight((edge.source, edge.target, weight).into());
            if potential_weight >= 0 {
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
                    logger.add_pot_update_dk();
                }
            }
        }
        logger.set_runtime_dk(timer.elapsed().as_nanos());

        // BiDijkstra
        timer = Instant::now();
        {
            let potential_weight = graph_bd.pot_weight((edge.source, edge.target, weight).into());
            if potential_weight >= 0 {
                graph_bd.update_weight(idx, weight);
                acc = true;
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
                        logger.add_pot_update_bd_f();
                    } else {
                        graph_bd.update_pot(node - graph_bd.n(), dist - db);
                        logger.add_pot_update_bd_b();
                    }
                }
                acc = true;
            }
        }
        logger.set_runtime_bd(timer.elapsed().as_nanos());
        logger.end_round(acc);

        if logger.round % skip_bf != 0 {
            logger.remove_empty_bf_insertion(acc);
        }

        if logger.round % log_step == 0 {
            let log_data = logger.extract_log_data(log_step, skip_bf);

            out_log
                .serialize(log_data)
                .expect("Could not serialize log-data!");

            if logger.round % (100 * log_step) == 0 {
                log_step *= 10;

                skip_bf = bfskip.min(log_step / 100);

                let (ins, pot, wgt) = logger.get_fixed_data();
                for ins_entry in ins {
                    out_ins
                        .serialize(ins_entry)
                        .expect("Could not serialize ins-data!");
                }
                for pot_entry in pot {
                    out_pot
                        .serialize(pot_entry)
                        .expect("Could not serialize pot-data!");
                }
                for wgt_entry in wgt {
                    out_weight
                        .serialize(wgt_entry)
                        .expect("Could not serialize weight-data!");
                }
            }
        }
    }
}
