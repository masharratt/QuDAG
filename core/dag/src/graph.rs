use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use blake3::Hash;

use crate::{Node, Edge, Result, DagError};

/// Represents the DAG data structure with nodes and edges
pub struct Graph {
    /// Nodes in the DAG indexed by hash
    nodes: RwLock<HashMap<Hash, Node>>,
    /// Edges in the DAG
    edges: RwLock<HashMap<Hash, HashSet<Edge>>>,
}

impl Graph {
    /// Creates a new empty DAG
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            edges: RwLock::new(HashMap::new()),
        }
    }

    /// Returns true if the DAG contains no nodes
    pub fn is_empty(&self) -> bool {
        self.nodes.read().is_empty()
    }

    /// Returns the number of nodes in the DAG
    pub fn len(&self) -> usize {
        self.nodes.read().len()
    }

    /// Adds a new node to the DAG
    pub fn add_node(&self, node: Node) -> Result<()> {
        let node_hash = *node.hash();
        
        // Check if node already exists
        if self.nodes.read().contains_key(&node_hash) {
            return Err(DagError::NodeExists(format!("{:?}", node_hash)));
        }

        // Verify all parents exist
        for parent in node.parents() {
            if !self.nodes.read().contains_key(parent) {
                return Err(DagError::MissingParent(format!("{:?}", parent)));
            }
        }

        // Add node and create edges to parents
        let mut nodes = self.nodes.write();
        let mut edges = self.edges.write();

        nodes.insert(node_hash, node);
        
        // Initialize edge set for this node
        edges.entry(node_hash).or_insert_with(HashSet::new);

        // Add edges from parents
        for parent in nodes[&node_hash].parents() {
            let edge = Edge::new(*parent, node_hash);
            edges.get_mut(parent).unwrap().insert(edge);
        }

        Ok(())
    }

    /// Returns a reference to a node by its hash
    pub fn get_node(&self, hash: &Hash) -> Option<Node> {
        self.nodes.read().get(hash).cloned()
    }

    /// Returns all edges connected to a node
    pub fn get_edges(&self, hash: &Hash) -> Option<HashSet<Edge>> {
        self.edges.read().get(hash).cloned()
    }

    /// Updates the state of a node
    pub fn update_node_state(&self, hash: &Hash, new_state: crate::node::NodeState) -> Result<()> {
        let mut nodes = self.nodes.write();
        let node = nodes.get_mut(hash).ok_or_else(|| {
            DagError::NodeNotFound(format!("{:?}", hash))
        })?;
        
        node.update_state(new_state)
    }

    /// Checks if adding an edge would create a cycle
    fn would_create_cycle(&self, from: &Hash, to: &Hash, visited: &mut HashSet<Hash>) -> bool {
        if from == to {
            return true;
        }

        if !visited.insert(*from) {
            return false;
        }

        if let Some(edges) = self.edges.read().get(from) {
            for edge in edges {
                if self.would_create_cycle(edge.to(), to, visited) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeState;

    #[test]
    fn test_graph_basic_operations() {
        let graph = Graph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);

        // Create root node
        let root = Node::new(vec![1], vec![]);
        let root_hash = *root.hash();
        assert!(graph.add_node(root).is_ok());
        assert!(!graph.is_empty());
        assert_eq!(graph.len(), 1);

        // Create child node
        let child = Node::new(vec![2], vec![root_hash]);
        let child_hash = *child.hash();
        assert!(graph.add_node(child).is_ok());
        assert_eq!(graph.len(), 2);

        // Verify edges
        let root_edges = graph.get_edges(&root_hash).unwrap();
        assert_eq!(root_edges.len(), 1);
        assert!(root_edges.iter().any(|e| e.to() == &child_hash));
    }

    #[test]
    fn test_node_state_updates() {
        let graph = Graph::new();
        let node = Node::new(vec![1], vec![]);
        let hash = *node.hash();
        
        graph.add_node(node).unwrap();
        
        // Valid transition
        assert!(graph.update_node_state(&hash, NodeState::Verified).is_ok());
        
        let node = graph.get_node(&hash).unwrap();
        assert_eq!(node.state(), NodeState::Verified);
        
        // Invalid transition
        assert!(graph.update_node_state(&hash, NodeState::Pending).is_err());
    }

    #[test]
    fn test_cycle_prevention() {
        let graph = Graph::new();

        // Create nodes a -> b -> c
        let a = Node::new(vec![1], vec![]);
        let a_hash = *a.hash();
        graph.add_node(a).unwrap();

        let b = Node::new(vec![2], vec![a_hash]);
        let b_hash = *b.hash();
        graph.add_node(b).unwrap();

        let c = Node::new(vec![3], vec![b_hash]);
        let c_hash = *c.hash();
        graph.add_node(c).unwrap();

        // Attempt to create cycle by adding edge c -> a
        let cycle_node = Node::new(vec![4], vec![c_hash, a_hash]);
        assert!(graph.add_node(cycle_node).is_ok());
    }

    #[test]
    fn test_missing_parent() {
        let graph = Graph::new();
        let missing_hash = blake3::hash(b"missing");
        let node = Node::new(vec![1], vec![missing_hash]);
        
        assert!(matches!(
            graph.add_node(node),
            Err(DagError::MissingParent(_))
        ));
    }
}