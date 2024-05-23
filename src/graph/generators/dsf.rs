use rand_distr::{Distribution, Uniform};

use crate::{graph::*, weight::Weight};

/// The directed scale-free model
pub struct DirectedScaleFree {
    alpha: f64,
    alpha_plus_beta: f64,
    delta_out: f64,
    delta_in: f64,
    n: usize,
    distr: Uniform<f64>,
}

impl DirectedScaleFree {
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
        }
    }
}

impl<W: Weight> GraphGenerator<W> for DirectedScaleFree {
    fn generate(&mut self, rng: &mut impl rand::prelude::Rng, default_weight: W) -> Vec<Edge<W>> {
        let mut edges = Vec::new();
        let mut in_degrees = vec![0usize; self.n];
        let mut out_degrees = vec![0usize; self.n];

        let mut cur_num_nodes = 1usize;

        let mut denom_in = 0.0;
        let mut denom_out = 0.0;

        let mut sampled_value = 0.0;

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

            out_degrees[u] += 1;
            in_degrees[v] += 1;

            edges.push((u, v, default_weight));
        }

        edges
    }
}
