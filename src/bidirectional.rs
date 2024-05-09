use radix_heap::RadixHeapMap;

use crate::{graph::*, utils::*, weight::Weight};

/// Possible VisitStates of a single node in the bidirectional search
#[derive(Debug, Clone, Copy, PartialEq)]
enum VisitState<W: Weight> {
    /// The node has not been found in either search
    Unvisited,
    /// The node has been queued in the forward-search
    QueuedForward(W),
    /// The node has been found in the backward-search
    QueuedBackward(W),
    /// The node has been queued in both searches    
    DoubleQueued(W, W),
    /// The node has been visited in the forward-search
    VisitedForward(W),
    /// The node has been visited in the backward-search
    VisitedBackward(W),
}

/// Keep track of all VisitStates
#[derive(Debug, Clone)]
struct VisitedDistances<W: Weight> {
    /// VisitStates of all nodes
    visit_map: Vec<VisitState<W>>,
    /// Vector of all seen nodes: might be faster for `o(n)` nodes
    seen_nodes: ReusableVec<Node>,
}

impl<W: Weight> VisitedDistances<W> {
    /// Creates a new instance
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            visit_map: vec![VisitState::Unvisited; n],
            seen_nodes: ReusableVec::with_capacity(n),
        }
    }

    /// Visits a node in the forward-search
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

    /// Visits a node in the backward-search
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

    /// Returns *true* if the node has been visited in any search
    #[inline]
    pub fn is_visited(&self, node: Node) -> bool {
        matches!(
            self.visit_map[node],
            VisitState::VisitedForward(_) | VisitState::VisitedBackward(_)
        )
    }

    /// Queues a node in the forward-search
    ///
    /// Returns `Some(bool)` if the queue was allowed and did go through/did not go through.
    /// Returns `None` if we have found a negative weight cycle
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

    /// Queues a node in the backward-search
    ///
    /// Returns `Some(bool)` if the queue was allowed and did go through/did not go through.
    /// Returns `None` if we have found a negative weight cycle
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

    /// Returns the node-distance pairs of all visited nodes.
    /// For nodes visited in the backward-search, we set the node-value to `node + n`
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

/// Bidirectional Dijkstra
pub struct BiDijkstra<W: Weight> {
    /// The Maxheap for the forward-search
    heapf: RadixHeapMap<<W as Weight>::RadixWeight, Node>,
    /// The Maxheap for the backward-search
    heapb: RadixHeapMap<<W as Weight>::RadixWeight, Node>,
    /// The VisitStates of all nodes
    visit_states: VisitedDistances<W>,
}

impl<W: Weight> BiDijkstra<W> {
    /// Creates a new instance
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heapf: RadixHeapMap::new(),
            heapb: RadixHeapMap::new(),
            visit_states: VisitedDistances::new(n),
        }
    }

    /// Runs bidirectional dijkstra on the given graph.
    ///
    /// Returns `None` if there exists a path from `source_node` to `target_node` with distance
    /// less than `max_distance`.
    /// Otherwise, return `Some(((df, db), it))` where `df` is the maximum visited distance in the
    /// forward-search, `db` the maximum visited distance in the backward-search and `it` an
    /// iterator over the node-distance pairs in the shortest path trees
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

            if self.heapf.is_empty() && self.heapb.is_empty() {
                df = max_distance - db;
                break;
            }
        }

        Some(((df, db), self.visit_states.get_distances()))
    }
}
