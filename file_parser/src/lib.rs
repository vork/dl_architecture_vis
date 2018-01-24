extern crate toml;

#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DLVisToml {
    start: usize,
    end: usize,
    nodes: Vec<NodeToml>
}

type Link = Option<usize>;

#[derive(Debug, Deserialize)]
pub struct NodeToml {
    pub id: usize,
    pub dimension: Vec<u32>,
    pub skip_connection_to: Link,
    pub operation: Option<OperationToml>,
    pub pass_to: Link,

    pub below_of: Link,
    pub above_of: Link,
    pub right_of: Link,
    pub left_of: Link,
}

#[derive(Debug, Deserialize)]
pub struct OperationToml {
    to: usize,
    convolution: Option<Convolution>,
    deconvolution: Option<Deconvolution>,
    fully_connected: Option<FullyConnected>
}

pub struct Node {
    pub id: usize,
    pub dimension: Vec<u32>,
    pub operations: Option<Vec<Operation>>,

    pub neighbors: Vec<Neighbors>,
    pub below_of: Link,
    pub above_of: Link,
    pub right_of: Link,
    pub left_of: Link,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Neighbors {
    Left, Right, Above, Below
}

impl Node {
    pub fn from_toml(toml_node: NodeToml) -> Result<Node, String> {
        let mut neighbors = Vec::new();
        if toml_node.below_of.is_some() {
            neighbors.push(Neighbors::Below)
        }
        if toml_node.above_of.is_some() {
            neighbors.push(Neighbors::Above)
        }
        if toml_node.right_of.is_some() {
            neighbors.push(Neighbors::Right)
        }
        if toml_node.left_of.is_some() {
            neighbors.push(Neighbors::Left)
        }

        let operation = Operation::from_toml(&toml_node);
        Ok(Node {
            id: toml_node.id,
            dimension: toml_node.dimension,
            operations: operation,
            neighbors,
            below_of: toml_node.below_of,
            above_of: toml_node.above_of,
            right_of: toml_node.right_of,
            left_of: toml_node.left_of
        })

    }

