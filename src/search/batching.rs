use std::sync::atomic::{AtomicU8, Ordering};

use array_init::array_init;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::{
    graph::{Edge, Graph, Node},
    search::bidijkstra::VisitedDistances,
    utils::{CappedVec, DoubleIterator, RadixHeap, ReusableVec},
    weight::Weight,
};

/// Rough implementation of a batched parallel bidirectional search.
///
/// Run multiple bidirectional searches at the same time from multiple edges.
/// If at some point, a node is visited by two searches, terminate all searches bigger than the
/// smaller one.
pub struct BatchedBiDijkstra<W: Weight, const MAX_THREADS: usize = 8>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    heaps_forward: [RadixHeap<W, Node>; MAX_THREADS],
    heaps_backward: [RadixHeap<W, Node>; MAX_THREADS],
    visit_states: [VisitedDistances<W>; MAX_THREADS],
    lowest_conflict: AtomicU8,
    nodes_found: Vec<AtomicU8>,
    pot_updates: Vec<W>,
    pot_nodes: ReusableVec<Node, MAX_THREADS>,
}

impl<W: Weight, const MAX_THREADS: usize> BatchedBiDijkstra<W, MAX_THREADS>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    /// Creates a new instance
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heaps_forward: array_init(|_| RadixHeap::new()),
            heaps_backward: array_init(|_| RadixHeap::new()),
            visit_states: array_init(|_| VisitedDistances::new(n)),
            lowest_conflict: AtomicU8::new(MAX_THREADS as u8),
            nodes_found: (0..n).map(|_| AtomicU8::new(MAX_THREADS as u8)).collect(),
            pot_updates: vec![W::MAX; n],
            pot_nodes: ReusableVec::with_capacity(n),
        }
    }

    /// Runs the algorithm on `num_edges < edges.size()` many edges
    pub fn run(
        &mut self,
        graph: &Graph<W>,
        edges: CappedVec<Edge<W>, MAX_THREADS>,
        num_edges: usize,
    ) -> (
        CappedVec<bool, MAX_THREADS>,
        impl Iterator<Item = (Node, W)> + '_,
    ) {
        self.lowest_conflict = AtomicU8::new(num_edges as u8);
        self.nodes_found
            .par_iter_mut()
            .for_each(|n| *n = AtomicU8::new(MAX_THREADS as u8));
        let mut distances = [(W::zero(), W::zero()); MAX_THREADS];
        let mut accepted = [false; MAX_THREADS];

        macro_rules! get_lowest_conflict {
            () => {
                self.lowest_conflict.load(Ordering::Acquire) as usize
            };
        }

        edges[..num_edges]
            .into_par_iter()
            .zip(self.heaps_forward.par_iter_mut())
            .zip(self.heaps_backward.par_iter_mut())
            .zip(self.visit_states.par_iter_mut())
            .zip(distances.par_iter_mut())
            .zip(accepted.par_iter_mut())
            .enumerate()
            .for_each(
                |(
                    i,
                    (
                        (
                            (
                                (
                                    (
                                        Edge {
                                            source,
                                            target,
                                            weight,
                                        },
                                        heapf,
                                    ),
                                    heapb,
                                ),
                                visit_states,
                            ),
                            (df, db),
                        ),
                        acc,
                    ),
                )| {
                    if get_lowest_conflict!() <= i {
                        return;
                    }
                    let i = i as u8;
                    let source = *source;
                    let target = *target;
                    let max_distance = *weight;

                    let prev_source_found =
                        self.nodes_found[source as usize].fetch_min(i, Ordering::AcqRel);

                    if prev_source_found < i {
                        self.lowest_conflict.fetch_min(i, Ordering::AcqRel);
                        return;
                    } else if prev_source_found != i {
                        self.lowest_conflict
                            .fetch_min(prev_source_found, Ordering::AcqRel);
                    }

                    if source == target {
                        if max_distance < W::zero() {
                            *acc = true;

                            visit_states.reset(max_distance);
                        }
                        return;
                    } else {
                        let prev_target_found =
                            self.nodes_found[target as usize].fetch_min(i, Ordering::AcqRel);

                        if prev_target_found < i {
                            self.lowest_conflict.fetch_min(i, Ordering::AcqRel);
                            return;
                        } else if prev_target_found != i {
                            self.lowest_conflict
                                .fetch_min(prev_target_found, Ordering::AcqRel);
                        }
                    }

                    visit_states.reset(max_distance);

                    if max_distance <= W::zero() {
                        *acc = true;
                        return;
                    }

                    heapf.clear();
                    heapb.clear();

                    visit_states.queue_node_forward(source, W::zero());
                    visit_states.queue_node_backward(target, W::zero());

                    heapf.push(W::zero(), source);
                    heapb.push(W::zero(), target);

                    loop {
                        if get_lowest_conflict!() as u8 <= i {
                            return;
                        }

                        if let Some((dist, heapf_node)) = heapf.pop() {
                            *df = dist;
                            if *df + *db >= max_distance {
                                *df = max_distance - *db;
                                break;
                            }

                            if !visit_states.is_visited_forward(heapf_node, dist) {
                                visit_states.visit_node_forward(heapf_node);
                                for edge in graph.out_neighbors(heapf_node) {
                                    let succ = edge.target;

                                    let prev_succ_found = self.nodes_found[succ as usize]
                                        .fetch_min(i, Ordering::AcqRel);
                                    if prev_succ_found < i {
                                        self.lowest_conflict.fetch_min(i, Ordering::AcqRel);
                                        return;
                                    } else if prev_succ_found != i {
                                        self.lowest_conflict
                                            .fetch_min(prev_succ_found, Ordering::AcqRel);
                                    }

                                    let mut cost = dist + graph.pot_weight(*edge);
                                    if cost >= max_distance - *db {
                                        continue;
                                    }

                                    cost.round_up(heapf.top());
                                    match visit_states.queue_node_forward(succ, cost) {
                                        None => {
                                            return;
                                        }
                                        Some(true) => {
                                            heapf.push(cost, succ);
                                        }
                                        _ => {}
                                    };
                                }
                            }
                        } else {
                            *df = max_distance - *db;
                            break;
                        }

                        if let Some((dist, heapb_node)) = heapb.pop() {
                            *db = dist;
                            if *df + *db >= max_distance {
                                *db = max_distance - *df;
                                break;
                            }

                            if !visit_states.is_visited_backward(heapb_node, dist) {
                                visit_states.visit_node_backward(heapb_node);
                                for edge in graph.in_neighbors(heapb_node) {
                                    let pred = edge.source;

                                    let prev_pred_found = self.nodes_found[pred as usize]
                                        .fetch_min(i, Ordering::AcqRel);
                                    if prev_pred_found < i {
                                        self.lowest_conflict.fetch_min(i, Ordering::AcqRel);
                                        return;
                                    } else if prev_pred_found != i {
                                        self.lowest_conflict
                                            .fetch_min(prev_pred_found, Ordering::AcqRel);
                                    }

                                    let mut cost = dist + graph.pot_weight(*edge);
                                    if cost >= max_distance - *df {
                                        continue;
                                    }

                                    cost.round_up(heapb.top());
                                    match visit_states.queue_node_backward(pred, cost) {
                                        None => {
                                            return;
                                        }
                                        Some(true) => {
                                            heapb.push(cost, pred);
                                        }
                                        _ => {}
                                    };
                                }
                            }
                        } else {
                            *db = max_distance - *df;
                            break;
                        }
                    }

                    *acc = true;
                },
            );

        let cap = get_lowest_conflict!();

        let accepted = CappedVec {
            data: accepted,
            size: cap,
        };

        self.reset_pot_updates();
        (0..cap).filter(|i| accepted.data[*i]).for_each(|i| {
            self.visit_states[i].get_distances().for_each(|(n, w)| {
                let node = n % graph.n();
                if self.pot_updates[node] == W::MAX {
                    self.pot_nodes.push(node);
                    self.pot_updates[node] = W::zero();
                }

                if n < graph.n() {
                    self.pot_updates[node] += distances[i].0 - w;
                } else {
                    self.pot_updates[node] += w - distances[i].1;
                }
            })
        });

        (accepted, self.get_pot_updates())
    }

    /// Reset the tentative potential changes
    fn reset_pot_updates(&mut self) {
        if self.pot_nodes.is_asymptotically_full() {
            self.pot_nodes.clear();
            self.pot_updates.par_iter_mut().for_each(|w| *w = W::MAX);
        } else {
            self.pot_nodes
                .iter()
                .for_each(|u| self.pot_updates[*u] = W::MAX);
            self.pot_nodes.clear();
        }
    }

    /// Get the potential changes
    fn get_pot_updates(&mut self) -> impl Iterator<Item = (Node, W)> + '_ {
        if self.pot_nodes.is_asymptotically_full() {
            DoubleIterator::IterA(
                self.pot_updates
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(|(_, d)| *d != W::MAX),
            )
        } else {
            DoubleIterator::IterB(self.pot_nodes.iter().map(|n| (*n, self.pot_updates[*n])))
        }
    }
}
