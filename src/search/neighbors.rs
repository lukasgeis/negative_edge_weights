use ez_bitset::bitset::BitSet;

use crate::{graph::*, logger::NeighborSeqLogger, utils::RadixHeap, weight::Weight};

use super::dijkstra::VisitedDistances;

/// Dijkstra from one source to multiple targets with a fixed max-length for each target
pub struct MultiDijkstra<W: Weight>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    heap: RadixHeap<W, Node>,
    visit_states: VisitedDistances<W>,
    zero_nodes: Vec<Node>,
    max_distances: Vec<W>,
    accepted: BitSet,
    last_nodes: Vec<usize>,
}

impl<W: Weight> MultiDijkstra<W>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    /// Create a new instance
    pub fn new(n: usize, d: usize) -> Self {
        Self {
            heap: RadixHeap::new(),
            visit_states: VisitedDistances::new(n),
            zero_nodes: Vec::new(),
            max_distances: vec![W::MIN; n],
            accepted: BitSet::new_all_set(d),
            last_nodes: Vec::with_capacity(d),
        }
    }

    /// Run the search
    pub fn run(
        &mut self,
        graph: &Graph<W>,
        source: Node,
        targets: &[(Node, W, W)],
        max_distance: W,
        logger: &mut NeighborSeqLogger,
    ) -> (
        impl Iterator<Item = usize> + '_,
        impl Iterator<Item = (Node, W)> + '_,
    ) {
        self.accepted.set_all();
        self.visit_states.reset();
        self.zero_nodes.clear();
        self.heap.clear();

        for u in &self.last_nodes {
            self.max_distances[*u] = W::MIN;
        }

        self.last_nodes.clear();

        for (n, _, w) in targets {
            self.max_distances[*n] = *w;
            self.last_nodes.push(*n);
        }

        self.visit_states.queue_node(source, W::zero());
        self.heap.push(W::zero(), source);
        logger.add_insertion();

        let mut not_rejected = targets.len();

        while let Some((dist, heap_node)) = self.heap.pop() {
            if not_rejected == 0 {
                break;
            }
            if self.visit_states.is_visited(heap_node, dist) {
                continue;
            }
            self.zero_nodes.push(heap_node);

            while let Some(node) = self.zero_nodes.pop() {
                if not_rejected == 0 {
                    break;
                }
                for edge in graph.out_neighbors(node) {
                    let succ = edge.target;
                    if succ == source {
                        continue;
                    }

                    let next = graph.pot_weight(*edge);
                    if next <= W::zero() && self.visit_states.queue_node(succ, dist) {
                        if dist < self.max_distances[succ] {
                            if let Some((i, _)) =
                                targets.iter().enumerate().find(|(_, (n, _, _))| *n == succ)
                            {
                                if self.accepted.unset_bit(i) {
                                    not_rejected -= 1;
                                }
                            }
                        }
                        self.zero_nodes.push(succ);
                        logger.add_insertion();
                        continue;
                    }

                    let mut cost = dist + next;
                    if cost > max_distance {
                        continue;
                    }

                    if cost < self.max_distances[succ] {
                        if let Some((i, _)) =
                            targets.iter().enumerate().find(|(_, (n, _, _))| *n == succ)
                        {
                            if self.accepted.unset_bit(i) {
                                not_rejected -= 1;
                            }
                        }
                    }

                    cost.round_up(self.heap.top());
                    if self.visit_states.queue_node(succ, cost) {
                        self.heap.push(cost, succ);
                        logger.add_insertion();
                    }
                }
            }
        }

        let len = targets.len();
        (
            self.accepted.iter().filter(move |u| *u < len),
            self.visit_states.get_distances(),
        )
    }
}
