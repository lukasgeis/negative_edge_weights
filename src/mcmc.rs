use crate::*;

pub(crate) fn run<W: Weight>(params: Parameters) {
    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let mut timer = Instant::now();
    let default_weight = W::from_f64(params.max_weight);
    let mut graph: Graph<W> = match params.source {
        Source::Gnp { nodes, avg_deg } => {
            assert!(nodes > 1 && avg_deg > 0.0);
            let prob = avg_deg / (nodes as f64);
            Graph::gen_gnp(&mut rng, nodes, prob, default_weight)
        }
        Source::Complete { nodes, loops } => Graph::gen_complete(nodes, loops, default_weight),
        Source::Cycle { nodes } => Graph::gen_cycle(nodes, default_weight),
    };

    #[cfg(not(feature = "no_print"))]
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

        #[cfg(not(feature = "no_print"))]
        println!(
            "NegativeCycleFinder run on starting graph in {}ms and found no negative cycle",
            timer.elapsed().as_millis()
        );
    }

    timer = Instant::now();
    #[cfg(not(feature = "bidir"))]
    run_mcmc(&mut rng, &mut graph, &params);

    #[cfg(feature = "bidir")]
    run_mcmc_bidirectional(&mut rng, &mut graph, &params);

    #[cfg(not(feature = "no_print"))]
    println!("MCMC run in {}ms", timer.elapsed().as_millis());

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
            "Resulting Graph has negative weight cycle"
        );

        #[cfg(not(feature = "no_print"))]
        println!(
            "NegativeCycleFinder run on resulting graph in {}ms and found no negative cycle",
            timer.elapsed().as_millis()
        );
    }

    #[cfg(not(feature = "no_print"))]
    println!(
        "Avg. Edge Weight: {}\nFraction of negative edges: {:.1}%",
        graph.avg_weight(),
        graph.frac_negative_edges() * 100.0,
    );

    if let Some(path) = params.output {
        timer = Instant::now();
        let file_handle = File::create(path).expect("Unable to create file");
        let mut writer = BufWriter::new(file_handle);
        graph.store_graph(&mut writer).unwrap();

        #[cfg(not(feature = "no_print"))]
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
                !has_negative_cycle(graph) && graph.is_feasible(),
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

/// Runs the MCMC using a bidirectional dijkstra search
#[cfg(feature = "bidir")]
fn run_mcmc_bidirectional<W: Weight>(
    rng: &mut impl Rng,
    graph: &mut Graph<W>,
    params: &Parameters,
) {
    let num_rounds = (graph.m() as f64 * params.rounds_per_edge).ceil() as u64;
    let mut dijkstra = BiDijkstra::new(graph.n());
    let sampler = Uniform::new(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );

    for _ in 0..num_rounds {
        let (idx, (u, v, w)) = graph.random_edge(rng);
        let weight = sampler.sample(rng);

        let potential_weight = graph.potential_weight((u, v, weight));
        if potential_weight >= W::zero() {
            graph.update_weight(idx, weight);
            #[cfg(feature = "bf_test")]
            assert!(
                !has_negative_cycle(graph),
                "BF found a negative weight cycle when BiDijkstra accepted directly"
            );
            continue;
        }

        if let Some(((df, db), shortest_path_tree)) = dijkstra.run(graph, v, u, -potential_weight) {
            graph.update_weight(idx, weight);
            for (node, dist) in shortest_path_tree {
                if node < graph.n() {
                    *graph.potential_mut(node) += df - dist;
                } else {
                    *graph.potential_mut(node - graph.n()) -= db - dist;
                }
            }

            #[cfg(feature = "bf_test")]
            assert!(
                !has_negative_cycle(graph) && graph.is_feasible(),
                "BF found a negative weight cycle when BiDijkstra accepted"
            );
        } else {
            #[cfg(feature = "bf_test")]
            {
                let old_weight = graph.weight(idx);
                graph.update_weight(idx, weight);
                assert!(
                    has_negative_cycle(graph),
                    "BF found no negative weight cycle when BiDijkstra rejected"
                );
                graph.update_weight(idx, old_weight);
            }
        }
    }
}
