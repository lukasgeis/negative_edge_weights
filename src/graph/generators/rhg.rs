use crate::graph::*;
use std::{
    f64::consts::{PI, TAU},
    vec,
};

use rand_distr::Uniform;

#[derive(Debug, Clone, Copy)]
struct Coord {
    id: usize,
    phi: f64,
    bid: usize,
    rad_cosh: f64,
    rad_sinh: f64,
    phi_cos: f64,
    phi_sin: f64,
}

impl PartialEq for Coord {
    fn eq(&self, other: &Self) -> bool {
        (self.bid, self.phi) == (other.bid, other.phi)
    }
}

impl PartialOrd for Coord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.bid, self.phi).partial_cmp(&(other.bid, other.phi))
    }
}

impl Eq for Coord {}

/// A RandomHyperbolicGraph-Generator for the threshold-case
///
/// TODO: rewrite code for better structure and readibility: currently just copy-pasted from
/// previous project
#[derive(Debug, Copy, Clone)]
pub struct Hyperbolic {
    /// Number of nodes
    nodes: usize,
    /// Radial dispersion
    alpha: f64,
    /// Radius of hyperbolic disk
    radius: Option<f64>,
    /// Average degree
    avg_deg: Option<f64>,
    /// Number of bands
    num_bands: Option<usize>,
    /// Probability for including two directed edges instead of an undirected one: both other
    /// directions are equally likely
    prob: f64,
    /// Uniform distrbution over [0,1]
    unif: Uniform<f64>,
}

/// Indicates which edges to include
#[derive(Debug, Copy, Clone)]
pub enum EdgeResult {
    Forward,
    Backward,
    Both,
}

impl Hyperbolic {
    /// Creates a new instance of the generator
    #[inline]
    pub fn new(
        nodes: usize,
        alpha: f64,
        radius: Option<f64>,
        avg_deg: Option<f64>,
        num_bands: Option<usize>,
        prob: f64,
    ) -> Self {
        assert!(nodes > 1);
        assert!(alpha > 0.0);

        Self {
            nodes,
            alpha,
            radius,
            avg_deg,
            num_bands,
            prob,
            unif: Uniform::new(0.0, 1.0),
        }
    }

    /// Decide which edges do add depending on `self.prob`
    #[inline]
    pub fn decide_edge(&self, rng: &mut impl Rng) -> EdgeResult {
        let sample = rng.sample(self.unif);
        if sample <= self.prob {
            return EdgeResult::Both;
        }

        let next_step = (1.0 + self.prob) / 2.0;
        if sample <= next_step {
            EdgeResult::Forward
        } else {
            EdgeResult::Backward
        }
    }
}

fn get_target_radius(n: f64, k: f64, alpha: f64) -> f64 {
    let gamma = 2.0 * alpha + 1.0;
    let xi_inv = (gamma - 2.0) / (gamma - 1.0);
    let v = k * (PI / 2.0) * xi_inv * xi_inv;
    let current_r = 2.0 * (n / v).ln();
    let mut lower_bound = current_r / 2.0;
    let mut upper_bound = current_r * 2.0;

    assert!(expected_degree(n, alpha, lower_bound) > k);
    assert!(expected_degree(n, alpha, upper_bound) < k);

    fn expected_degree(n: f64, alpha: f64, rad: f64) -> f64 {
        let gamma = 2.0 * alpha + 1.0;
        let xi = (gamma - 1.0) / (gamma - 2.0);
        let first_sum_term = (-rad / 2.0).exp();
        let second_sum_term = (-alpha * rad).exp()
            * (alpha
                * (rad / 2.0)
                * ((PI / 4.0) * (1.0 / alpha).powi(2) - (PI - 1.0) * (1.0 / alpha) + (PI - 2.0))
                - 1.0);
        (2.0 / PI) * xi * xi * n * (first_sum_term + second_sum_term)
    }

    loop {
        let current_r = (lower_bound + upper_bound) / 2.0;
        let current_k = expected_degree(n, alpha, current_r);

        if current_k < k {
            upper_bound = current_r;
        } else {
            lower_bound = current_r;
        }

        if (expected_degree(n, alpha, current_r) - k).abs() < 1e-5 {
            return current_r;
        }
    }
}

