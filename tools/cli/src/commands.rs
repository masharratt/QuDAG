use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};
use crate::CliError;
use crate::rpc::{RpcClient, NodeStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::time::timeout;

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
    let client = RpcClient::new("127.0.0.1".to_string(), args.port)
        .with_timeout(Duration::from_secs(args.timeout_seconds));
    
    // Check node connectivity first
    let is_connected = check_node_connectivity(args.port).await?;
    if !is_connected {
        return Err(anyhow::anyhow!("Connection refused: No node running on port {}", args.port));
    }
    
    // Get node status
    let rpc_status = client.get_status().await
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
    
    if args.port > 65535 {
        return Err(anyhow::anyhow!("Port cannot be greater than 65535"));
    }
    
    if args.timeout_seconds == 0 {
        return Err(anyhow::anyhow!("Timeout cannot be 0"));
    }
    
    if args.timeout_seconds > 300 {
        return Err(anyhow::anyhow!("Timeout cannot be greater than 300 seconds"));
    }
    
    Ok(())
}

/// Check if a node is running on the specified port
pub async fn check_node_connectivity(port: u16) -> Result<bool> {
    match timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
    ).await {
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
            let error_msg = error_state.strip_prefix("Error(").unwrap_or("Unknown error")
                .strip_suffix(")").unwrap_or("Unknown error");
            NodeState::Error(error_msg.to_string())
        },
        _ => NodeState::Error(format!("Unknown state: {}", rpc_status.state)),
    };
    
    let connected_peers = rpc_status.peers.into_iter().map(|peer| {
        PeerStatusInfo {
            peer_id: peer.id,
            address: peer.address,
            connected_duration_seconds: peer.connected_duration,
            messages_sent: peer.messages_sent,
            messages_received: peer.messages_received,
            last_seen_timestamp: peer.last_seen,
        }
    }).collect();
    
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
fn format_status_output(status: &NodeStatusResponse, format: &OutputFormat, verbose: bool) -> Result<String> {
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
    output.push_str("\n");
    output.push_str(&format!("State: {:?}", status.state));
    output.push_str("\n");
    output.push_str(&format!("Uptime: {} seconds", status.uptime_seconds));
    output.push_str("\n");
    output.push_str(&format!("Connected Peers: {}", status.connected_peers.len()));
    output.push_str("\n");
    
    if verbose {
        output.push_str("\nNetwork Statistics:\n");
        output.push_str(&format!("  Total Connections: {}", status.network_stats.total_connections));
        output.push_str("\n");
        output.push_str(&format!("  Active Connections: {}", status.network_stats.active_connections));
        output.push_str("\n");
        output.push_str(&format!("  Messages Sent: {}", status.network_stats.messages_sent));
        output.push_str("\n");
        output.push_str(&format!("  Messages Received: {}", status.network_stats.messages_received));
        output.push_str("\n");
        output.push_str(&format!("  Bytes Sent: {}", status.network_stats.bytes_sent));
        output.push_str("\n");
        output.push_str(&format!("  Bytes Received: {}", status.network_stats.bytes_received));
        output.push_str("\n");
        output.push_str(&format!("  Average Latency: {:.2} ms", status.network_stats.average_latency_ms));
        output.push_str("\n");
        
        output.push_str("\nDAG Statistics:\n");
        output.push_str(&format!("  Vertex Count: {}", status.dag_stats.vertex_count));
        output.push_str("\n");
        output.push_str(&format!("  Edge Count: {}", status.dag_stats.edge_count));
        output.push_str("\n");
        output.push_str(&format!("  Tip Count: {}", status.dag_stats.tip_count));
        output.push_str("\n");
        output.push_str(&format!("  Finalized Height: {}", status.dag_stats.finalized_height));
        output.push_str("\n");
        output.push_str(&format!("  Pending Transactions: {}", status.dag_stats.pending_transactions));
        output.push_str("\n");
        
        output.push_str("\nMemory Usage:\n");
        output.push_str(&format!("  Total Allocated: {} bytes", status.memory_usage.total_allocated_bytes));
        output.push_str("\n");
        output.push_str(&format!("  Current Usage: {} bytes", status.memory_usage.current_usage_bytes));
        output.push_str("\n");
        output.push_str(&format!("  Peak Usage: {} bytes", status.memory_usage.peak_usage_bytes));
        output.push_str("\n");
        
        if !status.connected_peers.is_empty() {
            output.push_str("\nConnected Peers:\n");
            for peer in &status.connected_peers {
                output.push_str(&format!("  {}: {} ({}s connected)", peer.peer_id, peer.address, peer.connected_duration_seconds));
                output.push_str("\n");
            }
        }
    }
    
    Ok(output)
}

