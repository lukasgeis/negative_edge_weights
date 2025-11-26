use crate::{graph::*, utils::*, weight::Weight};

/// An APSP implementation using repeated Dijkstra calls
pub struct APSP<W: Weight>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    heap: RadixHeap<W, Node>,
    visited: Vec<W>,
    zero_nodes: Vec<Node>,
}

impl<W: Weight> APSP<W>
where
    [(); W::NUM_BITS + 1]: Sized,
{
    pub fn new(n: usize) -> Self {
        Self {
            heap: RadixHeap::new(),
            visited: vec![W::MAX; n],
            zero_nodes: Vec::new(),
        }
    }

    fn dijkstra(&mut self, graph: &Graph<W>, start_node: Node) {
        // These two should be redundant
        self.heap.clear();
        self.zero_nodes.clear();

        self.visited.iter_mut().for_each(|w| *w = W::MAX);

        self.visited[start_node] = W::zero();
        self.heap.push(W::zero(), start_node);

        while let Some((dist, heap_node)) = self.heap.pop() {
            if self.visited[heap_node] < dist {
                continue;
            }

            self.zero_nodes.push(heap_node);

            while let Some(node) = self.zero_nodes.pop() {
                for edge in graph.out_neighbors(node) {
                    let succ = edge.target;
                    let next = graph.pot_weight(*edge);

                    if next <= W::zero() && dist < self.visited[succ] {
                        self.zero_nodes.push(succ);
                        self.visited[succ] = dist;
                        continue;
                    }

                    let mut cost = dist + next;
                    cost.round_up(self.heap.top());
                    if cost < self.visited[succ] {
                        self.heap.push(cost, succ);
                        self.visited[succ] = cost;
                    }
                }
            }
        }
    }

    pub fn run<'a>(&'a mut self, graph: &'a Graph<W>) -> impl Iterator<Item = Edge<W>> + 'a {
        assert_eq!(graph.n(), self.visited.len());
        assert!(graph.has_valid_potentials());

        (0..graph.n()).flat_map(move |u| {
            self.dijkstra(graph, u);
            self.visited
                .clone()
                .into_iter()
                .enumerate()
                .map(move |(v, w)| Edge {
                    source: u,
                    target: v,
                    weight: w,
                })
        })
    }
}

#[test]
fn test_apsp() {
    let graph = Graph::from_pos_edges(
        4,
        vec![
            (0, 1, 1).into(),
            (1, 2, 2).into(),
            (2, 3, 1).into(),
            (3, 0, 2).into(),
            (0, 2, 2).into(),
        ],
    );

    let mut apsp = APSP::new(4);
    let dist: Vec<Edge<i32>> = apsp.run(&graph).collect();

    assert_eq!(
        dist,
        vec![
            (0, 0, 0).into(),
            (0, 1, 1).into(),
            (0, 2, 2).into(),
            (0, 3, 3).into(),
            (1, 0, 5).into(),
            (1, 1, 0).into(),
            (1, 2, 2).into(),
            (1, 3, 3).into(),
            (2, 0, 3).into(),
            (2, 1, 4).into(),
            (2, 2, 0).into(),
            (2, 3, 1).into(),
            (3, 0, 2).into(),
            (3, 1, 3).into(),
            (3, 2, 4).into(),
            (3, 3, 0).into()
        ]
    );
}
