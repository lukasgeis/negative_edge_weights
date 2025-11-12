#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![allow(clippy::too_many_arguments)]

use negative_edge_weights::{
    graph::{
        generators::{GraphType, Source},
        tarjan::extract_largest_scc,
        Graph,
    },
    logger::EmptyLogger,
    search::bidijkstra::BiDijkstra,
    weight::InitialWeights,
};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

    /// Number of independent MCMC rounds to mix up the edges
    #[structopt(short = "p", default_value = "1")]
    pre_rounds: usize,

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

fn main() {
    let params = Parameters::from_args();
    assert!(params.min_weight < params.max_weight);

    (1..=params.iterations).into_par_iter().for_each(|iter| {
        let mut rng = if let Some(seed) = params.seed {
            Pcg64::seed_from_u64(seed ^ iter as u64)
        } else {
            Pcg64::from_entropy()
        };

        let (min_weight, max_weight) = (-1, 1);
        /*
        (
            i64::from_f64(params.min_weight),
            i64::from_f64(params.max_weight),
        );*/

        let graph: Graph<i64> = {
            let graph = Graph::from_source(
                &params.source,
                &mut rng,
                InitialWeights::Uniform,
                max_weight,
            );

            if params.scc {
                extract_largest_scc(graph)
            } else {
                graph
            }
        };

        /*
        println!(
            "[INFO] Loaded graph with {} nodes and {} edges in {}ms",
            graph.n(),
            graph.m(),
            timer.elapsed().as_millis(),
        );
        */

        let weight_sampler = Uniform::new_inclusive(min_weight, max_weight);

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

        let graph2 = Graph::from_pos_edges(
            graph.n(),
            graph
                .edges()
                .iter()
                .copied()
                .map(|mut e| {
                    e.weight = InitialWeights::Uniform.generate_weight(&mut rng, max_weight);
                    e
                })
                .collect(),
        );

        let prefix = format!(
            "{:?},{},{},{},{},{}",
            GraphType::from_source(&params.source),
            degree,
            graph.n(),
            graph.m(),
            params.pre_rounds,
            params.max_rounds
        );

        // Prints
        // GraphType,AvgDeg,n,m,PreRounds,MaxRounds,DeltaDist[Pre],NumRounds
        run_mcmc(
            graph,
            graph2,
            &mut rng,
            weight_sampler,
            params.pre_rounds,
            params.max_rounds,
            prefix,
        );

        //println!("[INFO] MCMC run in {}ms", timer.elapsed().as_millis());
    });
}

