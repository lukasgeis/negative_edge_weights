use std::collections::VecDeque;

use ez_bitset::bitset::BitSet;
use rand_distr::{Distribution, Uniform};

use crate::{graph::*, mcmc::NegWeightMCMC};

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
                            return false;
                        }
                        continue;
                    }

                    if !self.in_queue.set_bit(edge.target) {
                        self.queue.push_back(edge.target);
                    }
                }
            }
        }

        true
    }
}

/// Graph representation for the naive bellman-ford search
pub struct Graph<W: Weight> {
    /// List of all edges sorted by source node
    edges: Vec<Edge<W>>,
    /// `limits[u]` is the first edge in `edges` with source node `u`
    limits: Vec<usize>,
}

impl_debug_graph!(Graph);

impl<W: Weight> GraphStats for Graph<W> {
    #[inline]
    fn n(&self) -> usize {
        self.limits.len() - 1
    }

    #[inline]
    fn m(&self) -> usize {
        self.edges.len()
    }

    #[inline]
    fn avg_weight(&self) -> f64 {
        self.edges.iter().map(|e| e.weight).sum::<W>().to_f64() / self.m() as f64
    }

    #[inline]
    fn frac_negative_edges(&self) -> f64 {
        self.edges.iter().filter(|e| e.weight < W::zero()).count() as f64 / self.m() as f64
    }
}

impl<W: Weight> GraphNeigbors<W> for Graph<W> {
    fn out_neighbors(&self, u: Node) -> &[Edge<W>] {
        &self.edges[self.limits[u]..self.limits[u + 1]]
    }
}

impl<W: Weight> GraphEdgeList<W> for Graph<W> {
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

        Self { edges, limits }
    }

    #[inline]
    fn into_edges(self) -> Vec<Edge<W>> {
        self.edges
    }
}

impl<W: Weight> Graph<W> {
    #[inline]
    pub fn edge(&self, idx: usize) -> Edge<W> {
        self.edges[idx]
    }

    #[inline]
    pub fn potential_weight(&self, edge: Edge<W>) -> W {
        edge.weight
    }

    #[inline]
    pub fn update_weight(&mut self, idx: usize, weight: W) {
        self.edges[idx].weight = weight;
    }
}

impl<W> NegWeightMCMC<W> for Graph<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn run_mcmc<R: Rng, D: Distribution<W>>(
        &mut self,
        rng: &mut R,
        weight_sampler: D,
        rounds_factor: f64,
    ) {
        let num_rounds = (self.m() as f64 * rounds_factor).ceil() as u64;
        let mut bellman_ford = BellmanFord::new(self.n());
        let edge_sampler = Uniform::new(0, self.m());

        for _ in 0..num_rounds {
            let idx = edge_sampler.sample(rng);
            let edge = self.edge(idx);
            let weight = weight_sampler.sample(rng);

            if weight >= edge.weight || bellman_ford.run(self, edge.target, edge.source, -weight) {
                self.update_weight(idx, weight);
            }
        }
    }
}

/// Returns *true* if the graph has a negative weight cycle
#[inline]
pub fn has_negative_cycle<W: Weight, G: GraphNeigbors<W> + GraphStats>(graph: &G) -> bool {
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
fn shortest_path_tree_is_acyclic<W: Weight, G: GraphNeigbors<W> + GraphStats>(
    graph: &G,
    predecessors: &[Node],
) -> bool {
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
