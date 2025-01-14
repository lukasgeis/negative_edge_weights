//! # Loggers
//!
//! A collection of loggers used for the various experiments to collect data while running the
//! algorithms.

use std::ops::Index;

use serde_derive::{Deserialize, Serialize};

use crate::weight::{InitialWeights, Weight};

use super::{
    graph::{generators::GraphType, Edge, Node},
    Algorithm,
};

/// A generic trait for the base implementations of the algorithms.
/// Consists of (almost) all measurable metrics than can arise during the algorithm.
pub trait Logger<W: Weight> {
    /// Initialize the Logger
    fn init(n: usize, m: usize, deg: f64, gen: GraphType, init: InitialWeights) -> Self;

    /// Start a new round in the MCMC in which we try to update weight `old_weight` to `new_weight`
    fn new_round(&mut self, old_weight: W, new_weight: W);

    /// An insertion in BellmanFord happened
    fn add_insertion_bf(&mut self);

    /// An insertion in Dijkstra happened
    fn add_insertion_dk(&mut self);

    /// An insertion in BiDijkstra happened in the forward-search
    fn add_insertion_bd_f(&mut self);

    /// An insertion in BiDijkstra happened in the backward-search
    fn add_insertion_bd_b(&mut self);

    /// A potential update happened in Dijkstra
    fn add_pot_update_dk(&mut self);

    /// A potential update happened in BiDijkstra in the forward-search
    fn add_pot_update_bd_f(&mut self);

    /// A potential update happened in BiDijkstra in the backward-search
    fn add_pot_update_bd_b(&mut self);

    /// Set the runtime of BellmanFord
    fn set_runtime_bf(&mut self, ms: u128);

    /// Set the runtime of Dijkstra
    fn set_runtime_dk(&mut self, ms: u128);

    /// Set the runtime of BiDijkstra
    fn set_runtime_bd(&mut self, ms: u128);

    /// End the round with acceptance `acc`
    fn end_round(&mut self, acc: bool);
}

/// A empty logger that does not log data and only has blank implementations.
/// Allows the compiler to ignore the logging for the fastest implementation.
#[derive(Debug, Copy, Clone)]
pub struct EmptyLogger;

impl<W: Weight> Logger<W> for EmptyLogger {
    fn init(_: usize, _: usize, _: f64, _: GraphType, _: InitialWeights) -> Self {
        EmptyLogger
    }

    fn new_round(&mut self, _: W, _: W) {}

    fn add_insertion_bf(&mut self) {}

    fn add_insertion_dk(&mut self) {}

    fn add_insertion_bd_f(&mut self) {}

    fn add_insertion_bd_b(&mut self) {}

    fn add_pot_update_dk(&mut self) {}

    fn add_pot_update_bd_f(&mut self) {}

    fn add_pot_update_bd_b(&mut self) {}

    fn set_runtime_bf(&mut self, _ms: u128) {}

    fn set_runtime_dk(&mut self, _ms: u128) {}

    fn set_runtime_bd(&mut self, _ms: u128) {}

    fn end_round(&mut self, _: bool) {}
}

/// Helper function that adds `1` to `vec[len]`, filling up `vec` with `0` prior if `len` is out of
/// bounds
fn fill_and_push(vec: &mut Vec<usize>, len: usize) {
    while vec.len() < len + 1 {
        vec.push(0);
    }
    vec[len] += 1;
}

/// The logger for the sequential experiments `seq_exp`
#[derive(Debug, Clone)]
pub struct DiscreteSeqLogger {
    /// Number of edges
    pub m: usize,
    /// Average Degree
    pub deg: f64,
    /// Graph Model
    pub gen: GraphType,
    /// Initial Weight function
    pub initial_weights: InitialWeights,

    /// Current weight distribution
    pub weight_distr: DiscreteWeightDistribution,
    /// Number of negative edges
    pub num_neg_edges: usize,
    /// Number of accepted rounds
    pub num_acc_rounds: usize,

    /// Current round
    pub round: usize,
    /// Old Weight of the current edge
    pub old_weight: i64,
    /// (Proposed) new weight of the current edge
    pub new_weight: i64,

