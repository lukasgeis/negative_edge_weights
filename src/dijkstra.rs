use crate::{graph::*, radixheap::RadixHeap, utils::*, weight::Weight};

/// The state of a node in Dijkstra
#[derive(Debug, Clone, Copy, PartialEq)]
enum VisitState<W: Weight> {
    /// The node has not been found yet
    Unvisited,
    /// The node is in the queue with current value
    Queued(W),
    /// The node was visited with final value
    Visited(W),
}

#[derive(Debug, Clone)]
struct VisitedDistances<W: Weight> {
    /// Stores the state for each node in this iteration
    visit_map: Vec<VisitState<W>>,
    /// Stores which nodes were reached in this iteration: only beneficial if we have `o(n)` nodes
    /// seen in total
    seen_nodes: ReusableVec<Node>,
}

impl<W: Weight> VisitedDistances<W> {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            visit_map: vec![VisitState::Unvisited; n],
            seen_nodes: ReusableVec::with_capacity(n),
        }
    }

    /// Visits a node (if it is currently queued)
    #[inline]
    pub fn visit_node(&mut self, node: Node) {
        if let VisitState::Queued(dist) = self.visit_map[node] {
            self.visit_map[node] = VisitState::Visited(dist);
        }
    }

    /// Returns *true* if the node is already visisted
    #[inline]
    pub fn is_visited(&self, node: Node) -> bool {
        matches!(self.visit_map[node], VisitState::Visited(_))
    }

    /// Updates the distance of a node
    #[inline]
    pub fn queue_node(&mut self, node: Node, distance: W) -> bool {
        match self.visit_map[node] {
            VisitState::Unvisited => {
                self.visit_map[node] = VisitState::Queued(distance);
                self.seen_nodes.push(node);
                true
            }
            VisitState::Queued(dist) => {
                if distance < dist {
                    self.visit_map[node] = VisitState::Queued(distance);
                    true
                } else {
                    false
                }
            }
            VisitState::Visited(_) => false,
        }
    }

    /// Resets the data structure
    #[inline]
    pub fn reset(&mut self) {
        if self.seen_nodes.is_asymptotically_full() {
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

    /// Returns an iterator over all discovered nodes in the shortest path tree and their total distances  
    #[inline]
    pub fn get_distances(&mut self) -> impl Iterator<Item = (Node, W)> + '_ {
        if self.seen_nodes.is_asymptotically_full() {
            DoubleIterator::IterA(self.visit_map.iter().enumerate().filter_map(|(u, s)| {
                if let VisitState::Visited(w) = s {
                    Some((u, *w))
                } else {
                    None
                }
            }))
        } else {
            DoubleIterator::IterB(self.seen_nodes.iter().filter_map(|u| {
                if let VisitState::Visited(w) = self.visit_map[*u] {
                    Some((*u, w))
                } else {
                    None
                }
            }))
        }
    }
}

/// Dijkstra instance to reuse data structure for multiple runs
/// Note that this is meant to be used on graphs with the same number of nodes only
pub struct Dijkstra<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    /// MinHeap used for Dijkstra: implementation uses a MaxHeap, thus we need `Reverse`
    heap: RadixHeap<W, Node>,

    /// Stores which nodes have already been visited in which total distance
    visit_states: VisitedDistances<W>,

    /// A stack to keep track of nodes that can be visited directly without putting them on the
    /// heap
    zero_nodes: Vec<Node>,
}

impl<W> Dijkstra<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    /// Initializes Dijkstra for a graph with `n` nodes
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heap: RadixHeap::new(),
            visit_states: VisitedDistances::new(n),
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
    ) -> Option<impl Iterator<Item = (Node, W)> + '_> {
        if source_node == target_node {
            return None;
        }
    
        #[cfg(feature = "sptree_size")]
        let (mut nodes_visited, mut nodes_queued, mut edges_traversed) = (0usize, 0usize, 0usize);

        self.visit_states.reset();
        self.heap.clear();
        self.zero_nodes.clear();

        self.visit_states.queue_node(source_node, W::zero());

        self.heap.push(W::zero(), source_node);

        while let Some((dist, heap_node)) = self.heap.pop() {
            self.zero_nodes.push(heap_node);

            #[cfg(feature = "dfs_size")]
            let mut dfs = 0usize;

            while let Some(node) = self.zero_nodes.pop() {
                if self.visit_states.is_visited(node) {
                    continue;
                }

                self.visit_states.visit_node(node);

                #[cfg(feature = "sptree_size")]
                {
                    nodes_visited += 1;
                }

                for (_, succ, weight) in graph.neighbors(node) {
                    #[cfg(feature = "sptree_size")]
                    {
                        edges_traversed += 1;
                    }

                    let succ = *succ;
                    if self.visit_states.is_visited(succ) {
                        continue;
                    }

                    let mut next = graph.potential_weight((node, succ, *weight));
                    next.round_up(W::zero());

                    if next == W::zero() && self.visit_states.queue_node(succ, dist) {
                        if succ == target_node && dist < max_distance {
                            #[cfg(feature = "sptree_size")]
                            println!("{nodes_visited},{nodes_queued},{edges_traversed},dijkstra,total");
                            return None;
                        }

                        self.zero_nodes.push(succ);
                        
                        #[cfg(feature = "sptree_size")]
                        {
                            nodes_queued += 1;
                        }

                        #[cfg(feature = "dfs_size")]
                        {
                            dfs += 1;
                        }

                        continue;
                    }

                    let mut cost = dist + next;
                    cost.round_up(W::zero());
                    if cost > max_distance {
                        continue;
                    }

                    if succ == target_node && cost < max_distance {
                        #[cfg(feature = "sptree_size")]
                        println!("{nodes_visited},{nodes_queued},{edges_traversed},dijkstra,total");
                        return None;
                    }

                    cost.round_up(self.heap.top());
                    if self.visit_states.queue_node(succ, cost) {
                        #[cfg(feature = "sptree_size")]
                        {
                            nodes_queued += 1;
                        }
                        self.heap.push(cost, succ);
                    }
                }
            }

            #[cfg(feature = "dfs_size")]
            println!("{dfs}");
        }

        #[cfg(feature = "sptree_size")]
        println!("{nodes_visited},{nodes_queued},{edges_traversed},dijkstra,total");

        Some(self.visit_states.get_distances())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::*;

    #[test]
    fn test_dijkstra() {
        let mut graph = Graph::from_edge_list(5, EDGES.to_vec(), true);

        let mut dijsktra = Dijkstra::new(graph.n());

        for j in 0..EDGES.len() {
            graph.update_weight(j, GOOD_WEIGHTS[2][j]);
        }
        let res: Vec<Vec<f64>> = DISTANCES[2].into_iter().map(|s| s.to_vec()).collect();

        let targets: [Node; 5] = [4, 2, 4, 2, 3];

        for u in 0..graph.n() {
            let mut dj = vec![0.0; graph.n()];
            for (v, w) in dijsktra
                .run(&graph, u, targets[u], res[u][targets[u]])
                .unwrap()
            {
                dj[v] = w;
            }
            assert_eq!(res[u], dj);
        }
    }
}
