use radix_heap::RadixHeapMap;

use crate::{graph::*, weight::Weight};

#[derive(Debug, Clone, Copy, PartialEq)]
enum VisitState<W: Weight> {
    Unvisited,
    QueuedForward(W),
    QueuedBackward(W),
    DoubleQueued(W, W),
    VisitedForward(W),
    VisitedBackward(W),
}

#[derive(Debug, Clone)]
struct VisitedDistances<W: Weight> {
    visit_map: Vec<VisitState<W>>,
    seen_nodes: Vec<Node>,
}

impl<W: Weight> VisitedDistances<W> {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            visit_map: vec![VisitState::Unvisited; n],
            seen_nodes: Vec::new(),
        }
    }

    #[inline]
    pub fn visit_node_forward(&mut self, node: Node) {
        match self.visit_map[node] {
            VisitState::QueuedForward(dist) => {
                self.visit_map[node] = VisitState::VisitedForward(dist)
            }
            VisitState::DoubleQueued(dist, _) => {
                self.visit_map[node] = VisitState::VisitedForward(dist)
            }
            _ => (),
        };
    }

    #[inline]
    pub fn visit_node_backward(&mut self, node: Node) {
        match self.visit_map[node] {
            VisitState::QueuedBackward(dist) => {
                self.visit_map[node] = VisitState::VisitedBackward(dist)
            }
            VisitState::DoubleQueued(_, dist) => {
                self.visit_map[node] = VisitState::VisitedBackward(dist)
            }
            _ => (),
        };
    }

    #[inline]
    pub fn is_visited(&self, node: Node) -> bool {
        matches!(
            self.visit_map[node],
            VisitState::VisitedForward(_) | VisitState::VisitedBackward(_)
        )
    }

    #[inline]
    pub fn queue_node_forward(&mut self, node: Node, distance: W, max_distance: W) -> Option<bool> {
        match self.visit_map[node] {
            VisitState::Unvisited => {
                self.visit_map[node] = VisitState::QueuedForward(distance);
                self.seen_nodes.push(node);
                Some(true)
            }
            VisitState::QueuedForward(dist) => {
                if distance < dist {
                    self.visit_map[node] = VisitState::QueuedForward(distance);
                    Some(true)
                } else {
                    Some(false)
                }
            }
            VisitState::QueuedBackward(dist) => {
                if dist + distance < max_distance {
                    return None;
                }

                self.visit_map[node] = VisitState::DoubleQueued(distance, dist);
                Some(true)
            }
            VisitState::DoubleQueued(distf, distb) => {
                if distance >= distf {
                    return Some(false);
                }

                if distance + distb < max_distance {
                    return None;
                }

                self.visit_map[node] = VisitState::DoubleQueued(distance, distb);
                Some(true)
            }
            VisitState::VisitedBackward(dist) => {
                if dist + distance < max_distance {
                    return None;
                }

                Some(false)
            }
            _ => Some(false),
        }
    }

    #[inline]
    pub fn queue_node_backward(
        &mut self,
        node: Node,
        distance: W,
        max_distance: W,
    ) -> Option<bool> {
        match self.visit_map[node] {
            VisitState::Unvisited => {
                self.visit_map[node] = VisitState::QueuedBackward(distance);
                self.seen_nodes.push(node);
                Some(true)
            }
            VisitState::QueuedBackward(dist) => {
                if distance < dist {
                    self.visit_map[node] = VisitState::QueuedBackward(distance);
                    Some(true)
                } else {
                    Some(false)
                }
            }
            VisitState::QueuedForward(dist) => {
                if dist + distance < max_distance {
                    return None;
                }

                self.visit_map[node] = VisitState::DoubleQueued(dist, distance);
                Some(true)
            }
            VisitState::DoubleQueued(distf, distb) => {
                if distance >= distb {
                    return Some(false);
                }

                if distance + distf < max_distance {
                    return None;
                }

                self.visit_map[node] = VisitState::DoubleQueued(distf, distance);
                Some(true)
            }
            VisitState::VisitedForward(dist) => {
                if dist + distance < max_distance {
                    return None;
                }

                Some(false)
            }
            _ => Some(false),
        }
    }

    /// Returns *true* if we have seen `Omega(n)` nodes
    #[inline]
    fn is_asymptotically_full(&self) -> bool {
        self.seen_nodes.len() > self.visit_map.len() / 4
    }

    /// Resets the data structure
    #[inline]
    pub fn reset(&mut self) {
        if self.is_asymptotically_full() {
            self.seen_nodes.clear();
            self.visit_map
                .iter_mut()
                .for_each(|w| *w = VisitState::Unvisited);
        } else {
            self.seen_nodes
                .iter()
                .for_each(|u| self.visit_map[*u] = VisitState::Unvisited);
            self.seen_nodes.clear();
        }
    }

    pub fn get_distances(&mut self) -> impl Iterator<Item = (Node, W)> + '_ {
        if self.is_asymptotically_full() {
            DoubleIterator::IterA(
                self.visit_map
                    .iter()
                    .enumerate()
                    .filter_map(|(u, s)| match s {
                        VisitState::VisitedForward(w) => Some((u, *w)),
                        VisitState::VisitedBackward(w) => Some((u + self.visit_map.len(), *w)),
                        _ => None,
                    }),
            )
        } else {
            DoubleIterator::IterB(
                self.seen_nodes
                    .iter()
                    .filter_map(|u| match self.visit_map[*u] {
                        VisitState::VisitedForward(w) => Some((*u, w)),
                        VisitState::VisitedBackward(w) => Some((*u + self.visit_map.len(), w)),
                        _ => None,
                    }),
            )
        }
    }
}

