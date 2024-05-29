use core::panic;

use crate::{graph::*, weight::Weight};

/// Implement some common graph methods
macro_rules! impl_common_graph_fn {
    () => {
        #[inline]
        fn n(&self) -> usize {
            self.potentials.len()
        }

        #[inline]
        fn m(&self) -> usize {
            self.edges.len()
        }

        #[inline]
        fn edge(&self, idx: usize) -> Edge<W> {
            self.edges[idx]
        }

        #[inline]
        fn edges(&self) -> &[Edge<W>] {
            &self.edges
        }

        #[inline]
        fn potential(&self, u: Node) -> W {
            self.potentials[u]
        }

        #[inline]
        fn update_potential(&mut self, u: Node, delta: W) {
            self.potentials[u] += delta;
        }

        #[inline]
        fn out_neighbors(&self, u: Node) -> &[Edge<W>] {
            &self.edges[self.limits[u]..self.limits[u + 1]]
        }
    };
}

/// Graph representation for the onedirectional search
pub struct OneDirGraph<W: Weight> {
    /// Potentials of each node
    potentials: Vec<W>,
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the first edge in `edges` with source node `u`
    limits: Vec<usize>,
}

impl<W: Weight> Graph<W> for OneDirGraph<W> {
    impl_common_graph_fn!();

    #[cold]
    fn in_neighbors(&self, u: Node) -> &[Edge<W>] {
        panic!("This graph representation does not implement this method!");
    }

    #[inline]
    fn update_weight(&mut self, idx: usize, new_weight: W) {
        self.edges[idx].weight = new_weight;
    }

    fn from_edges(n: usize, mut edges: Vec<Edge<W>>) -> Self {
        assert!(edges.len() > 1);

        edges.sort_unstable();

        let mut curr_edge: usize = 0;
        let limits: Vec<usize> = (0..n)
            .map(|i| {
                while curr_edge < edges.len() && edges[curr_edge].source < i {
                    curr_edge += 1;
                }
                curr_edge
            })
            .chain(std::iter::once(edges.len()))
            .collect();

        Self {
            edges,
            limits,
            potentials: vec![W::zero(); n],
        }
    }
}

/// Graph representation for the bidirectional search
pub struct TwoDirGraph<W: Weight> {
    /// Potentials of all nodes
    potentials: Vec<W>,
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the first edge in `edges` with source node `u`
    limits: Vec<usize>,
    /// List of all edges sorted by target node
    rev_edges: Vec<Edge<W>>,
    /// `rev_limits[u]` is the first edge in `rev_edges` with target node `u`
    rev_limits: Vec<usize>,
}

impl<W: Weight> Graph<W> for TwoDirGraph<W> {
    impl_common_graph_fn!();

    #[inline]
    fn in_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.rev_edges[self.rev_limits[u]..self.rev_limits[u + 1]]
    }

    #[inline]
    fn update_weight(&mut self, idx: usize, new_weight: W) {
        let (u, v, w) = self.edges[idx].into();
        self.edges[idx].weight = new_weight;

        for i in self.rev_limits[v]..self.rev_limits[v + 1] {
            if self.rev_edges[i].source == u && self.rev_edges[i].weight == w {
                self.rev_edges[i].weight = new_weight;
                break;
            }
        }
    }

    fn from_edges(n: usize, mut edges: Vec<Edge<W>>) -> Self {
        assert!(edges.len() > 1);

        edges.sort_unstable();

        let mut curr_edge: usize = 0;
        let limits: Vec<usize> = (0..n)
            .map(|i| {
                while curr_edge < edges.len() && edges[curr_edge].source < i {
                    curr_edge += 1;
                }
                curr_edge
            })
            .chain(std::iter::once(edges.len()))
            .collect();

        let (rev_edges, rev_limits) = {
            let mut rev_edges = edges.clone();
            rev_edges
                .sort_unstable_by(|e1, e2| (e1.target, e1.source).cmp(&(e2.target, e2.source)));

            curr_edge = 0;
            let rev_limits: Vec<usize> = (0..n)
                .map(|i| {
                    while curr_edge < rev_edges.len() && rev_edges[curr_edge].target < i {
                        curr_edge += 1;
                    }
                    curr_edge
                })
                .chain(std::iter::once(rev_edges.len()))
                .collect();

            (rev_edges, rev_limits)
        };

        Self {
            edges,
            limits,
            potentials: vec![W::zero(); n],
            rev_edges,
            rev_limits,
        }
    }
}