    fn return_all_links(&self) -> Vec<usize> {
        let mut ret = Vec::new();
        if let Some(ref operations) = self.operations {
            for op in operations {
                ret.push(op.to);
            }
        }

        if let Some(below_of) = self.below_of {
            ret.push(below_of);
        }
        if let Some(above_of) = self.above_of {
            ret.push(above_of);
        }
        if let Some(right_of) = self.right_of {
            ret.push(right_of);
        }
        if let Some(left_of) = self.left_of {
            ret.push(left_of);
        }

        ret
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Op {
    Convolution{
        dimension: u32,
        kernel_size: u32,
        num_outputs: u32,
        stride: Option<Vec<u32>>,
        max_pool: Option<Vec<u32>>,
        activation_fn: Option<String>,
        normalization_fn: Option<String>},
    Deconvolution {
        dimension: u32,
        kernel_size: u32,
        num_outputs: u32,
        stride: Option<Vec<u32>>,
        max_pool: Option<Vec<u32>>,
        activation_fn: Option<String>,
        normalization_fn: Option<String>},
    FullyConnected {
        num_outputs: u32,
        activation_fn: Option<String>,
        normalization_fn: Option<String>},
    PassTo,
    SkipTo,
}
#[derive(Debug, Clone)]
pub struct Operation {
    pub to: usize,
    pub operation: Op
}

impl Operation {
    pub fn from_toml(node_toml: &NodeToml) -> Option<Vec<Operation>> {
        let optoml = &node_toml.operation;
        let mut operations = Vec::new();
        if let &Some(ref input) = optoml {
            if input.convolution.is_some() && input.deconvolution.is_none() && input.fully_connected.is_none() {
                let conv = input.convolution.clone().unwrap(); //TODO clone is not really the way to go
                let op: Op = Op::Convolution { dimension: conv.dimension, kernel_size: conv.kernel_size,
                    num_outputs: conv.num_outputs, stride: conv.stride, max_pool: conv.max_pool,
                    activation_fn: conv.activation_fn, normalization_fn: conv.normalization_fn };
                operations.push(Operation { to: input.to, operation: op });
            } else if input.convolution.is_none() && input.deconvolution.is_some() && input.fully_connected.is_none() {
                let deconv = input.deconvolution.clone().unwrap();
                let op: Op = Op::Deconvolution { dimension: deconv.dimension, kernel_size: deconv.kernel_size,
                    num_outputs: deconv.num_outputs, stride: deconv.stride, max_pool: deconv.max_pool,
                    activation_fn: deconv.activation_fn, normalization_fn: deconv.normalization_fn };
                operations.push(Operation { to: input.to, operation: op });
            } else if input.convolution.is_none() && input.deconvolution.is_none() && input.fully_connected.is_some() {
                let fc = input.fully_connected.clone().unwrap();
                let op: Op = Op::FullyConnected { num_outputs: fc.num_outputs,
                    activation_fn: fc.activation_fn, normalization_fn: fc.normalization_fn };
                operations.push(Operation { to: input.to, operation: op });
            }
        }
        if node_toml.pass_to.is_some() {
            operations.push(Operation { to: node_toml.pass_to.unwrap(), operation: Op::PassTo});
        }
        if node_toml.skip_connection_to.is_some() {
            operations.push(Operation { to: node_toml.skip_connection_to.unwrap(), operation: Op::SkipTo });
        }

        if operations.is_empty() {
            return None;
        } else {
            Some(operations)
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Convolution {
    dimension: u32,
    kernel_size: u32,
    num_outputs: u32,
    stride: Option<Vec<u32>>,
    max_pool: Option<Vec<u32>>,
    activation_fn: Option<String>,
    normalization_fn: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct Deconvolution {
    dimension: u32,
    kernel_size: u32,
    num_outputs: u32,
    stride: Option<Vec<u32>>,
    max_pool: Option<Vec<u32>>,
    activation_fn: Option<String>,
    normalization_fn: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct FullyConnected {
    num_outputs: u32,
    activation_fn: Option<String>,
    normalization_fn: Option<String>
}

pub struct DLVis {
    nodes: HashMap<usize, Node>,
    start: usize,
    end: usize
}

impl DLVis {
    pub fn from_toml_input(input: DLVisToml) -> Result<DLVis, String> {
        let mut node_map = HashMap::new();

        for node_toml in input.nodes {
            let node = Node::from_toml(node_toml);
            match node {
                Ok(val) => node_map.insert(val.id, val),
                Err(err) => return Err(err),
            };
        }

        //Check validity
        for node in node_map.values() {
            let links = node.return_all_links();
            for link in links {
                if !node_map.contains_key(&link) {
                    return Err(format!("Node {0} links to {1}. {1} is not known.", node.id, link));
                }
            }
        }

        Ok(DLVis { nodes: node_map, start: input.start, end: input.end })
    }

    pub fn get_node(&self, id: usize) -> Option<&Node> {
        return self.nodes.get(&id)
    }

    pub fn get_start(&self) -> &Node {
        self.nodes.get(&self.start).unwrap()
    }

    pub fn get_above_of(&self, node: &Node) -> Option<&Node> {
        if let Some(link) = node.above_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_below_of(&self, node: &Node) -> Option<&Node> {
        if let Some(link) = node.below_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_left_of(&self, node: &Node) -> Option<&Node> {
        if let Some(link) = node.left_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_right_of(&self, node: &Node) -> Option<&Node> {
        if let Some(link) = node.right_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_operation_to<'a>(&'a self, node: &'a Node) -> Option<Vec<(Option<&'a Node>, &'a Op)>> {
        if let Some(ref ops) = node.operations {
            let mut ret = Vec::new();
            for op in ops {
                ret.push((self.nodes.get(&op.to), &op.operation));
            }
            return Some(ret)
        } else {
            return None;
        }
    }

    pub fn get_neighbor(&self, node: &Node, neighbor: Neighbors) -> Option<&Node> {
        if neighbor == Neighbors::Left {
            return self.get_left_of(node);
        } else if neighbor == Neighbors::Right {
            return self.get_right_of(node);
        } else if neighbor == Neighbors::Above {
            return self.get_above_of(node);
        } else {
            return self.get_below_of(node);
        }
    }
}

pub fn parse_file(input: String) -> Result<DLVis, String> { //TODO return error enums
    match toml::from_str(&input) {
        Ok(res) => return DLVis::from_toml_input(res),
        Err(e) => return Err(format!("Error decoding the input: {:?}", e))
    }
}

#[test]
fn parse_test() {
    let toml_str = r#"
start = 1
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

    let decoded: DLVis = parse_file(toml_str.to_string()).unwrap();


    assert_eq!(decoded.start, 1);
    assert_eq!(decoded.end, 3);

    let node2 = decoded.get_node(2).unwrap();

    assert!(node2.operations.is_some());

    let operations = decoded.get_operation_to(node2).unwrap();
    assert_eq!(operations[0].0.unwrap().id, 3);
    match operations[0].1 {
        &Op::Convolution { .. }=> assert!(true),
        _ => assert!(false)
    }

    assert_eq!(decoded.get_node(2).unwrap().neighbors[0], Neighbors::Above);
    assert_eq!(decoded.get_node(1).unwrap().neighbors[0], Neighbors::Left);
}
