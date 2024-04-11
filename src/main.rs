use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use dijkstra::Dijkstra;
use graph::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use structopt::StructOpt;

use bellman_ford::has_negative_cycle;

#[cfg(test)]
pub(crate) use graph::test_graph_data as test_data;

mod bellman_ford;
mod dijkstra;
mod graph;

#[derive(StructOpt)]
struct Parameters {
    #[structopt(subcommand)]
    source: Source,

    #[structopt(short = "w", default_value = "-1")]
    min_weight: Weight,

    #[structopt(short = "W", default_value = "1")]
    max_weight: Weight,

    /// Carry out m * rounds_per_edge MCMC update steps
    #[structopt(short = "r", default_value = "1")]
    rounds_per_edge: f64,

    /// Seed for the PRNG
    #[structopt(short = "s")]
    seed: Option<u64>,

    /// Optional output path for the resulting weighted graph
    #[structopt(short = "o")]
    output: Option<PathBuf>,

    /// Check if the generated graphs have negative weight cycles
    #[structopt(long)]
    check: bool,
}

#[derive(StructOpt)]
enum Source {
    Gnp {
        #[structopt(short = "n")]
        nodes: Node,

        #[structopt(short = "d")]
        avg_deg: f64,
    },
}

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);
    assert!(params.rounds_per_edge > 0.0);

    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let mut timer = Instant::now();
    let mut graph: Graph = match params.source {
        Source::Gnp { nodes, avg_deg } => {
            assert!(nodes > 1 && avg_deg > 0.0);
            let prob = avg_deg / (nodes as f64);
            Graph::gen_gnp(&mut rng, nodes, prob, 1 as Weight)
        }
    };
    println!(
        "Loaded graph with {} nodes and {} edges in {}ms",
        graph.n(),
        graph.m(),
        timer.elapsed().as_millis(),
    );

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph),
            "Starting Graph has negative weight cycle"
        );
        println!(
            "NegativeCycleFinder run on starting graph in {}ms and found no negative cycle",
            timer.elapsed().as_millis()
        );
    }

    timer = Instant::now();
    run_mcmc(&mut rng, &mut graph, &params);
    println!("MCMC run in {}ms", timer.elapsed().as_millis());

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph),
            "Resulting Graph has negative weight cycle"
        );
        println!(
            "NegativeCycleFinder run on resulting graph in {}ms and found no negative cycle",
            timer.elapsed().as_millis()
        );
    }

    println!(
        "Avg. Edge Weight: {}\nFraction of negative edges: {:.1}%",
        graph.avg_weight(),
        graph.frac_negative_edges() * 100.0,
    );

    timer = Instant::now();
    if let Some(path) = params.output {
        let file_handle = File::create(path).expect("Unable to create file");
        let mut writer = BufWriter::new(file_handle);
        graph.store_graph(&mut writer)
    } else {
        graph.store_graph(&mut ::std::io::stderr())
    }
    .unwrap();
    println!("Graph stored in {}ms", timer.elapsed().as_millis());
}

/// Runs the MCMC on the graph with the specified parameters
fn run_mcmc(rng: &mut impl Rng, graph: &mut Graph, params: &Parameters) {
    let num_rounds = (graph.m() as f64 * params.rounds_per_edge).ceil() as u64;
    let mut dijkstra = Dijkstra::new(graph.n());
    for _ in 0..num_rounds {
        let (idx, (u, v, _)) = graph.random_edge(rng);
        let weight = rng.gen_range(params.min_weight..=params.max_weight);

        let potential_weight = graph.potential_weight((u, v, weight));
        if potential_weight >= 0.0 {
            graph.update_weight(idx, weight);
            continue;
        }

        if let Some(shortest_path_tree) = dijkstra.run(graph, v, u, -potential_weight) {
            for (node, dist) in shortest_path_tree {
                *graph.potential_mut(node) -= potential_weight + dist;
            }
        }
    }
}
