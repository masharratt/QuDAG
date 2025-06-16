use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    identity, noise, yamux, PeerId, Transport,
    kad::{Kademlia, KademliaConfig, store::MemoryStore},
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
};
use std::{error::Error, time::Duration};
use tokio::sync::mpsc;
use futures::StreamExt;
use tracing::{debug, error, info};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};
use rand::{thread_rng, RngCore};

use crate::routing::{Router, RoutePath, RoutingError};

/// Configuration for the P2P network node
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listening address
    pub listen_addr: String,
    /// Bootstrap peer addresses
    pub bootstrap_peers: Vec<String>,
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Traffic obfuscation key
    pub obfuscation_key: [u8; 32],
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let mut key = [0u8; 32];
        thread_rng().fill_bytes(&mut key);
        
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".to_string(),
            bootstrap_peers: vec![],
            timeout: Duration::from_secs(20),
            max_connections: 50,
            obfuscation_key: key,
        }
    }
}

/// Main P2P network node implementation
pub struct P2PNode {
    local_peer_id: PeerId,
    swarm: libp2p::Swarm<NetworkBehaviourImpl>,
    router: Router,
    cipher: ChaCha20Poly1305,
    message_rx: mpsc::Receiver<Vec<u8>>,
}

/// Custom network behaviour combining Kademlia DHT and custom protocols
#[derive(NetworkBehaviour)]
struct NetworkBehaviourImpl {
    kademlia: Kademlia<MemoryStore>,
    // Add custom protocols here
}

impl P2PNode {
    /// Creates a new P2P network node with the given configuration
    pub async fn new(config: NetworkConfig) -> Result<Self, Box<dyn Error>> {
        // Generate node identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        // Set up transport with noise encryption and yamux multiplexing
        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&local_key)?;

        let transport = TokioTcpConfig::new()
            .nodelay(true)
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        // Set up Kademlia DHT
        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::new(local_peer_id, store);
        
        // Initialize network behaviour
        let behaviour = NetworkBehaviourImpl {
            kademlia,
        };

        // Build the swarm
        let swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        // Set up message channels and router
        let (tx, rx) = mpsc::channel(128);
        let router = Router::new(tx);

        // Initialize traffic obfuscation
        let key = Key::from_slice(&config.obfuscation_key);
        let cipher = ChaCha20Poly1305::new(key);

        Ok(Self {
            local_peer_id,
            swarm,
            router,
            cipher,
            message_rx: rx,
        })
    }

    /// Starts the network node and handles events
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        // Listen on configured address
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // Event loop
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {:?}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            debug!("Connected to {:?}", peer_id);
                            // Add peer to routing table
                            self.router.add_peer_connection(self.local_peer_id, peer_id);
                            
                            // Add reverse path for bidirectional routing
                            self.router.add_peer_connection(peer_id, self.local_peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            debug!("Disconnected from {:?}", peer_id);
                            // Remove peer from routing table
                            self.router.remove_peer_connection(self.local_peer_id, peer_id);
                            self.router.remove_peer_connection(peer_id, self.local_peer_id);
                        }
                        _ => {}
                    }
                }
                Some(message) = self.message_rx.recv() => {
                    // Handle outgoing messages
                    self.handle_outgoing_message(message).await?;
                }
            }
        }
    }

    /// Obfuscates traffic using ChaCha20-Poly1305
    fn obfuscate_traffic(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        let mut encrypted = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| format!("Encryption error: {}", e))?;

        // Prepend nonce to encrypted data
        let mut result = nonce.to_vec();
        result.append(&mut encrypted);
        Ok(result)
    }

    /// Deobfuscates traffic using ChaCha20-Poly1305
    fn deobfuscate_traffic(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if data.len() < 12 {
            return Err("Data too short".into());
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let encrypted = &data[12..];

        self.cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| format!("Decryption error: {}", e).into())
    }

    /// Handles outgoing messages by routing them through multiple paths
    async fn handle_outgoing_message(&mut self, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
        // First obfuscate the message
        let obfuscated = self.obfuscate_traffic(&message)?;

        // Get available peers from Kademlia DHT
        let peers: Vec<PeerId> = self.swarm
            .behaviour()
            .kademlia
            .kbuckets()
            .peers()
            .map(|p| *p.node.key.preimage())
            .collect();

        // Route message through multiple paths
        for peer in peers {
            if let Err(e) = self.router.route_message(peer, obfuscated.clone()).await {
                match e {
                    RoutingError::NoRoute => continue, // Try next peer
                    _ => return Err(e.into()),
                }
            }
        }

        Ok(())
    }

    /// Returns the node's PeerId
    pub fn peer_id(&self) -> PeerId {
        self.local_peer_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_node_creation() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config).await.unwrap();
        assert!(node.peer_id().to_base58().len() > 0);
    }

    #[tokio::test]
    async fn test_traffic_obfuscation() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config).await.unwrap();
        
        let test_data = b"test message";
        let obfuscated = node.obfuscate_traffic(test_data).unwrap();
        let deobfuscated = node.deobfuscate_traffic(&obfuscated).unwrap();
        
        assert_eq!(test_data.to_vec(), deobfuscated);
    }

    #[tokio::test]
    async fn test_node_listening() {
        let config = NetworkConfig {
            listen_addr: "/ip4/127.0.0.1/tcp/0".to_string(),
            ..Default::default()
        };
        let mut node = P2PNode::new(config).await.unwrap();
        
        // Start node in background
        tokio::spawn(async move {
            node.start().await.unwrap();
        });

        // Allow time for node to start listening
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_peer_connections() {
        let config = NetworkConfig::default();
        let mut node = P2PNode::new(config).await.unwrap();
        
        let test_peer = PeerId::random();
        node.router.add_peer_connection(node.local_peer_id, test_peer);
        
        let paths = node.router.find_paths(test_peer);
        assert!(!paths.is_empty());
    }
}