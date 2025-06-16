#![deny(unsafe_code)]
#![deny(missing_docs)]

//! QuDAG Protocol Implementation
//! 
//! This module coordinates the core components of the QuDAG protocol:
//! - Cryptographic operations (ML-KEM, ML-DSA, HQC)
//! - DAG consensus (QR-Avalanche)
//! - Network communication
//!
//! The protocol ensures quantum resistance through post-quantum cryptographic primitives
//! while maintaining high performance and security guarantees.

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use tracing::{info, warn, error};

use qudag_crypto::{KeyPair, PublicKey};
use qudag_dag::QrDag;
use qudag_network::NetworkManager;

/// Protocol configuration parameters
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Network port for P2P communication
    pub network_port: u16,
    /// Bootstrap nodes for initial connection
    pub bootstrap_nodes: Vec<String>,
    /// Maximum number of peers to maintain
    pub max_peers: usize,
    /// Message validation timeout in milliseconds
    pub validation_timeout: u64,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            network_port: 8000,
            bootstrap_nodes: vec![],
            max_peers: 50,
            validation_timeout: 5000,
        }
    }
}

/// Protocol state machine states
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolState {
    /// Initial state
    Initialized,
    /// Protocol is running
    Running,
    /// Protocol has been stopped
    Stopped,
    /// Error state
    Error,
}

/// Main protocol coordinator
pub struct Coordinator {
    config: ProtocolConfig,
    state: Arc<RwLock<ProtocolState>>,
    crypto_manager: Option<KeyPair>,
    network_manager: Option<NetworkManager>,
    dag_manager: Option<Arc<QrDag>>,
}

impl Coordinator {
    /// Creates a new protocol coordinator
    pub async fn new(config: ProtocolConfig) -> Result<Self> {
        info!("Initializing QuDAG protocol coordinator");
        
        // Initialize crypto
        let keypair = KeyPair::generate()
            .context("Failed to generate keypair")?;
            
        // Initialize network
        let network = NetworkManager::new(config.network_port, config.max_peers)
            .context("Failed to initialize network")?;
            
        // Initialize DAG
        let dag = Arc::new(QrDag::new()
            .context("Failed to initialize DAG")?);
            
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ProtocolState::Initialized)),
            crypto_manager: Some(keypair),
            network_manager: Some(network),
            dag_manager: Some(dag),
        })
    }
    
    /// Starts the protocol
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting QuDAG protocol");
        
        // Start network manager
        if let Some(network) = &self.network_manager {
            network.start().await
                .context("Failed to start network manager")?;
        }
        
        // Update state
        let mut state = self.state.write().await;
        *state = ProtocolState::Running;
        
        info!("QuDAG protocol started successfully");
        Ok(())
    }
    
    /// Stops the protocol
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping QuDAG protocol");
        
        // Stop network manager
        if let Some(network) = &self.network_manager {
            network.stop().await
                .context("Failed to stop network manager")?;
        }
        
        // Update state
        let mut state = self.state.write().await;
        *state = ProtocolState::Stopped;
        
        info!("QuDAG protocol stopped successfully");
        Ok(())
    }
    
    /// Broadcasts a message through the network
    pub async fn broadcast_message(&self, message: Vec<u8>) -> Result<()> {
        // Sign message
        let signature = self.crypto_manager.as_ref()
            .context("Crypto manager not initialized")?
            .sign(&message)
            .context("Failed to sign message")?;
            
        // Add to DAG
        self.dag_manager.as_ref()
            .context("DAG manager not initialized")?
            .add_message(&message, &signature)
            .context("Failed to add message to DAG")?;
            
        // Broadcast through network
        self.network_manager.as_ref()
            .context("Network manager not initialized")?
            .broadcast(message)
            .await
            .context("Failed to broadcast message")?;
            
        Ok(())
    }
    
    /// Returns current protocol state
    pub async fn state(&self) -> ProtocolState {
        self.state.read().await.clone()
    }
    
    /// Returns whether protocol is initialized
    pub fn is_initialized(&self) -> bool {
        self.crypto_manager.is_some() && 
        self.network_manager.is_some() &&
        self.dag_manager.is_some()
    }
    
    /// Returns reference to crypto manager
    pub fn crypto_manager(&self) -> Option<&KeyPair> {
        self.crypto_manager.as_ref()
    }
    
    /// Returns reference to network manager
    pub fn network_manager(&self) -> Option<&NetworkManager> {
        self.network_manager.as_ref()
    }
    
    /// Returns reference to DAG manager
    pub fn dag_manager(&self) -> Option<&Arc<QrDag>> {
        self.dag_manager.as_ref()
    }
}