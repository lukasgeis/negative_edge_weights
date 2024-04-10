use ordered_float::NotNan;
use radix_heap::RadixHeapMap;
use std::cmp::Reverse;

use crate::graph::*;

struct VisitedDistances {
    /// Stores total distances for each node in the graph or `INFINITY` if the node was not yet reached
    /// in this iteration
    distances: Vec<Weight>,
    /// Stores which nodes were reached in this iteration: only beneficial if we have `o(n)` nodes
    /// visited in total
    visited_nodes: Vec<Node>,
}

impl VisitedDistances {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            distances: vec![Weight::INFINITY; n],
            // Might be beneficial to initialize with capacity `n` to prevent ever reallocating
            visited_nodes: Vec::new(),
        }
    }

    /// Returns *true* if the node is already visisted, i.e. if the stored distance is strictly
    /// less than the new distance
    #[inline]
    pub fn is_visited(&self, node: Node, distance: Weight) -> bool {
        self.distances[node] < distance
    }

    /// Updates the distance of a node
    #[inline]
    pub fn visit_node(&mut self, node: Node, distance: Weight) {
        if self.distances[node] == f64::INFINITY {
            self.visited_nodes.push(node);
        }
        self.distances[node] = distance;
    }

    /// Returns *true* if we have visited `Omega(n)` nodes
    #[inline]
    fn is_asymptotically_full(&self) -> bool {
        self.visited_nodes.len() > self.distances.len() / 4
    }

    /// Resets the data structure
    #[inline]
    pub fn reset(&mut self) {
        if self.is_asymptotically_full() {
            self.visited_nodes.clear();
            self.distances
                .iter_mut()
                .for_each(|w| *w = Weight::INFINITY);
        } else {
            self.visited_nodes
                .iter()
                .for_each(|u| self.distances[*u] = Weight::INFINITY);
            self.visited_nodes.clear();
        }
    }

    /// Returns an iterator over all discovered nodes in the shortest path tree and their total distances  
    #[inline]
    pub fn get_distances(&mut self) -> impl Iterator<Item = (Node, Weight)> + '_ {
        if self.is_asymptotically_full() {
            DoubleIterator::IterA(self.distances.iter().enumerate().filter_map(|(u, w)| {
                if *w < Weight::INFINITY {
                    Some((u, *w))
                } else {
                    None
                }
            }))
        } else {
            DoubleIterator::IterB(self.visited_nodes.iter().map(|u| (*u, self.distances[*u])))
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
    visited: VisitedDistances,
}

impl Dijkstra {
    /// Initializes Dijkstra for a graph with `n` nodes
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heap: RadixHeapMap::new(),
            visited: VisitedDistances::new(n),
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
    fn rounding_error_correction(&self, cost: &mut Weight) {
        let top = Self::radix_to_weight(self.heap.top().unwrap());
        if top > *cost && (top - *cost).abs() < 1e-8 {
            *cost = top;
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

        self.visited.reset();
        self.heap.clear();

        self.visited.visit_node(source_node, 0 as Weight);
        self.heap
            .push(Self::weight_to_radix(0 as Weight), source_node);

        while let Some((dist, node)) = self.heap.pop() {
            let dist = Self::radix_to_weight(dist);
            if self.visited.is_visited(node, dist) {
                continue;
            }

            for (_, succ, weight) in graph.neighbors(node) {
                let succ = *succ;
                let mut cost = dist + graph.potential_weight((node, succ, *weight));
                if self.visited.is_visited(succ, cost) || cost > max_distance {
                    continue;
                }

                if succ == target_node {
                    return None;
                }

                // `RadixHeapMap` panics if the inserted value is greater than the last popped
                // value `top`. Due to floating-point precision, this can throw unwanted errors that we
                // can prevent by rounding `cost` to `top` if they are very close to each other  
                self.rounding_error_correction(&mut cost);
                self.heap.push(Self::weight_to_radix(cost), succ);
                self.visited.visit_node(succ, cost);

            }
        }

        Some(self.visited.get_distances())
    }
}