    /// Total number of insertions for BellmanFord separated by acceptance
    pub tot_ins_bf: [usize; 2],
    /// Total number of insertions for Dijkstra separated by acceptance    
    pub tot_ins_dk: [usize; 2],
    /// Total number of insertions for BiDijkstra separated by acceptance    
    pub tot_ins_bd: [usize; 2],
    /// Total number of potential updates for Dijkstra
    pub tot_pot_dk: usize,
    /// Total number of potential updates for BiDijkstra
    pub tot_pot_bd: usize,

    /// Number of insertions for BellmanFord in current round    
    pub insertions_bf: usize,
    /// Number of insertions for Dijkstra in current round    
    pub insertions_dk: usize,
    /// Number of insertions for BiDijkstra in current round    
    pub insertions_bd: usize,
    /// Number of potential updates for Dijkstra in current round    
    pub pot_updates_dk: usize,
    /// Number of potential updates for BiDijkstra in current round    
    pub pot_updates_bd: usize,

    /// Occurences of insertions in accepted rounds for BellmanFord
    pub acc_insertions_bf: Vec<usize>,
    /// Occurences of insertions in accepted rounds for Dijkstra    
    pub acc_insertions_dk: Vec<usize>,
    /// Occurences of insertions in accepted rounds for BiDijkstra    
    pub acc_insertions_bd: Vec<usize>,

    /// Occurences of insertions in accepted rounds for BellmanFord
    pub rej_insertions_bf: Vec<usize>,
    /// Occurences of insertions in accepted rounds for Dijkstra
    pub rej_insertions_dk: Vec<usize>,
    /// Occurences of insertions in accepted rounds for BiDijkstra
    pub rej_insertions_bd: Vec<usize>,

    /// Occurences of potential updates for Dijkstra
    pub all_potentials_dk: Vec<usize>,
    /// Occurences of potential updates for BiDijkstra
    pub all_potentials_bd: Vec<usize>,

    /// Current cumulative runtimes of BellmanFord
    pub runtime_bf: u128,
    /// Current cumulative runtimes of Dijkstra
    pub runtime_dk: u128,
    /// Current cumulative runtimes of BiDijkstra
    pub runtime_bd: u128,
}

impl Logger<i64> for DiscreteSeqLogger {
    fn init(_: usize, m: usize, deg: f64, gen: GraphType, init: InitialWeights) -> Self {
        Self {
            m,
            deg,
            gen,
            initial_weights: init,
            num_neg_edges: 0,
            num_acc_rounds: 0,
            round: 0,
            old_weight: i64::MAX,
            new_weight: i64::MAX,
            tot_ins_bf: [0, 0],
            tot_ins_dk: [0, 0],
            tot_ins_bd: [0, 0],
            tot_pot_dk: 0,
            tot_pot_bd: 0,
            insertions_bf: 0,
            insertions_dk: 0,
            insertions_bd: 0,
            pot_updates_dk: 0,
            pot_updates_bd: 0,
            runtime_bf: 0,
            runtime_dk: 0,
            runtime_bd: 0,
            acc_insertions_bf: Vec::new(),
            acc_insertions_dk: Vec::new(),
            acc_insertions_bd: Vec::new(),
            rej_insertions_bf: Vec::new(),
            rej_insertions_dk: Vec::new(),
            rej_insertions_bd: Vec::new(),
            all_potentials_dk: Vec::new(),
            all_potentials_bd: Vec::new(),
            weight_distr: DiscreteWeightDistribution::default(),
        }
    }

    fn new_round(&mut self, old_weight: i64, new_weight: i64) {
        self.round += 1;
        self.old_weight = old_weight;
        self.new_weight = new_weight;

        self.insertions_bf = 0;
        self.insertions_dk = 0;
        self.insertions_bd = 0;

        self.pot_updates_dk = 0;
        self.pot_updates_bd = 0;
    }

    fn add_insertion_bf(&mut self) {
        self.insertions_bf += 1;
    }

    fn add_insertion_dk(&mut self) {
        self.insertions_dk += 1;
    }

