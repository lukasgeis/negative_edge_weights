use ordered_float::NotNan;
use radix_heap::RadixHeapMap;
use std::cmp::Reverse;

use crate::graph::*;

/// The state of a node in Dijkstra
#[derive(Debug, Clone, Copy, PartialEq)]
enum VisitState {
    /// The node has not been found yet
    Unvisisted,
    /// The node is in the queue with current value
    Queued(Weight),
    /// The node was visited with final value
    Visited(Weight),
}

#[derive(Debug, Clone)]
struct VisitedDistances {
    /// Stores the state for each node in this iteration
    visit_map: Vec<VisitState>,
    /// Stores which nodes were reached in this iteration: only beneficial if we have `o(n)` nodes
    /// seen in total
    seen_nodes: Vec<Node>,
}

impl VisitedDistances {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            visit_map: vec![VisitState::Unvisisted; n],
            // Might be beneficial to initialize with capacity `n` to prevent ever reallocating
            seen_nodes: Vec::new(),
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
    pub fn queue_node(&mut self, node: Node, distance: Weight) {
        match self.visit_map[node] {
            VisitState::Unvisisted => {
                self.visit_map[node] = VisitState::Queued(distance);
                self.seen_nodes.push(node);
            }
            VisitState::Queued(dist) => {
                if distance < dist {
                    self.visit_map[node] = VisitState::Queued(distance);
                }
            }
            VisitState::Visited(_) => {}
        };
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
                .for_each(|w| *w = VisitState::Unvisisted);
        } else {
            self.seen_nodes
                .iter()
                .for_each(|u| self.visit_map[*u] = VisitState::Unvisisted);
            self.seen_nodes.clear();
        }
    }

    /// Returns an iterator over all discovered nodes in the shortest path tree and their total distances  
    #[inline]
    pub fn get_distances(&mut self) -> impl Iterator<Item = (Node, Weight)> + '_ {
        if self.is_asymptotically_full() {
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

/// `f64` does not implement `Ord` as well as `Radix`, hence we need the wrapper `NotNan`.
/// Additionally, `RadixHeapMap` is a MaxHeap, but we require a MinHeap
type RadixWeight = Reverse<NotNan<Weight>>;

/// Dijkstra instance to reuse data structure for multiple runs
/// Note that this is meant to be used on graphs with the same number of nodes only
pub struct Dijkstra {
    /// MinHeap used for Dijkstra: implementation uses a MaxHeap, thus we need `Reverse`
    heap: RadixHeapMap<RadixWeight, Node>,
    /// Stores which nodes have already been visited in which total distance
    visit_states: VisitedDistances,
}

impl Dijkstra {
    /// Initializes Dijkstra for a graph with `n` nodes
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heap: RadixHeapMap::new(),
            visit_states: VisitedDistances::new(n),
        }
    }

    /// Converts a `Weight` into a `RadixWeight`
    #[inline]
    fn weight_to_radix(w: Weight) -> RadixWeight {
        Reverse(NotNan::new(w).expect("Some Weight was NaN"))
    }

    /// Converts a `RadixWeight` into a `Weight`
    #[inline]
    fn radix_to_weight(w: RadixWeight) -> Weight {
        w.0.into_inner()
    }

    #[inline]
    fn rounding_error_correction(&self, to_round: &mut Weight, value: Weight) {
        if value > *to_round && (value - *to_round).abs() < 1e-8 {
            *to_round = value;
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
        graph: &Graph,
        source_node: Node,
        target_node: Node,
        max_distance: Weight,
    ) -> Option<impl Iterator<Item = (Node, Weight)> + '_> {
        if source_node == target_node {
            return None;
        }

        self.visit_states.reset();
        self.heap.clear();

        self.visit_states.queue_node(source_node, 0 as Weight);
        self.heap
            .push(Self::weight_to_radix(0 as Weight), source_node);
        while let Some((dist, node)) = self.heap.pop() {
            if self.visit_states.is_visited(node) {
                continue;
            }

            self.visit_states.visit_node(node);

            let dist = Self::radix_to_weight(dist);
            for (_, succ, weight) in graph.neighbors(node) {
                let succ = *succ;
                let mut cost = dist + graph.potential_weight((node, succ, *weight));
                self.rounding_error_correction(&mut cost, 0.0);
                if self.visit_states.is_visited(succ) || cost > max_distance {
                    continue;
                }

                if succ == target_node {
                    if cost == max_distance {
                        return Some(self.visit_states.get_distances());
                    } else {
                        return None;
                    }
                }

                // `RadixHeapMap` panics if the inserted value is greater than the last popped
                // value `top`. Due to floating-point precision, this can throw unwanted errors that we
                // can prevent by rounding `cost` to `top` if they are very close to each other
                let top = Self::radix_to_weight(self.heap.top().unwrap());
                self.rounding_error_correction(&mut cost, top);
                self.heap.push(Self::weight_to_radix(cost), succ);
                self.visit_states.queue_node(succ, cost);
            }
        }

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
        let res: Vec<Vec<Weight>> = DISTANCES[2].into_iter().map(|s| s.to_vec()).collect();

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
