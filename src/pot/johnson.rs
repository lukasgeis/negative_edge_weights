use std::collections::VecDeque;

use ez_bitset::bitset::BitSet;

use self::checks::TopoCheck;

use super::*;

pub struct Johnson;

impl<W> Potentials<W> for Johnson
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn compute_pot(graph: &Graph<W>) -> Result<Vec<W>, NegativeCycleFound> {
        let mut pred: Vec<Node> = vec![graph.n(); graph.n()];

        let mut dist: Vec<W> = vec![W::zero(); graph.n()];
        let mut queue = VecDeque::from((0..graph.n()).collect::<Vec<Node>>());
        let mut in_queue = BitSet::new_all_set(graph.n());

        let mut num_relax = 0usize;
        let mut topo_check = TopoCheck::new(graph.n());

        while let Some(u) = queue.pop_front() {
            in_queue.unset_bit(u);

            for (v, w) in graph.neighbors(u) {
                if dist[u] + *w < dist[*v] {
                    dist[*v] = dist[u] + *w;
                    pred[*v] = u;
                    num_relax += 1;

                    if num_relax == 4 * graph.n() {
                        num_relax = 0;
                        if !topo_check.check(&pred) {
                            return Err(NegativeCycleFound);
                        }
                    }

                    if !in_queue.set_bit(*v) {
                        queue.push_back(*v);
                    }
                }
            }
        }

        Ok(dist)
    }
}