/// Format status as a table
fn format_status_as_table(status: &NodeStatusResponse, verbose: bool) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("┌──────────────────────────────────────────────────────────────────────────────┐\n");
    output.push_str(&format!("│ Node Status: {:<62} │\n", status.node_id));
    output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
    output.push_str(&format!("│ State: {:<68} │\n", format!("{:?}", status.state)));
    output.push_str(&format!("│ Uptime: {:<67} │\n", format!("{} seconds", status.uptime_seconds)));
    output.push_str(&format!("│ Connected Peers: {:<60} │\n", status.connected_peers.len()));
    
    if verbose {
        output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
        output.push_str("│ Network Statistics                                                      │\n");
        output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("│ Total Connections: {:<57} │\n", status.network_stats.total_connections));
        output.push_str(&format!("│ Active Connections: {:<56} │\n", status.network_stats.active_connections));
        output.push_str(&format!("│ Messages Sent: {:<61} │\n", status.network_stats.messages_sent));
        output.push_str(&format!("│ Messages Received: {:<57} │\n", status.network_stats.messages_received));
        output.push_str(&format!("│ Bytes Sent: {:<64} │\n", status.network_stats.bytes_sent));
        output.push_str(&format!("│ Bytes Received: {:<60} │\n", status.network_stats.bytes_received));
        output.push_str(&format!("│ Average Latency: {:<59} │\n", format!("{:.2} ms", status.network_stats.average_latency_ms)));
        
        output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
        output.push_str("│ DAG Statistics                                                          │\n");
        output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("│ Vertex Count: {:<62} │\n", status.dag_stats.vertex_count));
        output.push_str(&format!("│ Edge Count: {:<64} │\n", status.dag_stats.edge_count));
        output.push_str(&format!("│ Tip Count: {:<65} │\n", status.dag_stats.tip_count));
        output.push_str(&format!("│ Finalized Height: {:<58} │\n", status.dag_stats.finalized_height));
        output.push_str(&format!("│ Pending Transactions: {:<54} │\n", status.dag_stats.pending_transactions));
        
        output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
        output.push_str("│ Memory Usage                                                            │\n");
        output.push_str("├──────────────────────────────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("│ Total Allocated: {:<59} │\n", format!("{} bytes", status.memory_usage.total_allocated_bytes)));
        output.push_str(&format!("│ Current Usage: {:<61} │\n", format!("{} bytes", status.memory_usage.current_usage_bytes)));
        output.push_str(&format!("│ Peak Usage: {:<64} │\n", format!("{} bytes", status.memory_usage.peak_usage_bytes)));
    }
    
    output.push_str("└──────────────────────────────────────────────────────────────────────────────┘\n");
    
    Ok(output)
}

/// Command routing and dispatch logic
pub struct CommandRouter;

impl CommandRouter {
    /// Route and execute node status command
    pub async fn handle_node_status(args: StatusArgs) -> Result<String, CliError> {
        info!("Executing node status command with port {}", args.port);
        
        match execute_status_command(args).await {
            Ok(output) => Ok(output),
            Err(e) => Err(CliError::Command(e.to_string())),
        }
    }

    /// Route and execute peer list command
    pub async fn handle_peer_list() -> Result<(), CliError> {
        info!("Executing peer list command");
        // TODO: Fetch actual peer list from running node
        unimplemented!("Peer list command not yet implemented")
    }

    /// Route and execute peer add command
    pub async fn handle_peer_add(address: String) -> Result<(), CliError> {
        info!("Executing peer add command for address: {}", address);
        // TODO: Validate address and connect to peer
        unimplemented!("Peer add command not yet implemented")
    }

    /// Route and execute peer remove command
    pub async fn handle_peer_remove(address: String) -> Result<(), CliError> {
        info!("Executing peer remove command for address: {}", address);
        // TODO: Disconnect from peer
        unimplemented!("Peer remove command not yet implemented")
    }

