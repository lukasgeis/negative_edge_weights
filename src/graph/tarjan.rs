use std::{iter::FusedIterator, marker::PhantomData};

use crate::weight::Weight;

use super::{Edge, GraphNeigbors, GraphStats, Node};

/// Implementation of Tarjan's Algorithm for Strongly Connected Components.
/// It is designed as an iterator that emits the nodes of one strongly connected component at a
/// time. Observe that the order of nodes within a component is non-deterministic; the order of the
/// components themselves are in the reverse topological order of the SCCs (i.e. if each SCC
/// were contracted into a single node).
pub struct StronglyConnected<'a, W: Weight, G: GraphStats + GraphNeigbors<W>> {
    graph: &'a G,
    idx: Node,

    states: Vec<NodeState>,
    potentially_unvisited: usize,

    include_singletons: bool,

    path_stack: Vec<Node>,

    call_stack: Vec<StackFrame<'a, W, G>>,

    phantom: PhantomData<W>,
}

impl<'a, W: Weight, G: GraphStats + GraphNeigbors<W>> StronglyConnected<'a, W, G> {
    /// Construct the iterator for some graph
    pub fn new(graph: &'a G) -> Self {
        Self {
            graph,
            idx: 0,
            states: vec![Default::default(); graph.n()],
            potentially_unvisited: 0,

            include_singletons: true,

            path_stack: Vec::with_capacity(32),
            call_stack: Vec::with_capacity(32),

            phantom: Default::default(),
        }
    }

    /// Each node that is not part of a circle is returned as its own SCC.
    /// By setting `include = false`, those nodes are not returned (which can lead to a significant
    /// performance boost)
    pub fn set_include_singletons(&mut self, include: bool) {
        self.include_singletons = include;
    }

    /// Just like in a classic DFS where we want to compute a spanning-forest, we will need to
    /// to visit each node at least once. We start we node 0, and cover all nodes reachable from
    /// there in `search`. Then, we search for an untouched node here, and start over.
    fn next_unvisited_node(&mut self) -> Option<Node> {
        while self.potentially_unvisited < self.graph.n() {
            if !self.states[self.potentially_unvisited].visited {
                let v = self.potentially_unvisited as Node;
                self.push_node(v, None);
                return Some(v);
            }

            self.potentially_unvisited += 1;
        }
        None
    }

    /// Put a pristine stack frame on the call stack. Roughly speaking, this is the first step
    /// to a recursive call of search.
    fn push_node(&mut self, node: Node, parent: Option<Node>) {
        self.call_stack.push(StackFrame::<W, G> {
            node,
            parent: parent.unwrap_or(node),
            initial_stack_len: 0,
            first_call: true,
            has_loop: false,
            neighbors: self.graph.out_neighbors(node),
            phantom: Default::default(),
        });
    }

    fn search(&mut self) -> Option<Vec<Node>> {
        /*
        Tarjan's algorithm is typically described in a recursive fashion similarly to DFS
        with some extra steps. This design has two issues:
         1.) We cannot easily build an iterator from it
         2.) For large graphs we get stack overflows

        To overcome these issues, we use the explicit call stack `self.call_stack` that simulates
        recursive calls. On first visit of a node v it is assigned a "DFS rank"ish index and
        additionally the same low_link value. This value stores the smallest known index of any node known to be
        reachable from v. We then process all of its neighbors (which may trigger recursive calls).
        Eventually, all nodes in an SCC will have the same low_link and the unique node with this
        index becomes the arbitrary representative of this SCC (known as root).

        The key design is that the whole computation is wrapped in a `while` loop and all state
        (including iterators) is stored in `self.call_stack`. So we continue execution directly with
        another iteration. Alternative, we can pause processing, return an value and resume by
        reentering the function.
        */

        'recurse: while let Some(frame) = self.call_stack.last_mut() {
            let v = frame.node;

            if frame.first_call {
                frame.first_call = false;
                frame.initial_stack_len = self.path_stack.len() as Node;

                self.states[v as usize].visit(self.idx);
                self.idx += 1;

                self.path_stack.push(v);
            }

            for w in frame.neighbors.as_ref() {
                let w = w.target;
                let w_state = self.states[w];
                frame.has_loop |= w == v;

                if !w_state.visited {
                    self.push_node(w, Some(v));
                    continue 'recurse;
                } else if w_state.on_stack {
                    self.states[frame.node as usize].try_lower_link(w_state.index);
                }
            }

            let frame = self.call_stack.pop().unwrap();
            let state = self.states[v as usize];

            self.states[frame.parent as usize].try_lower_link(state.low_link);

            if state.is_root() {
                if !self.include_singletons
                    && *self.path_stack.last().unwrap() == v
                    && !frame.has_loop
                {
                    // skip producing component descriptor, since we have a singleton node
                    // but we need to undo
                    self.states[v as usize].on_stack = false;
                    self.path_stack.pop();
                } else {
                    // this component goes into the result, so produce a descriptor and clean-up stack
                    // while doing so
                    let component: Vec<_> = self.path_stack
                        [frame.initial_stack_len as usize..self.path_stack.len()]
                        .iter()
                        .copied()
                        .collect();

                    self.path_stack.truncate(frame.initial_stack_len as usize);

                    for &w in &component {
                        self.states[w as usize].on_stack = false;
                    }

                    debug_assert_eq!(*component.first().unwrap(), v);

                    return Some(component);
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
struct StackFrame<'a, W: Weight, G: GraphStats + GraphNeigbors<W> + 'a> {
    node: Node,
    parent: Node,
    initial_stack_len: Node,
    first_call: bool,
    has_loop: bool,
    neighbors: &'a [Edge<W>],
    phantom: PhantomData<G>,
}

#[derive(Debug, Clone, Copy, Default)]
struct NodeState {
    visited: bool,
    on_stack: bool,
    index: Node,
    low_link: Node,
}

impl NodeState {
    fn visit(&mut self, u: Node) {
        debug_assert!(!self.visited);
        self.index = u;
        self.low_link = u;
        self.visited = true;
        self.on_stack = true;
    }

    fn try_lower_link(&mut self, l: Node) {
        self.low_link = self.low_link.min(l);
    }

    fn is_root(&self) -> bool {
        self.index == self.low_link
    }
}

impl<'a, W: Weight, G: GraphStats + GraphNeigbors<W>> Iterator for StronglyConnected<'a, W, G> {
    type Item = Vec<Node>;

    /// Returns either a vector of node ids that form an SCC or None if no further SCC was found
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(x) = self.search() {
                return Some(x);
            }

            self.next_unvisited_node()?;
        }
    }
}

impl<'a, W: Weight, G: GraphStats + GraphNeigbors<W>> FusedIterator
    for StronglyConnected<'a, W, G>
{
}
