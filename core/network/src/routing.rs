use std::collections::{HashMap, HashSet};
use std::time::Duration;
use libp2p::PeerId;
use thiserror::Error;
use tokio::sync::mpsc;
use rand::seq::SliceRandom;
use crate::shadow_address::{ShadowAddress, ShadowAddressError, ShadowAddressResolver};

/// Errors that can occur during routing operations
#[derive(Error, Debug)]
pub enum RoutingError {
    #[error("No route available to destination")]
    NoRoute,
    #[error("Message too large")]
    MessageTooLarge,
    #[error("Channel send error")]
    ChannelError,
    #[error("Shadow address error: {0}")]
    ShadowAddressError(#[from] ShadowAddressError),
}

/// Message destination type
#[derive(Debug, Clone)]
pub enum Destination {
    /// Direct peer routing
    Peer(PeerId),
    /// Shadow address routing
    Shadow(ShadowAddress),
}

impl From<PeerId> for Destination {
    fn from(peer_id: PeerId) -> Self {
        Destination::Peer(peer_id)
    }
}

impl From<ShadowAddress> for Destination {
    fn from(addr: ShadowAddress) -> Self {
        Destination::Shadow(addr)
    }
}

/// Represents a path through the network
#[derive(Clone, Debug)]
pub struct RoutePath {
    hops: Vec<PeerId>,
    latency: Duration,
    reliability: f64,
}

/// Multi-path router implementation
pub struct Router {
    /// Known peers and their connections
    peers: HashMap<PeerId, HashSet<PeerId>>,
    /// Path metrics
    path_metrics: HashMap<(PeerId, PeerId), RoutePath>,
    /// Message channel
    message_tx: mpsc::Sender<Vec<u8>>,
    /// Shadow address resolver
    shadow_resolver: Option<Box<dyn ShadowAddressResolver>>,
}

impl Router {
    /// Creates a new router instance
    pub fn new(message_tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            peers: HashMap::new(),
            path_metrics: HashMap::new(),
            message_tx,
            shadow_resolver: None,
        }
    }
    
    /// Set the shadow address resolver
    pub fn set_shadow_resolver(&mut self, resolver: Box<dyn ShadowAddressResolver>) {
        self.shadow_resolver = Some(resolver);
    }
    
    /// Find paths for a shadow address
    fn find_shadow_paths(&self, addr: &ShadowAddress) -> Result<Vec<RoutePath>, RoutingError> {
        // Resolve shadow address to onetime address
        let _resolved = if let Some(resolver) = &self.shadow_resolver {
            resolver.resolve_address(addr)?
        } else {
            return Err(RoutingError::NoRoute);
        };
        
        // Find random set of peers to use as intermediaries
        let mut rng = rand::thread_rng();
        let peer_count = 3; // Use 3 intermediate hops
        let mut available_peers: Vec<_> = self.peers.keys().collect();
        available_peers.shuffle(&mut rng);
        
        let selected_peers: Vec<_> = available_peers.into_iter()
            .take(peer_count)
            .cloned()
            .collect();
            
        if selected_peers.len() < peer_count {
            return Err(RoutingError::NoRoute);
        }
        
        // Create path through selected peers
        Ok(vec![RoutePath {
            hops: selected_peers,
            latency: Duration::from_millis(50),
            reliability: 0.95,
        }])
    }

    /// Adds a peer connection to the routing table
    pub fn add_peer_connection(&mut self, from: PeerId, to: PeerId) {
        self.peers.entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    /// Removes a peer connection from the routing table
    pub fn remove_peer_connection(&mut self, from: PeerId, to: PeerId) {
        if let Some(connections) = self.peers.get_mut(&from) {
            connections.remove(&to);
            if connections.is_empty() {
                self.peers.remove(&from);
            }
        }
    }

    /// Updates path metrics between two peers
    pub fn update_path_metrics(
        &mut self,
        from: PeerId,
        to: PeerId,
        path: RoutePath,
    ) {
        self.path_metrics.insert((from, to), path);
    }

    /// Finds multiple disjoint paths to a destination
    pub fn find_paths(&self, destination: PeerId) -> Vec<RoutePath> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();

        fn dfs(
            router: &Router,
            current: PeerId,
            destination: PeerId,
            path: Vec<PeerId>,
            visited: &mut HashSet<PeerId>,
            paths: &mut Vec<RoutePath>,
        ) {
            if current == destination {
                // Path found
                paths.push(RoutePath {
                    hops: path,
                    latency: Duration::from_millis(50), // TODO: Calculate actual latency
                    reliability: 0.95, // TODO: Calculate actual reliability
                });
                return;
            }

            if let Some(connections) = router.peers.get(&current) {
                for next in connections {
                    if !visited.contains(next) {
                        visited.insert(*next);
                        let mut new_path = path.clone();
                        new_path.push(*next);
                        dfs(router, *next, destination, new_path, visited, paths);
                        visited.remove(next);
                    }
                }
            }
        }

        if let Some(connections) = self.peers.get(&destination) {
            for start in connections {
                visited.insert(*start);
                dfs(self, *start, destination, vec![*start], &mut visited, &mut paths);
                visited.remove(start);
            }
        }

        paths
    }

