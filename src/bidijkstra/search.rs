use crate::{graph::*, utils::*, weight::Weight};

use super::Graph;

/// Keep track of all VisitStates
#[derive(Debug, Clone)]
pub struct VisitedDistances<W: Weight> {
    /// VisitStates of all nodes: first entry is the tentative distance in the forward-search,
    /// second entry the tentative distance in the backward-search. An entry of -1 means that the
    /// node was visited in the other search
    visit_map: Vec<(W, W)>,
    /// Vector of all seen nodes: might be faster for `o(n)` nodes
    seen_nodes: ReusableVec<Node>,
}

impl<W: Weight> VisitedDistances<W> {
    /// Creates a new instance
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            visit_map: vec![(W::MAX, W::MAX); n],
            seen_nodes: ReusableVec::with_capacity(n),
        }
    }

    /// Visits a node in the forward-search
    #[inline]
    pub fn visit_node_forward(&mut self, node: Node) {
        self.visit_map[node].1 = -W::one()
    }

    /// Visits a node in the backward-search
    #[inline]
    pub fn visit_node_backward(&mut self, node: Node) {
        self.visit_map[node].0 = -W::one()
    }

    /// Returns *true* if the node has been visited in the forward-search
    #[inline]
    pub fn is_visited_forward(&self, node: Node, dist: W) -> bool {
        self.visit_map[node].0 < dist
    }

    /// Returns *true* if the node has been visited in the backward-search
    #[inline]
    pub fn is_visited_backward(&self, node: Node, dist: W) -> bool {
        self.visit_map[node].1 < dist
    }

    /// Queues a node in the forward-search
    ///
    /// Returns `Some(bool)` if the queue was allowed and did go through/did not go through.
    /// Returns `None` if we have found a negative weight cycle
    #[inline]
    pub fn queue_node_forward(&mut self, node: Node, distance: W, max_distance: W) -> Option<bool> {
        if distance < self.visit_map[node].0 {
            if self.visit_map[node].1 < W::MAX && distance + self.visit_map[node].1 < max_distance {
                return None;
            }

            if self.visit_map[node] == (W::MAX, W::MAX) {
                self.seen_nodes.push(node);
            }
            self.visit_map[node].0 = distance;
            Some(true)
        } else {
            Some(false)
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
        if distance < self.visit_map[node].1 {
            if self.visit_map[node].0 < W::MAX && distance + self.visit_map[node].0 < max_distance {
                return None;
            }

            if self.visit_map[node] == (W::MAX, W::MAX) {
                self.seen_nodes.push(node);
            }
            self.visit_map[node].1 = distance;
            Some(true)
        } else {
            Some(false)
        }
    }

    /// Resets the data structure
    #[inline]
    pub fn reset(&mut self) {
        if self.seen_nodes.is_asymptotically_full() {
            self.seen_nodes.clear();
            self.visit_map
                .iter_mut()
                .for_each(|w| *w = (W::MAX, W::MAX));
        } else {
            self.seen_nodes
                .iter()
                .for_each(|u| self.visit_map[*u] = (W::MAX, W::MAX));
            self.seen_nodes.clear();
        }
    }

    /// Returns the node-distance pairs of all visited nodes.
    /// For nodes visited in the backward-search, we set the node-value to `node + n`
    pub fn get_distances(&mut self) -> impl Iterator<Item = (Node, W)> + '_ {
        if self.seen_nodes.is_asymptotically_full() {
            DoubleIterator::IterA(self.visit_map.iter().enumerate().filter_map(|(u, s)| {
                if s.0 == -W::one() {
                    Some((u + self.visit_map.len(), s.1))
                } else if s.1 == -W::one() {
                    Some((u, s.0))
                } else {
                    None
                }
            }))
        } else {
            DoubleIterator::IterB(self.seen_nodes.iter().filter_map(|u| {
                if self.visit_map[*u].0 == -W::one() {
                    Some((*u + self.visit_map.len(), self.visit_map[*u].1))
                } else if self.visit_map[*u].1 == -W::one() {
                    Some((*u, self.visit_map[*u].0))
                } else {
                    None
                }
            }))
        }
    }
}

/// Bidirectional Dijkstra
pub struct BiDijkstra<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    /// The Maxheap for the forward-search
    heapf: RadixHeap<W, Node>,
    /// The Maxheap for the backward-search
    heapb: RadixHeap<W, Node>,
    /// The VisitStates of all nodes
    visit_states: VisitedDistances<W>,
}

impl<W> BiDijkstra<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    /// Creates a new instance
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heapf: RadixHeap::new(),
            heapb: RadixHeap::new(),
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

        self.heapf.push(W::zero(), source_node);
        self.heapb.push(W::zero(), target_node);

        let (mut df, mut db) = (W::zero(), W::zero());

        loop {
            if let Some((dist, heapf_node)) = self.heapf.pop() {
                df = dist;
                if df + db >= max_distance {
                    df = max_distance - db;
                    break;
                }
                if !self.visit_states.is_visited_forward(heapf_node, dist) {
                    self.visit_states.visit_node_forward(heapf_node);
                    for edge in graph.out_neighbors(heapf_node) {
                        let succ = edge.target;
                        let mut cost = dist + graph.potential_weight(*edge);
                        cost.round_up(self.heapf.top());
                        match self
                            .visit_states
                            .queue_node_forward(succ, cost, max_distance)
                        {
                            None => {
                                return None;
                            }
                            Some(true) => {
                                self.heapf.push(cost, succ);
                            }
                            _ => (),
                        };
                    }
                }
            }

            if let Some((dist, heapb_node)) = self.heapb.pop() {
                db = dist;
                if df + db >= max_distance {
                    db = max_distance - df;
                    break;
                }

                if !self.visit_states.is_visited_backward(heapb_node, dist) {
                    self.visit_states.visit_node_backward(heapb_node);
                    for edge in graph.in_neighbors(heapb_node) {
                        let pred = edge.source;
                        let mut cost = dist + graph.potential_weight(*edge);
                        cost.round_up(self.heapb.top());
                        match self
                            .visit_states
                            .queue_node_backward(pred, cost, max_distance)
                        {
                            None => {
                                return None;
                            }
                            Some(true) => {
                                self.heapb.push(cost, pred);
                            }
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
