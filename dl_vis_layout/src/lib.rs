extern crate cassowary;

extern crate file_parser as parser;

use std::collections::{HashMap, HashSet, VecDeque};

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
    // Open nodes
    let mut open_set = VecDeque::new();
    //Already visited nodes
    let mut closed_set = HashSet::new();

    //Contains meta information about placement
    let mut position_meta = HashMap::new();
    let mut operation_meta = HashMap::new();

    let start = graph.get_start();

    open_set.push_front(start.id);

    //Store all nodes for the cassowary algorithm
    let mut layout = HashMap::new();
    layout.insert(start.id, Node::new());

    let mut nodes = HashMap::new();
    nodes.insert(start.id, start);

    while !open_set.is_empty() {
        let parent_id = open_set.pop_front();
        let parent = nodes.get(parent_id).expect(format!("Parent {} couldn't be found in Hashmap", parent_id));

        for neighbor in parent.neighbors {
            let child = graph.get_neighbor(cur_node, neighbor).expect(format!("Neighbor {:?} not found for id {}", neighbor, cur_node.id));

            //Already visited. No need to create a cycle
            if closed_set.contains(child.id) {
                continue;
            }

            //Create new node for child
            if !layout.contains_key(child.id) {
                layout.insert(child.id, Node::new());
            }

            if !nodes.contains_key(child.id) {
                nodes.insert(child.id, child);
            }

            if !open_set.contains(child.id) {
                open_set.push_front(child.id)
            }

            //Set the positional relationship to the child
            let positions = position_meta.entry(parent.id).or_insert(Vec::new());
            *positions.push((child.id, neighbor));
        }

        if parent.operations.is_some() {
            for operation in parent.operations.unwrap {
                if !layout.contains_key(child.id) {
                    layout.insert(child.id, Node::new());
                }
                if !nodes.contains_key(child.id) {
                    nodes.insert(child.id, child);
                }

                let operation = operation_meta.entry(parent.id).or_insert(Vec::new());
                *operation.push((operation.to, operation.operation));
            }
        }

        closed_set.insert(child.id);
    }






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