fn run_mcmc<R: Rng, D: Distribution<i64>>(
    mut graph1: Graph<i64>,
    mut graph2: Graph<i64>,
    rng: &mut R,
    weight_sampler: D,
    pre_rounds: usize,
    max_rounds: usize,
    mut prefix: String,
) {
    let edge_sampler = Uniform::new(0usize, graph1.m());

    let mut bd1 = BiDijkstra::new(graph1.n());
    let mut bd2 = BiDijkstra::new(graph2.n());

    // Pre-Mixing
    for _ in 0..pre_rounds {
        // BiDijkstra1
        {
            let idx = edge_sampler.sample(rng);
            let weight = weight_sampler.sample(rng);
            let edge1 = graph1.edge(idx);

            let potential_weight = graph1.pot_weight((edge1.source, edge1.target, weight).into());
            if potential_weight >= 0 {
                graph1.update_weight(idx, weight);
            } else if let Some(((df, db), shortest_path_tree)) = bd1.run(
                &graph1,
                edge1.target,
                edge1.source,
                -potential_weight,
                &mut EmptyLogger,
            ) {
                graph1.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    if node < graph1.n() {
                        graph1.update_pot(node, df - dist);
                    } else {
                        graph1.update_pot(node - graph1.n(), dist - db);
                    }
                }
            }
        }

        // BiDijkstra2
        {
            let idx = edge_sampler.sample(rng);
            let weight = weight_sampler.sample(rng);
            let edge2 = graph1.edge(idx);

            let potential_weight = graph2.pot_weight((edge2.source, edge2.target, weight).into());
            if potential_weight >= 0 {
                graph2.update_weight(idx, weight);
            } else if let Some(((df, db), shortest_path_tree)) = bd2.run(
                &graph2,
                edge2.target,
                edge2.source,
                -potential_weight,
                &mut EmptyLogger,
            ) {
                graph2.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    if node < graph2.n() {
                        graph2.update_pot(node, df - dist);
                    } else {
                        graph2.update_pot(node - graph2.n(), dist - db);
                    }
                }
            }
        }
    }

    let mut delta_dist = (0..graph1.m())
        .filter(|&idx| graph1.edge(idx).weight != graph2.edge(idx).weight)
        .count();
    prefix.push_str(format!(",{delta_dist}").as_str());

    let mut w1: i64 = graph1.edges().iter().map(|e| e.weight).sum();
    let mut w2: i64 = graph2.edges().iter().map(|e| e.weight).sum();

    //println!("[INFO] Pre-run MCMC for Delta={delta_dist} W[1] = {w1} W[2] = {w2}");

    for round in 1..=max_rounds {
        let idx = edge_sampler.sample(rng);
        let weight = weight_sampler.sample(rng);

        let edge1 = graph1.edge(idx);
        let edge2 = graph2.edge(idx);

        // If weight stays equal, this will be reversed
        delta_dist += (edge1.weight == edge2.weight) as usize;

        let cw1 = w1;
        let cw2 = w2;

        // BiDijkstra1
        {
            let weight = if cw1 >= cw2 && edge1.weight != edge2.weight {
                match weight {
                    0 => -1,
                    -1 => 0,
                    1 => 1,
                    _ => unreachable!("Hard coded"),
                }
            } else {
                weight
            };

            let potential_weight = graph1.pot_weight((edge1.source, edge1.target, weight).into());
            if potential_weight >= 0 {
                w1 += weight - edge1.weight;
                graph1.update_weight(idx, weight);
            } else if let Some(((df, db), shortest_path_tree)) = bd1.run(
                &graph1,
                edge1.target,
                edge1.source,
                -potential_weight,
                &mut EmptyLogger,
            ) {
                w1 += weight - edge1.weight;
                graph1.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    if node < graph1.n() {
                        graph1.update_pot(node, df - dist);
                    } else {
                        graph1.update_pot(node - graph1.n(), dist - db);
                    }
                }
            }
        }

        // BiDijkstra2
        {
            let weight = if cw2 > cw1 && edge1.weight != edge2.weight {
                match weight {
                    0 => -1,
                    -1 => 0,
                    1 => 1,
                    _ => unreachable!("Hard coded"),
                }
            } else {
                weight
            };

            let potential_weight = graph2.pot_weight((edge2.source, edge2.target, weight).into());
            if potential_weight >= 0 {
                w2 += weight - edge2.weight;
                graph2.update_weight(idx, weight);
            } else if let Some(((df, db), shortest_path_tree)) = bd2.run(
                &graph2,
                edge2.target,
                edge2.source,
                -potential_weight,
                &mut EmptyLogger,
            ) {
                w2 += weight - edge2.weight;
                graph2.update_weight(idx, weight);
                for (node, dist) in shortest_path_tree {
                    if node < graph2.n() {
                        graph2.update_pot(node, df - dist);
                    } else {
                        graph2.update_pot(node - graph2.n(), dist - db);
                    }
                }
            }
        }

        // If weight was unequal before, this will decrease the delta-distance
        delta_dist -= (graph1.edge(idx).weight == graph2.edge(idx).weight) as usize;

        if delta_dist == 0 {
            eprintln!("{prefix},{round}");
            return;
        }
    }

    eprintln!("{prefix},INF");
}
