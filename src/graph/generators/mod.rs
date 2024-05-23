use rand::Rng;

use crate::{graph::*, weight::Weight};

mod dsf;
mod gnp;

pub use dsf::*;
pub use gnp::*;

/// A base trait for all graph generators
pub trait GraphGenerator<W: Weight> {
    fn generate(&mut self, rng: &mut impl Rng, default_weight: W) -> Vec<Edge<W>>;
}

/// Generator for complete graphs with/without self-loops
pub struct Complete {
    n: usize,
    loops: bool,
}

impl Complete {
    #[inline]
    pub fn new(n: usize, loops: bool) -> Self {
        Self { n, loops }
    }
}

impl<W: Weight> GraphGenerator<W> for Complete {
    #[inline]
    fn generate(&mut self, _: &mut impl Rng, default_weight: W) -> Vec<Edge<W>> {
        (0..(self.n * self.n))
            .filter_map(|x| {
                let u = (x / self.n) as Node;
                let v = (x % self.n) as Node;

                if u != v || self.loops {
                    Some((u, v, default_weight))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// A generator for a simple cycle
pub struct Cycle {
    n: usize,
}

impl Cycle {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl<W: Weight> GraphGenerator<W> for Cycle {
    #[inline]
    fn generate(&mut self, _: &mut impl Rng, default_weight: W) -> Vec<Edge<W>> {
        (0..self.n)
            .map(|u| (u as Node, ((u + 1) % self.n) as Node, default_weight))
            .collect()
    }
}
