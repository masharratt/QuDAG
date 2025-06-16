//! Protocol configuration implementation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Node configuration
    pub node: NodeConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// Consensus configuration
    pub consensus: ConsensusConfig,
}

/// Node-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node ID
    pub node_id: String,
    
    /// Data directory
    pub data_dir: PathBuf,
    
    /// Log level
    pub log_level: String,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen port
    pub port: u16,
    
    /// Maximum number of peers
    pub max_peers: usize,
    
    /// Connection timeout
    pub connect_timeout: Duration,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Finality threshold
    pub finality_threshold: f64,
    
    /// Round timeout
    pub round_timeout: Duration,
    
    /// Maximum rounds
    pub max_rounds: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node: NodeConfig::default(),
            network: NetworkConfig::default(),
            consensus: ConsensusConfig::default(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: "node-0".to_string(),
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            max_peers: 50,
            connect_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            finality_threshold: 0.67,
            round_timeout: Duration::from_secs(10),
            max_rounds: 100,
        }
    }
}