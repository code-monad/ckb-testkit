use crate::Node;
use std::collections::hash_map::{Keys, Values};
use std::collections::HashMap;

pub struct Nodes {
    _inner: HashMap<String, Node>,
}

impl From<HashMap<String, Node>> for Nodes {
    fn from(nodes: HashMap<String, Node>) -> Self {
        Nodes { _inner: nodes }
    }
}

impl From<Vec<Node>> for Nodes {
    fn from(nodes: Vec<Node>) -> Self {
        nodes
            .into_iter()
            .map(|node| (node.node_name().to_string(), node))
            .collect::<HashMap<_, _>>()
            .into()
    }
}

impl From<Nodes> for HashMap<String, Node> {
    fn from(nodes: Nodes) -> Self {
        nodes._inner
    }
}

impl AsRef<HashMap<String, Node>> for Nodes {
    fn as_ref(&self) -> &HashMap<String, Node> {
        &self._inner
    }
}

impl Nodes {
    pub fn get_node(&self, node_name: &str) -> &Node {
        assert!(self._inner.contains_key(node_name));
        self._inner.get(node_name).expect("checked above")
    }

    pub fn get_node_mut(&mut self, node_name: &str) -> &mut Node {
        assert!(self._inner.contains_key(node_name));
        self._inner.get_mut(node_name).expect("checked above")
    }

    pub fn node_names(&self) -> Keys<String, Node> {
        self._inner.keys()
    }

    pub fn nodes(&self) -> Values<String, Node> {
        self._inner.values()
    }
}
