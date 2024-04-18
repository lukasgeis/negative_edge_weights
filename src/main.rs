use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use dijkstra::Dijkstra;
use graph::*;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;
use structopt::StructOpt;

use bellman_ford::has_negative_cycle;

#[cfg(test)]
pub(crate) use graph::test_graph_data as test_data;
use weight::{Weight, WeightType};

mod bellman_ford;
mod dijkstra;
mod graph;
mod weight;

#[derive(StructOpt, Debug, Clone)]
struct Parameters {
    #[structopt(subcommand)]
    source: Source,

    #[structopt(short = "w", default_value = "-1")]
    min_weight: f64,

    #[structopt(short = "W", default_value = "1")]
    max_weight: f64,

    #[structopt(short = "t", default_value = "f64")]
    weight_type: WeightType,

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

#[derive(StructOpt, Debug, Clone)]
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

    let skip_store = false;
    match params.weight_type {
        WeightType::F32 => run::<f32>(params, skip_store),
        WeightType::F64 => run::<f64>(params, skip_store),
        WeightType::I8 => run::<i8>(params, skip_store),
        WeightType::I16 => run::<i16>(params, skip_store),
        WeightType::I32 => run::<i32>(params, skip_store),
        WeightType::I64 => run::<i64>(params, skip_store),
    };
}

fn run<W: Weight>(params: Parameters, skip_store: bool) {
    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let mut timer = Instant::now();
    let mut graph: Graph<W> = match params.source {
        Source::Gnp { nodes, avg_deg } => {
            assert!(nodes > 1 && avg_deg > 0.0);
            let prob = avg_deg / (nodes as f64);
            Graph::gen_gnp(&mut rng, nodes, prob, W::from_f64(params.max_weight))
        }
    };
    #[cfg(not(feature = "hops"))]
    println!(
        "Loaded graph with {} nodes and {} edges in {}ms",
        graph.n(),
        graph.m(),
        timer.elapsed().as_millis(),
    );

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
            "Starting Graph has negative weight cycle"
        );
        #[cfg(not(feature = "hops"))]
        println!(
            "NegativeCycleFinder run on starting graph in {}ms and found no negative cycle",
            timer.elapsed().as_millis()
        );
    }

    timer = Instant::now();
    run_mcmc(&mut rng, &mut graph, &params);
    #[cfg(not(feature = "hops"))]
    println!("MCMC run in {}ms", timer.elapsed().as_millis());

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
            "Resulting Graph has negative weight cycle"
        );
        #[cfg(not(feature = "hops"))]
        println!(
            "NegativeCycleFinder run on resulting graph in {}ms and found no negative cycle",
            timer.elapsed().as_millis()
        );
    }

    #[cfg(not(feature = "hops"))]
    println!(
        "Avg. Edge Weight: {}\nFraction of negative edges: {:.1}%",
        graph.avg_weight(),
        graph.frac_negative_edges() * 100.0,
    );

    if !skip_store {
        timer = Instant::now();
        if let Some(path) = params.output {
            let file_handle = File::create(path).expect("Unable to create file");
            let mut writer = BufWriter::new(file_handle);
            graph.store_graph(&mut writer)
        } else {
            graph.store_graph(&mut ::std::io::stderr())
        }
        .unwrap();
        #[cfg(not(feature = "hops"))]
        println!("Graph stored in {}ms", timer.elapsed().as_millis());
    }
}

/// Runs the MCMC on the graph with the specified parameters
fn run_mcmc<W: Weight>(rng: &mut impl Rng, graph: &mut Graph<W>, params: &Parameters) {
    let num_rounds = (graph.m() as f64 * params.rounds_per_edge).ceil() as u64;
    let mut dijkstra = Dijkstra::new(graph.n());
    let sampler = Uniform::new(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );
    for _ in 0..num_rounds {
        let (idx, (u, v, _)) = graph.random_edge(rng);
        let weight = sampler.sample(rng);

        let potential_weight = graph.potential_weight((u, v, weight));
        if potential_weight >= W::zero() {
            graph.update_weight(idx, weight);
            #[cfg(feature = "bf_test")]
            assert!(
                !has_negative_cycle(graph),
                "BF found a negative weight cycle when Dijkstra accepted directly"
            );
            continue;
        }

        if let Some(shortest_path_tree) = dijkstra.run(graph, v, u, -potential_weight) {
            graph.update_weight(idx, weight);
            for (node, dist) in shortest_path_tree {
                *graph.potential_mut(node) -= potential_weight + dist;
            }
            #[cfg(feature = "bf_test")]
            assert!(
                !has_negative_cycle(graph),
                "BF found a negative weight cycle when Dijkstra accepted"
            );
        } else {
            #[cfg(feature = "bf_test")]
            {
                let old_weight = graph.weight(idx);
                graph.update_weight(idx, weight);
                assert!(
                    has_negative_cycle(graph),
                    "BF found no negative weight cycle when Dijkstra rejected"
                );
                graph.update_weight(idx, old_weight);
            }
        }
    }
}

#[cfg(feature = "bf_test")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dijkstra_vs_bf() {
        for (a, b) in [(-1.0, 1.0), (-2.0, 5.0), (-3.0, 10.0)] {
            let params = Parameters {
                source: Source::Gnp {
                    nodes: 100,
                    avg_deg: 5.0,
                },
                min_weight: a,
                max_weight: b,
                weight_type: WeightType::F64,
                rounds_per_edge: 5.0,
                seed: Some(1234),
                output: None,
                check: true,
            };

            run::<i64>(params.clone(), true);
            run::<f64>(params, true);
        }
    }
}
