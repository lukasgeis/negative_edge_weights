use crate::weight::Weight;

use super::{GraphNeigbors, GraphStats};


#[derive(Debug, Copy, Clone)]
struct NodeState {
    index: usize,
    lowlink: usize,
    on_stack: bool,
}

impl NodeState {
    #[inline]
    pub fn new(n: usize) -> Vec<Self> {
        vec![
            Self {
                index: n,
                lowlink: n,
                on_stack: false,
            }; n
        ]
    }
}

pub fn num_sccs<W: Weight,G: GraphNeigbors<W> + GraphStats>(graph: &G) -> usize {
    let mut index = 0usize;
    let mut stack = Vec::new();

    let mut states = NodeState::new(graph.n());

    let mut sccs: Vec<Vec<usize>> = Vec::new();

    for v in 0..graph.n() {
        if states[v].index == graph.n() {
            strongconnect(graph, &mut sccs, &mut states, &mut stack, &mut index, v);
        }
    }

    fn strongconnect<W: Weight,G: GraphNeigbors<W> + GraphStats>(graph: &G, sccs: &mut Vec<Vec<usize>>, states: &mut [NodeState], stack: &mut Vec<usize>, index: &mut usize, v: usize) {
        states[v].index = *index;
        states[v].lowlink = *index;
        *index += 1;
        stack.push(v);
        states[v].on_stack = true;

        for edge in graph.out_neighbors(v) {
            let w = edge.target;
            if states[w].index == graph.n() {
                strongconnect(graph, sccs, states, stack, index, w);
                states[v].lowlink = states[v].lowlink.min(states[w].lowlink);
            } else if states[v].on_stack {
                states[v].lowlink = states[v].lowlink.min(states[w].index);
            }
        }

        if states[v].lowlink == states[v].index {
            sccs.push(stack.clone());            
            while let Some(w) = stack.pop() {
                states[w].on_stack = false;
            }
        }
    }

    sccs.len() 
}
