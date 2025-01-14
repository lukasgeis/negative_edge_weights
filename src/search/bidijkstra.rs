use crate::{graph::*, logger::Logger, utils::*, weight::Weight};

/// Keep track of all VisitStates
#[derive(Debug, Clone)]
pub struct VisitedDistances<W: Weight> {
    /// VisitStates of all nodes: first entry is the tentative distance in the forward-search,
    /// second entry the tentative distance in the backward-search. An entry of -1 means that the
    /// node was visited in the other search
    visit_map: Vec<(W, W)>,
    /// Vector of all seen nodes: might be faster for `o(n)` nodes
    seen_nodes: ReusableVec<Node>,
    /// Max Distance of the current BiDijkstra-Run
    max_distance: W,
}

impl<W: Weight> VisitedDistances<W> {
    /// Creates a new instance
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            visit_map: vec![(W::MAX, W::MAX); n],
            seen_nodes: ReusableVec::with_capacity(n),
            max_distance: W::MAX,
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
    pub fn queue_node_forward(&mut self, node: Node, distance: W) -> Option<bool> {
        if distance < self.visit_map[node].0 {
            if self.visit_map[node].1 < W::MAX
                && distance + self.visit_map[node].1 < self.max_distance
            {
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
    pub fn queue_node_backward(&mut self, node: Node, distance: W) -> Option<bool> {
        if distance < self.visit_map[node].1 {
            if self.visit_map[node].0 < W::MAX
                && distance + self.visit_map[node].0 < self.max_distance
            {
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
    pub fn reset(&mut self, max_distance: W) {
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
        self.max_distance = max_distance;
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
pub struct BiDijkstra<W: Weight>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    /// The Minheap for the forward-search
    heapf: RadixHeap<W, Node>,
    /// The Minheap for the backward-search
    heapb: RadixHeap<W, Node>,
    /// The VisitStates of all nodes
    visit_states: VisitedDistances<W>,
}

impl<W: Weight> BiDijkstra<W>
where
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
    pub fn run<L: Logger<W>>(
        &mut self,
        graph: &Graph<W>,
        source_node: Node,
        target_node: Node,
        max_distance: W,
        logger: &mut L,
    ) -> Option<((W, W), impl Iterator<Item = (Node, W)> + '_)> {
        if source_node == target_node {
            return None;
        }

        self.visit_states.reset(max_distance);
        self.heapf.clear();
        self.heapb.clear();

        self.visit_states.queue_node_forward(source_node, W::zero());
        self.visit_states
            .queue_node_backward(target_node, W::zero());

        self.heapf.push(W::zero(), source_node);
        self.heapb.push(W::zero(), target_node);

        logger.add_insertion_bd_f();
        logger.add_insertion_bd_b();

        let (mut df, mut db);
        db = W::zero();

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
                        let mut cost = dist + graph.pot_weight(*edge);
                        if cost >= max_distance - db {
                            continue;
                        }

                        cost.round_up(self.heapf.top());
                        match self.visit_states.queue_node_forward(succ, cost) {
                            None => {
                                return None;
                            }
                            Some(true) => {
                                self.heapf.push(cost, succ);
                                logger.add_insertion_bd_f();
                            }
                            _ => (),
                        };
                    }
                }
            } else {
                df = max_distance - db;
                break;
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
                        let mut cost = dist + graph.pot_weight(*edge);
                        if cost >= max_distance - df {
                            continue;
                        }

                        cost.round_up(self.heapb.top());
                        match self.visit_states.queue_node_backward(pred, cost) {
                            None => {
                                return None;
                            }
                            Some(true) => {
                                self.heapb.push(cost, pred);
                                logger.add_insertion_bd_b();
                            }
                            _ => (),
                        };
                    }
                }
            } else {
                db = max_distance - df;
                break;
            }
        }

        Some(((df, db), self.visit_states.get_distances()))
    }
}
