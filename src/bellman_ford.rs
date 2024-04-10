use crate::graph::*;

pub fn bellman_ford(graph: &Graph, source_node: Node) -> Option<Vec<Weight>> {
    let mut distances = vec![Weight::INFINITY; graph.n()];
    distances[source_node] = 0 as Weight;

    for _ in 0..(graph.n() - 1) {
        for (u, v, w) in graph.edges() {
            if distances[*u] + *w < distances[*v] {
                distances[*v] = distances[*u] + *w;
            }
        }
    }

    for (u, v, w) in graph.edges() {
        if distances[*u] + *w < distances[*v] {
            return None;
        } 
    }

    Some(distances)
}

pub fn contains_negative_weight_cycle(graph: &Graph) -> bool {
    for u in 0..graph.n() {
        if let Some(distances) = bellman_ford(graph, u) {
            if distances.into_iter().all(|w| w < Weight::INFINITY) {
                return false;
            }
        } else {
            return true;
        }
    }
    false 
}