fn sample_coordinates(
    rng: &mut impl Rng,
    n: Node,
    disk_rad: f64,
    alpha: f64,
    band_limits: &[f64],
) -> (Vec<Coord>, Vec<usize>) {
    let min = 1.0_f64.next_up();
    let max = (alpha * disk_rad).cosh();
    let mut band_sizes = vec![0; band_limits.len()];
    (
        (0..n)
            .map(|id| {
                let phi = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                let rad = rng.gen_range(min..max).acosh() / alpha;
                assert!(0.0 <= rad);
                assert!(rad <= disk_rad);
                let bid = get_band_id(rad, band_limits);
                band_sizes[bid] += 1;
                Coord {
                    id,
                    phi,
                    bid,
                    rad_cosh: rad.cosh(),
                    rad_sinh: rad.sinh(),
                    phi_sin: phi.sin(),
                    phi_cos: phi.cos(),
                }
            })
            .collect(),
        band_sizes,
    )
}

#[inline]
fn get_band_id(rad: f64, band_limits: &[f64]) -> usize {
    band_limits
        .iter()
        .enumerate()
        .rev()
        .find(|(_, &limit)| rad >= limit)
        .unwrap()
        .0
}

#[inline]
fn get_band_bounds(band_sizes: &[usize]) -> Vec<usize> {
    let mut band_bounds = vec![0]; // lower bound of lowest band
    band_bounds.append(
        &mut band_sizes
            .iter()
            .scan(0, |sum, i| {
                *sum += i;
                Some(*sum)
            }) // prefix sum
            .collect::<Vec<_>>(),
    );
    band_bounds
}

fn binary_search_partition(val: f64, points: &[Coord]) -> usize {
    if points.is_empty() {
        return 0;
    }

    let mut left = 0usize;
    let mut right = points.len() - 1;

    while left < right {
        let mid = (left + right) / 2;

        if (points[mid].phi <= val && points[mid + 1].phi > val) || mid == 0 {
            return mid;
        } else if points[mid].phi > val {
            right = mid - 1;
        } else {
            left = mid + 1;
        }
    }

    left
}

