#![feature(is_sorted)]

use std::collections::{HashMap, HashSet};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use structopt::StructOpt;

type WeightEncoding = u32;
type WeightType = i8;
const MAX_NODES: usize = 16;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct Weights {
    weights: [WeightType; MAX_NODES],
    total_weight: i32,
}

impl Weights {
    fn new() -> Weights {
        Weights {
            weights: [0; MAX_NODES],
            total_weight: 0,
        }
    }

    fn try_update(&mut self, node: usize, weight: WeightType) -> bool {
        let new_total = self.total_weight - self.weights[node] as i32 + weight as i32;
        if new_total < 0 {
            return false;
        }
        self.weights[node] = weight;
        self.total_weight = new_total;
        true
    }

    fn update(&mut self, node: usize, weight: WeightType) {
        self.total_weight = self.total_weight - self.weights[node] as i32 + weight as i32;
        self.weights[node] = weight;
    }

    #[allow(dead_code)]
    fn get_weight(&self, node: usize) -> WeightType {
        self.weights[node]
    }

    fn encode(&self, nodes: usize, min: WeightType, max: WeightType) -> WeightEncoding {
        let mut encoding = 0;
        let mut digit: WeightEncoding = 1;
        for i in 0..nodes {
            encoding += (self.weights[i] - min) as WeightEncoding * digit;
            digit *= 1 + (max - min) as WeightEncoding;
        }
        encoding
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "n", long, default_value = "8")]
    nodes: usize,

    #[structopt(short = "a", long, default_value = "-1")]
    min_weight: WeightType,

    #[structopt(short = "b", long, default_value = "1")]
    max_weight: WeightType,

    #[structopt(short = "m", long, default_value = "16")]
    steps: u64,

    #[structopt(short = "k", long, default_value = "1")]
    runs: u32,

    #[structopt(short = "r", long, default_value = "100")]
    repetitions: u32,

    #[structopt(short = "e", long)]
    quit_early: bool,
}

#[derive(Debug, serde::Serialize)]
struct Result {
    nodes: usize,
    min_weight: WeightType,
    max_weight: WeightType,
    steps: u64,
    runs: u32,
    num_valid_encodings: usize,
    count_of_frequencies: Vec<(u64, u64)>,
    completion_run: Option<u64>,
    runtime_ms: u64,
}

fn valid_encodings(nodes: usize, min: WeightType, max: WeightType) -> HashSet<WeightEncoding> {
    let mut encodings = HashSet::new();
    let mut node = Weights::new();

    fn recurse(
        node: &mut Weights,
        encodings: &mut HashSet<WeightEncoding>,
        node_id: usize,
        nodes: usize,
        min: WeightType,
        max: WeightType,
    ) {
        for w in min..=max {
            node.update(node_id, w);
            if node_id == nodes - 1 {
                if node.total_weight >= 0 {
                    encodings.insert(node.encode(nodes, min, max));
                }
            } else {
                recurse(node, encodings, node_id + 1, nodes, min, max);
            }
        }
    }

    recurse(&mut node, &mut encodings, 0, nodes, min, max);
    encodings
}

fn compute_frequency_counts(
    opt: &Opt,
    num_steps: u64,
    num_runs: u64,
    num_possible_encodings: WeightEncoding,
    valid_encodings: &HashSet<WeightEncoding>,
) -> (Vec<(u64, u64)>, Option<u64>) {
    let random_encodings = (0..num_runs)
        .into_par_iter()
        .map_init(Pcg64::from_entropy, |rng, _| {
            let mut node = Weights::new();
            if num_steps == 0 {
                loop {
                    let encoding = rng.gen_range(0..num_possible_encodings);
                    if valid_encodings.contains(&encoding) {
                        break encoding;
                    }
                }
            } else {
                for _ in 0..num_steps {
                    let node_id = rng.gen_range(0..opt.nodes);
                    let weight = rng.gen_range(opt.min_weight..=opt.max_weight);
                    node.try_update(node_id, weight);
                }
                node.encode(opt.nodes, opt.min_weight, opt.max_weight)
            }
        })
        .collect::<Vec<_>>();

    // check that all encodings are valid
    assert!(random_encodings
        .par_iter()
        .all(|encoding| valid_encodings.contains(encoding)));

    let mut encoding_frequency = vec![0; *valid_encodings.iter().max().unwrap() as usize + 1];
    let mut completion_time = None;
    let mut num_unseen = valid_encodings.len();
    let threshold = valid_encodings.len() / 100;
    for (step, &encoding) in random_encodings.iter().enumerate() {
        encoding_frequency[encoding as usize] += 1;
        num_unseen -= (encoding_frequency[encoding as usize] == 1) as usize;

        if completion_time.is_none() && num_unseen == threshold {
            completion_time = Some(step as u64);
            if opt.quit_early {
                return (vec![], completion_time);
            }
        }
    }

    let mut count_of_frequencies: HashMap<u64, u64> = Default::default();
    for (_, &frequency) in encoding_frequency
        .iter()
        .enumerate()
        .filter(|(encoding, _)| valid_encodings.contains(&(*encoding as WeightEncoding)))
    {
        *count_of_frequencies.entry(frequency).or_default() += 1;
    }

    let mut count_counts: Vec<_> = count_of_frequencies.into_iter().collect();
    count_counts.sort_by_key(|(count, _)| *count);

    assert_eq!(
        count_counts.iter().map(|&(a, b)| a * b).sum::<u64>(),
        num_runs
    );

    (count_counts, completion_time)
}

fn main() {
    let opt = Opt::from_args();

    let valid_encodings = valid_encodings(opt.nodes, opt.min_weight, opt.max_weight);
    let possible_encodings =
        ((1 + opt.max_weight - opt.min_weight) as i64).pow(opt.nodes as u32) as WeightEncoding;

    println!(
        "There are {} valid encodings out of {} possibles",
        valid_encodings.len(),
        possible_encodings
    );

    let num_runs = {
        let k = valid_encodings.len() as f64;
        opt.runs as u64 * (k * k.log(k).ceil()) as u64
    };

    (0..opt.repetitions).into_par_iter().for_each(|_| {
        for steps in 0..opt.steps {
            let scaled_steps = opt.nodes as u64 * steps / 4;

            let start = std::time::Instant::now();
            let (count_of_frequencies, completion_run) = compute_frequency_counts(
                &opt,
                scaled_steps,
                num_runs,
                possible_encodings,
                &valid_encodings,
            );
            let runtime_ms = start.elapsed().as_millis() as u64;

            let result = Result {
                nodes: opt.nodes,
                min_weight: opt.min_weight,
                max_weight: opt.max_weight,
                num_valid_encodings: valid_encodings.len(),
                steps: scaled_steps,
                runs: opt.runs,
                count_of_frequencies,
                completion_run,
                runtime_ms,
            };

            eprintln!("{}", serde_json::to_string(&result).unwrap());
        }
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn node_try_update() {
        let mut node = Weights::new();
        assert!(node.try_update(0, 1));
        assert!(!node.try_update(0, -1));
        assert!(node.try_update(1, 1));
        assert!(node.try_update(0, -1));
    }

    #[test]
    #[allow(clippy::identity_op)]
    fn node_encoding() {
        let mut node = Weights::new();
        node.try_update(0, 0);
        assert_eq!(node.encode(1, -1, 1), 1);

        node.try_update(0, 1);
        assert_eq!(node.encode(1, -1, 1), 2);

        assert_eq!(node.encode(2, -1, 1), 2 + 3);
        assert_eq!(node.encode(3, -1, 1), 2 + 3 + 9);
    }
}
