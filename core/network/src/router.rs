use crate::types::{NetworkMessage, PeerId, RoutingStrategy, NetworkError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Information about a hop in a route
#[derive(Debug, Clone)]
pub struct HopInfo {
    peer_id: PeerId,
    known_peers: HashSet<PeerId>,
    layer_keys: HashMap<usize, Vec<u8>>,
}

impl HopInfo {
    /// Check if this hop can decrypt a specific layer
    pub fn can_decrypt_layer(&self, layer: usize) -> bool {
        self.layer_keys.contains_key(&layer)
    }
    
    /// Check if this hop knows about a specific peer
    pub fn knows_peer(&self, peer: &PeerId) -> bool {
        self.known_peers.contains(peer)
    }
}

/// Anonymous router for network messages
#[derive(Debug, Clone)]
pub struct Router {
    /// Known peers in the network
    peers: Arc<RwLock<HashSet<PeerId>>>,
    /// Hop information for each peer
    hop_info: Arc<RwLock<HashMap<PeerId, HopInfo>>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashSet::new())),
            hop_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a peer to the network
    pub async fn add_peer(&self, peer_id: PeerId) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id);
        
        // Create hop info for this peer
        let mut hop_info = self.hop_info.write().await;
        let mut known_peers = HashSet::new();
        
        // Each peer knows about a random subset of other peers (simulating network topology)
        let all_peers: Vec<_> = peers.iter().filter(|&&p| p != peer_id).cloned().collect();
        let mut rng = thread_rng();
        let subset_size = (all_peers.len() / 2).max(1).min(3); // Know about 1-3 peers
        let known_subset: Vec<_> = all_peers.choose_multiple(&mut rng, subset_size).cloned().collect();
        
        for peer in known_subset {
            known_peers.insert(peer);
        }
        
        // Generate layer keys for this peer (simulating onion routing capabilities)
        let mut layer_keys = HashMap::new();
        for i in 0..5 { // Support up to 5 layers
            layer_keys.insert(i, vec![i as u8; 32]); // Simple key generation
        }
        
        hop_info.insert(peer_id, HopInfo {
            peer_id,
            known_peers,
            layer_keys,
        });
    }
    
    /// Route a message using the specified strategy
    pub async fn route(&self, message: &NetworkMessage, strategy: RoutingStrategy) -> Result<Vec<PeerId>, NetworkError> {
        match strategy {
            RoutingStrategy::Anonymous { hops } => {
                self.route_anonymous(message, hops).await
            }
            RoutingStrategy::Direct(peer_bytes) => {
                // Convert bytes to PeerId if possible
                if peer_bytes.len() == 32 {
                    let mut peer_id_bytes = [0u8; 32];
                    peer_id_bytes.copy_from_slice(&peer_bytes);
                    Ok(vec![PeerId::from_bytes(peer_id_bytes)])
                } else {
                    Err(NetworkError::RoutingError("Invalid peer ID format".into()))
                }
            }
            RoutingStrategy::Flood => {
                let peers = self.peers.read().await;
                Ok(peers.iter().cloned().collect())
            }
            RoutingStrategy::RandomSubset(count) => {
                let peers = self.peers.read().await;
                let mut rng = thread_rng();
                let selected: Vec<_> = peers.iter().choose_multiple(&mut rng, count).cloned().collect();
                Ok(selected)
            }
        }
    }
    
    /// Route a message anonymously using onion routing
    async fn route_anonymous(&self, message: &NetworkMessage, hops: usize) -> Result<Vec<PeerId>, NetworkError> {
        let peers = self.peers.read().await;
        
        // Filter out source and destination from available peers
        let source_peer = if message.source.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&message.source);
            Some(PeerId::from_bytes(bytes))
        } else {
            None
        };
        
        let dest_peer = if message.destination.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&message.destination);
            Some(PeerId::from_bytes(bytes))
        } else {
            None
        };
        
        let available_peers: Vec<_> = peers.iter()
            .filter(|&&p| Some(p) != source_peer && Some(p) != dest_peer)
            .cloned()
            .collect();
            
        if available_peers.len() < hops {
            return Err(NetworkError::RoutingError("Not enough peers for anonymous routing".into()));
        }
        
        // Select random peers for the route
        let mut rng = thread_rng();
        let route: Vec<_> = available_peers.choose_multiple(&mut rng, hops).cloned().collect();
        
        // Update hop info to simulate onion routing knowledge
        self.update_hop_knowledge(&route).await;
        
        Ok(route)
    }
    
    /// Update hop knowledge to simulate onion routing properties
    async fn update_hop_knowledge(&self, route: &[PeerId]) {
        let mut hop_info = self.hop_info.write().await;
        
        for (i, &peer_id) in route.iter().enumerate() {
            if let Some(info) = hop_info.get_mut(&peer_id) {
                // Clear previous knowledge
                info.known_peers.clear();
                
                // Each hop only knows about its immediate neighbors
                if i > 0 {
                    info.known_peers.insert(route[i - 1]);
                }
                if i < route.len() - 1 {
                    info.known_peers.insert(route[i + 1]);
                }
                
                // Update layer keys - each hop can only decrypt its own layer
                info.layer_keys.clear();
                info.layer_keys.insert(i, vec![i as u8; 32]);
            }
        }
    }
    
    /// Get hop information for a peer
    pub async fn get_hop_info(&self, peer_id: &PeerId) -> Result<HopInfo, NetworkError> {
        let hop_info = self.hop_info.read().await;
        hop_info.get(peer_id)
            .cloned()
            .ok_or_else(|| NetworkError::RoutingError("Hop information not found".into()))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessagePriority;
    use std::time::Duration;

    #[tokio::test]
    async fn test_router_creation() {
        let router = Router::new();
        let peers = router.peers.read().await;
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn test_add_peer() {
        let router = Router::new();
        let peer_id = PeerId::random();
        
        router.add_peer(peer_id).await;
        
        let peers = router.peers.read().await;
        assert!(peers.contains(&peer_id));
    }

    #[tokio::test]
    async fn test_anonymous_routing() {
        let router = Router::new();
        
        // Add test peers
        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }
        
        // Create test message
        let msg = NetworkMessage {
            id: "test".into(),
            source: peers[0].to_bytes().to_vec(),
            destination: peers[4].to_bytes().to_vec(),
            payload: vec![1, 2, 3],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };
        
        // Test anonymous routing
        let route = router.route(&msg, RoutingStrategy::Anonymous { hops: 3 }).await.unwrap();
        
        assert_eq!(route.len(), 3);
        assert!(!route.contains(&peers[0])); // Should not include source
        assert!(!route.contains(&peers[4])); // Should not include destination
    }
}