use crate::peer_manager::{PeerManager, PeerManagerConfig};
use crate::rpc::{NodeStatus, RpcClient};
use crate::CliError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{info, warn};

/// Status command arguments
#[derive(Debug, Clone)]
pub struct StatusArgs {
    pub port: u16,
    pub format: OutputFormat,
    pub timeout_seconds: u64,
    pub verbose: bool,
}

impl Default for StatusArgs {
    fn default() -> Self {
        Self {
            port: 8000,
            format: OutputFormat::Text,
            timeout_seconds: 30,
            verbose: false,
        }
    }
}

/// Output format options
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Table,
}

/// Node status response structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeStatusResponse {
    pub node_id: String,
    pub state: NodeState,
    pub uptime_seconds: u64,
    pub connected_peers: Vec<PeerStatusInfo>,
    pub network_stats: NetworkStatistics,
    pub dag_stats: DagStatistics,
    pub memory_usage: MemoryUsage,
}

/// Node state enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeState {
    Running,
    Stopped,
    Syncing,
    Error(String),
}

/// Peer connection information for status display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerStatusInfo {
    pub peer_id: String,
    pub address: String,
    pub connected_duration_seconds: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub last_seen_timestamp: u64,
}

/// Network statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkStatistics {
    pub total_connections: usize,
    pub active_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub average_latency_ms: f64,
}

/// DAG statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagStatistics {
    pub vertex_count: usize,
    pub edge_count: usize,
    pub tip_count: usize,
    pub finalized_height: u64,
    pub pending_transactions: usize,
}

/// Memory usage information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_allocated_bytes: usize,
    pub current_usage_bytes: usize,
    pub peak_usage_bytes: usize,
}

/// Execute status command with the given arguments
pub async fn execute_status_command(args: StatusArgs) -> Result<String> {
    // Validate arguments
    validate_status_args(&args)?;

    // Create RPC client
    let client = RpcClient::new_tcp("127.0.0.1".to_string(), args.port)
        .with_timeout(Duration::from_secs(args.timeout_seconds));

    // Check node connectivity first
    let is_connected = check_node_connectivity(args.port).await?;
    if !is_connected {
        return Err(anyhow::anyhow!(
            "Connection refused: No node running on port {}",
            args.port
        ));
    }

    // Get node status
    let rpc_status = client
        .get_status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get node status: {}", e))?;

    // Convert RPC status to our status response format
    let status_response = convert_rpc_status_to_response(rpc_status);

    // Format output based on requested format
    let output = format_status_output(&status_response, &args.format, args.verbose)?;

    Ok(output)
}

/// Validate status command arguments
fn validate_status_args(args: &StatusArgs) -> Result<()> {
    if args.port == 0 {
        return Err(anyhow::anyhow!("Port cannot be 0"));
    }

    // Note: u16 cannot be greater than 65535, so this check is redundant
    // Keeping for clarity but it will be optimized away by the compiler

    if args.timeout_seconds == 0 {
        return Err(anyhow::anyhow!("Timeout cannot be 0"));
    }

    if args.timeout_seconds > 300 {
        return Err(anyhow::anyhow!(
            "Timeout cannot be greater than 300 seconds"
        ));
    }

    Ok(())
}

/// Check if a node is running on the specified port
pub async fn check_node_connectivity(port: u16) -> Result<bool> {
    match timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)),
    )
    .await
    {
        Ok(Ok(_)) => Ok(true),
        Ok(Err(_)) => Ok(false),
        Err(_) => Ok(false), // timeout
    }
}

