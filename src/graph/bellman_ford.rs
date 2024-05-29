//! This module is a port of `https://github.com/Qiskit/rustworkx/blob/main/rustworkx-core/src/shortest_path/bellman_ford.rs`

use std::collections::VecDeque;

use ez_bitset::bitset::BitSet;

use crate::{graph::*, weight::Weight};

/// Runs BF on the given graph and starting node
///
/// Returns `Some(distances)` where distances is the distance vector of every node or `None` if a
/// negative cycle exists
#[inline]
pub fn bellman_ford<W: Weight>(graph: &Graph<W>, source_node: Node) -> Option<Vec<W>> {
    inner_bellman_ford(graph, Some(source_node))
}

/// Returns *true* if the graph has a negative weight cycle
#[inline]
pub fn has_negative_cycle<W: Weight>(graph: &Graph<W>) -> bool {
    inner_bellman_ford(graph, None).is_none()
}

/// Implementation of the SPFA heuristic with cycle-checks every `n` relaxations  
/// If `source_node` is `None`, run from all nodes in graph
fn inner_bellman_ford<W: Weight>(graph: &Graph<W>, source_node: Option<Node>) -> Option<Vec<W>> {
    // A value of `n` means: no predecessor set yet
    let mut predecessors: Vec<Node> = vec![graph.n() as Node; graph.n()];

    let (mut distances, mut queue, mut in_queue) = if let Some(source_node) = source_node {
        let mut distances = vec![W::MAX; graph.n()];
        let mut queue = VecDeque::with_capacity(graph.n());
        let mut in_queue = BitSet::new(graph.n());

        distances[source_node] = W::zero();
        queue.push_back(source_node);
        in_queue.set_bit(source_node);

        (distances, queue, in_queue)
    } else {
        (
            vec![W::zero(); graph.n()],
            VecDeque::from((0..graph.n()).collect::<Vec<Node>>()),
            BitSet::new_all_set(graph.n()),
        )
    };

    let mut num_relaxations = 0usize;

    while let Some(u) = queue.pop_front() {
        in_queue.unset_bit(u);

        for edge in graph.neighbors(u) {
            if distances[u] + edge.weight < distances[edge.target] {
                distances[edge.target] = distances[u] + edge.weight;
                predecessors[edge.target] = u;
                num_relaxations += 1;
                if num_relaxations == graph.n() {
                    num_relaxations = 0;
                    if !shortest_path_tree_is_acyclic(graph, &predecessors) {
                        return None;
                    }
                }

                if !in_queue.set_bit(edge.target) {
                    queue.push_back(edge.target);
                }
            }
        }
    }

    Some(distances)
}

// Check if the shortest path tree is acyclic via TopoSearch
fn shortest_path_tree_is_acyclic<W: Weight>(graph: &Graph<W>, predecessors: &[Node]) -> bool {
    let mut unused_nodes = BitSet::new_all_set(graph.n());
    let mut successors: Vec<Vec<Node>> = vec![Vec::new(); graph.n()];
    let mut stack: Vec<Node> = predecessors
        .iter()
        .enumerate()
        .filter_map(|(v, u)| {
            if *u >= graph.n() {
                Some(v as Node)
            } else {
                successors[*u].push(v as Node);
                None
            }
        })
        .collect();

    while let Some(u) = stack.pop() {
        unused_nodes.unset_bit(u);

        for v in &successors[u] {
            // In the SP-Tree, every node has only one incoming edge
            stack.push(*v);
        }
    }

    unused_nodes.cardinality() == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::*;

    #[test]
    fn test_negative_cycle_finder() {
        let mut graph = Graph::from_edge_list(5, EDGES.into_iter().map(|e| e.into()).collect());

        for weights in GOOD_WEIGHTS {
            for i in 0..EDGES.len() {
                graph.update_weight(i, 0.0, weights[i]);
            }
            assert!(!has_negative_cycle(&graph));
        }

        for weights in BAD_WEIGHTS {
            for i in 0..EDGES.len() {
                graph.update_weight(i, 0.0, weights[i]);
            }
            assert!(has_negative_cycle(&graph));
        }
    }

    #[test]
    fn test_bellman_ford() {
        let mut graph = Graph::from_edge_list(5, EDGES.into_iter().map(|e| e.into()).collect());

        for i in 0..GOOD_WEIGHTS.len() {
            for j in 0..EDGES.len() {
                graph.update_weight(j, 0.0, GOOD_WEIGHTS[i][j]);
            }
            let res: Vec<Vec<f64>> = DISTANCES[i].into_iter().map(|s| s.to_vec()).collect();

            for u in 0..graph.n() {
                let bf = bellman_ford(&graph, u).unwrap();
                assert_eq!(res[u], bf);
            }
        }
    }
}
