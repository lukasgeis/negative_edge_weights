#![allow(clippy::type_complexity)]

use std::{convert::Infallible, path::PathBuf, str::FromStr};

use negative_edge_weights::{
    graph::generators::GraphType,
    logger::{ConflictData, DiscreteSeqInsData, DiscreteSeqPotData, DiscreteSeqWeightData},
    weight::InitialWeights,
    Algorithm,
};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
struct Parameters {
    #[structopt(parse(from_os_str))]
    inpath: PathBuf,

    #[structopt(parse(from_os_str))]
    outpath: PathBuf,

    #[structopt(default_value = "i")]
    data_type: DataTypes,

    #[structopt(short = "m", long)]
    mult: Option<usize>,
}

#[derive(Debug, Copy, Clone)]
enum DataTypes {
    Insertions,
    Potentials,
    Weights,
    ConflictLens,
}

impl FromStr for DataTypes {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().next() {
            match c {
                'p' => Ok(DataTypes::Potentials),
                'w' => Ok(DataTypes::Weights),
                'c' => Ok(DataTypes::ConflictLens),
                _ => Ok(DataTypes::Insertions),
            }
        } else {
            Ok(DataTypes::Insertions)
        }
    }
}

fn main() {
    let params = Parameters::from_args();

    match params.data_type {
        DataTypes::Insertions => compress_insertions(params.inpath, params.outpath, params.mult),
        DataTypes::Potentials => compress_potentials(params.inpath, params.outpath, params.mult),
        DataTypes::Weights => compress_weights(params.inpath, params.outpath, params.mult),
        DataTypes::ConflictLens => {
            compress_conflict_lens(params.inpath, params.outpath, params.mult)
        }
    }
}

