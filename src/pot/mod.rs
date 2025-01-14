//! # Potential Computation
//!
//! Some implementations of algorithms to compute feasible potential functions.
//! This module is only a mockup and thus not documented.
//! It is not used in any experiment.

use std::{
    fmt::Debug,
    io::{BufRead, Error, ErrorKind},
};

use crate::weight::Weight;

pub mod alternating;
pub mod checks;
pub mod det_mc;
pub mod johnson;

pub type Node = usize;
pub type Edge<W> = (Node, Node, W);

#[derive(Debug, Clone)]
pub struct Graph<W: Weight> {
    neighbors: Vec<(Node, W)>,
    offsets: Vec<usize>,
    neg_offsets: Vec<usize>,
}

impl<W: Weight> Graph<W> {
    #[inline]
    pub fn n(&self) -> usize {
        self.neg_offsets.len()
    }

    #[inline]
    pub fn m(&self) -> usize {
        self.neighbors.len()
    }

    #[inline]
    pub fn neighbors(&self, u: Node) -> &[(Node, W)] {
        &self.neighbors[self.offsets[u]..self.offsets[u + 1]]
    }

    #[inline]
    pub fn pos_neighbors(&self, u: Node) -> &[(Node, W)] {
        &self.neighbors[self.neg_offsets[u]..self.offsets[u + 1]]
    }

    #[inline]
    pub fn neg_neighbors(&self, u: Node) -> &[(Node, W)] {
        &self.neighbors[self.offsets[u]..self.neg_offsets[u]]
    }

    #[inline]
    pub fn has_neg_neighbors(&self, u: Node) -> bool {
        self.neg_offsets[u] > self.offsets[u]
    }

    #[inline]
    pub fn pot_weight(&self, edge: &Edge<W>, pot: &[W]) -> W {
        edge.2 + pot[edge.1] - pot[edge.0]
    }

    pub fn from_edges(n: usize, mut edges: Vec<Edge<W>>) -> Self {
        edges.sort_unstable_by(|(u1, _, w1), (u2, _, w2)| (u1, w1).partial_cmp(&(u2, w2)).unwrap());

        let mut neighbors = Vec::with_capacity(edges.len());
        let mut offsets = Vec::with_capacity(n + 1);

        let mut curr_node = 0usize;
        offsets.push(0usize);

        edges.into_iter().enumerate().for_each(|(i, (u, v, w))| {
            neighbors.push((v, w));

            while curr_node < u {
                offsets.push(i);
                curr_node += 1;
            }
        });
        for _ in curr_node..n {
            offsets.push(neighbors.len());
        }

        let neg_offsets = (0..n)
            .map(|u| {
                (offsets[u]..offsets[u + 1])
                    .find(|i| neighbors[*i].1 >= W::zero())
                    .unwrap_or(offsets[u + 1])
            })
            .collect();

        Self {
            neighbors,
            offsets,
            neg_offsets,
        }
    }
}

/// Returns an IO-Error with a custom error message.
#[inline]
fn io_error<O>(msg: &str) -> Result<O, Error> {
    Err(Error::new(ErrorKind::Other, msg))
}

/// Reads a graph from file
pub fn read_graph_from_file<R: BufRead, W: Weight>(
    reader: R,
) -> Result<(usize, Vec<Edge<W>>), Error> {
    let mut lines = reader.lines().filter_map(|x| -> Option<String> {
        if let Ok(line) = x {
            if !line.starts_with('%') {
                return Some(line);
            }
        }
        None
    });

    let (n, m) = parse_header(&mut lines)?;

    let mut edges = Vec::with_capacity(m);

    for (line, content) in lines.enumerate() {
        if line >= m {
            return io_error("Too many edges given");
        }

        let edge: Vec<_> = content.trim().split(',').collect();
        if edge.len() != 3 {
            return io_error(
                format!(
                    "Line {}: An edge should consist of exactly 2 nodes and a weight!",
                    line + 1
                )
                .as_str(),
            );
        }

        let u: Node = match edge[0].parse::<Node>() {
            Ok(u) => u,
            Err(_) => {
                return io_error(format!("Line {}: Cannot parse first node!", line + 1).as_str())
            }
        };

        let v: Node = match edge[1].parse::<Node>() {
            Ok(v) => v,
            Err(_) => {
                return io_error(format!("Line {}: Cannot parse second node!", line + 1).as_str())
            }
        };

        let w: W = match edge[2].parse::<W>() {
            Ok(w) => w,
            Err(_) => return io_error(format!("Line {}: Cannot parse weight!", line + 1).as_str()),
        };

        if u >= n as Node || v >= n as Node {
            return io_error(
                format!("Line {}: Node {u} in edge is bigger than n={n}!", line + 1).as_str(),
            );
        }

        edges.push((u, v, w));
    }

    Ok((n, edges))
}

/// Parses the header of a graph file and returns (name, n, m) or an IO-Error.
#[inline]
fn parse_header<I: Iterator<Item = String>>(lines: &mut I) -> Result<(usize, usize), Error> {
    if let Some(header) = lines.next() {
        let fields: Vec<_> = header.split(' ').collect();
        if fields.len() < 3 {
            return io_error("Expected at least 3 header fields");
        }

        let n: usize = match fields[1].parse() {
            Ok(n) => n,
            Err(_) => return io_error("Cannot parse number of nodes"),
        };

        let m: usize = match fields[2].parse() {
            Ok(m) => m,
            Err(_) => return io_error("Cannot parse number of edges"),
        };

        Ok((n, m))
    } else {
        io_error("Cannot read header")
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NegativeCycleFound;

pub trait Potentials<W>
where
    W: Weight,
    [(); W::NUM_BITS + 1]: Sized,
{
    fn compute_pot(graph: &Graph<W>) -> Result<Vec<W>, NegativeCycleFound>;
}
