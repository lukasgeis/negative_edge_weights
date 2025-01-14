#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![allow(clippy::unnecessary_cast)]

use serde_derive::{Deserialize, Serialize};

pub mod graph;
pub mod logger;
pub mod pot;
pub mod search;
pub mod utils;
pub mod weight;

/// All possible search algorithms
#[derive(Debug, Copy, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Algorithm {
    /// BellmanFord
    BF = 0,
    /// Dijkstra
    DK = 1,
    /// BiDijkstra
    BD = 2,
}

impl Algorithm {
    /// Allows iterating over all possible values
    pub const ALL: [Algorithm; 3] = [Algorithm::BF, Algorithm::DK, Algorithm::BD];
}
