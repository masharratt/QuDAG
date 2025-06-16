#![deny(unsafe_code)]

use crate::types::{ConnectionStatus, NetworkMetrics};
use anyhow::Result;
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manages network connections and their states
pub struct ConnectionManager {
    /// Maximum concurrent connections
    max_connections: usize,
    /// Connection status by peer ID
    connections: Arc<RwLock<HashMap<PeerId, ConnectionStatus>>>,
    /// Network performance metrics
    metrics: Arc<RwLock<NetworkMetrics>>,
}

impl ConnectionManager {
    /// Creates a new connection manager
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            connections: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
        }
    }

    /// Initiates a connection to a peer
    pub async fn connect(&self, peer_id: PeerId) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if connections.len() >= self.max_connections {
            warn!("Max connections reached");
            return Ok(());
        }
        
        connections.insert(peer_id, ConnectionStatus::Connecting);
        Ok(())
    }

    /// Updates connection status for a peer
    pub async fn update_status(&self, peer_id: PeerId, status: ConnectionStatus) {
        let mut connections = self.connections.write().await;
        connections.insert(peer_id, status);
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.connections = connections.len();
    }

    /// Disconnects from a peer
    pub async fn disconnect(&self, peer_id: &PeerId) {
        let mut connections = self.connections.write().await;
        connections.remove(peer_id);
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.connections = connections.len();
    }

    /// Returns current connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Returns connection status for a peer
    pub async fn get_status(&self, peer_id: &PeerId) -> Option<ConnectionStatus> {
        self.connections.read().await.get(peer_id).copied()
    }

    /// Updates network metrics
    pub async fn update_metrics(&self, messages_per_second: f64, avg_latency_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.messages_per_second = messages_per_second;
        metrics.avg_latency = std::time::Duration::from_millis(avg_latency_ms);
    }

    /// Returns current network metrics
    pub async fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_management() {
        let manager = ConnectionManager::new(2);
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        // Test connection limit
        assert!(manager.connect(peer1).await.is_ok());
        assert!(manager.connect(peer2).await.is_ok());
        assert!(manager.connect(peer3).await.is_ok()); // Should be ignored due to limit

        assert_eq!(manager.connection_count().await, 2);

        // Test status updates
        manager.update_status(peer1, ConnectionStatus::Connected).await;
        assert_eq!(manager.get_status(&peer1).await, Some(ConnectionStatus::Connected));

        // Test disconnection
        manager.disconnect(&peer1).await;
        assert_eq!(manager.get_status(&peer1).await, None);
        assert_eq!(manager.connection_count().await, 1);

        // Test metrics
        manager.update_metrics(1000.0, 50).await;
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.messages_per_second, 1000.0);
        assert_eq!(metrics.connections, 1);
    }
}