    fn add_insertion_bd_f(&mut self) {
        self.insertions_bd += 1;
    }

    fn add_insertion_bd_b(&mut self) {
        self.insertions_bd += 1;
    }

    fn add_pot_update_dk(&mut self) {
        self.pot_updates_dk += 1;
    }

    fn add_pot_update_bd_f(&mut self) {
        self.pot_updates_bd += 1;
    }

    fn add_pot_update_bd_b(&mut self) {
        self.pot_updates_bd += 1;
    }

    fn set_runtime_bf(&mut self, ns: u128) {
        self.runtime_bf += ns;
    }

    fn set_runtime_dk(&mut self, ns: u128) {
        self.runtime_dk += ns;
    }

    fn set_runtime_bd(&mut self, ns: u128) {
        self.runtime_bd += ns;
    }

    fn end_round(&mut self, acc: bool) {
        if acc {
            self.num_acc_rounds += 1;
            self.num_neg_edges += (self.new_weight < 0) as usize - (self.old_weight < 0) as usize;

            self.weight_distr
                .update_weight(self.old_weight, self.new_weight);
        }

        if self.new_weight < self.old_weight {
            if acc {
                fill_and_push(&mut self.acc_insertions_bf, self.insertions_bf);
                fill_and_push(&mut self.acc_insertions_dk, self.insertions_dk);
                fill_and_push(&mut self.acc_insertions_bd, self.insertions_bd);
            } else {
                fill_and_push(&mut self.rej_insertions_bf, self.insertions_bf);
                fill_and_push(&mut self.rej_insertions_dk, self.insertions_dk);
                fill_and_push(&mut self.rej_insertions_bd, self.insertions_bd);
            }
            fill_and_push(&mut self.all_potentials_dk, self.pot_updates_dk);
            fill_and_push(&mut self.all_potentials_bd, self.pot_updates_bd);
        }

        self.tot_ins_bf[acc as usize] += self.insertions_bf;
        self.tot_ins_dk[acc as usize] += self.insertions_dk;
        self.tot_ins_bd[acc as usize] += self.insertions_bd;
        self.tot_pot_dk += self.pot_updates_dk;
        self.tot_pot_bd += self.pot_updates_bd;
    }
}

/// Helper struct to store the current weight distribution as well as important metrics
///
/// Only works if weights are a discrete (ie `i64`) interval
#[derive(Debug, Clone, Default)]
pub struct DiscreteWeightDistribution {
    /// Current occurences of weights
    pub weights: Vec<usize>,
    /// Start and end of weight-range
    pub range: (i64, i64),
    /// Total sum of weight
    pub total: i64,
    /// Current TVD to `Unif[(range.0)..=(range.1)]`
    pub tvd: f64,
    /// Number of edges
    pub m: i64,
}

impl DiscreteWeightDistribution {
    /// Initialize struct with range bounds and current edges (edge-weights)
    pub fn init(&mut self, (a, b): (i64, i64), edges: &[Edge<i64>]) {
        let size = (b - a + 1) as usize;
        self.weights.resize(size, 0);

        for Edge { weight, .. } in edges {
            self.weights[(weight - a) as usize] += 1;
            self.total += weight;
        }

        self.range = (a, b);

        self.m = edges.len() as i64;

        let mut cum_tvd = 0usize;
        for x in &self.weights {
            cum_tvd += (self.m - (*x * size) as i64).unsigned_abs() as usize;
        }
        self.tvd = cum_tvd as f64 / self.m as f64 / size as f64 / 2.0;
    }

    /// Change a weight `old` to `new`
    ///
    /// It is not important which edge got its weight changed.
    pub fn update_weight(&mut self, old: i64, new: i64) {
        self.total += new - old;

        let size = self.range.1 - self.range.0 + 1;

        let mut tvd_delta = 0i64;
        tvd_delta -= (self.m - size * self[old] as i64).abs();
        tvd_delta -= (self.m - size * self[new] as i64).abs();

        self.weights[(old - self.range.0) as usize] -= 1;
        self.weights[(new - self.range.0) as usize] += 1;

        tvd_delta += (self.m - size * self[old] as i64).abs();
        tvd_delta += (self.m - size * self[new] as i64).abs();

        self.tvd += tvd_delta as f64 / self.m as f64 / size as f64 / 2.0;
    }

