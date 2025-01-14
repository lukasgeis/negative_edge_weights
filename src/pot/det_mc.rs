use crate::{search::dijkstra::VisitedDistances, utils::RadixHeap};

use super::*;

pub struct IterativeInsertions;

impl<W> Potentials<W> for IterativeInsertions
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn compute_pot(graph: &Graph<W>) -> Result<Vec<W>, NegativeCycleFound> {
        let mut pot: Vec<W> = vec![W::zero(); graph.n()];

        let mut heap: RadixHeap<W, Node> = RadixHeap::new();
        let mut visit_states: VisitedDistances<W> = VisitedDistances::new(graph.n());

        for i in 0..graph.n() {
            if graph.has_neg_neighbors(i) {
                visit_states.reset();

                let mut min = W::zero();
                for (v, w) in graph.neg_neighbors(i) {
                    if min > *w + pot[*v] {
                        min = *w + pot[*v];
                    }
                }
                heap.set_top(min - pot[i]);

                for (v, w) in graph.neg_neighbors(i) {
                    let pot_weight = *w + pot[*v] - pot[i];
                    if pot_weight < W::zero() {
                        heap.push(pot_weight, *v);
                        visit_states.queue_node(*v, pot_weight);
                    }
                }

                while let Some((d, u)) = heap.pop() {
                    if visit_states.is_visited(u, d) {
                        continue;
                    }

                    for (v, w) in graph.neighbors(u) {
                        let w = if u <= i || *w >= W::zero() {
                            *w
                        } else {
                            W::zero()
                        };

                        let next = d + w + pot[*v] - pot[u];
                        if next >= W::zero() {
                            continue;
                        }

                        if *v == i {
                            return Err(NegativeCycleFound);
                        }

                        if visit_states.queue_node(*v, next) {
                            heap.push(next, *v);
                        }
                    }
                }

                for (node, dist) in visit_states.get_distances() {
                    pot[node] -= dist;
                }
            }
        }

        Ok(pot)
    }
}
