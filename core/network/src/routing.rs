#![deny(unsafe_code)]

use crate::types::{NetworkMessage, RoutingStrategy};
use anyhow::Result;
use libp2p::PeerId;
use qudag_crypto::encryption::hqc::HQCEncryption;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use rand::seq::IteratorRandom;
use qudag_crypto::kem::ml_kem::MLKem;

const MIN_CIRCUIT_LENGTH: usize = 3;
const MAX_CIRCUIT_LENGTH: usize = 7;

/// Anonymous routing implementation
pub struct Router {
    /// Connected peers
    peers: Arc<RwLock<HashSet<PeerId>>>,
    /// Routing table
    routes: Arc<RwLock<HashMap<Vec<u8>, Vec<PeerId>>>>,
    /// Encryption for anonymous routing
    encryption: HQCEncryption,
    /// KEM for circuit building
    kem: MLKem,
    /// Active circuits
    circuits: Arc<RwLock<HashMap<Vec<u8>, Vec<PeerId>>>>
}

impl Router {
    /// Creates a new router instance
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashSet::new())),
            routes: Arc::new(RwLock::new(HashMap::new())),
            encryption: HQCEncryption::new(),
            kem: MLKem::new(),
            circuits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds a peer to the routing table
    pub async fn add_peer(&self, peer_id: PeerId) {
        self.peers.write().await.insert(peer_id);
    }

    /// Removes a peer from the routing table
    pub async fn remove_peer(&self, peer_id: &PeerId) {
        self.peers.write().await.remove(peer_id);
    }

    /// Routes a message according to the specified strategy
    pub async fn route(&self, msg: &NetworkMessage, strategy: RoutingStrategy) -> Result<Vec<PeerId>> {
        let peers = self.peers.read().await;
        
        match strategy {
            RoutingStrategy::Direct(dest) => {
                if let Some(route) = self.routes.read().await.get(&dest) {
                    Ok(route.clone())
                } else {
                    warn!("No route found for destination");
                    Ok(vec![])
                }
            }
            
            RoutingStrategy::Flood => {
                Ok(peers.iter().copied().collect())
            }
            
            RoutingStrategy::RandomSubset(count) => {
                let mut rng = rand::thread_rng();
                Ok(peers.iter()
                    .copied()
                    .choose_multiple(&mut rng, count)
                    .collect())
            }
            
            RoutingStrategy::Anonymous { hops } => {
                let circuit_id = msg.id.clone();
                let mut circuits = self.circuits.write().await;
                
                // Check for existing circuit
                if let Some(route) = circuits.get(&circuit_id) {
                    return Ok(route.clone());
                }
                
                // Build new circuit
                let hop_count = hops.clamp(MIN_CIRCUIT_LENGTH, MAX_CIRCUIT_LENGTH);
                let mut rng = rand::thread_rng();
                let mut selected_peers = Vec::with_capacity(hop_count);
                let mut excluded = HashSet::new();
                
                // Select diverse path
                for _ in 0..hop_count {
                    if let Some(peer) = peers.iter()
                        .filter(|p| !excluded.contains(*p))
                        .choose(&mut rng) {
                        selected_peers.push(*peer);
                        excluded.insert(peer);
                    }
                }
                
                // Setup circuit encryption
                let mut encrypted = msg.payload.clone();
                for hop in selected_peers.iter().rev() {
                    // Generate ephemeral keys for this hop
                    let (eph_pub, eph_priv) = self.kem.keygen()?;
                    
                    // Layer encryption
                    let shared_secret = self.kem.encapsulate(&eph_pub)?;
                    encrypted = self.encryption.encrypt_with_key(&encrypted, &shared_secret)?;
                }
                
                // Store circuit
                circuits.insert(circuit_id, selected_peers.clone());
                
                Ok(selected_peers)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::types::MessagePriority;

    #[tokio::test]
    async fn test_routing() {
        let router = Router::new();
        
        // Add test peers
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();
        
        router.add_peer(peer1).await;
        router.add_peer(peer2).await;
        router.add_peer(peer3).await;
        
        let msg = NetworkMessage {
            id: "test".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Test flood routing
        let flood_peers = router.route(&msg, RoutingStrategy::Flood).await.unwrap();
        assert_eq!(flood_peers.len(), 3);

        // Test random subset routing
        let subset = router.route(&msg, RoutingStrategy::RandomSubset(2)).await.unwrap();
        assert_eq!(subset.len(), 2);

        // Test anonymous routing
        let route = router.route(&msg, RoutingStrategy::Anonymous { hops: 2 }).await.unwrap();
        assert_eq!(route.len(), 2);
    }
}