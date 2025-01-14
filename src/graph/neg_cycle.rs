use std::collections::VecDeque;

use ez_bitset::bitset::BitSet;

use super::*;

/// Returns *true* if the graph has a negative weight cycle
#[inline]
pub fn has_negative_cycle<W: Weight>(graph: &Graph<W>) -> bool {
    // A value of `n` means: no predecessor set yet
    let mut predecessors: Vec<Node> = vec![graph.n() as Node; graph.n()];

    let mut distances = vec![W::zero(); graph.n()];
    let mut queue = VecDeque::from((0..graph.n()).collect::<Vec<Node>>());
    let mut in_queue = BitSet::new_all_set(graph.n());

    let mut num_relaxations = 0usize;

    while let Some(u) = queue.pop_front() {
        in_queue.unset_bit(u);

        for edge in graph.out_neighbors(u) {
            if distances[u] + edge.weight < distances[edge.target] {
                distances[edge.target] = distances[u] + edge.weight;
                predecessors[edge.target] = u;
                num_relaxations += 1;
                if num_relaxations == graph.n() {
                    num_relaxations = 0;
                    if !shortest_path_tree_is_acyclic(graph, &predecessors) {
                        return true;
                    }
                }

                if !in_queue.set_bit(edge.target) {
                    queue.push_back(edge.target);
                }
            }
        }
    }

    false
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
