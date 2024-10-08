use fxhash::FxHashSet;
use rand_distr::{Distribution, Uniform};

use crate::graph::*;

/// The directed scale-free model
///
/// Naive generator inspired by [NetworkX](https://networkx.org/documentation/networkx-2.2/_modules/networkx/generators/directed.html#scale_free_graph)
pub struct DirectedScaleFree {
    /// Probability for adding a new node with an outgoing edge
    alpha: f64,
    /// `= 1 - gamma`: probability for **not** adding a new node with an incoming edge
    alpha_plus_beta: f64,
    ///  Bias for choosing nodes from out-degree distribution
    delta_out: f64,
    /// Bias for choosing nodes from in-degree distribution
    delta_in: f64,
    /// Number of nodes
    n: usize,
    /// Uniform distrbution in [0,1]
    distr: Uniform<f64>,

    seen_edges: FxHashSet<(Node, Node)>,
}

impl DirectedScaleFree {
    /// Creates the generator using the specified parameters
    #[inline]
    pub fn new(n: usize, alpha: f64, beta: f64, delta_out: f64, delta_in: f64) -> Self {
        assert!(alpha + beta <= 1.0);
        assert!(n > 1);
        assert!(delta_out > 0.0);
        assert!(delta_in > 0.0);

        Self {
            alpha,
            alpha_plus_beta: alpha + beta,
            delta_out,
            delta_in,
            n,
            distr: Uniform::new(0.0, 1.0),
            seen_edges: FxHashSet::with_hasher(Default::default())
        }
    }
}

impl GraphGenerator for DirectedScaleFree {
    fn generate(&mut self, rng: &mut impl rand::prelude::Rng) -> Vec<(Node, Node)> {
        let mut edges = Vec::new();
        let mut in_degrees = vec![0usize; self.n];
        let mut out_degrees = vec![0usize; self.n];

        let mut cur_num_nodes = 1usize;

        let mut denom_in;
        let mut denom_out;
        let mut sampled_value;

        let choose_node = |n: usize, deg: &[usize], delta: f64, sampled_times_denom: f64| -> Node {
            let mut cumsum = 0.0;
            let mut node = 0;

            while node < n - 1 {
                cumsum += delta + deg[node] as f64;
                if sampled_times_denom < cumsum {
                    break;
                }
                node += 1;
            }

            node
        };

        while cur_num_nodes < self.n {
            denom_in = edges.len() as f64 + self.delta_in * cur_num_nodes as f64;
            denom_out = edges.len() as f64 + self.delta_out * cur_num_nodes as f64;

            sampled_value = self.distr.sample(rng);

            let (u, v) = if sampled_value < self.alpha {
                let v = choose_node(
                    cur_num_nodes,
                    &in_degrees,
                    self.delta_in,
                    denom_in * self.distr.sample(rng),
                );
                let u = cur_num_nodes as Node;

                cur_num_nodes += 1;

                (u, v)
            } else if sampled_value < self.alpha_plus_beta {
                let u = choose_node(
                    cur_num_nodes,
                    &out_degrees,
                    self.delta_out,
                    denom_out * self.distr.sample(rng),
                );
                let v = choose_node(
                    cur_num_nodes,
                    &in_degrees,
                    self.delta_in,
                    denom_in * self.distr.sample(rng),
                );

                (u, v)
            } else {
                let u = choose_node(
                    cur_num_nodes,
                    &out_degrees,
                    self.delta_out,
                    denom_out * self.distr.sample(rng),
                );
                let v = cur_num_nodes as Node;

                cur_num_nodes += 1;

                (u, v)
            };

            if !self.seen_edges.insert((u, v)) {
                continue;
            }

            out_degrees[u] += 1;
            in_degrees[v] += 1;

            edges.push((u, v));
        }
        
        edges
    }
}

/// Compute `alpha` and `beta` according to given alpha,beta,gamma and average degree
#[inline]
pub fn compute_dsf_params(
    alpha: Option<f64>,
    beta: Option<f64>,
    gamma: Option<f64>,
    avg_deg: Option<f64>,
) -> (f64, f64) {
    let (alpha, beta) = if let Some(d) = avg_deg {
        let b = (d - 1.0) / d;
        match (alpha, gamma) {
            (Some(a), _) => (a, b),
            (_, Some(g)) => (two(b, g), b),
            (_, _) => (one(b), b),
        }
    } else {
        match (alpha, beta, gamma) {
            (Some(a), Some(b), _) => (a, b),
            (Some(a), _, Some(g)) => (a, two(a, g)),
            (Some(a), _, _) => (a, one(a)),
            (_, Some(b), Some(g)) => (two(b, g), b),
            (_, Some(b), _) => (one(b), b),
            (_, _, Some(g)) => (one(g), one(g)),
            (_, _, _) => (1.0 / 3.0, 1.0 / 3.0),
        }
    };

    assert!(alpha >= 0.0 && beta >= 0.0 && alpha + beta <= 1.0);

    (alpha, beta)
}

#[inline]
fn one(p: f64) -> f64 {
    (1.0 - p) / 2.0
}

#[inline]
fn two(p1: f64, p2: f64) -> f64 {
    1.0 - p1 - p2
}
