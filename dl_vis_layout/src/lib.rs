extern crate cassowary;

extern crate file_parser as parser;

use std::collections::{HashMap, HashSet, VecDeque};

use cassowary::{ Solver, Variable };
use cassowary::WeightedRelation::*;
use cassowary::strength::{ WEAK, MEDIUM, STRONG, REQUIRED };

use parser::DLVis;

struct Node {
    left: Variable,
    right: Variable,
    upper: Variable,
    lower: Variable
}

impl Node {
    pub fn new() -> Self {
        Node { left: Variable::new(), right: Variable::new(), upper: Variable::new(), lower: Variable::new() }
    }
}

struct Line {
    x1: Variable,
    y1: Variable,
    x2: Variable,
    y2: Variable
}

impl Line {
    pub fn new() -> Self {
        Line { x1: Variable::new(), y1: Variable::new(), x2: Variable::new(), y2: Variable::new() }
    }
}

const NODE_SIZE: f32 = 100.0;
const NODE_SPACING: f32 = NODE_SIZE / 2.0;
const NODE_OVERLAY: f32 = 0.1;
const LINE_SPACING: f32 = NODE_SPACING / 3.0;

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


fn print_changes(names: &HashMap<Variable, String>, changes: &[(Variable, f64)]) {
    println!("Changes:");
    for &(ref var, ref val) in changes {
        println!("{}: {}", names[var], val);
    }
}

