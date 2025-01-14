use crate::utils::RadixHeap;

use self::checks::TopoCheck;

use super::*;

pub struct Alternating;

impl<W> Potentials<W> for Alternating
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn compute_pot(graph: &Graph<W>) -> Result<Vec<W>, NegativeCycleFound> {
        let mut dist = vec![W::zero(); graph.n()];
        let mut pred = vec![graph.n(); graph.n()];

        let mut marked: Vec<Node> = (0..graph.n()).collect();

        let mut heap: RadixHeap<W, Node> = RadixHeap::new();

        let mut num_relax = 0usize;
        let mut topo_check = TopoCheck::new(graph.n());

        loop {
            // Dijkstra
            while let Some((d, u)) = heap.pop() {
                if dist[u] < d {
                    continue;
                }
                marked.push(u);

                for (v, w) in graph.pos_neighbors(u) {
                    if dist[u] + *w < dist[*v] {
                        dist[*v] = dist[u] + *w;
                        pred[*v] = u;

                        num_relax += 1;
                        heap.push(dist[u] + *w, *v);
                    }
                }
            }

            // Update `top` value as distances can be negative after BF phases
            heap.set_top(W::MIN);

            // BellmanFord
            while let Some(u) = marked.pop() {
                for (v, w) in graph.neg_neighbors(u) {
                    if dist[u] + *w < dist[*v] {
                        dist[*v] = dist[u] + *w;
                        pred[*v] = u;

                        num_relax += 1;
                        heap.push(dist[u] + *w, *v);
                    }
                }
            }

            if heap.is_empty() {
                return Ok(dist);
            }

            if num_relax >= 4 * graph.n() {
                num_relax = 0;
                if !topo_check.check(&pred) {
                    return Err(NegativeCycleFound);
                }
            }
        }
    }
}