    /// Iterate over all weights and their occurences
    pub fn weights_iter(&self) -> impl Iterator<Item = (i64, usize)> + '_ {
        (self.range.0..=self.range.1).map(|w| (w, self.weights[(w - self.range.0) as usize]))
    }
}

impl Index<i64> for DiscreteWeightDistribution {
    type Output = usize;

    fn index(&self, index: i64) -> &Self::Output {
        &self.weights[(index - self.range.0) as usize]
    }
}

/// Experiment data logged every `log` steps
///
/// Field names are similar to those of `DiscreteSeqLogger`
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DiscreteSeqLogData {
    pub graph: GraphType,
    pub m: usize,
    pub degree: f64,
    pub initial: InitialWeights,
    pub round: usize,
    pub num_acc_rounds: usize,
    pub average_weight: f64,
    pub tvd: f64,
    pub num_neg_edges: usize,
    pub num_ins_bf_acc: usize,
    pub num_ins_bf_rej: usize,
    pub num_ins_dk_acc: usize,
    pub num_ins_dk_rej: usize,
    pub num_ins_bd_acc: usize,
    pub num_ins_bd_rej: usize,
    pub num_pot_dk: usize,
    pub num_pot_bd: usize,
    pub time_bf: u128,
    pub time_dk: u128,
    pub time_bd: u128,
}

/// Experiment data for occurences of queue insertions
/// Logged at every `10^i`th round  
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DiscreteSeqInsData {
    pub graph: GraphType,
    pub degree: f64,
    pub initial: InitialWeights,
    pub algo: Algorithm,
    pub round: usize,
    pub acc: bool,
    pub ins: usize,
    pub num: usize,
}

/// Experiment data for occurences of potential updates
/// Logged at every `10^i`th round
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DiscreteSeqPotData {
    pub graph: GraphType,
    pub degree: f64,
    pub initial: InitialWeights,
    pub algo: Algorithm,
    pub round: usize,
    pub pot: usize,
    pub num: usize,
}

/// Experiment data for weight distributions
/// Logged at every `10^i`th round
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DiscreteSeqWeightData {
    pub graph: GraphType,
    pub degree: f64,
    pub initial: InitialWeights,
    pub round: usize,
    pub weight: i64,
    pub num: usize,
}

impl DiscreteSeqLogger {
    /// Get the current `DiscreteSeqLogData` from the logger (and reset the logger)
    pub fn extract_log_data(&mut self, int_size: usize, bf_scale: usize) -> DiscreteSeqLogData {
        let log_data = DiscreteSeqLogData {
            graph: self.gen,
            m: self.m,
            degree: self.deg,
            initial: self.initial_weights,
            round: self.round,
            num_acc_rounds: self.num_acc_rounds,
            average_weight: self.weight_distr.total as f64 / self.m as f64,
            tvd: self.weight_distr.tvd,
            num_neg_edges: self.num_neg_edges,
            num_ins_bf_acc: self.tot_ins_bf[1] * bf_scale,
            num_ins_bf_rej: self.tot_ins_bf[0] * bf_scale,
            num_ins_dk_acc: self.tot_ins_dk[1],
            num_ins_dk_rej: self.tot_ins_dk[0],
            num_ins_bd_acc: self.tot_ins_bd[1],
            num_ins_bd_rej: self.tot_ins_bd[0],
            num_pot_dk: self.tot_pot_dk,
            num_pot_bd: self.tot_pot_bd,
            time_bf: self.runtime_bf * bf_scale as u128 / int_size as u128,
            time_dk: self.runtime_dk / int_size as u128,
            time_bd: self.runtime_bd / int_size as u128,
        };

        self.runtime_bf = 0;
        self.runtime_dk = 0;
        self.runtime_bd = 0;

        log_data
    }