fn solve_layout(start: usize, end: usize,
                start_align_left: bool,
                start_align_right: bool,
                start_align_up: bool,
                start_align_down: bool,
                nodes: &HashMap<usize, &parser::Node>,
                layout: &HashMap<usize, Node>,
                position_meta: &HashMap<usize, Vec<(usize, parser::Neighbors)>>,
                operation_meta: &HashMap<usize, Vec<(usize, parser::Op)>>) -> HashMap<usize, Node> {
    let mut names = HashMap::new();

    let window_width = Variable::new();
    let window_height = Variable::new();

    names.insert(window_width,"window_width".to_string());
    names.insert(window_height,"window_height".to_string());

    let mut solver = Solver::new();

    solver.add_constraints(&[window_width |GE(REQUIRED)| 0.0,
        window_height |GE(REQUIRED)| 0.0]).unwrap(); // Positive window size

    if start_align_left {
        solver.add_constraint(layout.get(&start).expect("Start node wasn't found!").left |EQ(REQUIRED)| 0.0).unwrap();
    }
    if start_align_right {
        solver.add_constraint(layout.get(&start).expect("Start node wasn't found!").right |EQ(REQUIRED)| window_width).unwrap();
    }
    if start_align_up {
        solver.add_constraint(layout.get(&start).expect("Start node wasn't found!").upper |EQ(REQUIRED)| 0.0).unwrap();
    }
    if start_align_down {
        solver.add_constraint(layout.get(&start).expect("Start node wasn't found!").lower |EQ(REQUIRED)| window_height).unwrap();
    }
    /*solver.add_constraints(&[layout.get(&start).expect("Start node wasn't found!").left |EQ(REQUIRED)| 0.0, //Left align
        layout.get(&start).expect("Start node wasn't found!").upper |EQ(REQUIRED)| 0.0, //Up align
        layout.get(&end).expect("Start node wasn't found!").right |EQ(REQUIRED)| window_width, //Right align
        layout.get(&end).expect("Start node wasn't found!").upper |EQ(REQUIRED)| 0.0 //Up align
    ]).unwrap();*/

    for (id, ps) in position_meta {
        for &(to, alignment) in ps {
            let cur_node = layout.get(&id).expect(&format!("Couldn't find node {}", id));
            let align_to = layout.get(&to).expect(&format!("Couldn't find node {}", to));

            match alignment {
                parser::Neighbors::Left => {
                    solver.add_constraints(&[
                        cur_node.right | LE(REQUIRED) | align_to.left,
                        align_to.left - cur_node.right | GE(STRONG) | NODE_SPACING,
                        cur_node.upper | EQ(WEAK) | align_to.upper
                    ]).unwrap();
                },
                parser::Neighbors::Right => {
                    solver.add_constraints(&[
                        cur_node.left  | GE(REQUIRED) | align_to.right,
                        cur_node.left - align_to.right | GE(STRONG) | NODE_SPACING,
                        cur_node.upper | EQ(WEAK) | align_to.upper
                    ]).unwrap();
                },
                parser::Neighbors::Above => {
                    solver.add_constraints(&[
                        cur_node.lower | LE(REQUIRED) | align_to.upper,
                        align_to.upper - cur_node.lower  | GE(STRONG) | NODE_SPACING,
                        cur_node.left | EQ(WEAK) | align_to.left
                    ]).unwrap();
                },
                parser::Neighbors::Below => {
                    solver.add_constraints(&[
                        cur_node.upper | GE(REQUIRED) | align_to.lower,
                        align_to.lower - cur_node.upper  | GE(STRONG) | NODE_SPACING,
                        cur_node.left | EQ(WEAK) | align_to.left
                    ]).unwrap();
                }
            };
        }
    }

    for (id, node) in layout {
        let node_data = nodes.get(id).unwrap();

        let dims = &node_data.dimension;

        let mut final_size = NODE_SIZE;
        if dims.len() > 3 {
            let depth = dims[0];
            final_size += NODE_SIZE * depth as f32 * NODE_OVERLAY ;
        }

        solver.add_constraints(&[
            node.left |LE(REQUIRED)| node.right,
            node.upper |LE(REQUIRED)| node.lower,
            node.right - node.left |EQ(REQUIRED)| final_size,
            node.lower - node.upper |EQ(REQUIRED)| final_size,
        ]).unwrap();
        let name_left = format!("Node{}.Left", id);
        let name_right = format!("Node{}.Right", id);
        let name_upper = format!("Node{}.Upper", id);
        let name_lower = format!("Node{}.Lower", id);
        names.insert(node.left, name_left);
        names.insert(node.right, name_right);
        names.insert(node.upper, name_upper);
        names.insert(node.lower, name_lower);
    }

    print_changes(&names, solver.fetch_changes());

    for (id, os) in operation_meta {
        for &(to, ref operation) in os {
            let from_node = layout.get(&id).expect(&format!("Couldn't find node {}", id));
            let to_node = layout.get(&to).expect(&format!("Couldn't find node {}", to));

            match operation {
                &parser::Op::PassTo => {
                    let line = Line::new();
                    names.insert(line.x1, format!("Pass{}To{}.x1", id, to));
                    names.insert(line.y1, format!("Pass{}To{}.y1", id, to));
                    names.insert(line.x2, format!("Pass{}To{}.x2", id, to));
                    names.insert(line.y2, format!("Pass{}To{}.y2", id, to));

                    let is_node_left = if from_node.left < to_node.left { true } else { false };
                    let is_node_above = if from_node.upper < to_node.upper { true } else { false };

                    // This is ugly! and only works due to line 250
                    let horizontal_diff = solver.get_value(from_node.left) - solver.get_value(to_node.left);
                    let vertical_diff = solver.get_value(from_node.upper) - solver.get_value(to_node.upper);

                    let horizontal_line = if horizontal_diff >= vertical_diff { true } else { false };

                    if horizontal_line { // Horizontal line.
                        let start_point = if is_node_left {
                            (from_node.right, from_node.upper + (from_node.lower - from_node.upper) / 2.0)
                        } else {
                            (from_node.left, from_node.upper + (from_node.lower - from_node.upper) / 2.0)
                        };
                        let end_point = if is_node_left {
                            (to_node.left, to_node.upper + (to_node.lower - to_node.upper) / 2.0)
                        } else {
                            (to_node.right, to_node.upper + (to_node.lower - to_node.upper) / 2.0)
                        };
                        solver.add_constraints(&[
                            line.x1 | EQ(REQUIRED) | start_point.0 + LINE_SPACING,
                            line.y1 | EQ(REQUIRED) | start_point.1,
                            line.x2 | EQ(REQUIRED) | end_point.0 - LINE_SPACING,
                            line.y2 | EQ(REQUIRED) | end_point.1,
                        ]);
                    } else { //Vertical line
                        let start_point = if is_node_above {
                            (from_node.lower, from_node.left + (from_node.right - from_node.left) / 2.0)
                        } else {
                            (from_node.upper,  from_node.left + (from_node.right - from_node.left) / 2.0)
                        };
                        let end_point = if is_node_above {
                            (to_node.upper, to_node.left + (to_node.right - to_node.left) / 2.0)
                        } else {
                            (to_node.lower, to_node.left + (to_node.right - to_node.left) / 2.0)
                        };
                        solver.add_constraints(&[
                            line.x1 | EQ(REQUIRED) | start_point.1,
                            line.y1 | EQ(REQUIRED) | start_point.0 + LINE_SPACING,
                            line.x2 | EQ(REQUIRED) | end_point.1,
                            line.y2 | EQ(REQUIRED) | end_point.0 - LINE_SPACING,
                        ]);
                    }


                },
                &parser::Op::SkipTo => {
                    let line = Line::new();
                    names.insert(line.x1, format!("Skip{}To{}.x1", id, to));
                    names.insert(line.y1, format!("Skip{}To{}.y1", id, to));
                    names.insert(line.x2, format!("Skip{}To{}.x2", id, to));
                    names.insert(line.y2, format!("Skip{}To{}.y2", id, to));

                    let is_node_left = if from_node.left < to_node.left { true } else { false };

                    let start_point = if is_node_left {
                        (from_node.right, from_node.upper + (from_node.lower - from_node.upper) / 2.0)
                    } else {
                        (from_node.left, from_node.upper + (from_node.lower - from_node.upper) / 2.0)
                    };
                    let end_point = if is_node_left {
                        (to_node.left, to_node.upper + (to_node.lower - to_node.upper) / 2.0)
                    } else {
                        (to_node.right, to_node.upper + (to_node.lower - to_node.upper) / 2.0)
                    };
                    solver.add_constraints(&[
                        line.x1 | EQ(REQUIRED) | start_point.0 + (LINE_SPACING*2.0),
                        line.y1 | EQ(REQUIRED) | start_point.1,
                        line.x2 | EQ(REQUIRED) | end_point.0 - LINE_SPACING,
                        line.y2 | EQ(REQUIRED) | end_point.1,
                    ]);


                    let first_dim = nodes.get(&id).unwrap().dimension[0];

                    for i in 0..first_dim {
                        let connect_line = Line::new();
                        names.insert(connect_line.x1, format!("Connect{}.{}.x1", id, i));
                        names.insert(connect_line.y1, format!("Connect{}.{}.y1", id, i));
                        names.insert(connect_line.x2, format!("Connect{}.{}.x2", id, i));
                        names.insert(connect_line.y2, format!("Connect{}.{}.y2", id, i));

                        let node_point = if is_node_left {
                            (from_node.right, from_node.lower - (NODE_SIZE / 2.0) - (i as f32 * NODE_SIZE * NODE_OVERLAY))
                        } else {
                            (from_node.left, from_node.lower - (NODE_SIZE / 2.0) - (i as f32 * NODE_SIZE * NODE_OVERLAY))
                        };

                        solver.add_constraints(&[
                            connect_line.x1 | EQ(REQUIRED) | node_point.0 + LINE_SPACING,
                            connect_line.y1 | EQ(REQUIRED) | node_point.1,
                            connect_line.x2 | EQ(REQUIRED) | line.x1,
                            connect_line.y2 | EQ(REQUIRED) | line.y1,
                        ]);
                    }
                }
                _ => { //Ignore other operations for now
                    continue;
                },
            }
        }

    }

    solver.add_edit_variable(window_width, STRONG).unwrap();
    solver.add_edit_variable(window_height, STRONG).unwrap();

    solver.suggest_value(window_width, 1280.0).unwrap();
    solver.suggest_value(window_height, 600.0).unwrap();

    print_changes(&names, solver.fetch_changes());

    return HashMap::new()
}

