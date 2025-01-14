use ez_bitset::bitset::BitSet;

use super::*;

pub fn is_feasible<W: Weight>(graph: &Graph<W>, pot: &[W], rev: bool) -> bool {
    (0..graph.n()).all(|u| {
        graph.neighbors(u).iter().all(|(v, w)| {
            let mut inv_gradient = pot[u] - pot[*v];
            if rev {
                inv_gradient = -inv_gradient;
            }

            *w >= inv_gradient
        })
    })
}

pub struct TopoCheck {
    unused_nodes: BitSet,
    successors: Vec<Vec<Node>>,
    stack: Vec<Node>,
}

impl TopoCheck {
    #[inline]
    pub fn new(n: usize) -> Self {
        Self {
            unused_nodes: BitSet::new_all_set(n),
            successors: vec![Vec::with_capacity(32); n],
            stack: Vec::with_capacity(n),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.unused_nodes.set_all();
        self.successors.iter_mut().for_each(|v| v.clear());
    }

    pub fn check(&mut self, pred: &[Node]) -> bool {
        self.clear();

        pred.iter().enumerate().for_each(|(v, u)| {
            if *u >= self.successors.len() {
                self.stack.push(v);
            } else {
                self.successors[*u].push(v);
            }
        });

        while let Some(u) = self.stack.pop() {
            self.unused_nodes.unset_bit(u);

            for v in &self.successors[u] {
                self.stack.push(*v);
            }
        }

        self.unused_nodes.cardinality() == 0
    }
}