    /// Helper function to remove a `0`-insertion from BellmanFord in case no BellmanFord-iteration
    /// is run but an insertion (or rather no insertion) is logged
    pub fn remove_empty_bf_insertion(&mut self, acc: bool) {
        if acc {
            self.acc_insertions_bf[0] -= 1;
        } else {
            self.rej_insertions_bf[0] -= 1;
        }
    }

    /// Helper function to generate `DiscreteSeqInsData`
    fn gen_ins_data(
        &self,
        algo: Algorithm,
        (i, n): (usize, &usize),
        acc: bool,
    ) -> Option<DiscreteSeqInsData> {
        if *n > 0 {
            Some(DiscreteSeqInsData {
                graph: self.gen,
                degree: self.deg,
                initial: self.initial_weights,
                algo,
                round: self.round,
                acc,
                ins: i,
                num: *n,
            })
        } else {
            None
        }
    }

    /// Helper function to generate `DiscreteSeqPotData`
    fn gen_pot_data(&self, algo: Algorithm, (i, n): (usize, &usize)) -> Option<DiscreteSeqPotData> {
        if *n > 0 {
            Some(DiscreteSeqPotData {
                graph: self.gen,
                degree: self.deg,
                initial: self.initial_weights,
                algo,
                round: self.round,
                pot: i,
                num: *n,
            })
        } else {
            None
        }
    }

    /// Extract all `DiscreteSeqInsData`, `DiscreteSeqPotData`, `DiscreteSeqWeightData` from the
    /// logger
    pub fn get_fixed_data(
        &self,
    ) -> (
        Vec<DiscreteSeqInsData>,
        Vec<DiscreteSeqPotData>,
        Vec<DiscreteSeqWeightData>,
    ) {
        (
            self.acc_insertions_bf
                .iter()
                .enumerate()
                .filter_map(|x| self.gen_ins_data(Algorithm::BF, x, true))
                .chain(
                    self.acc_insertions_dk
                        .iter()
                        .enumerate()
                        .filter_map(|x| self.gen_ins_data(Algorithm::DK, x, true)),
                )
                .chain(
                    self.acc_insertions_bd
                        .iter()
                        .enumerate()
                        .filter_map(|x| self.gen_ins_data(Algorithm::BD, x, true)),
                )
                .chain(
                    self.rej_insertions_bf
                        .iter()
                        .enumerate()
                        .filter_map(|x| self.gen_ins_data(Algorithm::BF, x, false)),
                )
                .chain(
                    self.rej_insertions_dk
                        .iter()
                        .enumerate()
                        .filter_map(|x| self.gen_ins_data(Algorithm::DK, x, false)),
                )
                .chain(
                    self.rej_insertions_bd
                        .iter()
                        .enumerate()
                        .filter_map(|x| self.gen_ins_data(Algorithm::BD, x, false)),
                )
                .collect(),
            self.all_potentials_dk
                .iter()
                .enumerate()
                .filter_map(|x| self.gen_pot_data(Algorithm::DK, x))
                .chain(
                    self.all_potentials_bd
                        .iter()
                        .enumerate()
                        .filter_map(|x| self.gen_pot_data(Algorithm::BD, x)),
                )
                .collect(),
            self.weight_distr
                .weights_iter()
                .map(|(w, n)| DiscreteSeqWeightData {
                    graph: self.gen,
                    degree: self.deg,
                    initial: self.initial_weights,
                    round: self.round,
                    weight: w,
                    num: n,
                })
                .collect(),
        )
    }
}

/// A logger for measuring conflict lengths of the MCMC
#[derive(Debug, Clone)]
pub struct ConflictLogger {
    /// Graph Model
    pub graph: GraphType,
    /// Average Degree
    pub degree: f64,
    /// Initial Weight function
    pub initial: InitialWeights,
    /// Current round
    pub round: usize,

    /// Round-index for each node when this node was last seen in Dijkstra
    pub nodes_last_seen_dk: Vec<usize>,
    /// Current length of consecutive conflict-free rounds in Dijkstra
    pub cur_round_conflict_dk: usize,
    /// Last round since no conflict happened in Dijkstra
    pub last_round_conflict_dk: usize,
    /// Occurences of lengths of consecutive conflict-free rounds in Dijkstra
    pub round_conflicts_dk: Vec<usize>,
    /// Occurences of lengths of consecutive conflict-free nodes in BDijkstra
    pub node_conflicts_dk: Vec<usize>,

