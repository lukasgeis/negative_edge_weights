use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use rand_pcg::Pcg64;

use crate::bidijkstra::Graph as Graph2;
use crate::dijkstra::Graph as Graph1;
use crate::graph::bellman_ford::{has_negative_cycle, Graph as Graph3};
use crate::weight::Weight;
use crate::{graph::*, Algorithm, Parameters};

/// The MCMC used for generating negative edge weights
pub trait NegWeightMCMC<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    /// Given a graph and an RNG, sample weights from `weight_sampler` for `self.m() * rounds_factor` rounds
    fn run_mcmc<R: Rng, D: Distribution<W>>(
        &mut self,
        rng: &mut R,
        weight_sampler: D,
        rounds_factor: f64,
    );
}

/// Run the standard MCMC with additional informations
#[inline]
pub fn run<W>(params: Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    match params.algorithm {
        Algorithm::Dijkstra => run_with_graph::<W, Graph1<W>>(params),
        Algorithm::BiDijkstra => run_with_graph::<W, Graph2<W>>(params),
        Algorithm::BellmanFord => run_with_graph::<W, Graph3<W>>(params),
    };
}

/// Private specified helper for `run`
#[inline]
fn run_with_graph<W, G>(params: Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
    G: GraphStats + GraphEdgeList<W> + GraphFromSource<W> + GraphNeigbors<W> + NegWeightMCMC<W>,
{
    let mut rng = if let Some(seed) = params.seed {
        Pcg64::seed_from_u64(seed)
    } else {
        Pcg64::from_entropy()
    };

    let mut timer = Instant::now();
    let max_weight = W::from_f64(params.max_weight);
    let mut graph: G = G::from_source(&params.source, &mut rng, params.initial_weights, max_weight);

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

        println!(
            "[TEST] No negative cycle found in starting graph in {}ms",
            timer.elapsed().as_millis()
        );
    }

    timer = Instant::now();
    let weight_sampler = Uniform::new_inclusive(
        W::from_f64(params.min_weight),
        W::from_f64(params.max_weight),
    );
    graph.run_mcmc(&mut rng, weight_sampler, params.rounds_per_edge);

    println!("[INFO] MCMC run in {}ms", timer.elapsed().as_millis());

    if params.check {
        timer = Instant::now();
        assert!(
            !has_negative_cycle(&graph), // alternatively we can use `graph.is_feasible()`
            "[TEST] Resulting Graph has negative weight cycle"
        );

        println!(
            "[TEST] No negative cycle found in resulting graph in {}ms",
            timer.elapsed().as_millis()
        );
    }

    println!(
        "[INFO] Avg. Edge Weight: {}\n[INFO] Fraction of negative edges: {:.1}%",
        graph.avg_weight(),
        graph.frac_negative_edges() * 100.0,
    );

    if let Some(path) = params.output {
        timer = Instant::now();
        let file_handle = File::create(path).expect("Unable to create file");
        let mut writer = BufWriter::new(file_handle);
        store_graph(graph, &mut writer).unwrap();

        println!("[INFO] Graph stored in {}ms", timer.elapsed().as_millis());
    }
}
