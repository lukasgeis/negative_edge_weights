//! This module is a port of `https://github.com/Qiskit/rustworkx/blob/main/rustworkx-core/src/shortest_path/bellman_ford.rs`

use std::collections::VecDeque;

use ez_bitset::bitset::BitSet;

use crate::graph::*;

/// Runs BF on the given graph and starting node
///
/// Returns `Some(distances)` where distances is the distance vector of every node or `None` if a
/// negative cycle exists
#[allow(unused)]
#[inline]
pub fn bellman_ford(graph: &Graph, source_node: Node) -> Option<Vec<Weight>> {
    inner_bellman_ford(graph, Some(source_node))
}

/// Returns *true* if the graph has a negative weight cycle
#[inline]
pub fn has_negative_cycle(graph: &Graph) -> bool {
    inner_bellman_ford(graph, None).is_none()
}

/// Implementation of the SPFA heuristic with cycle-checks every `n` relaxations  
/// If `source_node` is `None`, run from all nodes in graph
fn inner_bellman_ford(graph: &Graph, source_node: Option<Node>) -> Option<Vec<Weight>> {
    // A value of `n` means: no predecessor set yet
    let mut predecessors: Vec<Node> = vec![graph.n() as Node; graph.n()];

    let (mut distances, mut queue, mut in_queue) = if let Some(source_node) = source_node {
        let mut distances = vec![Weight::INFINITY; graph.n()];
        let mut queue = VecDeque::with_capacity(graph.n());
        let mut in_queue = BitSet::new(graph.n());

        distances[source_node] = 0 as Weight;
        queue.push_back(source_node);
        in_queue.set_bit(source_node);

        (distances, queue, in_queue)
    } else {
        (
            vec![0 as Weight; graph.n()],
            VecDeque::from((0..graph.n()).collect::<Vec<Node>>()),
            BitSet::new_all_set(graph.n()),
        )
    };

    let mut num_relaxations = 0usize;

    while let Some(u) = queue.pop_front() {
        in_queue.unset_bit(u);

        for (_, v, w) in graph.neighbors(u) {
            if distances[u] + *w < distances[*v] {
                distances[*v] = distances[u] + *w;
                predecessors[*v] = u;
                num_relaxations += 1;
                if num_relaxations == graph.n() {
                    num_relaxations = 0;

                    if !shortest_path_tree_is_acyclic(graph, &predecessors) {
                        return None;
                    }
                }

                if !in_queue.set_bit(*v) {
                    queue.push_back(*v);
                }
            }
        }
    }

    Some(distances)
}

// Check if the shortest path tree is acyclic via TopoSearch
fn shortest_path_tree_is_acyclic(graph: &Graph, predecessors: &[Node]) -> bool {
    let mut unused_nodes = BitSet::new_all_set(graph.n());
    let mut stack: Vec<Node> = predecessors
        .iter()
        .enumerate()
        .filter_map(|(v, u)| {
            if *u >= graph.n() {
                Some(v as Node)
            } else {
                None
            }
        })
        .collect();

    while let Some(u) = stack.pop() {
        unused_nodes.unset_bit(u);

        for (_, v, _) in graph.neighbors(u) {
            // In the SP-Tree, every node has only one incoming edge
            if unused_nodes[*v] {
                stack.push(*v);
            }
        }
    }

    unused_nodes.cardinality() == 0
}