    /// Round-index for each node when this node was last seen in BiDijkstra
    pub nodes_last_seen_bd: Vec<usize>,
    /// Current length of consecutive conflict-free rounds in BiDijkstra
    pub cur_round_conflict_bd: usize,
    /// Last round since no conflict happened in BiDijkstra
    pub last_round_conflict_bd: usize,
    /// Occurences of lengths of consecutive conflict-free rounds in BiDijkstra
    pub round_conflicts_bd: Vec<usize>,
    /// Occurences of lengths of consecutive conflict-free nodes in BiDijkstra
    pub node_conflicts_bd: Vec<usize>,
}

/// Experiment data for conflict lengths (either round or nodes)
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ConflictData {
    pub graph: GraphType,
    pub degree: f64,
    pub initial: InitialWeights,
    pub algo: Algorithm,
    pub round: usize,
    pub len: usize,
    pub num: usize,
    pub node: bool,
}

impl ConflictLogger {
    /// Creates a new logger
    pub fn new(graph: GraphType, n: usize, degree: f64, initial: InitialWeights) -> Self {
        Self {
            graph,
            degree,
            initial,
            round: 0,
            nodes_last_seen_dk: vec![usize::MAX; n],
            cur_round_conflict_dk: usize::MAX,
            last_round_conflict_dk: 0,
            round_conflicts_dk: Vec::new(),
            node_conflicts_dk: Vec::new(),
            nodes_last_seen_bd: vec![usize::MAX; n],
            cur_round_conflict_bd: usize::MAX,
            last_round_conflict_bd: 0,
            round_conflicts_bd: Vec::new(),
            node_conflicts_bd: Vec::new(),
        }
    }

    /// Start a new round
    pub fn new_round(&mut self, source: Node, target: Node) {
        self.round += 1;
        self.cur_round_conflict_dk = usize::MAX;
        self.cur_round_conflict_bd = usize::MAX;

        self.see_node_dk(source);
        self.see_node_dk(target);

        self.see_node_bd(source);
        self.see_node_bd(target);
    }

    /// End a round
    pub fn end_round(&mut self) {
        if self.cur_round_conflict_bd != usize::MAX {
            fill_and_push(&mut self.round_conflicts_bd, self.cur_round_conflict_bd);
            self.last_round_conflict_bd = self.round;
            self.cur_round_conflict_bd = usize::MAX;
        }

        if self.cur_round_conflict_dk != usize::MAX {
            fill_and_push(&mut self.round_conflicts_dk, self.cur_round_conflict_dk);
            self.last_round_conflict_dk = self.round;
            self.cur_round_conflict_dk = usize::MAX;
        }
    }

    /// Visit a node in Dijkstra
    pub fn see_node_dk(&mut self, node: Node) {
        if self.nodes_last_seen_dk[node as usize] == usize::MAX {
            self.nodes_last_seen_dk[node as usize] = self.round;
            return;
        }

        let conflict_len = self.round - self.nodes_last_seen_dk[node as usize];
        if conflict_len == 0 {
            return;
        }
        fill_and_push(&mut self.node_conflicts_dk, conflict_len);

        if self.nodes_last_seen_dk[node as usize] >= self.last_round_conflict_dk {
            self.cur_round_conflict_dk = self.round - self.last_round_conflict_dk;
            self.last_round_conflict_dk = self.round;
        }

        self.nodes_last_seen_dk[node as usize] = self.round;
    }

    /// Visit a node in BiDijkstra
    pub fn see_node_bd(&mut self, node: Node) {
        if self.nodes_last_seen_bd[node as usize] == usize::MAX {
            self.nodes_last_seen_bd[node as usize] = self.round;
            return;
        }

        let conflict_len = self.round - self.nodes_last_seen_bd[node as usize];
        if conflict_len == 0 {
            return;
        }
        fill_and_push(&mut self.node_conflicts_bd, conflict_len);

        if self.nodes_last_seen_bd[node as usize] >= self.last_round_conflict_bd {
            self.cur_round_conflict_bd = self.round - self.last_round_conflict_bd;
            self.last_round_conflict_bd = self.round;
        }

        self.nodes_last_seen_bd[node as usize] = self.round;
    }

