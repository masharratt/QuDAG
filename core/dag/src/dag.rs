use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, Mutex};
use thiserror::Error;
use tracing::error;

use crate::vertex::{Vertex, VertexId, VertexError};
use crate::consensus::{ConsensusError, QRAvalanche};

/// Errors that can occur during DAG operations
#[derive(Error, Debug)]
pub enum DagError {
    #[error("Vertex error: {0}")]
    VertexError(#[from] VertexError),
    
    #[error("Consensus error: {0}")]
    ConsensusError(#[from] ConsensusError),
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Conflict detected")]
    ConflictDetected,
    
    #[error("State sync failed")]
    StateSyncFailed,
}

/// Message type for DAG processing
#[derive(Debug, Clone)]
pub struct DagMessage {
    /// Unique message ID
    pub id: VertexId,
    /// Message payload
    pub payload: Vec<u8>,
    /// Parent vertex IDs
    pub parents: HashSet<VertexId>,
    /// Message timestamp
    pub timestamp: u64,
}

/// Represents the current state of message processing
#[derive(Debug)]
struct ProcessingState {
    /// Messages currently being processed
    processing: HashSet<VertexId>,
    /// Known conflicts between messages
    conflicts: HashMap<VertexId, HashSet<VertexId>>,
}

/// Main DAG structure for parallel message processing
#[derive(Debug, Clone)]
pub struct Dag {
    /// Vertices in the DAG
    vertices: Arc<RwLock<HashMap<VertexId, Vertex>>>,
    /// Current processing state
    state: Arc<RwLock<ProcessingState>>,
    /// Message processing channel
    msg_tx: mpsc::Sender<DagMessage>,
    /// Consensus mechanism
    consensus: Arc<Mutex<QRAvalanche>>,
    /// Maximum concurrent messages
    max_concurrent: usize,
}

impl Dag {
    /// Creates a new DAG instance
    pub fn new(max_concurrent: usize) -> Self {
        let (msg_tx, mut msg_rx) = mpsc::channel::<DagMessage>(1024);
        let vertices = Arc::new(RwLock::new(HashMap::new()));
        let state = Arc::new(RwLock::new(ProcessingState {
            processing: HashSet::new(),
            conflicts: HashMap::new(),
        }));
        let consensus = Arc::new(Mutex::new(QRAvalanche::new()));
        
        let vertices_clone = vertices.clone();
        let state_clone = state.clone();
        let consensus_clone = consensus.clone();
        
        // Spawn message processing task
        tokio::spawn(async move {
            while let Some(msg) = msg_rx.recv().await {
                let mut state = state_clone.write().await;
                if state.processing.len() >= max_concurrent {
                    // Wait for some messages to complete
                    continue;
                }
                state.processing.insert(msg.id);
                drop(state);
                
                let vertices = vertices_clone.clone();
                let state = state_clone.clone();
                let consensus = consensus_clone.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = Self::process_message(msg, vertices, state.clone(), consensus).await {
                        error!("Message processing failed: {}", e);
                    }
                    let mut state = state.write().await;
                    state.processing.remove(&msg.id);
                });
            }
        });
        
        Self {
            vertices,
            state,
            msg_tx,
            consensus,
            max_concurrent,
        }
    }
    
    /// Submits a message for processing
    pub async fn submit_message(&self, msg: DagMessage) -> Result<(), DagError> {
        self.msg_tx.send(msg).await.map_err(|_| DagError::ChannelClosed)
    }
    
