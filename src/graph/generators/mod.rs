use crate::graph::*;

mod dsf;
mod gnp;
mod rhg;

pub use dsf::*;
pub use gnp::*;
pub use rhg::*;

/// A base trait for all graph generators
pub trait GraphGenerator {
    fn generate(&mut self, rng: &mut impl Rng) -> Vec<(Node, Node)>;
}

/// Generator for complete graphs with/without self-loops
pub struct Complete {
    /// Number of nodes
    n: usize,
    /// Are self-loops allowed?
    loops: bool,
}

impl Complete {
    /// Creates the generator with given parameters
    #[inline]
    pub fn new(n: usize, loops: bool) -> Self {
        Self { n, loops }
    }
}

impl GraphGenerator for Complete {
    #[inline]
    fn generate(&mut self, _: &mut impl Rng) -> Vec<(Node, Node)> {
        (0..(self.n * self.n))
            .filter_map(|x| {
                let u = (x / self.n) as Node;
                let v = (x % self.n) as Node;

                if u != v || self.loops {
                    Some((u, v))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// A generator for a simple cycle
pub struct Cycle {
    /// Number of nodes
    n: usize,
}

impl Cycle {
    /// Creates the generator with given parameters
    #[inline]
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl GraphGenerator for Cycle {
    #[inline]
    fn generate(&mut self, _: &mut impl Rng) -> Vec<(Node, Node)> {
        (0..self.n)
            .map(|u| (u as Node, ((u + 1) % self.n) as Node))
            .collect()
    }
}