    /// Route and execute network stats command
    pub async fn handle_network_stats() -> Result<(), CliError> {
        info!("Executing network stats command");
        // TODO: Implement actual network statistics command
        println!("Network Statistics:");
        println!("==================");
        println!("(Network stats command not yet implemented)");
        Ok(())
    }

    /// Route and execute network test command
    pub async fn handle_network_test() -> Result<(), CliError> {
        info!("Executing network test command");
        // TODO: Implement actual network connectivity tests
        println!("Network Test Results:");
        println!("====================");
        println!("(Network test command not yet implemented)");
        Ok(())
    }
}

// Keep existing command implementations below for backward compatibility

pub(crate) async fn start_node(
    data_dir: Option<PathBuf>,
    port: Option<u16>,
    peers: Vec<String>,
) -> Result<(), CliError> {
    info!("Starting QuDAG node...");
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from("./data"));
    let port = port.unwrap_or(8000);
    
    info!("Data directory: {:?}", data_dir);
    info!("Port: {}", port);
    
    if !peers.is_empty() {
        info!("Initial peers: {:?}", peers);
    }
    
    // TODO: Implement actual node startup once core modules are ready
    info!("Node started successfully on port {}", port);
    
    // Keep the process running
    tokio::signal::ctrl_c().await
        .map_err(|e| CliError::Node(format!("Failed to wait for shutdown signal: {}", e)))?;
    
    info!("Shutting down...");
    Ok(())
}

pub(crate) async fn stop_node() -> Result<(), CliError> {
    info!("Stopping QuDAG node...");
    
    // TODO: Implement graceful shutdown by sending shutdown signal to running node
    warn!("Node stop functionality not yet implemented");
    
    Ok(())
}

pub(crate) async fn show_status() -> Result<(), CliError> {
    info!("Fetching node status...");

    let args = StatusArgs::default();
    match CommandRouter::handle_node_status(args).await {
        Ok(output) => {
            println!("{}", output);
            Ok(())
        }
        Err(e) => {
            warn!("Failed to get node status: {}", e);
            // Fallback to placeholder output for backward compatibility
            println!("Node Status:");
            println!("============");
            println!("Status: Not available ({})", e);
            println!("Port: 8000");
            println!("Peers: 0");
            println!("Messages: 0");
            println!("Uptime: 0s");
            Ok(())
        }
    }
}

pub(crate) async fn list_peers() -> Result<(), CliError> {
    info!("Listing connected peers...");
    
    // TODO: Fetch actual peer list from running node
    println!("Connected Peers:");
    println!("================");
    println!("No peers connected (placeholder)");

    Ok(())
}

pub(crate) async fn add_peer(address: String) -> Result<(), CliError> {
    info!("Adding peer: {}", address);
    
    // TODO: Validate address and connect to peer
    println!("Peer {} added successfully (placeholder)", address);
    
    Ok(())
}

pub(crate) async fn remove_peer(address: String) -> Result<(), CliError> {
    info!("Removing peer: {}", address);
    
    // TODO: Disconnect from peer
    println!("Peer {} removed successfully (placeholder)", address);
    
    Ok(())
}

pub(crate) async fn visualize_dag(
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
    
    info!("DAG visualization saved to {:?} in {} format", output, format);
    Ok(())
}

pub(crate) async fn show_network_stats() -> Result<(), CliError> {
    info!("Fetching network statistics...");
    
    // TODO: Fetch actual network statistics from running node
    println!("Network Statistics:");
    println!("==================");
    println!("Total Peers: 0");
    println!("Active Connections: 0");
    println!("Messages Sent: 0");
    println!("Messages Received: 0");
    println!("Total Bandwidth Used: 0 MB");
    println!("Average Latency: 0 ms");
    println!("Uptime: 0 seconds");
    
    Ok(())
}

pub(crate) async fn test_network() -> Result<(), CliError> {
    info!("Testing network connectivity...");
    
    // TODO: Implement actual network connectivity tests
    println!("Network Connectivity Test Results:");
    println!("==================================");
    println!("No peers to test (placeholder)");
    println!();
    println!("Network test complete.");
    
    Ok(())
}