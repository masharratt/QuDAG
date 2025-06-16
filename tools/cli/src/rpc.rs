use anyhow::{anyhow, Result};
use qudag_network::{NetworkAddress, PeerId};
use qudag_protocol::{Node, NodeConfig, ProtocolState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: Uuid,
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: Uuid,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
}

/// RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub state: String,
    pub uptime: u64,
    pub peers: Vec<PeerInfo>,
    pub network_stats: NetworkStats,
    pub dag_stats: DagStats,
    pub memory_usage: MemoryStats,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub connected_duration: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub last_seen: u64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub average_latency: f64,
}

/// DAG statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStats {
    pub vertex_count: usize,
    pub edge_count: usize,
    pub tip_count: usize,
    pub finalized_height: u64,
    pub pending_transactions: usize,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
}

/// Wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub public_key: String,
    pub balance: u64,
    pub address: String,
    pub key_type: String,
}

/// Network test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTestResult {
    pub peer_id: String,
    pub address: String,
    pub reachable: bool,
    pub latency: Option<f64>,
    pub error: Option<String>,
}

/// RPC client for communicating with QuDAG nodes
pub struct RpcClient {
    address: String,
    port: u16,
    timeout: Duration,
}

impl RpcClient {
    /// Create new RPC client
    pub fn new(address: String, port: u16) -> Self {
        Self {
            address,
            port,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send RPC request
    async fn send_request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: method.to_string(),
            params,
        };

        let request_data = serde_json::to_vec(&request)?;
        
        // Connect to node
        let mut stream = timeout(
            self.timeout,
            TcpStream::connect(format!("{}:{}", self.address, self.port))
        ).await
        .map_err(|_| anyhow!("Connection timeout"))?
        .map_err(|e| anyhow!("Failed to connect: {}", e))?;

        // Send request
        stream.write_u32(request_data.len() as u32).await?;
        stream.write_all(&request_data).await?;

        // Read response
        let response_len = stream.read_u32().await?;
        let mut response_data = vec![0u8; response_len as usize];
        stream.read_exact(&mut response_data).await?;

        let response: RpcResponse = serde_json::from_slice(&response_data)?;

        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }

        response.result.ok_or_else(|| anyhow!("Empty response"))
    }

    /// Get node status
    pub async fn get_status(&self) -> Result<NodeStatus> {
        let result = self.send_request("get_status", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Start node
    pub async fn start_node(&self, config: NodeConfig) -> Result<()> {
        let params = serde_json::to_value(config)?;
        self.send_request("start", params).await?;
        Ok(())
    }

    /// Stop node
    pub async fn stop_node(&self) -> Result<()> {
        self.send_request("stop", serde_json::Value::Null).await?;
        Ok(())
    }

    /// Restart node
    pub async fn restart_node(&self) -> Result<()> {
        self.send_request("restart", serde_json::Value::Null).await?;
        Ok(())
    }

    /// Add peer
    pub async fn add_peer(&self, address: String) -> Result<()> {
        let params = serde_json::json!({ "address": address });
        self.send_request("add_peer", params).await?;
        Ok(())
    }

    /// Remove peer
    pub async fn remove_peer(&self, peer_id: String) -> Result<()> {
        let params = serde_json::json!({ "peer_id": peer_id });
        self.send_request("remove_peer", params).await?;
        Ok(())
    }

    /// List peers
    pub async fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        let result = self.send_request("list_peers", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        let result = self.send_request("get_network_stats", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Test network connectivity
    pub async fn test_network(&self) -> Result<Vec<NetworkTestResult>> {
        let result = self.send_request("test_network", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get wallet information
    pub async fn get_wallet_info(&self) -> Result<WalletInfo> {
        let result = self.send_request("get_wallet_info", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Create new wallet
    pub async fn create_wallet(&self, password: String) -> Result<String> {
        let params = serde_json::json!({ "password": password });
        let result = self.send_request("create_wallet", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Import wallet from seed
    pub async fn import_wallet(&self, seed: String, password: String) -> Result<()> {
        let params = serde_json::json!({ "seed": seed, "password": password });
        self.send_request("import_wallet", params).await?;
        Ok(())
    }

    /// Export wallet seed
    pub async fn export_wallet(&self, password: String) -> Result<String> {
        let params = serde_json::json!({ "password": password });
        let result = self.send_request("export_wallet", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get DAG visualization data
    pub async fn get_dag_data(&self) -> Result<serde_json::Value> {
        self.send_request("get_dag_data", serde_json::Value::Null).await
    }

    /// Debug network
    pub async fn debug_network(&self) -> Result<serde_json::Value> {
        self.send_request("debug_network", serde_json::Value::Null).await
    }

    /// Debug consensus
    pub async fn debug_consensus(&self) -> Result<serde_json::Value> {
        self.send_request("debug_consensus", serde_json::Value::Null).await
    }

    /// Debug performance
    pub async fn debug_performance(&self) -> Result<serde_json::Value> {
        self.send_request("debug_performance", serde_json::Value::Null).await
    }

    /// Security audit
    pub async fn security_audit(&self) -> Result<serde_json::Value> {
        self.send_request("security_audit", serde_json::Value::Null).await
    }

    /// Get configuration
    pub async fn get_config(&self) -> Result<serde_json::Value> {
        self.send_request("get_config", serde_json::Value::Null).await
    }

    /// Update configuration
    pub async fn update_config(&self, config: serde_json::Value) -> Result<()> {
        self.send_request("update_config", config).await?;
        Ok(())
    }

    /// Validate configuration
    pub async fn validate_config(&self, config: serde_json::Value) -> Result<bool> {
        let params = serde_json::json!({ "config": config });
        let result = self.send_request("validate_config", params).await?;
        Ok(serde_json::from_value(result)?)
    }
}

/// Check if node is running
pub async fn is_node_running(port: u16) -> bool {
    match TcpStream::connect(format!("127.0.0.1:{}", port)).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Wait for node to start
pub async fn wait_for_node_start(port: u16, timeout_secs: u64) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout_duration {
        if is_node_running(port).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Err(anyhow!("Node failed to start within {} seconds", timeout_secs))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rpc_request_serialization() {
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: "test_method".to_string(),
            params: serde_json::json!({"key": "value"}),
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RpcRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(request.method, deserialized.method);
    }
    
    #[test]
    fn test_rpc_response_serialization() {
        let response = RpcResponse {
            id: Uuid::new_v4(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };
        
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: RpcResponse = serde_json::from_str(&serialized).unwrap();
        
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }
}