fn compress_insertions(inpath: PathBuf, outpath: PathBuf, mult: Option<usize>) {
    let mut reader = csv::Reader::from_path(inpath).expect("Could not open infile");
    let mut writer = csv::Writer::from_path(outpath).expect("Could not open outfile");

    /// Indexing:
    /// - GraphType (6)
    /// - InitialWeights (3)
    /// - Algorithm (3)
    /// - Acceptance (2)
    /// - Degree
    /// - Round
    /// - Insertions
    #[derive(Default, Debug, Clone)]
    struct InsIndexTree([[[[Vec<(f64, Vec<(usize, Vec<usize>)>)>; 2]; 3]; 3]; 6]);

    impl InsIndexTree {
        pub fn get_mut_or_create(
            &mut self,
            g: GraphType,
            i: InitialWeights,
            a: Algorithm,
            acc: bool,
            d: f64,
            r: usize,
        ) -> &mut Vec<usize> {
            let mut d_idx: Option<usize> = None;
            let mut r_idx: Option<usize> = None;

            macro_rules! idx_arr {
                () => {
                    self.0[g as usize][i as usize][a as usize][acc as usize]
                };
            }

            for j in 0..idx_arr!().len() {
                if idx_arr!()[j].0 == d {
                    d_idx = Some(j);
                    break;
                }
            }

            if let Some(di) = d_idx {
                for j in 0..idx_arr!()[di].1.len() {
                    if idx_arr!()[di].1[j].0 == r {
                        r_idx = Some(j);
                        break;
                    }
                }

                if r_idx.is_none() {
                    idx_arr!()[di].1.push((r, Vec::new()));
                    r_idx = Some(idx_arr!()[di].1.len() - 1);
                }
            } else {
                idx_arr!().push((d, vec![(r, Vec::new())]));
                d_idx = Some(idx_arr!().len() - 1);
                r_idx = Some(0);
            }

            &mut idx_arr!()[d_idx.unwrap()].1[r_idx.unwrap()].1
        }
    }

    let mut data = InsIndexTree::default();

    for result in reader.deserialize() {
        let row: DiscreteSeqInsData = result.expect("Could not parse row!");
        let DiscreteSeqInsData {
            graph,
            mut degree,
            initial,
            algo,
            round,
            acc,
            ins,
            num,
        } = row;

        if let Some(m) = mult {
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

        let entry = data.get_mut_or_create(graph, initial, algo, acc, degree, round);
        fill_with_zero(entry, ins);
        entry[ins] += num;
    }

    for g in GraphType::ALL {
        for i in InitialWeights::ALL {
            for a in Algorithm::ALL {
                for acc in [false, true] {
                    for (d, rounds) in &data.0[g as usize][i as usize][a as usize][acc as usize] {
                        for (r, insertions) in rounds {
                            for (ins, num) in insertions.iter().enumerate() {
                                let out_data: DiscreteSeqInsData = DiscreteSeqInsData {
                                    graph: g,
                                    degree: *d,
                                    initial: i,
                                    algo: a,
                                    round: *r,
                                    acc,
                                    ins,
                                    num: *num,
                                };

                                writer
                                    .serialize(out_data)
                                    .expect("Could not serialize Out-Data");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn compress_potentials(inpath: PathBuf, outpath: PathBuf, mult: Option<usize>) {
    let mut reader = csv::Reader::from_path(inpath).expect("Could not open infile");
    let mut writer = csv::Writer::from_path(outpath).expect("Could not open outfile");

    /// Indexing:
    /// - GraphType (6)
    /// - InitialWeights (3)
    /// - Algorithm (3)
    /// - Degree
    /// - Round
    /// - Insertions
    #[derive(Default, Debug, Clone)]
    struct PotIndexTree([[[Vec<(f64, Vec<(usize, Vec<usize>)>)>; 3]; 3]; 6]);

    impl PotIndexTree {
        pub fn get_mut_or_create(
            &mut self,
            g: GraphType,
            i: InitialWeights,
            a: Algorithm,
            d: f64,
            r: usize,
        ) -> &mut Vec<usize> {
            let mut d_idx: Option<usize> = None;
            let mut r_idx: Option<usize> = None;

            macro_rules! idx_arr {
                () => {
                    self.0[g as usize][i as usize][a as usize]
                };
            }

            for j in 0..idx_arr!().len() {
                if idx_arr!()[j].0 == d {
                    d_idx = Some(j);
                    break;
                }
            }

            if let Some(di) = d_idx {
                for j in 0..idx_arr!()[di].1.len() {
                    if idx_arr!()[di].1[j].0 == r {
                        r_idx = Some(j);
                        break;
                    }
                }

                if r_idx.is_none() {
                    idx_arr!()[di].1.push((r, Vec::new()));
                    r_idx = Some(idx_arr!()[di].1.len() - 1);
                }
            } else {
                idx_arr!().push((d, vec![(r, Vec::new())]));
                d_idx = Some(idx_arr!().len() - 1);
                r_idx = Some(0);
            }

            &mut idx_arr!()[d_idx.unwrap()].1[r_idx.unwrap()].1
        }
    }

    let mut data = PotIndexTree::default();

    for result in reader.deserialize() {
        let row: DiscreteSeqPotData = result.expect("Could not parse row!");
        let DiscreteSeqPotData {
            graph,
            mut degree,
            initial,
            algo,
            round,
            pot,
            num,
        } = row;

        if let Some(m) = mult {
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

        let entry = data.get_mut_or_create(graph, initial, algo, degree, round);
        fill_with_zero(entry, pot);
        entry[pot] += num;
    }

    for g in GraphType::ALL {
        for i in InitialWeights::ALL {
            for a in Algorithm::ALL {
                for (d, rounds) in &data.0[g as usize][i as usize][a as usize] {
                    for (r, potentials) in rounds {
                        for (pot, num) in potentials.iter().enumerate() {
                            let out_data: DiscreteSeqPotData = DiscreteSeqPotData {
                                graph: g,
                                degree: *d,
                                initial: i,
                                algo: a,
                                round: *r,
                                pot,
                                num: *num,
                            };

                            writer
                                .serialize(out_data)
                                .expect("Could not serialize Out-Data");
                        }
                    }
                }
            }
        }
    }
}

fn compress_weights(inpath: PathBuf, outpath: PathBuf, mult: Option<usize>) {
    let mut reader = csv::Reader::from_path(inpath).expect("Could not open infile");
    let mut writer = csv::Writer::from_path(outpath).expect("Could not open outfile");

    /// Indexing:
    /// - GraphType (6)
    /// - InitialWeights (3)
    /// - Degree
    /// - Round
    /// - Insertions
    #[derive(Default, Debug, Clone)]
    struct WeightIndexTree([[Vec<(f64, Vec<(usize, Vec<(i64, usize)>)>)>; 3]; 6]);

    impl WeightIndexTree {
        pub fn get_mut_or_create(
            &mut self,
            g: GraphType,
            i: InitialWeights,
            d: f64,
            r: usize,
        ) -> &mut Vec<(i64, usize)> {
            let mut d_idx: Option<usize> = None;
            let mut r_idx: Option<usize> = None;

            macro_rules! idx_arr {
                () => {
                    self.0[g as usize][i as usize]
                };
            }

            for j in 0..idx_arr!().len() {
                if idx_arr!()[j].0 == d {
                    d_idx = Some(j);
                    break;
                }
            }

            if let Some(di) = d_idx {
                for j in 0..idx_arr!()[di].1.len() {
                    if idx_arr!()[di].1[j].0 == r {
                        r_idx = Some(j);
                        break;
                    }
                }

                if r_idx.is_none() {
                    idx_arr!()[di].1.push((r, Vec::new()));
                    r_idx = Some(idx_arr!()[di].1.len() - 1);
                }
            } else {
                idx_arr!().push((d, vec![(r, Vec::new())]));
                d_idx = Some(idx_arr!().len() - 1);
                r_idx = Some(0);
            }

            &mut idx_arr!()[d_idx.unwrap()].1[r_idx.unwrap()].1
        }
    }

    let mut data = WeightIndexTree::default();

    for result in reader.deserialize() {
        let row: DiscreteSeqWeightData = result.expect("Could not parse row!");
        let DiscreteSeqWeightData {
            graph,
            mut degree,
            initial,
            round,
            weight,
            num,
        } = row;

        if let Some(m) = mult {
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

        let entry = data.get_mut_or_create(graph, initial, degree, round);
        let index = entry.iter_mut().find(|(w, _)| *w == weight);
        if let Some(i) = index {
            i.1 += num;
        } else {
            entry.push((weight, num));
        }
    }

    for g in GraphType::ALL {
        for i in InitialWeights::ALL {
            for (d, rounds) in &data.0[g as usize][i as usize] {
                for (r, weights) in rounds {
                    for (weight, num) in weights {
                        let out_data: DiscreteSeqWeightData = DiscreteSeqWeightData {
                            graph: g,
                            degree: *d,
                            initial: i,
                            round: *r,
                            weight: *weight,
                            num: *num,
                        };

                        writer
                            .serialize(out_data)
                            .expect("Could not serialize Out-Data");
                    }
                }
            }
        }
    }
}

fn fill_with_zero(vec: &mut Vec<usize>, len: usize) {
    while vec.len() < len + 1 {
        vec.push(0);
    }
}

fn compress_conflict_lens(inpath: PathBuf, outpath: PathBuf, mult: Option<usize>) {
    let mut reader = csv::Reader::from_path(inpath).expect("Could not open infile");
    let mut writer = csv::Writer::from_path(outpath).expect("Could not open outfile");

    /// Indexing:
    /// - GraphType (6)
    /// - InitialWeights (3)
    /// - Algorithm (3)
    /// - Acceptance (2)
    /// - Degree
    /// - Round
    /// - Insertions
    #[derive(Default, Debug, Clone)]
    struct ConflictIndexTree([[[[Vec<(f64, Vec<(usize, Vec<usize>)>)>; 2]; 3]; 3]; 6]);

    impl ConflictIndexTree {
        pub fn get_mut_or_create(
            &mut self,
            g: GraphType,
            i: InitialWeights,
            a: Algorithm,
            d: f64,
            r: usize,
            node: bool,
        ) -> &mut Vec<usize> {
            let mut d_idx: Option<usize> = None;
            let mut r_idx: Option<usize> = None;

            macro_rules! idx_arr {
                () => {
                    self.0[g as usize][i as usize][a as usize][node as usize]
                };
            }

            for j in 0..idx_arr!().len() {
                if idx_arr!()[j].0 == d {
                    d_idx = Some(j);
                    break;
                }
            }

            if let Some(di) = d_idx {
                for j in 0..idx_arr!()[di].1.len() {
                    if idx_arr!()[di].1[j].0 == r {
                        r_idx = Some(j);
                        break;
                    }
                }

                if r_idx.is_none() {
                    idx_arr!()[di].1.push((r, Vec::new()));
                    r_idx = Some(idx_arr!()[di].1.len() - 1);
                }
            } else {
                idx_arr!().push((d, vec![(r, Vec::new())]));
                d_idx = Some(idx_arr!().len() - 1);
                r_idx = Some(0);
            }

            &mut idx_arr!()[d_idx.unwrap()].1[r_idx.unwrap()].1
        }
    }

    let mut data = ConflictIndexTree::default();

    for result in reader.deserialize() {
        let row: ConflictData = result.expect("Could not parse row!");
        let ConflictData {
            graph,
            mut degree,
            initial,
            algo,
            round,
            len,
            num,
            node,
        } = row;

        if let Some(m) = mult {
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

        let entry = data.get_mut_or_create(graph, initial, algo, degree, round, node);
        fill_with_zero(entry, len);
        entry[len] += num;
    }

    for g in GraphType::ALL {
        for i in InitialWeights::ALL {
            for a in Algorithm::ALL {
                for node in [false, true] {
                    for (d, rounds) in &data.0[g as usize][i as usize][a as usize][node as usize] {
                        for (r, lens) in rounds {
                            for (len, num) in lens.iter().enumerate() {
                                let out_data: ConflictData = ConflictData {
                                    graph: g,
                                    degree: *d,
                                    initial: i,
                                    algo: a,
                                    round: *r,
                                    len,
                                    num: *num,
                                    node,
                                };

                                writer
                                    .serialize(out_data)
                                    .expect("Could not serialize Out-Data");
                            }
                        }
                    }
                }
            }
        }
    }
}