/// Quick hack to allow a function to return two different iterators over the same item
pub enum DoubleIterator<I, A, B>
where
    A: Iterator<Item = I>,
    B: Iterator<Item = I>,
{
    IterA(A),
    IterB(B),
}

impl<I, A, B> Iterator for DoubleIterator<I, A, B>
where
    A: Iterator<Item = I>,
    B: Iterator<Item = I>,
{
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DoubleIterator::IterA(iter) => iter.next(),
            DoubleIterator::IterB(iter) => iter.next(),
        }
    }
}

pub struct BiDijkstra<W: Weight> {
    heapf: RadixHeapMap<<W as Weight>::RadixWeight, Node>,
    heapb: RadixHeapMap<<W as Weight>::RadixWeight, Node>,
    visit_states: VisitedDistances<W>,
}

impl<W: Weight> BiDijkstra<W> {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heapf: RadixHeapMap::new(),
            heapb: RadixHeapMap::new(),
            visit_states: VisitedDistances::new(n),
        }
    }

    pub fn run(
        &mut self,
        graph: &Graph<W>,
        source_node: Node,
        target_node: Node,
        max_distance: W,
    ) -> Option<((W, W), impl Iterator<Item = (Node, W)> + '_)> {
        if source_node == target_node {
            return None;
        }

        self.visit_states.reset();
        self.heapf.clear();
        self.heapb.clear();

        self.visit_states
            .queue_node_forward(source_node, W::zero(), max_distance);
        self.visit_states
            .queue_node_backward(target_node, W::zero(), max_distance);

        self.heapf.push(W::to_radix(W::zero()), source_node);
        self.heapb.push(W::to_radix(W::zero()), target_node);

        let (mut df, mut db) = (W::zero(), W::zero());

        loop {
            if let Some((dist, heapf_node)) = self.heapf.pop() {
                let dist = W::from_radix(dist);

                df = dist;
                if df + db >= max_distance {
                    df = max_distance - db;
                    break;
                }
                if !self.visit_states.is_visited(heapf_node) {
                    self.visit_states.visit_node_forward(heapf_node);

                    for (_, succ, weight) in graph.neighbors(heapf_node) {
                        let succ = *succ;
                        let mut cost = dist + graph.potential_weight((heapf_node, succ, *weight));
                        let top = W::from_radix(self.heapf.top().unwrap());
                        cost.round_up(top);
                        match self
                            .visit_states
                            .queue_node_forward(succ, cost, max_distance)
                        {
                            None => return None,
                            Some(true) => self.heapf.push(W::to_radix(cost), succ),
                            _ => (),
                        };
                    }
                }
            }

            if let Some((dist, heapb_node)) = self.heapb.pop() {
                let dist = W::from_radix(dist);

                db = dist;
                if df + db >= max_distance {
                    db = max_distance - df;
                    break;
                }

                if !self.visit_states.is_visited(heapb_node) {
                    self.visit_states.visit_node_backward(heapb_node);

                    for (pred, _, weight) in graph.in_neighbors(heapb_node) {
                        let pred = *pred;
                        let mut cost = dist + graph.potential_weight((pred, heapb_node, *weight));
                        let top = W::from_radix(self.heapb.top().unwrap());
                        cost.round_up(top);

                        match self
                            .visit_states
                            .queue_node_backward(pred, cost, max_distance)
                        {
                            None => return None,
                            Some(true) => self.heapb.push(W::to_radix(cost), pred),
                            _ => (),
                        };
                    }
                }
            }
        }

        Some(((df, db), self.visit_states.get_distances()))
    }
}