/// Convert RPC NodeStatus to our NodeStatusResponse format
fn convert_rpc_status_to_response(rpc_status: NodeStatus) -> NodeStatusResponse {
    let state = match rpc_status.state.as_str() {
        "Running" => NodeState::Running,
        "Stopped" => NodeState::Stopped,
        "Syncing" => NodeState::Syncing,
        error_state if error_state.starts_with("Error") => {
            let error_msg = error_state
                .strip_prefix("Error(")
                .unwrap_or("Unknown error")
                .strip_suffix(")")
                .unwrap_or("Unknown error");
            NodeState::Error(error_msg.to_string())
        }
        _ => NodeState::Error(format!("Unknown state: {}", rpc_status.state)),
    };

    let connected_peers = rpc_status
        .peers
        .into_iter()
        .map(|peer| PeerStatusInfo {
            peer_id: peer.id,
            address: peer.address,
            connected_duration_seconds: peer.connected_duration,
            messages_sent: peer.messages_sent,
            messages_received: peer.messages_received,
            last_seen_timestamp: peer.last_seen,
        })
        .collect();

    let network_stats = NetworkStatistics {
        total_connections: rpc_status.network_stats.total_connections,
        active_connections: rpc_status.network_stats.active_connections,
        messages_sent: rpc_status.network_stats.messages_sent,
        messages_received: rpc_status.network_stats.messages_received,
        bytes_sent: rpc_status.network_stats.bytes_sent,
        bytes_received: rpc_status.network_stats.bytes_received,
        average_latency_ms: rpc_status.network_stats.average_latency,
    };

    let dag_stats = DagStatistics {
        vertex_count: rpc_status.dag_stats.vertex_count,
        edge_count: rpc_status.dag_stats.edge_count,
        tip_count: rpc_status.dag_stats.tip_count,
        finalized_height: rpc_status.dag_stats.finalized_height,
        pending_transactions: rpc_status.dag_stats.pending_transactions,
    };

    let memory_usage = MemoryUsage {
        total_allocated_bytes: rpc_status.memory_usage.total_allocated,
        current_usage_bytes: rpc_status.memory_usage.current_usage,
        peak_usage_bytes: rpc_status.memory_usage.peak_usage,
    };

    NodeStatusResponse {
        node_id: rpc_status.node_id,
        state,
        uptime_seconds: rpc_status.uptime,
        connected_peers,
        network_stats,
        dag_stats,
        memory_usage,
    }
}

/// Format status output based on the requested format
fn format_status_output(
    status: &NodeStatusResponse,
    format: &OutputFormat,
    verbose: bool,
) -> Result<String> {
    match format {
        OutputFormat::Json => {
            if verbose {
                Ok(serde_json::to_string_pretty(status)?)
            } else {
                Ok(serde_json::to_string(status)?)
            }
        }
        OutputFormat::Text => format_status_as_text(status, verbose),
        OutputFormat::Table => format_status_as_table(status, verbose),
    }
}

/// Format status as human-readable text
fn format_status_as_text(status: &NodeStatusResponse, verbose: bool) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format!("Node Status: {}", status.node_id));
    output.push('\n');
    output.push_str(&format!("State: {:?}", status.state));
    output.push('\n');
    output.push_str(&format!("Uptime: {} seconds", status.uptime_seconds));
    output.push('\n');
    output.push_str(&format!(
        "Connected Peers: {}",
        status.connected_peers.len()
    ));
    output.push('\n');

    if verbose {
        output.push_str("\nNetwork Statistics:\n");
        output.push_str(&format!(
            "  Total Connections: {}",
            status.network_stats.total_connections
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Active Connections: {}",
            status.network_stats.active_connections
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Messages Sent: {}",
            status.network_stats.messages_sent
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Messages Received: {}",
            status.network_stats.messages_received
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Bytes Sent: {}",
            status.network_stats.bytes_sent
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Bytes Received: {}",
            status.network_stats.bytes_received
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Average Latency: {:.2} ms",
            status.network_stats.average_latency_ms
        ));
        output.push('\n');

        output.push_str("\nDAG Statistics:\n");
        output.push_str(&format!(
            "  Vertex Count: {}",
            status.dag_stats.vertex_count
        ));
        output.push('\n');
        output.push_str(&format!("  Edge Count: {}", status.dag_stats.edge_count));
        output.push('\n');
        output.push_str(&format!("  Tip Count: {}", status.dag_stats.tip_count));
        output.push('\n');
        output.push_str(&format!(
            "  Finalized Height: {}",
            status.dag_stats.finalized_height
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Pending Transactions: {}",
            status.dag_stats.pending_transactions
        ));
        output.push('\n');

        output.push_str("\nMemory Usage:\n");
        output.push_str(&format!(
            "  Total Allocated: {} bytes",
            status.memory_usage.total_allocated_bytes
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Current Usage: {} bytes",
            status.memory_usage.current_usage_bytes
        ));
        output.push('\n');
        output.push_str(&format!(
            "  Peak Usage: {} bytes",
            status.memory_usage.peak_usage_bytes
        ));
        output.push('\n');

        if !status.connected_peers.is_empty() {
            output.push_str("\nConnected Peers:\n");
            for peer in &status.connected_peers {
                output.push_str(&format!(
                    "  {}: {} ({}s connected)",
                    peer.peer_id, peer.address, peer.connected_duration_seconds
                ));
                output.push('\n');
            }
        }
    }

    Ok(output)
}

