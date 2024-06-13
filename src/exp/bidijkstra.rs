use crate::{
    bidijkstra::search::VisitedDistances, bidijkstra::Graph, graph::*, utils::*, weight::Weight,
};

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

        #[cfg(feature = "insertions")]
        let mut num_insertions = 2usize;

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
                                #[cfg(feature = "insertions")]
                                println!("{num_insertions},rej,bd");
                                return None;
                            }
                            Some(true) => {
                                self.heapf.push(cost, succ);
                                #[cfg(feature = "insertions")]
                                {
                                    num_insertions += 1;
                                }
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
                                #[cfg(feature = "insertions")]
                                println!("{num_insertions},rej,bd");
                                return None;
                            }
                            Some(true) => {
                                self.heapb.push(cost, pred);
                                #[cfg(feature = "insertions")]
                                {
                                    num_insertions += 1;
                                }
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

        #[cfg(feature = "insertions")]
        println!("{num_insertions},acc,bd");

        Some(((df, db), self.visit_states.get_distances()))
    }
}
