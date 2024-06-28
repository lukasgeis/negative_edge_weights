use crate::{bidijkstra::Graph, utils::RadixHeap, weight::Weight};

use super::{GraphNeigbors, GraphStats, Node};

pub struct CompleteDijkstra<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    heap: RadixHeap<W, Node>,
    visit_states: Vec<W>,
    zero_nodes: Vec<Node>,
}

impl<W> CompleteDijkstra<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            heap: RadixHeap::new(),
            visit_states: vec![W::MAX; n],
            zero_nodes: Vec::new(),
        }
    }

    pub fn run(&mut self, graph: &Graph<W>, source_node: Node) -> (W, W) {
        self.visit_states.iter_mut().for_each(|w| *w = W::MAX);
        self.heap.clear();
        self.zero_nodes.clear();

        self.visit_states[source_node] = W::zero();
        self.heap.push(W::zero(), source_node);

        while let Some((dist, heap_node)) = self.heap.pop() {
            if self.visit_states[heap_node] < dist {
                continue;
            }
            self.zero_nodes.push(heap_node);

            while let Some(node) = self.zero_nodes.pop() {
                for edge in graph.out_neighbors(node) {
                    let succ = edge.target;
                    let next = graph.potential_weight(*edge);
                    if next <= W::zero() && self.visit_states[succ] > dist {
                        self.zero_nodes.push(succ);
                        self.visit_states[succ] = dist;
                        continue;
                    }

                    let mut cost = dist + next;
                    cost.round_up(self.heap.top());
                    if self.visit_states[succ] > cost {
                        self.heap.push(cost, succ);
                        self.visit_states[succ] = cost;
                    }
                }
            }
        }

        let mut sum_path = W::zero();
        let mut max_path = W::zero();

        for u in 0..graph.n() {
            let path_weight =
                self.visit_states[u] + graph.potential(u) - graph.potential(source_node);
            sum_path += path_weight;
            if path_weight > max_path {
                max_path = path_weight;
            }
        }

        (sum_path, max_path)
    }
}

pub fn mean_max_paths<W>(graph: &Graph<W>) -> (f64, f64)
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    let mut dijkstra = CompleteDijkstra::new(graph.n());

    let mut sum_path = W::zero();
    let mut max_path = W::zero();
    for u in 0..graph.n() {
        let (sp, mp) = dijkstra.run(graph, u);
        sum_path += sp;
        if mp > max_path {
            max_path = mp;
        }
    }

    (
        sum_path.to_f64() / (graph.n() * graph.n()) as f64,
        max_path.to_f64(),
    )
}