/// Format status as a table
fn format_status_as_table(status: &NodeStatusResponse, verbose: bool) -> Result<String> {
    let mut output = String::new();

    output.push_str(
        "┌──────────────────────────────────────────────────────────────────────────────┐\n",
    );
    output.push_str(&format!("│ Node Status: {:<62} │\n", status.node_id));
    output.push_str(
        "├──────────────────────────────────────────────────────────────────────────────┤\n",
    );
    output.push_str(&format!(
        "│ State: {:<68} │\n",
        format!("{:?}", status.state)
    ));
    output.push_str(&format!(
        "│ Uptime: {:<67} │\n",
        format!("{} seconds", status.uptime_seconds)
    ));
    output.push_str(&format!(
        "│ Connected Peers: {:<60} │\n",
        status.connected_peers.len()
    ));

    if verbose {
        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────────┤\n",
        );
        output.push_str(
            "│ Network Statistics                                                      │\n",
        );
        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────────┤\n",
        );
        output.push_str(&format!(
            "│ Total Connections: {:<57} │\n",
            status.network_stats.total_connections
        ));
        output.push_str(&format!(
            "│ Active Connections: {:<56} │\n",
            status.network_stats.active_connections
        ));
        output.push_str(&format!(
            "│ Messages Sent: {:<61} │\n",
            status.network_stats.messages_sent
        ));
        output.push_str(&format!(
            "│ Messages Received: {:<57} │\n",
            status.network_stats.messages_received
        ));
        output.push_str(&format!(
            "│ Bytes Sent: {:<64} │\n",
            status.network_stats.bytes_sent
        ));
        output.push_str(&format!(
            "│ Bytes Received: {:<60} │\n",
            status.network_stats.bytes_received
        ));
        output.push_str(&format!(
            "│ Average Latency: {:<59} │\n",
            format!("{:.2} ms", status.network_stats.average_latency_ms)
        ));

        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────────┤\n",
        );
        output.push_str(
            "│ DAG Statistics                                                          │\n",
        );
        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────────┤\n",
        );
        output.push_str(&format!(
            "│ Vertex Count: {:<62} │\n",
            status.dag_stats.vertex_count
        ));
        output.push_str(&format!(
            "│ Edge Count: {:<64} │\n",
            status.dag_stats.edge_count
        ));
        output.push_str(&format!(
            "│ Tip Count: {:<65} │\n",
            status.dag_stats.tip_count
        ));
        output.push_str(&format!(
            "│ Finalized Height: {:<58} │\n",
            status.dag_stats.finalized_height
        ));
        output.push_str(&format!(
            "│ Pending Transactions: {:<54} │\n",
            status.dag_stats.pending_transactions
        ));

        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────────┤\n",
        );
        output.push_str(
            "│ Memory Usage                                                            │\n",
        );
        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────────┤\n",
        );
        output.push_str(&format!(
            "│ Total Allocated: {:<59} │\n",
            format!("{} bytes", status.memory_usage.total_allocated_bytes)
        ));
        output.push_str(&format!(
            "│ Current Usage: {:<61} │\n",
            format!("{} bytes", status.memory_usage.current_usage_bytes)
        ));
        output.push_str(&format!(
            "│ Peak Usage: {:<64} │\n",
            format!("{} bytes", status.memory_usage.peak_usage_bytes)
        ));
    }

    output.push_str(
        "└──────────────────────────────────────────────────────────────────────────────┘\n",
    );

    Ok(output)
}