fn generate_threshold_rhg(
    rhg: &Hyperbolic,
    rng: &mut impl Rng,
    band_limits: &[f64],
    band_bounds: &[usize],
    coords: &[Coord],
) -> Vec<(Node, Node)> {
    let band_cosh = band_limits.iter().map(|b| b.cosh()).collect::<Vec<f64>>();
    let radius_cosh = *band_cosh.last().unwrap();
    let band_sinh = band_limits.iter().map(|b| b.sinh()).collect::<Vec<f64>>();
    coords
        .iter()
        .flat_map(|v| {
            let mut edges = Vec::<(Node, Node)>::new();
            // `rhs_safe` is used to find the borders of the inner circle, wherein every node is definitely near enough to v.
            // It is defined hear as the inner rectangle of the current band is the outer rectangle of the next band,
            // so we can reuse the value.
            let mut rhs_safe = ((v.rad_cosh * band_cosh[v.bid] - radius_cosh)
                / (v.rad_sinh * band_sinh[v.bid]))
                .acos();
            let mut min_safe;
            let mut max_safe;
            if rhs_safe.is_nan() {
                min_safe = -1.0;
                max_safe = -1.0;
            } else {
                min_safe = if v.phi - rhs_safe >= 0.0 {
                    v.phi - rhs_safe
                } else {
                    v.phi - rhs_safe + TAU
                };
                max_safe = if v.phi + rhs_safe < TAU {
                    v.phi + rhs_safe
                } else {
                    v.phi + rhs_safe - TAU
                };
            }
            (v.bid..band_limits.len() - 1).for_each(|bid| {
                let slab = &coords[band_bounds[bid]..band_bounds[bid + 1]];
                let nodes_to_test;
                let rhs = rhs_safe;
                if rhs.is_nan() {
                    nodes_to_test = slab.iter().chain(&[]);
                } else {
                    let min = min_safe;
                    let max = max_safe;
                    let start = binary_search_partition(min, slab);
                    let end = binary_search_partition(max, slab) + 1;
                    if start < end {
                        nodes_to_test = slab[start..end].iter().chain(&[]);
                    } else {
                        nodes_to_test = slab[..end].iter().chain(&slab[start..]);
                    }
                }
                rhs_safe = ((v.rad_cosh * band_cosh[bid + 1] - radius_cosh)
                    / (v.rad_sinh * band_sinh[bid + 1]))
                    .acos();
                if rhs_safe.is_nan() {
                    min_safe = -1.0;
                    max_safe = -1.0;
                } else {
                    min_safe = if v.phi - rhs_safe >= 0.0 {
                        v.phi - rhs_safe
                    } else {
                        v.phi - rhs_safe + TAU
                    };
                    max_safe = if v.phi + rhs_safe < TAU {
                        v.phi + rhs_safe
                    } else {
                        v.phi + rhs_safe - TAU
                    };
                }
                nodes_to_test.for_each(|w| {
                    if bid > v.bid || v.id < w.id {
                        let within_inner = if min_safe <= max_safe {
                            min_safe < w.phi && w.phi < max_safe
                        } else {
                            min_safe < w.phi || w.phi < max_safe
                        };
                        if within_inner {
                            match rhg.decide_edge(rng) {
                                EdgeResult::Both => {
                                    edges.push((v.id, w.id));
                                    edges.push((w.id, v.id));
                                }
                                EdgeResult::Forward => {
                                    edges.push((v.id, w.id));
                                }
                                EdgeResult::Backward => {
                                    edges.push((w.id, v.id));
                                }
                            };
                        } else {
                            let dist_cosh = v.rad_cosh * w.rad_cosh
                                - v.rad_sinh
                                    * w.rad_sinh
                                    * (v.phi_cos * w.phi_cos + v.phi_sin * w.phi_sin);
                            if dist_cosh < radius_cosh {
                                match rhg.decide_edge(rng) {
                                    EdgeResult::Both => {
                                        edges.push((v.id, w.id));
                                        edges.push((w.id, v.id));
                                    }
                                    EdgeResult::Forward => {
                                        edges.push((v.id, w.id));
                                    }
                                    EdgeResult::Backward => {
                                        edges.push((w.id, v.id));
                                    }
                                };
                            }
                        }
                    }
                })
            });
            edges
        })
        .collect()
}

impl GraphGenerator for Hyperbolic {
    fn generate(&mut self, rng: &mut impl Rng) -> Vec<(Node, Node)> {
        let radius = if let Some(deg) = self.avg_deg {
            assert!(
                self.radius.is_none(),
                "Specify exactly one of `radius` and `avg_deg`"
            );
            assert!(deg + 1.0 < self.nodes as f64);
            get_target_radius(self.nodes as f64, deg, self.alpha)
        } else {
            self.radius
                .expect("Specify exactly one of `radius` and `avg_deg`")
        };

        let num_bands = if let Some(b) = self.num_bands {
            assert!(b > 1);
            b
        } else {
            // inspired by "Communication-free Massively Distributed Graph Generation" [Funke et al.]
            2.max((radius * self.alpha / 2.0 / std::f64::consts::LN_2).ceil() as usize)
        };

        let band_limits: Vec<f64> = [0.0, radius / 2.0]
            .into_iter()
            .chain(
                (1..num_bands)
                    .map(|i| radius / 2.0 / (num_bands - 1) as f64 * i as f64 + radius / 2.0),
            )
            .collect();

        let (mut coords, band_sizes) =
            sample_coordinates(rng, self.nodes, radius, self.alpha, &band_limits);
        coords.sort_unstable_by(|u, v| u.partial_cmp(v).unwrap());
        let band_bounds = get_band_bounds(&band_sizes);

        generate_threshold_rhg(self, rng, &band_limits, &band_bounds, &coords)
    }
}
