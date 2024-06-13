use rand_distr::Geometric;

use crate::graph::*;

/// The G(n,p) graph generator
pub struct Gnp {
    /// Number of nodes
    n: u64,
    /// Geometric distrbution with specified probability `p`
    distr: Geometric,
}

impl Gnp {
    /// Creates a new instance of the generator
    #[inline]
    pub fn new(n: usize, p: f64) -> Self {
        assert!((0.0..=1.0).contains(&p));

        Self {
            n: n as u64,
            distr: Geometric::new(p).unwrap(),
        }
    }
}

impl GraphGenerator for Gnp {
    fn generate(&mut self, rng: &mut impl Rng) -> Vec<(Node, Node)> {
        let mut edges = Vec::new();

        let mut cur = 0u64;
        let end = self.n * self.n;

        loop {
            let skip = rng.sample(self.distr);
            cur = match (cur + 1).checked_add(skip) {
                Some(x) => x,
                None => break,
            };

            if cur > end {
                break;
            }

            let u = ((cur - 1) / self.n) as Node;
            let v = ((cur - 1) % self.n) as Node;

            edges.push((u, v));
        }

        edges
    }
}
