extern crate cassowary;

extern crate file_parser as parser;

use std::collections::HashMap;

use cassowary::{ Solver, Variable };
use cassowary::WeightedRelation::*;
use cassowary::strength::{ WEAK, MEDIUM, STRONG, REQUIRED };

use parser::DLVis;

struct Node {
    left: Variable,
    right: Variable
}

impl Node {
    pub fn new() -> Self {
        Node { left: Variable::new(), right: Variable::new() }
    }
}

const NODE_SIZE: i32 = 100;

fn iterate_graph(graph: DLVis) {
    let mut layout = HashMap::new();

    let root = graph.get_start();

    layout.insert(root.id, Node::new());

    let mut cur_node = root;
    while !cur_node.neighbors.is_empty() {
        for neighbor in cur_node.neighbors {
            let node = graph.get_neighbor(cur_node, neighbor).expect(format!("Neighbor {:?} not found for id {}", neighbor, cur_node.id));
            layout.insert(node.id, Node::new())
        }
    }
}

pub fn render_file(toml: String) {
    match parser::parse_file(toml) {
        Ok(dlvis) => iterate_graph(dlvis),
        Err(err) => panic!(err),
    }
}