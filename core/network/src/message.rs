#![deny(unsafe_code)]

use crate::types::{MessagePriority, NetworkMessage};
use anyhow::Result;
use std::collections::VecDeque;
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// High-performance message queue for network messages
pub struct MessageQueue {
    /// High priority message queue
    high_priority: Arc<Mutex<VecDeque<NetworkMessage>>>,
    /// Normal priority message queue  
    normal_priority: Arc<Mutex<VecDeque<NetworkMessage>>>,
    /// Low priority message queue
    low_priority: Arc<Mutex<VecDeque<NetworkMessage>>>,
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
    pub async fn enqueue(&self, msg: NetworkMessage) -> Result<()> {
        let queue = match msg.priority {
            MessagePriority::High => &self.high_priority,
            MessagePriority::Normal => &self.normal_priority,
            MessagePriority::Low => &self.low_priority,
        };
        
        queue.lock().await.push_back(msg);
        let _ = self.notify_tx.send(()).await;
        Ok(())
    }

    /// Dequeues the next message by priority
    pub async fn dequeue(&self) -> Option<NetworkMessage> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_message_queue() {
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
        assert!(queue.enqueue(msg2.clone()).await.is_ok());
        assert_eq!(queue.len().await, 2);

        // Test priority dequeue
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.id, "1"); // High priority dequeued first
        
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.id, "2"); // Normal priority dequeued second
    }
}