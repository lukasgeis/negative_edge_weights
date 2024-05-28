use crate::*;

pub(crate) fn run<W>(params: Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let mut timer = Instant::now();
    let default_weight = W::from_f64(params.max_weight);
    let mut graph: Graph<W> = Graph::from_source(&params.source, &mut rng, default_weight);

    #[cfg(not(feature = "no_print"))]
    println!(
        "[INFO] Loaded graph with {} nodes and {} edges in {}ms",
        graph.n(),
        graph.m(),
        timer.elapsed().as_millis(),
    );

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
            "[TEST] Starting Graph has negative weight cycle"
        );

        #[cfg(not(feature = "no_print"))]
        println!(
            "[TEST] No negative cycle found in starting graph in {}ms",
            timer.elapsed().as_millis()
        );
    }

    timer = Instant::now();
    match params.bidir {
        false => run_mcmc(&mut rng, &mut graph, &params),
        true => run_mcmc_bidirectional(&mut rng, &mut graph, &params),
    };

    #[cfg(not(feature = "no_print"))]
    println!("[INFO] MCMC run in {}ms", timer.elapsed().as_millis());

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
            "[TEST] Resulting Graph has negative weight cycle"
        );

        #[cfg(not(feature = "no_print"))]
        println!(
            "[TEST] No negative cycle found in resulting graph in {}ms",
            timer.elapsed().as_millis()
        );
    }

    #[cfg(not(feature = "no_print"))]
    println!(
        "[INFO] Avg. Edge Weight: {}\n[INFO] Fraction of negative edges: {:.1}%",
        graph.avg_weight(),
        graph.frac_negative_edges() * 100.0,
    );

    if let Some(path) = params.output {
        timer = Instant::now();
        let file_handle = File::create(path).expect("Unable to create file");
        let mut writer = BufWriter::new(file_handle);
        graph.store_graph(&mut writer).unwrap();

        #[cfg(not(feature = "no_print"))]
        println!("[INFO] Graph stored in {}ms", timer.elapsed().as_millis());
    }
}

/// Runs the MCMC on the graph with the specified parameters
pub(crate) fn run_mcmc<W>(rng: &mut impl Rng, graph: &mut Graph<W>, params: &Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    let num_rounds = (graph.m() as f64 * params.rounds_per_edge).ceil() as u64;
    let mut dijkstra = Dijkstra::new(graph.n());
    let sampler = Uniform::new(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );

    for _ in 0..num_rounds {
        let (idx, (u, v, w)) = graph.random_edge(rng);
        let weight = sampler.sample(rng);

        let potential_weight = graph.potential_weight((u, v, weight));
        if potential_weight >= W::zero() {
            graph.update_weight(idx, w, weight);

            if params.bftest {
                assert!(
                    !has_negative_cycle(graph),
                    "[FAIL] BF found a negative weight cycle when Dijkstra accepted directly"
                );
            }
            continue;
        }

        if let Some(shortest_path_tree) = dijkstra.run(graph, v, u, -potential_weight) {
            graph.update_weight(idx, w, weight);
            for (node, dist) in shortest_path_tree {
                *graph.potential_mut(node) -= potential_weight + dist;
            }

            if params.bftest {
                assert!(
                    !has_negative_cycle(graph),
                    "[FAIL] BF found a negative weight cycle when Dijkstra accepted"
                );
            }
        } else if params.bftest {
            graph.update_weight(idx, w, weight);
            assert!(
                has_negative_cycle(graph),
                "[FAIL] BF found no negative weight cycle when Dijkstra rejected"
            );
            graph.update_weight(idx, weight, w);
        }
    }
}

/// Runs the MCMC using a bidirectional dijkstra search
pub(crate) fn run_mcmc_bidirectional<W>(
    rng: &mut impl Rng,
    graph: &mut Graph<W>,
    params: &Parameters,
) where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
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
            graph.update_weight(idx, w, weight);

            if params.bftest {
                assert!(
                    !has_negative_cycle(graph),
                    "[FAIL] BF found a negative weight cycle when BiDijkstra accepted directly"
                );
            }
            continue;
        }

        if let Some(((df, db), shortest_path_tree)) = dijkstra.run(graph, v, u, -potential_weight) {
            graph.update_weight(idx, w, weight);
            for (node, dist) in shortest_path_tree {
                if node < graph.n() {
                    *graph.potential_mut(node) += df - dist;
                } else {
                    *graph.potential_mut(node - graph.n()) -= db - dist;
                }
            }

            if params.bftest {
                assert!(
                    !has_negative_cycle(graph),
                    "[FAIL] BF found a negative weight cycle when BiDijkstra accepted"
                );
            }
        } else if params.bftest {
            graph.update_weight(idx, w, weight);
            assert!(
                has_negative_cycle(graph),
                "[FAIL] BF found no negative weight cycle when BiDijkstra rejected"
            );
            graph.update_weight(idx, weight, w);
        }
    }
}
