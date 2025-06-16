#![deny(unsafe_code)]

use crate::types::{MessagePriority, NetworkMessage, NetworkError};
use serde::{Serialize, Deserialize};
use blake3::Hash;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use std::collections::VecDeque;
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// High-performance message queue for network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// The actual message
    pub message: NetworkMessage,
    /// Message hash for integrity
    pub hash: Hash,
    /// Timestamp
    pub timestamp: u64,
    /// Signature
    pub signature: Option<Vec<u8>>,
}

impl MessageEnvelope {
    pub fn new(message: NetworkMessage) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bincode::serialize(&message).unwrap());
        hasher.update(&timestamp.to_le_bytes());
        
        Self {
            message,
            hash: hasher.finalize(),
            timestamp,
            signature: None,
        }
    }
    
    pub fn verify(&self) -> bool {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bincode::serialize(&self.message).unwrap());
        hasher.update(&self.timestamp.to_le_bytes());
        
        self.hash == hasher.finalize()
    }
    
    pub fn sign(&mut self, key: &[u8]) -> Result<(), NetworkError> {
        // Sign the message hash
        let signature = ring::signature::Ed25519KeyPair::from_seed_unchecked(key)
            .map_err(|e| NetworkError::EncryptionError(e.to_string()))?;
            
        self.signature = Some(signature.sign(self.hash.as_bytes()).as_ref().to_vec());
        Ok(())
    }
    
    pub fn verify_signature(&self, public_key: &[u8]) -> Result<bool, NetworkError> {
        match &self.signature {
            Some(sig) => {
                let peer_public_key = ring::signature::UnparsedPublicKey::new(
                    &ring::signature::ED25519,
                    public_key
                );
                
                peer_public_key.verify(self.hash.as_bytes(), sig)
                    .map(|_| true)
                    .map_err(|e| NetworkError::EncryptionError(e.to_string()))
            }
            None => Ok(false)
        }
    }
}

pub struct MessageQueue {
    /// High priority message queue
    high_priority: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Normal priority message queue  
    normal_priority: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Low priority message queue
    low_priority: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Channel for message notifications
    notify_tx: mpsc::Sender<()>,
}

impl MessageQueue {
    /// Creates a new message queue
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(1000);
        
        let queue = Self {
            high_priority: Arc::new(Mutex::new(VecDeque::with_capacity(10000))),
            normal_priority: Arc::new(Mutex::new(VecDeque::with_capacity(50000))),
            low_priority: Arc::new(Mutex::new(VecDeque::with_capacity(100000))),
            notify_tx: tx,
        };
        
        (queue, rx)
    }

    /// Enqueues a message with the specified priority
    pub async fn enqueue(&self, msg: NetworkMessage) -> Result<(), NetworkError> {
        let envelope = MessageEnvelope::new(msg.clone());
        
        // Verify message integrity
        if !envelope.verify() {
            return Err(NetworkError::Internal("Message integrity check failed".into()));
        }
        let queue = match msg.priority {
            MessagePriority::High => &self.high_priority,
            MessagePriority::Normal => &self.normal_priority,
            MessagePriority::Low => &self.low_priority,
        };
        
        queue.lock().await.push_back(envelope);
        let _ = self.notify_tx.send(()).await;
        Ok(())
    }

    /// Dequeues the next message by priority
    pub async fn dequeue(&self) -> Option<MessageEnvelope> {
        if let Some(msg) = self.high_priority.lock().await.pop_front() {
            return Some(msg);
        }
        
        if let Some(msg) = self.normal_priority.lock().await.pop_front() {
            return Some(msg);
        }
        
        self.low_priority.lock().await.pop_front()
    }

    /// Returns the total number of queued messages
    pub async fn len(&self) -> usize {
        let high = self.high_priority.lock().await.len();
        let normal = self.normal_priority.lock().await.len();
        let low = self.low_priority.lock().await.len();
        high + normal + low
    }

    /// Purge expired messages
    pub async fn purge_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // Purge high priority
        let mut high = self.high_priority.lock().await;
        high.retain(|env| env.message.ttl.as_secs() + env.timestamp > now);
        
        // Purge normal priority
        let mut normal = self.normal_priority.lock().await;
        normal.retain(|env| env.message.ttl.as_secs() + env.timestamp > now);
        
        // Purge low priority
        let mut low = self.low_priority.lock().await;
        low.retain(|env| env.message.ttl.as_secs() + env.timestamp > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_message_queue() {
        use std::thread;
        
        let (queue, _rx) = MessageQueue::new();
        
        // Create test messages
        let msg1 = NetworkMessage {
            id: "1".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0; 100],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };

        let msg2 = NetworkMessage {
            id: "2".into(),
            source: vec![1],
            destination: vec![2], 
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Test enqueue
        assert!(queue.enqueue(msg1.clone()).await.is_ok());
        
        // Test message verification
        let envelope = queue.dequeue().await.unwrap();
        assert!(envelope.verify());
        assert!(queue.enqueue(msg2.clone()).await.is_ok());
        assert_eq!(queue.len().await, 2);

        // Test priority dequeue
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.message.id, "1"); // High priority dequeued first
        
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.message.id, "2"); // Normal priority dequeued second
        
        // Test message expiry
        let msg3 = NetworkMessage {
            id: "3".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0; 100],
            priority: MessagePriority::Low,
            ttl: Duration::from_secs(1), // Short TTL
        };
        
        assert!(queue.enqueue(msg3).await.is_ok());
        assert_eq!(queue.len().await, 1);
        
        // Wait for message to expire
        thread::sleep(Duration::from_secs(2));
        queue.purge_expired().await;
        assert_eq!(queue.len().await, 0);
    }
}