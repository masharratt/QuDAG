use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Data directory
    pub data_dir: PathBuf,
    /// Network port
    pub port: u16,
    /// Initial peers
    pub peers: Vec<String>,
    /// Log level
    pub log_level: String,
    /// Node identity
    pub identity: IdentityConfig,
    /// Network configuration
    pub network: NetworkConfig,
}

/// Node identity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Node ID
    pub node_id: Option<String>,
    /// Private key file
    pub key_file: Option<PathBuf>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: String,
    /// External address
    pub external_addr: Option<String>,
    /// Maximum peers
    pub max_peers: usize,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            port: 8000,
            peers: Vec::new(),
            log_level: "info".to_string(),
            identity: IdentityConfig {
                node_id: None,
                key_file: None,
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0".to_string(),
                external_addr: None,
                max_peers: 50,
                bootstrap_nodes: Vec::new(),
            },
        }
    }
}

impl NodeConfig {
    /// Load configuration from file
    pub fn load(path: PathBuf) -> Result<Self> {
        let config = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&config)?)
    }

    /// Save configuration to file
    pub fn save(&self, path: PathBuf) -> Result<()> {
        let config = serde_json::to_string_pretty(self)?;
        std::fs::write(path, config)?;
        Ok(())
    }
}