extern crate toml;

#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DLVisToml {
    start: usize,
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
    pub skip_connection_to: Link,
    pub operation: Option<Operation>,
    pub pass_to: Link,

    pub below_of: Link,
    pub above_of: Link,
    pub right_of: Link,
    pub left_of: Link,
}

impl Node {
    pub fn from_toml(toml_node: NodeToml) -> Result<Node, String> {
        let operation = Operation::from_toml(toml_node.operation);

        match operation  {
            Ok(op) =>
                Ok(Node { id: toml_node.id, dimension: toml_node.dimension,
                    skip_connection_to: toml_node.skip_connection_to,
                    operation: op, pass_to: toml_node.pass_to, below_of: toml_node.below_of,
                    above_of: toml_node.above_of, right_of: toml_node.right_of, left_of: toml_node.left_of }),
            Err(err) => Err(err)
        }
    }

    fn return_all_links(&self) -> Vec<usize> {
        let mut ret = Vec::new();
        if let Some(skip) = self.skip_connection_to {
            ret.push(skip);
        }
        if let Some(ref operation) = self.operation {
            ret.push(operation.to);
        }
        if let Some(pass_to) = self.pass_to {
            ret.push(pass_to);
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
        normalization_fn: Option<String>}
}

pub struct Operation {
    to: usize,
    operation: Op
}

impl Operation {
    pub fn from_toml(optoml: Option<OperationToml>) -> Result<Option<Operation>, String> {
        if let Some(input) = optoml {
            if input.convolution.is_some() && input.deconvolution.is_none() && input.fully_connected.is_none() {
                let conv = input.convolution.unwrap();
                let op: Op = Op::Convolution { dimension: conv.dimension, kernel_size: conv.kernel_size,
                    num_outputs: conv.num_outputs, stride: conv.stride, max_pool: conv.max_pool,
                    activation_fn: conv.activation_fn, normalization_fn: conv.normalization_fn };
                return Ok( Some(Operation { to: input.to, operation: op }) )
            } else if input.convolution.is_none() && input.deconvolution.is_some() && input.fully_connected.is_none() {
                let deconv = input.deconvolution.unwrap();
                let op: Op = Op::Deconvolution { dimension: deconv.dimension, kernel_size: deconv.kernel_size,
                    num_outputs: deconv.num_outputs, stride: deconv.stride, max_pool: deconv.max_pool,
                    activation_fn: deconv.activation_fn, normalization_fn: deconv.normalization_fn };
                return Ok( Some(Operation { to: input.to, operation: op }) )
            } else if input.convolution.is_none() && input.deconvolution.is_none() && input.fully_connected.is_some() {
                let fc = input.fully_connected.unwrap();
                let op: Op = Op::FullyConnected { num_outputs: fc.num_outputs,
                    activation_fn: fc.activation_fn, normalization_fn: fc.normalization_fn };
                return Ok( Some(Operation { to: input.to, operation: op }) )
            } else {
                return Err("Only one operation is allowed!".to_string())
            }
        } else {
            return Ok( None )
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Convolution {
    dimension: u32,
    kernel_size: u32,
    num_outputs: u32,
    stride: Option<Vec<u32>>,
    max_pool: Option<Vec<u32>>,
    activation_fn: Option<String>,
    normalization_fn: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Deconvolution {
    dimension: u32,
    kernel_size: u32,
    num_outputs: u32,
    stride: Option<Vec<u32>>,
    max_pool: Option<Vec<u32>>,
    activation_fn: Option<String>,
    normalization_fn: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct FullyConnected {
    num_outputs: u32,
    activation_fn: Option<String>,
    normalization_fn: Option<String>
}

pub struct DLVis {
    nodes: HashMap<usize, Node>,
    start: usize,
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

        Ok(DLVis { nodes: node_map, start: input.start })
    }

    pub fn get_start(&self) -> &Node {
        self.nodes.get(&self.start).unwrap()
    }

    pub fn get_above_of(&self, node: Node) -> Option<&Node> {
        if let Some(link) = node.above_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_below_of(&self, node: Node) -> Option<&Node> {
        if let Some(link) = node.below_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_left_of(&self, node: Node) -> Option<&Node> {
        if let Some(link) = node.left_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_right_of(&self, node: Node) -> Option<&Node> {
        if let Some(link) = node.right_of {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_operation_to(&self, node: Node) -> Option<(Option<&Node>, Op)> {
        if let Some(op) = node.operation {
            return Some((self.nodes.get(&op.to), op.operation))
        } else {
            return None;
        }
    }

    pub fn get_pass_to(&self, node: Node) -> Option<&Node> {
        if let Some(link) = node.pass_to {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }

    pub fn get_skip_connection_to(&self, node: Node) -> Option<&Node> {
        if let Some(link) = node.skip_connection_to {
            return self.nodes.get(&link)
        } else {
            return None;
        }
    }
}

fn parse_file(input: String) -> Result<DLVis, String> { //TODO return error enums
    match toml::from_str(&input) {
        Ok(res) => return DLVis::from_toml_input(res),
        Err(e) => return Err(format!("Error decoding the input: {:?}", e))
    }
}


