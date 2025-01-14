#![allow(unused)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

use std::{fs::File, io::BufReader, path::PathBuf, time::Instant};

use negative_edge_weights::{
    pot::{
        alternating::Alternating, checks::is_feasible, det_mc::IterativeInsertions,
        johnson::Johnson, read_graph_from_file, Graph, Potentials,
    },
    weight::{Weight, WeightType},
};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
struct Parameters {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short = "t", default_value = "f64")]
    weight_type: WeightType,
}

fn main() {
    let params = Parameters::from_args();

    match params.weight_type {
        WeightType::F32 => run::<f32>(params),
        WeightType::F64 => run::<f64>(params),
        WeightType::I8 => run::<i8>(params),
        WeightType::I16 => run::<i16>(params),
        WeightType::I32 => run::<i32>(params),
        WeightType::I64 => run::<i64>(params),
    };
}

fn run<W>(params: Parameters)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    let file = File::open(params.input).expect("Could not open file!");
    let reader = BufReader::new(file);
    let (n, edges) = read_graph_from_file(reader).unwrap();

    let graph: Graph<W> = Graph::from_edges(n, edges);

    let mut timer;

    // Johnson
    {
        timer = Instant::now();
        let pot = Johnson::compute_pot(&graph).expect("Somehow there were negative cycles!");
        println!(
            "Computed Potentials using Johnson in {}ms",
            timer.elapsed().as_millis()
        );
        assert!(
            is_feasible(&graph, &pot, true),
            "Potentials from Johnson were not feasible!"
        );
    }

    // Alternating
    {
        timer = Instant::now();
        let pot = Alternating::compute_pot(&graph).expect("Somehow there were negative cycles!");
        println!(
            "Computed Potentials using Alternating in {}ms",
            timer.elapsed().as_millis()
        );
        assert!(
            is_feasible(&graph, &pot, true),
            "Potentials from Alternating were not feasible!"
        );
    }

    // IterativeInsertions
    {
        timer = Instant::now();
        let pot =
            IterativeInsertions::compute_pot(&graph).expect("Somehow there were negative cycles!");
        println!(
            "Computed Potentials using IterativeInsertions in {}ms",
            timer.elapsed().as_millis()
        );
        assert!(
            is_feasible(&graph, &pot, false),
            "Potentials from IterativeInsertions were not feasible!"
        );
    }
}