/// Command routing and dispatch logic
pub struct CommandRouter {
    /// Peer manager instance
    peer_manager: Option<Arc<Mutex<PeerManager>>>,
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRouter {
    /// Create new CommandRouter
    pub fn new() -> Self {
        Self {
            peer_manager: None,
        }
    }
    
    /// Create CommandRouter with initialized PeerManager
    pub async fn with_peer_manager() -> Result<Self, CliError> {
        let config = PeerManagerConfig::default();
        let peer_manager = PeerManager::new(config).await
            .map_err(|e| CliError::Config(format!("Failed to initialize peer manager: {}", e)))?;
        
        Ok(Self {
            peer_manager: Some(Arc::new(Mutex::new(peer_manager))),
        })
    }
    
    /// Get or create peer manager instance
    async fn get_peer_manager(&self) -> Result<Arc<Mutex<PeerManager>>, CliError> {
        if let Some(ref pm) = self.peer_manager {
            Ok(Arc::clone(pm))
        } else {
            Err(CliError::Config("Peer manager not initialized".to_string()))
        }
    }
    
    /// Route and execute node status command
    pub async fn handle_node_status(args: StatusArgs) -> Result<String, CliError> {
        info!("Executing node status command with port {}", args.port);

        match execute_status_command(args).await {
            Ok(output) => Ok(output),
            Err(e) => Err(CliError::Command(e.to_string())),
        }
    }

    /// Route and execute peer list command
    pub async fn handle_peer_list(&self, port: Option<u16>) -> Result<(), CliError> {
        info!("Executing peer list command");
        
        // Try to use peer manager first for comprehensive peer information
        if let Ok(peer_manager) = self.get_peer_manager().await {
            let manager = peer_manager.lock().await;
            match manager.list_peers().await {
                Ok(peers) => {
                    if peers.is_empty() {
                        println!("No peers in database");
                    } else {
                        println!("Known Peers ({}):", peers.len());
                        println!("{:<16} {:<30} {:<12} {:<10} {:<12} {:<20}", 
                               "Peer ID", "Address", "Trust", "Status", "Latency", "Nickname");
                        println!("{}", "-".repeat(110));
                        
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        for peer in peers {
                            let id_short = if peer.id.len() > 16 {
                                format!("{}...", &peer.id[..13])
                            } else {
                                peer.id.clone()
                            };
                            
                            let status = if now - peer.last_seen < 300 {
                                "Active"
                            } else {
                                "Inactive"
                            };
                            
                            let latency = peer.avg_latency_ms
                                .map(|l| format!("{:.1}ms", l))
                                .unwrap_or_else(|| "N/A".to_string());
                            
                            let nickname = peer.nickname.unwrap_or_else(|| "-".to_string());
                            
                            println!("{:<16} {:<30} {:<12} {:<10} {:<12} {:<20}", 
                                   id_short, peer.address, peer.trust_level, status, latency, nickname);
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to list peers from manager: {}", e);
                    // Fall back to RPC method
                }
            }
        }
        
        // Fallback to RPC client method
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.list_peers().await {
            Ok(peers) => {
                if peers.is_empty() {
                    println!("No peers connected");
                } else {
                    println!("Connected Peers ({}):", peers.len());
                    println!("{:<20} {:<30} {:<15} {:<12} {:<12}", 
                           "Peer ID", "Address", "Status", "Messages In", "Messages Out");
                    println!("{}", "-".repeat(95));
                    
                    for peer in peers {
                        println!("{:<20} {:<30} {:<15} {:<12} {:<12}", 
                               peer.id, peer.address, peer.status,
                               peer.messages_received, peer.messages_sent);
                    }
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to fetch peer list: {}", e);
                Err(CliError::Command(format!("Failed to fetch peer list: {}", e)))
            }
        }
    }

    /// Route and execute peer add command
    pub async fn handle_peer_add(&self, address: String, port: Option<u16>, nickname: Option<String>) -> Result<(), CliError> {
        info!("Executing peer add command for address: {}", address);
        
        // Validate address format
        if !is_valid_peer_address(&address) {
            return Err(CliError::Command(format!("Invalid peer address format: {}", address)));
        }
        
        // Try to use peer manager first
        if let Ok(peer_manager) = self.get_peer_manager().await {
            println!("Connecting to peer: {}", address);
            
            let manager = peer_manager.lock().await;
            match manager.add_peer(address.clone(), nickname.clone()).await {
                Ok(peer_id) => {
                    println!("✓ Successfully connected to peer");
                    println!("  Peer ID: {}", peer_id);
                    println!("  Address: {}", address);
                    if let Some(nick) = nickname {
                        println!("  Nickname: {}", nick);
                    }
                    
                    // Save peers after successful connection
                    if let Err(e) = manager.save_peers().await {
                        warn!("Failed to save peer data: {}", e);
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to add peer via manager: {}", e);
                    // Fall back to RPC method
                }
            }
        }
        
        // Fallback to RPC client method
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.add_peer(address.clone()).await {
            Ok(message) => {
                println!("✓ {}", message);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to add peer {}: {}", address, e);
                Err(CliError::Command(format!("Failed to add peer: {}", e)))
            }
        }
    }

    /// Route and execute peer remove command
    pub async fn handle_peer_remove(&self, peer_id: String, port: Option<u16>, force: bool) -> Result<(), CliError> {
        info!("Executing peer remove command for peer: {}", peer_id);
        
        // Show confirmation prompt unless forced
        if !force {
            print!("Are you sure you want to remove peer {}? [y/N] ", peer_id);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            
            let mut response = String::new();
            io::stdin().read_line(&mut response).unwrap();
            
            if !response.trim().eq_ignore_ascii_case("y") {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        
        // Try to use peer manager first
        if let Ok(peer_manager) = self.get_peer_manager().await {
            let manager = peer_manager.lock().await;
            match manager.remove_peer(peer_id.clone()).await {
                Ok(()) => {
                    println!("✓ Successfully removed peer: {}", peer_id);
                    
                    // Save peers after removal
                    if let Err(e) = manager.save_peers().await {
                        warn!("Failed to save peer data: {}", e);
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to remove peer via manager: {}", e);
                    // Fall back to RPC method
                }
            }
        }
        
        // Fallback to RPC client method
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.remove_peer(peer_id.clone()).await {
            Ok(message) => {
                println!("✓ {}", message);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to remove peer {}: {}", peer_id, e);
                Err(CliError::Command(format!("Failed to remove peer: {}", e)))
            }
        }
    }

    /// Route and execute network stats command
    pub async fn handle_network_stats(&self, port: Option<u16>, verbose: bool) -> Result<(), CliError> {
        info!("Executing network stats command");
        
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.get_network_stats().await {
            Ok(stats) => {
                println!("Network Statistics:");
                println!("==================");
                println!("Total Connections: {}", stats.total_connections);
                println!("Active Connections: {}", stats.active_connections);
                println!("Messages Sent: {}", stats.messages_sent);
                println!("Messages Received: {}", stats.messages_received);
                
                if verbose {
                    println!("Bytes Sent: {}", format_bytes(stats.bytes_sent));
                    println!("Bytes Received: {}", format_bytes(stats.bytes_received));
                    println!("Average Latency: {:.2} ms", stats.average_latency);
                    println!("Uptime: {}", format_duration(stats.uptime));
                }
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to fetch network stats: {}", e);
                Err(CliError::Command(format!("Failed to fetch network stats: {}", e)))
            }
        }
    }

    /// Route and execute network test command
    pub async fn handle_network_test(&self, port: Option<u16>) -> Result<(), CliError> {
        info!("Executing network test command");
        
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(60)); // Longer timeout for network tests
        
        println!("Testing network connectivity...");
        
        match client.test_network().await {
            Ok(results) => {
                println!("\nNetwork Test Results:");
                println!("====================\n");
                
                if results.is_empty() {
                    println!("No peers to test");
                    return Ok(());
                }
                
                for result in results {
                    let status = if result.reachable { "✓ REACHABLE" } else { "✗ UNREACHABLE" };
                    println!("Peer: {} ({})", result.peer_id, result.address);
                    println!("Status: {}", status);
                    
                    if let Some(latency) = result.latency {
                        println!("Latency: {:.2} ms", latency);
                    }
                    
                    if let Some(error) = result.error {
                        println!("Error: {}", error);
                    }
                    
                    println!();
                }
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to run network test: {}", e);
                Err(CliError::Command(format!("Failed to run network test: {}", e)))
            }
        }
    }
    
    /// Route and execute peer info command
    pub async fn handle_peer_info(&self, peer_id: String, port: Option<u16>) -> Result<(), CliError> {
        info!("Executing peer info command for peer: {}", peer_id);
        
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.get_peer_info(peer_id.clone()).await {
            Ok(peer) => {
                println!("Peer Information:");
                println!("================\n");
                println!("Peer ID: {}", peer.id);
                println!("Address: {}", peer.address);
                println!("Status: {}", peer.status);
                println!("Connected Duration: {} seconds", peer.connected_duration);
                println!("Messages Sent: {}", peer.messages_sent);
                println!("Messages Received: {}", peer.messages_received);
                println!("Last Seen: {} (timestamp)", peer.last_seen);
                
                if let Some(latency) = peer.latency {
                    println!("Latency: {:.2} ms", latency);
                }
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to get peer info for {}: {}", peer_id, e);
                Err(CliError::Command(format!("Failed to get peer info: {}", e)))
            }
        }
    }
    
    /// Route and execute peer ban command
    pub async fn handle_peer_ban(&self, peer_id: String, port: Option<u16>) -> Result<(), CliError> {
        info!("Executing peer ban command for peer: {}", peer_id);
        
        // Try to use peer manager first
        if let Ok(peer_manager) = self.get_peer_manager().await {
            let manager = peer_manager.lock().await;
            match manager.ban_peer(peer_id.clone()).await {
                Ok(()) => {
                    println!("✓ Successfully banned peer: {}", peer_id);
                    println!("  The peer has been blacklisted and disconnected");
                    
                    // Save peers after banning
                    if let Err(e) = manager.save_peers().await {
                        warn!("Failed to save peer data: {}", e);
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to ban peer via manager: {}", e);
                    // Fall back to RPC method
                }
            }
        }
        
        // Fallback to RPC client method
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.ban_peer(peer_id.clone()).await {
            Ok(message) => {
                println!("✓ {}", message);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to ban peer {}: {}", peer_id, e);
                Err(CliError::Command(format!("Failed to ban peer: {}", e)))
            }
        }
    }
    
    /// Route and execute peer unban command
    pub async fn handle_peer_unban(&self, address: String, port: Option<u16>) -> Result<(), CliError> {
        info!("Executing peer unban command for address: {}", address);
        
        // Try to use peer manager first
        if let Ok(peer_manager) = self.get_peer_manager().await {
            let manager = peer_manager.lock().await;
            match manager.unban_peer(address.clone()).await {
                Ok(()) => {
                    println!("✓ Successfully unbanned peer with address: {}", address);
                    println!("  The peer can now connect again");
                    
                    // Save peers after unbanning
                    if let Err(e) = manager.save_peers().await {
                        warn!("Failed to save peer data: {}", e);
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to unban peer via manager: {}", e);
                    // Fall back to RPC method
                }
            }
        }
        
        // Fallback to RPC client method
        let port = port.unwrap_or(8000);
        let client = RpcClient::new_tcp("127.0.0.1".to_string(), port)
            .with_timeout(Duration::from_secs(30));
        
        match client.unban_peer(address.clone()).await {
            Ok(message) => {
                println!("✓ {}", message);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to unban peer {}: {}", address, e);
                Err(CliError::Command(format!("Failed to unban peer: {}", e)))
            }
        }
    }
    
    /// Route and execute peer import command
    pub async fn handle_peer_import(&self, path: PathBuf, merge: bool) -> Result<(), CliError> {
        info!("Executing peer import command from: {:?}", path);
        
        if !path.exists() {
            return Err(CliError::Command(format!("File not found: {:?}", path)));
        }
        
        let peer_manager = self.get_peer_manager().await?;
        let manager = peer_manager.lock().await;
        
        match manager.import_peers(path.clone(), merge).await {
            Ok(count) => {
                println!("✓ Successfully imported {} peers from {:?}", count, path);
                if merge {
                    println!("  Peers were merged with existing database");
                } else {
                    println!("  Existing peer database was replaced");
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to import peers: {}", e);
                Err(CliError::Command(format!("Failed to import peers: {}", e)))
            }
        }
    }
    
    /// Route and execute peer export command
    pub async fn handle_peer_export(&self, path: PathBuf, tags: Option<Vec<String>>) -> Result<(), CliError> {
        info!("Executing peer export command to: {:?}", path);
        
        let peer_manager = self.get_peer_manager().await?;
        let manager = peer_manager.lock().await;
        
        match manager.export_peers(path.clone(), tags.clone()).await {
            Ok(count) => {
                println!("✓ Successfully exported {} peers to {:?}", count, path);
                if let Some(t) = tags {
                    println!("  Filtered by tags: {}", t.join(", "));
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to export peers: {}", e);
                Err(CliError::Command(format!("Failed to export peers: {}", e)))
            }
        }
    }
    
    /// Route and execute peer test command
    pub async fn handle_peer_test(&self) -> Result<(), CliError> {
        info!("Executing peer test command");
        
        let peer_manager = self.get_peer_manager().await?;
        let manager = peer_manager.lock().await;
        
        println!("Testing connectivity to all known peers...");
        println!();
        
        let progress_callback = |current: usize, total: usize| {
            print!("\rTesting peer {}/{}...", current, total);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        };
        
        match manager.test_all_peers(progress_callback).await {
            Ok(results) => {
                println!("\r\nTest Results:");
                println!("=============\n");
                
                let mut success_count = 0;
                let mut total_latency = 0.0;
                let mut latency_count = 0;
                
                for (peer_id, success, latency) in &results {
                    let status = if *success { "✓ SUCCESS" } else { "✗ FAILED" };
                    print!("{:<16} {}", 
                        if peer_id.len() > 16 { 
                            format!("{}...", &peer_id[..13]) 
                        } else { 
                            peer_id.clone() 
                        },
                        status
                    );
                    
                    if let Some(lat) = latency {
                        print!(" ({:.1}ms)", lat);
                        total_latency += lat;
                        latency_count += 1;
                    }
                    println!();
                    
                    if *success {
                        success_count += 1;
                    }
                }
                
                println!("\nSummary:");
                println!("--------");
                println!("Total peers tested: {}", results.len());
                println!("Successful connections: {} ({:.1}%)", 
                    success_count, 
                    (success_count as f64 / results.len() as f64) * 100.0
                );
                
                if latency_count > 0 {
                    println!("Average latency: {:.1}ms", total_latency / latency_count as f64);
                }
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to test peers: {}", e);
                Err(CliError::Command(format!("Failed to test peers: {}", e)))
            }
        }
    }
}

// Keep existing command implementations below for backward compatibility

pub async fn start_node(
    data_dir: Option<PathBuf>,
    port: Option<u16>,
    peers: Vec<String>,
) -> Result<(), CliError> {
    use crate::node_manager::{NodeManager, NodeManagerConfig};
    
    info!("Starting QuDAG node...");

    // Create node manager with default config
    let config = NodeManagerConfig::default();
    let manager = NodeManager::new(config)
        .map_err(|e| CliError::Node(format!("Failed to create node manager: {}", e)))?;

    // Start the node
    manager
        .start_node(port, data_dir, peers, true) // Run in foreground
        .await
        .map_err(|e| CliError::Node(format!("Failed to start node: {}", e)))?;

    Ok(())
}

pub async fn stop_node() -> Result<(), CliError> {
    use crate::node_manager::{NodeManager, NodeManagerConfig};
    
    info!("Stopping QuDAG node...");

    // Create node manager
    let config = NodeManagerConfig::default();
    let manager = NodeManager::new(config)
        .map_err(|e| CliError::Node(format!("Failed to create node manager: {}", e)))?;

    // Stop the node
    manager
        .stop_node(false) // Graceful shutdown
        .await
        .map_err(|e| CliError::Node(format!("Failed to stop node: {}", e)))?;

    Ok(())
}

pub async fn show_status() -> Result<(), CliError> {
    use crate::node_manager::{NodeManager, NodeManagerConfig};
    
    info!("Fetching node status...");

    // First check if node is running locally
    let config = NodeManagerConfig::default();
    let manager = NodeManager::new(config)
        .map_err(|e| CliError::Node(format!("Failed to create node manager: {}", e)))?;
    
    let local_status = manager.get_status().await
        .map_err(|e| CliError::Node(format!("Failed to get local status: {}", e)))?;
    
    if local_status.is_running {
        // Node is running, try to get detailed status via RPC
        let args = StatusArgs::default();
        match CommandRouter::handle_node_status(args).await {
            Ok(output) => {
                println!("{}", output);
                Ok(())
            }
            Err(e) => {
                // RPC failed but node is running, show basic status
                warn!("Failed to get detailed status via RPC: {}", e);
                println!("Node Status:");
                println!("============");
                println!("Status: Running (PID: {})", local_status.pid.unwrap_or(0));
                println!("Port: {}", local_status.port);
                println!("Data Directory: {:?}", local_status.data_dir);
                println!("Log File: {:?}", local_status.log_file);
                if let Some(uptime) = local_status.uptime_seconds {
                    println!("Uptime: {} seconds", uptime);
                }
                println!("\nNote: RPC connection failed, showing local status only");
                Ok(())
            }
        }
    } else {
        println!("Node Status:");
        println!("============");
        println!("Status: Not running");
        println!("Port: {} (configured)", local_status.port);
        println!("Data Directory: {:?}", local_status.data_dir);
        println!("Log File: {:?}", local_status.log_file);
        println!("\nUse 'qudag start' to start the node");
        Ok(())
    }
}

pub async fn list_peers() -> Result<(), CliError> {
    let router = CommandRouter::with_peer_manager().await?;
    router.handle_peer_list(None).await
}

pub async fn add_peer(address: String) -> Result<(), CliError> {
    let router = CommandRouter::with_peer_manager().await?;
    router.handle_peer_add(address, None, None).await
}

pub async fn remove_peer(peer_id: String) -> Result<(), CliError> {
    let router = CommandRouter::with_peer_manager().await?;
    router.handle_peer_remove(peer_id, None, false).await
}

pub async fn visualize_dag(
    output: Option<PathBuf>,
    format: Option<String>,
) -> Result<(), CliError> {
    info!("Generating DAG visualization...");

    let output = output.unwrap_or_else(|| PathBuf::from("dag_visualization.dot"));
    let format = format.unwrap_or_else(|| "dot".to_string());

    // TODO: Generate actual DAG visualization
    use std::fs::File;
    use std::io::Write;

    let dot_content = r#"digraph DAG {
    node [shape=box];
    "genesis" -> "block1";
    "genesis" -> "block2";
    "block1" -> "block3";
    "block2" -> "block3";
}
"#;

    let mut file = File::create(&output)
        .map_err(|e| CliError::Visualization(format!("Failed to create output file: {}", e)))?;

    file.write_all(dot_content.as_bytes())
        .map_err(|e| CliError::Visualization(format!("Failed to write visualization: {}", e)))?;

    info!(
        "DAG visualization saved to {:?} in {} format",
        output, format
    );
    Ok(())
}

pub async fn show_network_stats() -> Result<(), CliError> {
    let router = CommandRouter::new();
    router.handle_network_stats(None, false).await
}

pub async fn test_network() -> Result<(), CliError> {
    let router = CommandRouter::new();
    router.handle_network_test(None).await
}

/// Validate peer address format
fn is_valid_peer_address(address: &str) -> bool {
    // Check basic format: IP:PORT or hostname:PORT
    if let Some((host, port_str)) = address.rsplit_once(':') {
        if host.is_empty() || port_str.is_empty() {
            return false;
        }
        
        // Validate port
        if let Ok(port) = port_str.parse::<u16>() {
            if port == 0 {
                return false;
            }
        } else {
            return false;
        }
        
        // Basic validation for host (IP or hostname)
        if host.parse::<std::net::IpAddr>().is_ok() {
            return true; // Valid IP address
        }
        
        // Basic hostname validation
        if host.len() <= 253 && !host.is_empty() {
            return host.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-');
        }
    }
    
    false
}

/// Format bytes in human readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human readable format
fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
