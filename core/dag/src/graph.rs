use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use blake3::Hash;
use dashmap::DashMap;
use rayon::prelude::*;
use std::time::Instant;
use tracing::{debug, warn, info};

use crate::{Node, Edge, Result, DagError};

/// Represents the DAG data structure with nodes and edges
/// Graph performance metrics
#[derive(Debug, Default)]
pub struct GraphMetrics {
    /// Average vertex processing time in nanoseconds
    pub avg_vertex_time_ns: u64,
    /// Number of vertices processed
    pub vertices_processed: u64,
    /// Cache hit rate for vertex lookups
    pub cache_hit_rate: f64,
}

/// Represents the DAG data structure with high-performance concurrent access
pub struct Graph {
    /// Nodes in the DAG with concurrent access
    nodes: DashMap<Hash, Node>,
    /// Edges in the DAG with concurrent access
    edges: DashMap<Hash, HashSet<Edge>>,
    /// Performance metrics
    metrics: RwLock<GraphMetrics>,
}

impl Graph {
    /// Creates a new empty DAG
    pub fn new() -> Self {
        // Initialize with reasonable capacity
        let initial_capacity = 10_000;
        Self::with_capacity(initial_capacity)
    }

    /// Creates a new Graph with specified initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: DashMap::with_capacity(capacity),
            edges: DashMap::with_capacity(capacity),
            metrics: RwLock::new(GraphMetrics::default()),
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
        let start = Instant::now();
        let node_hash = *node.hash();
        
        // Check if node already exists using lockless read
        if self.nodes.contains_key(&node_hash) {
            return Err(DagError::NodeExists(format!("{:?}", node_hash)));
        }

        // Verify all parents exist concurrently
        let missing_parent = node.parents().par_iter().find_first(|parent| {
            !self.nodes.contains_key(parent)
        });

        if let Some(parent) = missing_parent {
            return Err(DagError::MissingParent(format!("{:?}", parent)));
        }

        // Add node
        self.nodes.insert(node_hash, node);
        
        // Initialize edge set
        self.edges.entry(node_hash).or_insert_with(HashSet::new);

        // Add edges from parents in parallel
        let node_parents = self.nodes.get(&node_hash).unwrap().parents().to_vec();
        node_parents.par_iter().for_each(|parent| {
            let edge = Edge::new(*parent, node_hash);
            if let Some(mut parent_edges) = self.edges.get_mut(parent) {
                parent_edges.insert(edge);
            }
        });

        // Update metrics
        let elapsed = start.elapsed().as_nanos() as u64;
        let mut metrics = self.metrics.write();
        metrics.vertices_processed += 1;
        metrics.avg_vertex_time_ns = 
            (metrics.avg_vertex_time_ns * (metrics.vertices_processed - 1) + elapsed) / 
            metrics.vertices_processed;

        Ok(())
    }

    /// Returns a reference to a node by its hash
    pub fn get_node(&self, hash: &Hash) -> Option<Node> {
        // Fast concurrent lookup
        self.nodes.get(hash).map(|node| node.clone())
    }

    /// Returns all edges connected to a node
    pub fn get_edges(&self, hash: &Hash) -> Option<HashSet<Edge>> {
        // Fast concurrent lookup
        self.edges.get(hash).map(|edges| edges.clone())
    }

    /// Updates the state of a node
    pub fn update_node_state(&self, hash: &Hash, new_state: crate::node::NodeState) -> Result<()> {
        // Get node with write access
        let mut entry = self.nodes.get_mut(hash)
            .ok_or_else(|| DagError::NodeNotFound(format!("{:?}", hash)))?;
            
        entry.value_mut().update_state(new_state)
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