    /// Routes a message through multiple paths using either PeerId or ShadowAddress
    pub async fn route_message(
        &self,
        destination: impl Into<Destination>,
        message: Vec<u8>,
    ) -> Result<(), RoutingError> {
        let dest = destination.into();
        
        // Get routing paths based on destination type
        let paths = match dest {
            Destination::Peer(peer_id) => self.find_paths(peer_id),
            Destination::Shadow(shadow_addr) => self.find_shadow_paths(&shadow_addr)?,
        };
        
        if paths.is_empty() {
            return Err(RoutingError::NoRoute);
        }

        // Split message into chunks for multi-path routing
        let chunk_size = message.len() / paths.len();
        let chunks: Vec<Vec<u8>> = message
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        // Send chunks through different paths
        for (chunk, path) in chunks.into_iter().zip(paths) {
            // Add routing header with path information
            let mut routed_message = Vec::new();
            routed_message.extend_from_slice(&path.hops.len().to_le_bytes());
            for hop in path.hops {
                routed_message.extend_from_slice(&hop.to_bytes());
            }
            routed_message.extend_from_slice(&chunk);

            // Send through channel
            self.message_tx.send(routed_message).await
                .map_err(|_| RoutingError::ChannelError)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use crate::shadow_address::{NetworkType, ShadowMetadata};

    // Mock shadow address resolver for testing
    struct MockResolver;
    
    impl ShadowAddressResolver for MockResolver {
        fn resolve_address(&self, _: &ShadowAddress) -> Result<Vec<u8>, ShadowAddressError> {
            Ok(vec![1, 2, 3, 4])
        }
        
        fn check_address(&self, _: &ShadowAddress, onetime: &[u8]) -> Result<bool, ShadowAddressError> {
            Ok(onetime == &[1, 2, 3, 4])
        }
    }

    fn setup_test_router() -> (Router, mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(128);
        let mut router = Router::new(tx);
        router.set_shadow_resolver(Box::new(MockResolver));
        (router, rx)
    }
    
    fn create_test_shadow_address() -> ShadowAddress {
        ShadowAddress {
            view_key: vec![1, 2, 3, 4],
            spend_key: vec![5, 6, 7, 8],
            payment_id: None,
            metadata: ShadowMetadata {
                version: 1,
                network: NetworkType::Testnet,
                expires_at: None,
                flags: 0,
            },
        }
    }

    #[test]
    fn test_add_remove_peer() {
        let (mut router, _) = setup_test_router();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        router.add_peer_connection(peer1, peer2);
        assert!(router.peers.get(&peer1).unwrap().contains(&peer2));

        router.remove_peer_connection(peer1, peer2);
        assert!(!router.peers.contains_key(&peer1));
    }

    #[tokio::test]
    async fn test_route_message() {
        let (mut router, mut rx) = setup_test_router();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        // Set up a path
        router.add_peer_connection(peer1, peer2);
        router.add_peer_connection(peer2, peer3);

        let test_msg = vec![1, 2, 3, 4];
        router.route_message(peer3, test_msg.clone()).await.unwrap();

        // Verify message was sent
        let received = rx.recv().await.unwrap();
        assert!(!received.is_empty());
    }

    #[test]
    fn test_find_paths() {
        let (mut router, _) = setup_test_router();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        router.add_peer_connection(peer1, peer2);
        router.add_peer_connection(peer2, peer3);

        let paths = router.find_paths(peer3);
        assert!(!paths.is_empty());
    }
    
    #[tokio::test]
    async fn test_route_shadow_message() {
        let (mut router, mut rx) = setup_test_router();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        // Set up some peers
        router.add_peer_connection(peer1, peer2);
        router.add_peer_connection(peer2, peer3);

        // Try routing to a shadow address
        let shadow_addr = create_test_shadow_address();
        let test_msg = vec![1, 2, 3, 4];
        router.route_message(shadow_addr, test_msg.clone()).await.unwrap();

        // Verify message was sent
        let received = rx.recv().await.unwrap();
        assert!(!received.is_empty());
    }
}