pub fn render_file(toml: String) {
    match parser::parse_file(toml) {
        Ok(dlvis) => {
            let (nodes, layout, position_meta, operation_meta) = iterate_graph(&dlvis);
            solve_layout(dlvis.start, dlvis.end,
                         dlvis.start_align_left,
                         dlvis.start_align_right,
                         dlvis.start_align_up,
                         dlvis.start_align_down,
                         &nodes, &layout, &position_meta, &operation_meta);

        },
        Err(err) => panic!(err),
    }
}

#[test]
fn graph_test() {
    let toml_str = r#"
start = 1
start_align_left = true
start_align_up = true
end = 3

[[nodes]]
	id = 1
	dimension = [5, 512, 512, 1]
	pass_to = 2
	left_of = 2

[[nodes]]
	id = 2
	dimension = [5, 512, 512, 1]
	above_of = 3
	[nodes.operation]
		to = 3
		[nodes.operation.convolution]
			dimension = 3
			kernel_size = 3
			num_outputs = 128
			stride = [1, 2, 2]
			activation_fn = "relu"

[[nodes]]
	id = 3
	dimension = [5, 256, 256, 1]
    "#;

    let dlvis = parser::parse_file(toml_str.to_string()).unwrap();

    let (nodes, layout, position_meta, operation_meta) = iterate_graph(&dlvis);

    assert_eq!(layout.len(), 3);
    assert_eq!(nodes.len(), 3);
    assert_eq!(position_meta.len(), 2);
    assert_eq!(operation_meta.len(), 2);

    assert_eq!(position_meta.get(&1).unwrap()[0], (2, parser::Neighbors::Left));
    assert_eq!(position_meta.get(&2).unwrap()[0], (3, parser::Neighbors::Above));

    assert_eq!(operation_meta.get(&1).unwrap()[0], (2, parser::Op::PassTo));

    assert_eq!(solve_layout(dlvis.start, dlvis.end, dlvis.start_align_left,
                            dlvis.start_align_right,
                            dlvis.start_align_up,
                            dlvis.start_align_down, &nodes, &layout, &position_meta, &operation_meta).len(), 0);
}