    /// Extract all `ConflictData`
    pub fn get_data(&self) -> impl Iterator<Item = ConflictData> + '_ {
        self.node_conflicts_bd
            .iter()
            .enumerate()
            .filter_map(|x| self.gen_conflict_data(Algorithm::BD, x, true))
            .chain(
                self.node_conflicts_dk
                    .iter()
                    .enumerate()
                    .filter_map(|x| self.gen_conflict_data(Algorithm::DK, x, true)),
            )
            .chain(
                self.round_conflicts_bd
                    .iter()
                    .enumerate()
                    .filter_map(|x| self.gen_conflict_data(Algorithm::BD, x, false)),
            )
            .chain(
                self.round_conflicts_dk
                    .iter()
                    .enumerate()
                    .filter_map(|x| self.gen_conflict_data(Algorithm::DK, x, false)),
            )
    }

    /// Helper function to create `ConflictData`
    fn gen_conflict_data(
        &self,
        algorithm: Algorithm,
        (len, num): (usize, &usize),
        node: bool,
    ) -> Option<ConflictData> {
        if *num > 0 && len > 0 {
            Some(ConflictData {
                graph: self.graph,
                degree: self.degree,
                initial: self.initial,
                algo: algorithm,
                round: self.round,
                len,
                num: *num,
                node,
            })
        } else {
            None
        }
    }
}

/// A logger for the neighbor-MCMC
#[derive(Debug, Clone)]
pub struct NeighborSeqLogger {
    pub m: usize,
    pub deg: f64,
    pub gen: GraphType,
    pub initial_weights: InitialWeights,

    // Number of Acceptances/Rejections
    pub num_acc: usize,
    pub num_rej: usize,

    pub round: usize,

    // Number of Insertions/Potential Updates
    pub tot_ins: usize,
    pub tot_pot: usize,

    pub runtime: u128,
}

/// Experiment data for the neighbor-MCMC
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct NeighborSeqData {
    pub graph: GraphType,
    pub m: usize,
    pub degree: f64,
    pub initial: InitialWeights,
    pub round: usize,
    pub num_acc: usize,
    pub num_rej: usize,
    pub num_ins: usize,
    pub num_pot: usize,
    pub runtime: u128,
}

impl NeighborSeqLogger {
    /// Create a new logger
    pub fn new(m: usize, deg: f64, gen: GraphType, initial_weights: InitialWeights) -> Self {
        Self {
            m,
            deg,
            gen,
            initial_weights,
            num_acc: 0,
            num_rej: 0,
            round: 0,
            tot_ins: 0,
            tot_pot: 0,
            runtime: 0,
        }
    }

    /// Start a new round
    pub fn new_round(&mut self) {
        self.round += 1;
    }

    /// Add an insertion
    pub fn add_insertion(&mut self) {
        self.tot_ins += 1;
    }

    /// Add a potential update
    pub fn add_pot_update(&mut self) {
        self.tot_pot += 1;
    }

    /// Add the runtime of an algorithm
    pub fn add_runtime(&mut self, ns: u128) {
        self.runtime += ns;
    }

    /// End the current round
    pub fn end_round(&mut self, num_acc: usize, num_rej: usize) {
        self.num_acc += num_acc;
        self.num_rej += num_rej;
    }

    /// Extract `NeighborSeqData` and reset the logger
    pub fn get_data(&mut self, int_size: usize) -> NeighborSeqData {
        let data = NeighborSeqData {
            graph: self.gen,
            m: self.m,
            degree: self.deg,
            initial: self.initial_weights,
            round: self.round,
            num_acc: self.num_acc,
            num_rej: self.num_rej,
            num_ins: self.tot_ins,
            num_pot: self.tot_pot,
            runtime: self.runtime / int_size as u128,
        };

        self.runtime = 0;
        data
    }
}
