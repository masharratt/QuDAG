use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use rand::seq::SliceRandom;
use std::fmt;

/// Error types for routing operations
#[derive(Error, Debug)]
pub enum RouteError {
    /// Path selection failed
    #[error("path selection failed: {0}")]
    PathSelectionError(String),
    
    /// Invalid route configuration
    #[error("invalid route config: {0}")]
    ConfigError(String),
    
    /// Route validation failed
    #[error("route validation failed: {0}")]
    ValidationError(String),
}

/// Router configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Minimum number of hops in a route
    pub min_hops: usize,
    /// Maximum number of hops in a route
    pub max_hops: usize,
    /// Maximum number of path attempts
    pub max_attempts: usize,
    /// Required node properties for routing
    pub required_props: HashSet<String>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            min_hops: 3,
            max_hops: 10,
            max_attempts: 50,
            required_props: HashSet::new(),
        }
    }
}

/// Router interface for path selection and validation
pub trait Router: Send + Sync {
    /// Selects a route path through the network
    fn select_path(
        &self,
        destination: Vec<u8>,
        config: &RouterConfig,
    ) -> Result<Vec<Vec<u8>>, RouteError>;
    
    /// Validates a proposed route path
    fn validate_path(&self, path: &[Vec<u8>]) -> Result<(), RouteError>;
    
    /// Updates router with new network information
    fn update_network(&mut self, peers: Vec<Vec<u8>>);
}

/// Implementation of the QuDAG router
pub struct QuDagRouter {
    /// Known network peers
    peers: Vec<Vec<u8>>,
    /// Router configuration
    config: RouterConfig,
}

impl QuDagRouter {
    /// Creates a new QuDAG router with the given configuration
    pub fn new(config: RouterConfig) -> Self {
        Self {
            peers: Vec::new(),
            config,
        }
    }

    /// Selects random peers for path excluding specific peers
    fn select_random_peers(
        &self,
        count: usize,
        exclude: &HashSet<Vec<u8>>
    ) -> Option<Vec<Vec<u8>>> {
        let mut rng = rand::thread_rng();
        let available: Vec<_> = self.peers.iter()
            .filter(|p| !exclude.contains(*p))
            .cloned()
            .collect();
        
        if available.len() < count {
            return None;
        }
        
        Some(available.choose_multiple(&mut rng, count).cloned().collect())
    }
}

impl Router for QuDagRouter {
    fn select_path(
        &self,
        destination: Vec<u8>,
        config: &RouterConfig,
    ) -> Result<Vec<Vec<u8>>, RouteError> {
        if config.min_hops < 2 {
            return Err(RouteError::ConfigError("minimum hops must be at least 2".into()));
        }
        
        let mut attempts = 0;
        let mut exclude = HashSet::new();
        exclude.insert(destination.clone());
        
        while attempts < config.max_attempts {
            let hop_count = rand::thread_rng().gen_range(config.min_hops..=config.max_hops);
            
            if let Some(mut path) = self.select_random_peers(hop_count - 1, &exclude) {
                path.push(destination.clone());
                if self.validate_path(&path).is_ok() {
                    return Ok(path);
                }
            }
            
            attempts += 1;
        }
        
        Err(RouteError::PathSelectionError("failed to find valid path".into()))
    }
    
    fn validate_path(&self, path: &[Vec<u8>]) -> Result<(), RouteError> {
        if path.len() < self.config.min_hops {
            return Err(RouteError::ValidationError(
                format!("path length {} below minimum {}", path.len(), self.config.min_hops)
            ));
        }
        
        if path.len() > self.config.max_hops {
            return Err(RouteError::ValidationError(
                format!("path length {} exceeds maximum {}", path.len(), self.config.max_hops)
            ));
        }
        
        let mut seen = HashSet::new();
        for peer in path {
            if !seen.insert(peer) {
                return Err(RouteError::ValidationError("duplicate peer in path".into()));
            }
        }
        
        Ok(())
    }
    
    fn update_network(&mut self, peers: Vec<Vec<u8>>) {
        self.peers = peers;
    }
}