    /// Processes a single message
    async fn process_message(
        msg: DagMessage,
        vertices: Arc<RwLock<HashMap<VertexId, Vertex>>>,
        state: Arc<RwLock<ProcessingState>>,
        consensus: Arc<Mutex<QRAvalanche>>,
    ) -> Result<(), DagError> {
        // Validate parents exist
        {
            let vertices = vertices.read().await;
            for parent in &msg.parents {
                if !vertices.contains_key(parent) {
                    return Err(DagError::VertexError(VertexError::ParentNotFound));
                }
            }
        }
        
        // Check for conflicts
        let conflicts = Self::detect_conflicts(&msg, &vertices).await?;
        if !conflicts.is_empty() {
            let mut state = state.write().await;
            state.conflicts.insert(msg.id, conflicts);
            return Err(DagError::ConflictDetected);
        }
        
        // Create new vertex
        let vertex = Vertex::new(msg.id.clone(), msg.payload, msg.parents);
        
        // Add to DAG
        {
            let mut vertices = vertices.write().await;
            vertices.insert(msg.id.clone(), vertex);
        }
        
        // Update consensus
        {
            let mut consensus = consensus.lock().await;
            consensus.process_vertex(msg.id)?;
        }
        
        Ok(())
    }
    
    /// Detects conflicts between messages
    async fn detect_conflicts(
        msg: &DagMessage,
        vertices: &Arc<RwLock<HashMap<VertexId, Vertex>>>,
    ) -> Result<HashSet<VertexId>, DagError> {
        let vertices = vertices.read().await;
        let mut conflicts = HashSet::new();
        
        // Simple conflict detection based on overlapping parents
        for (id, vertex) in vertices.iter() {
            if vertex.parents().intersection(&msg.parents).count() > 0 {
                conflicts.insert(id.clone());
            }
        }
        
        Ok(conflicts)
    }
    
    /// Synchronizes state with another DAG instance
    pub async fn sync_state(&self, other: &Dag) -> Result<(), DagError> {
        let other_vertices = other.vertices.read().await;
        let mut vertices = self.vertices.write().await;
        
        for (id, vertex) in other_vertices.iter() {
            if !vertices.contains_key(id) {
                vertices.insert(id.clone(), vertex.clone());
            }
        }
        
        let mut consensus = self.consensus.lock().await;
        consensus.sync()?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_parallel_message_processing() {
        let dag = Dag::new(4);
        
        let mut messages = Vec::new();
        for i in 0..10 {
            messages.push(DagMessage {
                id: VertexId::new(),
                payload: vec![i as u8],
                parents: HashSet::new(),
                timestamp: i as u64,
            });
        }
        
        // Submit messages concurrently
        let mut handles = Vec::new();
        for msg in messages {
            let dag = dag.clone();
            handles.push(tokio::spawn(async move {
                dag.submit_message(msg).await
            }));
        }
        
        // Wait for all messages to be processed
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        
        sleep(Duration::from_millis(100)).await;
        
        let vertices = dag.vertices.read().await;
        assert_eq!(vertices.len(), 10);
    }
    
    #[tokio::test]
    async fn test_conflict_detection() {
        let dag = Dag::new(4);
        
        // Create two messages with overlapping parents
        let parent_id = VertexId::new();
        let mut parents = HashSet::new();
        parents.insert(parent_id);
        
        let msg1 = DagMessage {
            id: VertexId::new(),
            payload: vec![1],
            parents: parents.clone(),
            timestamp: 1,
        };
        
        let msg2 = DagMessage {
            id: VertexId::new(),
            payload: vec![2],
            parents,
            timestamp: 2,
        };
        
        // Submit first message
        dag.submit_message(msg1.clone()).await.unwrap();
        sleep(Duration::from_millis(50)).await;
        
        // Second message should detect conflict
        let result = dag.submit_message(msg2).await;
        assert!(result.is_err());
        match result {
            Err(DagError::ConflictDetected) => (),
            _ => panic!("Expected conflict detection"),
        }
    }
    
    #[tokio::test]
    async fn test_state_sync() {
        let dag1 = Dag::new(4);
        let dag2 = Dag::new(4);
        
        // Add messages to first DAG
        let msg = DagMessage {
            id: VertexId::new(),
            payload: vec![1],
            parents: HashSet::new(),
            timestamp: 1,
        };
        
        dag1.submit_message(msg).await.unwrap();
        sleep(Duration::from_millis(50)).await;
        
        // Sync state to second DAG
        dag2.sync_state(&dag1).await.unwrap();
        
        let vertices1 = dag1.vertices.read().await;
        let vertices2 = dag2.vertices.read().await;
        assert_eq!(vertices1.len(), vertices2.len());
    }
}