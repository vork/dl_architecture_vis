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

fn iterate_graph<'a>(graph: &'a DLVis) ->
                               (HashMap<usize, &'a parser::Node>,
                                HashMap<usize, Node>,
                                HashMap<usize, Vec<(usize, parser::Neighbors)>>,
                                HashMap<usize, Vec<(usize, parser::Op)>>) {
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
        if parent_id.is_some() {
            let pid = parent_id.unwrap();

            let neighbors = nodes.get(&pid)
                .expect(&format!("Parent {} couldn't be found in Hashmap", pid))
                .neighbors.clone();
            for neighbor in neighbors {
                let child = graph.get_neighbor(nodes.get(&pid)
                                                   .expect(&format!("Parent {} couldn't be found in Hashmap",
                                                                    pid)), neighbor)
                    .expect(&format!("Neighbor {:?} not found for id {}", neighbor, pid));

                //Already visited. No need to create a cycle
                if closed_set.contains(&child.id) {
                    continue;
                }

                //Create new node for child
                if !layout.contains_key(&child.id) {
                    layout.insert(child.id, Node::new());
                }

                if !nodes.contains_key(&child.id) {
                    nodes.insert(child.id, child);
                }

                if !open_set.contains(&child.id) {
                    open_set.push_front(child.id)
                }

                //Set the positional relationship to the child
                let positions = position_meta.entry(pid).or_insert(Vec::new());
                (*positions).push((child.id, neighbor));
            }

            let is_operation_some = nodes.get(&pid)
                .expect(&format!("Parent {} couldn't be found in Hashmap", pid))
                .operations.is_some();

            if is_operation_some {
                let operations = nodes.get(&pid).unwrap().operations.clone().unwrap();
                for operation in operations {
                    let id = operation.to;
                    let node = graph.get_node(id);

                    if node.is_some() {
                        if !layout.contains_key(&id) {
                            layout.insert(id, Node::new());
                        }
                        if !nodes.contains_key(&id) {
                            nodes.insert(id, node.unwrap());
                        }

                        let op = operation_meta
                            .entry(pid).or_insert(Vec::new());
                        (*op).push((operation.to, operation.operation));
                    }
                }
            }

            closed_set.insert(pid);
        }
    }
    return (nodes, layout, position_meta, operation_meta)
}

pub fn render_file(toml: String) {
    match parser::parse_file(toml) {
        Ok(dlvis) => {
            let (nodes, layout, position_meta, operation_meta) = iterate_graph(&dlvis);
        },
        Err(err) => panic!(err),
    }
}