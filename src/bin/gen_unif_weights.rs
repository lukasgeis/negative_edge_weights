#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use negative_edge_weights::{
    graph::{
        generators::Source, neg_cycle::has_negative_cycle, tarjan::extract_largest_scc, Graph,
    },
    logger::EmptyLogger,
    search::bidijkstra::BiDijkstra,
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
    #[structopt(short = "r", default_value = "1")]
    rounds_per_edge: f64,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Initial starting weights
    #[structopt(short = "i", default_value = "max")]
    initial_weights: InitialWeights,

    /// Optional output path for the resulting weighted graph
    #[structopt(short = "o")]
    output: Option<PathBuf>,

    /// Extract the largest SCC and run the MCMC on it
    #[structopt(long)]
    scc: bool,
}

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);
    assert!(params.rounds_per_edge > 0.0);

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

    let mut timer = Instant::now();
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

    println!(
        "[INFO] Loaded graph with {} nodes and {} edges in {}ms",
        graph.n(),
        graph.m(),
        timer.elapsed().as_millis(),
    );

    timer = Instant::now();
    let weight_sampler = Uniform::new_inclusive(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );
    run_mcmc(&mut graph, &mut rng, weight_sampler, params.rounds_per_edge);

    println!("[INFO] MCMC run in {}ms", timer.elapsed().as_millis());

    timer = Instant::now();
    assert!(
        !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
        "[FAIL] Resulting Graph has negative weight cycle"
    );

    println!(
        "[TEST] No negative cycle found in resulting graph in {}ms",
        timer.elapsed().as_millis()
    );

    println!(
        "[INFO] Avg. Edge Weight: {}\n[INFO] Fraction of negative edges: {:.1}%",
        graph.avg_weight(),
        graph.frac_neg_edges() * 100.0,
    );

    if let Some(path) = params.output {
        timer = Instant::now();
        let file_handle = File::create(path).expect("Unable to create file");
        let mut writer = BufWriter::new(file_handle);
        graph.store(&mut writer).unwrap();

        println!("[INFO] Graph stored in {}ms", timer.elapsed().as_millis());
    }
}

fn run_mcmc<W: Weight, R: Rng, D: Distribution<W>>(
    graph: &mut Graph<W>,
    rng: &mut R,
    weight_sampler: D,
    rounds_factor: f64,
) where
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut no_logging = EmptyLogger;
    let num_rounds = (graph.m() as f64 * rounds_factor).ceil() as u64;
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
