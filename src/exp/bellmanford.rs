use std::collections::VecDeque;

use ez_bitset::bitset::BitSet;

use crate::{graph::*, weight::Weight};


pub struct BellmanFord<W: Weight> {
    distances: Vec<W>,
    queue: VecDeque<Node>,
    in_queue: BitSet,
}

impl<W: Weight> BellmanFord<W> {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            distances: vec![W::MAX; n],
            queue: VecDeque::with_capacity(n),
            in_queue: BitSet::new(n),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.distances.iter_mut().for_each(|d| *d = W::MAX);
        self.queue.clear();
        self.in_queue.unset_all();
    }

    #[inline]
    pub fn run<G: GraphNeigbors<W>>(
        &mut self,
        graph: &G,
        source_node: Node,
        target_node: Node,
        min_distance: W,
    ) -> bool {
        if source_node == target_node {
            return min_distance <= W::zero();
        }

        #[cfg(feature = "insertions")]
        let mut num_insertions = 1usize;

        self.clear();

        self.distances[source_node] = W::zero();
        self.queue.push_back(source_node);
        self.in_queue.set_bit(source_node);

        while let Some(u) = self.queue.pop_front() {
            self.in_queue.unset_bit(u);

            for edge in graph.out_neighbors(u) {
                if self.distances[u] + edge.weight < self.distances[edge.target] {
                    self.distances[edge.target] = self.distances[u] + edge.weight;

                    if edge.target == target_node {
                        if self.distances[edge.target] < min_distance {
                            #[cfg(feature = "insertions")]
                            println!("{num_insertions},rej,bf");
                            return false;
                        }
                        continue;
                    }

                    if !self.in_queue.set_bit(edge.target) {
                        self.queue.push_back(edge.target);
                        #[cfg(feature = "insertions")]
                        {
                            num_insertions += 1;
                        }
                    }
                }
            }
        }

        #[cfg(feature = "insertions")]
        println!("{num_insertions},acc,bf");

        true
    }
}
