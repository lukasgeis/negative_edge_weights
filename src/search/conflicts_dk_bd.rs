use crate::{
    graph::*,
    logger::ConflictLogger,
    search::{bidijkstra::VisitedDistances as VisDisBD, dijkstra::VisitedDistances as VisDisDK},
    utils::RadixHeap,
    weight::Weight,
};

/// Bidirectional Dijkstra for conflict logging
pub struct BiDijkstra<W: Weight>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    /// The Maxheap for the forward-search
    heapf: RadixHeap<W, Node>,
    /// The Maxheap for the backward-search
    heapb: RadixHeap<W, Node>,
    /// The VisitStates of all nodes
    visit_states: VisDisBD<W>,
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
            visit_states: VisDisBD::new(n),
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
        logger: &mut ConflictLogger,
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

                        logger.see_node_bd(succ);

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

                        logger.see_node_bd(pred);

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

/// Dijkstra for conflict logging
pub struct Dijkstra<W: Weight>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    /// MinHeap
    heap: RadixHeap<W, Node>,

    /// Stores which nodes have already been visited in which total distance
    visit_states: VisDisDK<W>,

    /// Stack to keep track of nodes that can be visited directly without using the heap
    zero_nodes: Vec<Node>,
}

impl<W: Weight> Dijkstra<W>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    /// Initializes Dijkstra for a graph with `n` nodes
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heap: RadixHeap::new(),
            visit_states: VisDisDK::new(n),
            zero_nodes: Vec::new(),
        }
    }

    /// Runs dijkstra on the given graph from `source_node` until either
    /// (1) All nodes with total distance <= `max_distance` have been found
    /// (2) `target_node` is found with total distance <= `max_distance`
    ///
    /// In case (1) return `Some(SP)` where `SP` is an iterator over the shortest path tree found
    /// by dijkstra. In case (2) return `None`.
    pub fn run(
        &mut self,
        graph: &Graph<W>,
        source_node: Node,
        target_node: Node,
        max_distance: W,
        logger: &mut ConflictLogger,
    ) -> Option<impl Iterator<Item = (Node, W)> + '_> {
        if source_node == target_node {
            return None;
        }

        self.visit_states.reset();
        self.heap.clear();
        self.zero_nodes.clear();

        self.visit_states.queue_node(source_node, W::zero());
        self.heap.push(W::zero(), source_node);

        while let Some((dist, heap_node)) = self.heap.pop() {
            if self.visit_states.is_visited(heap_node, dist) {
                continue;
            }
            self.zero_nodes.push(heap_node);

            while let Some(node) = self.zero_nodes.pop() {
                for edge in graph.out_neighbors(node) {
                    let succ = edge.target;

                    logger.see_node_dk(succ);

                    let next = graph.pot_weight(*edge);
                    if next <= W::zero() && self.visit_states.queue_node(succ, dist) {
                        if succ == target_node && dist < max_distance {
                            return None;
                        }

                        self.zero_nodes.push(succ);
                        continue;
                    }

                    let mut cost = dist + next;
                    if cost > max_distance {
                        continue;
                    }

                    if succ == target_node && cost < max_distance {
                        return None;
                    }

                    cost.round_up(self.heap.top());
                    if self.visit_states.queue_node(succ, cost) {
                        self.heap.push(cost, succ);
                    }
                }
            }
        }

        Some(self.visit_states.get_distances())
    }
}
