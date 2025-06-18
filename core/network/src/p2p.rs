use libp2p::{
    core::{
        multiaddr::{Multiaddr, Protocol},
        transport::{Boxed, MemoryTransport, Transport as LibP2PTransport},
        upgrade::{self, SelectUpgrade},
    },
    dcutr,
    gossipsub::{self, MessageAuthenticity, ValidationMode, IdentTopic, Config as GossipsubConfig, ConfigBuilder as GossipsubConfigBuilder},
    identify::{self},
    identity::{self, Keypair},
    kad::{self, store::MemoryStore, QueryResult},
    mdns::{self},
    noise,
    ping::{self},
    relay,
    request_response::{self, ProtocolSupport},
    swarm::{
        behaviour::toggle::Toggle, NetworkBehaviour,
        SwarmEvent,
    },
    SwarmBuilder,
    tcp, websocket, yamux, PeerId as LibP2PPeerId, StreamProtocol,
};

/// Combined network behaviour event
#[derive(Debug)]
pub enum NetworkBehaviourEvent {
    Kademlia(kad::Event),
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Ping(ping::Event),
    Identify(identify::Event),
    Relay(relay::Event),
    Dcutr(dcutr::Event),
    RequestResponse(request_response::Event<QuDagRequest, QuDagResponse>),
}

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, RwLock};
use futures::{channel::oneshot, prelude::*, select};
use tracing::{debug, error, info, warn};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    routing::{Router, RoutePath, RoutingError},
    types::{NetworkError, PeerId},
};

/// Configuration for the P2P network node
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listening addresses
    pub listen_addrs: Vec<String>,
    /// Bootstrap peer addresses
    pub bootstrap_peers: Vec<String>,
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Traffic obfuscation key
    pub obfuscation_key: [u8; 32],
    /// Enable MDNS for local peer discovery
    pub enable_mdns: bool,
    /// Enable relay for NAT traversal
    pub enable_relay: bool,
    /// Enable QUIC transport
    pub enable_quic: bool,
    /// Enable WebSocket transport
    pub enable_websocket: bool,
    /// Gossipsub configuration
    pub gossipsub_config: Option<GossipsubConfig>,
    /// Kademlia replication factor
    pub kad_replication_factor: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let mut key = [0u8; 32];
        thread_rng().fill_bytes(&mut key);

        Self {
            listen_addrs: vec![
                "/ip4/0.0.0.0/tcp/0".to_string(),
                "/ip6/::/tcp/0".to_string(),
            ],
            bootstrap_peers: vec![],
            timeout: Duration::from_secs(20),
            max_connections: 50,
            obfuscation_key: key,
            enable_mdns: true,
            enable_relay: true,
            enable_quic: false,
            enable_websocket: true,
            gossipsub_config: None,
            kad_replication_factor: 20,
        }
    }
}

/// Request-response protocol for custom messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuDagRequest {
    pub request_id: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuDagResponse {
    pub request_id: String,
    pub payload: Vec<u8>,
}

/// Combined network behaviour for the P2P node
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NetworkBehaviourEvent")]
pub struct NetworkBehaviourImpl {
    /// Kademlia DHT for peer discovery and content routing
    pub kademlia: kad::Behaviour<MemoryStore>,
    /// Gossipsub for pub/sub messaging
    pub gossipsub: gossipsub::Behaviour,
    /// MDNS for local peer discovery
    pub mdns: Toggle<mdns::tokio::Behaviour>,
    /// Ping for keep-alive and latency measurement
    pub ping: ping::Behaviour,
    /// Identify protocol for peer identification
    pub identify: identify::Behaviour,
    /// Relay for NAT traversal
    pub relay: relay::Behaviour,
    /// Direct connection upgrade through relay
    pub dcutr: dcutr::Behaviour,
    /// Request-response protocol for custom messages
    pub request_response: request_response::cbor::Behaviour<QuDagRequest, QuDagResponse>,
}

/// Events emitted by the P2P network
#[derive(Debug)]
pub enum P2PEvent {
    /// New peer discovered
    PeerDiscovered(LibP2PPeerId),
    /// Peer connection established
    PeerConnected(LibP2PPeerId),
    /// Peer disconnected
    PeerDisconnected(LibP2PPeerId),
    /// Message received via gossipsub
    MessageReceived {
        peer_id: LibP2PPeerId,
        topic: String,
        data: Vec<u8>,
    },
    /// Request received
    RequestReceived {
        peer_id: LibP2PPeerId,
        request: QuDagRequest,
        channel: oneshot::Sender<QuDagResponse>,
    },
    /// Response received
    ResponseReceived {
        peer_id: LibP2PPeerId,
        response: QuDagResponse,
    },
    /// Routing table updated
    RoutingTableUpdated,
}

/// Main P2P network node implementation
pub struct P2PNode {
    /// Local peer ID
    local_peer_id: LibP2PPeerId,
    /// Swarm instance
    swarm: libp2p::Swarm<NetworkBehaviourImpl>,
    /// Router for message routing
    router: Arc<RwLock<Router>>,
    /// Traffic obfuscation cipher
    cipher: ChaCha20Poly1305,
    /// Event channel sender
    event_tx: mpsc::UnboundedSender<P2PEvent>,
    /// Event channel receiver
    event_rx: mpsc::UnboundedReceiver<P2PEvent>,
    /// Connected peers
    connected_peers: Arc<RwLock<HashSet<LibP2PPeerId>>>,
    /// Pending requests
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<QuDagResponse>>>>,
    /// Metrics recorder
    metrics: Option<()>, // TODO: Use proper metrics type
    /// Network configuration
    config: NetworkConfig,
}

impl P2PNode {
    /// Creates a new P2P network node with the given configuration
    pub async fn new(config: NetworkConfig) -> Result<Self, Box<dyn Error>> {
        // Generate node identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = LibP2PPeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        // Build the transport
        let transport = build_transport(&local_key, &config)?;

        // Set up Kademlia DHT
        let store = MemoryStore::new(local_peer_id);
        let mut kad_config = kad::Config::default();
        kad_config.set_replication_factor(
            std::num::NonZeroUsize::new(config.kad_replication_factor)
                .expect("Replication factor must be > 0"),
        );
        let kademlia = kad::Behaviour::with_config(local_peer_id, store, kad_config);

        // Set up Gossipsub
        let gossipsub_config = config.gossipsub_config.clone().unwrap_or_else(|| {
            GossipsubConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(ValidationMode::Strict)
                .build()
                .expect("Valid gossipsub config")
        });

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Set up MDNS
        let mdns = if config.enable_mdns {
            Toggle::from(Some(mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                local_peer_id,
            )?))
        } else {
            Toggle::from(None)
        };

        // Set up other protocols
        let ping = ping::Behaviour::new(ping::Config::new());
        let identify = identify::Behaviour::new(identify::Config::new(
            "/qudag/1.0.0".to_string(),
            local_key.public(),
        ));

        let relay = relay::Behaviour::new(local_peer_id, Default::default());
        let dcutr = dcutr::Behaviour::new(local_peer_id);

        // Set up request-response protocol
        let protocols = std::iter::once((
            StreamProtocol::new("/qudag/req/1.0.0"),
            ProtocolSupport::Full,
        ));
        let request_response = request_response::cbor::Behaviour::new(
            protocols,
            request_response::Config::default(),
        );

        // Create the network behaviour
        let behaviour = NetworkBehaviourImpl {
            kademlia,
            gossipsub,
            mdns,
            ping,
            identify,
            relay,
            dcutr,
            request_response,
        };

        // Build the swarm
        let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

        // Set up channels and state
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (router_tx, _) = mpsc::channel(1024);
        let router = Arc::new(RwLock::new(Router::new(router_tx)));

        // Initialize traffic obfuscation
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&config.obfuscation_key));

        // Initialize metrics if enabled
        let metrics = if std::env::var("QUDAG_METRICS").is_ok() {
            Some(()) // TODO: Initialize proper metrics
        } else {
            None
        };

        Ok(Self {
            local_peer_id,
            swarm,
            router,
            cipher,
            event_tx,
            event_rx,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            metrics,
            config,
        })
    }

    /// Starts the network node and begins listening on configured addresses
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        // Listen on all configured addresses
        for addr_str in &self.config.listen_addrs {
            let addr: Multiaddr = addr_str.parse()?;
            self.swarm.listen_on(addr)?;
        }

        // Add bootstrap peers to Kademlia
        for peer_addr_str in &self.config.bootstrap_peers {
            let peer_addr: Multiaddr = peer_addr_str.parse()?;
            if let Some(peer_id) = extract_peer_id(&peer_addr) {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, peer_addr);
            }
        }

        // Bootstrap Kademlia
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            warn!("Kademlia bootstrap failed: {}", e);
        }

        info!("P2P node started");
        Ok(())
    }

    /// Main event loop for the P2P node
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            select! {
                swarm_event = self.swarm.next() => {
                    if let Some(event) = swarm_event {
                        self.handle_swarm_event(event).await?;
                    }
                }
                complete => break,
            }
        }
        Ok(())
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<NetworkBehaviourEvent>,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                endpoint,
                num_established,
                ..
            } => {
                info!(
                    "Connection established with {} at {} ({} total connections)",
                    peer_id,
                    endpoint.get_remote_address(),
                    num_established
                );
                self.connected_peers.write().await.insert(peer_id);
                self.event_tx.send(P2PEvent::PeerConnected(peer_id))?;

                // Update router
                let router = self.router.write().await;
                if let Ok(socket_addr) = endpoint.get_remote_address().to_string().parse() {
                    router.add_discovered_peer(peer_id, crate::discovery::DiscoveredPeer::new(
                        peer_id,
                        socket_addr,
                        crate::discovery::DiscoveryMethod::Kademlia,
                    )).await;
                }
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established,
                ..
            } => {
                info!(
                    "Connection closed with {} ({} remaining connections)",
                    peer_id, num_established
                );
                if num_established == 0 {
                    self.connected_peers.write().await.remove(&peer_id);
                    self.event_tx.send(P2PEvent::PeerDisconnected(peer_id))?;

                    // Update router
                    let router = self.router.write().await;
                    router.remove_discovered_peer(peer_id).await;
                }
            }
            SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(behaviour_event).await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle behaviour events
    async fn handle_behaviour_event(
        &mut self,
        event: NetworkBehaviourEvent,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            NetworkBehaviourEvent::Kademlia(kad_event) => {
                self.handle_kademlia_event(kad_event).await?;
            }
            NetworkBehaviourEvent::Gossipsub(gossipsub_event) => {
                self.handle_gossipsub_event(gossipsub_event).await?;
            }
            NetworkBehaviourEvent::Mdns(mdns_event) => {
                self.handle_mdns_event(mdns_event).await?;
            }
            NetworkBehaviourEvent::Ping(ping_event) => {
                self.handle_ping_event(ping_event).await?;
            }
            NetworkBehaviourEvent::Identify(identify_event) => {
                self.handle_identify_event(identify_event).await?;
            }
            NetworkBehaviourEvent::RequestResponse(req_res_event) => {
                self.handle_request_response_event(req_res_event).await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Kademlia events
    async fn handle_kademlia_event(
        &mut self,
        event: kad::Event,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            kad::Event::RoutingUpdated {
                peer, addresses, ..
            } => {
                debug!("Kademlia routing updated for peer {}", peer);
                for addr in addresses {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer, addr);
                }
                self.event_tx.send(P2PEvent::RoutingTableUpdated)?;
            }
            kad::Event::UnroutablePeer { peer } => {
                warn!("Peer {} is unroutable", peer);
            }
            kad::Event::InboundRequest { request } => {
                debug!("Kademlia inbound request: {:?}", request);
            }
            kad::Event::OutboundQueryProgressed { result, .. } => match result {
                QueryResult::GetClosestPeers(result) => {
                    match result {
                        Ok(ok) => {
                            for peer in ok.peers {
                                debug!("Found closest peer: {}", peer);
                                self.event_tx.send(P2PEvent::PeerDiscovered(peer))?;
                            }
                        }
                        Err(e) => warn!("Get closest peers error: {:?}", e),
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    /// Handle Gossipsub events
    async fn handle_gossipsub_event(
        &mut self,
        event: gossipsub::Event,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            } => {
                let topic = message.topic.to_string();
                let data = message.data;

                // Deobfuscate if needed
                let decrypted_data = match self.deobfuscate_traffic(&data) {
                    Ok(d) => d,
                    Err(_) => data, // Assume not obfuscated
                };

                self.event_tx.send(P2PEvent::MessageReceived {
                    peer_id: propagation_source,
                    topic,
                    data: decrypted_data,
                })?;
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                debug!("Peer {} subscribed to topic {}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                debug!("Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle MDNS events
    async fn handle_mdns_event(&mut self, event: mdns::Event) -> Result<(), Box<dyn Error>> {
        match event {
            mdns::Event::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    debug!("MDNS discovered peer {} at {}", peer_id, addr);
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                    self.event_tx.send(P2PEvent::PeerDiscovered(peer_id))?;
                }
            }
            mdns::Event::Expired(peers) => {
                for (peer_id, _) in peers {
                    debug!("MDNS peer expired: {}", peer_id);
                }
            }
        }
        Ok(())
    }

    /// Handle ping events
    async fn handle_ping_event(&mut self, event: ping::Event) -> Result<(), Box<dyn Error>> {
        match event.result {
            Ok(duration) => {
                debug!("Ping to {} successful: {:?}", event.peer, duration);
            }
            Err(e) => {
                debug!("Ping to {} failed: {}", event.peer, e);
            }
        }
        Ok(())
    }

    /// Handle identify events
    async fn handle_identify_event(
        &mut self,
        event: identify::Event,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            identify::Event::Received { peer_id, info } => {
                debug!(
                    "Identified peer {}: protocols={:?}, agent={}",
                    peer_id,
                    info.protocols,
                    info.agent_version
                );

                // Add observed addresses to Kademlia
                for addr in info.listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                }
            }
            identify::Event::Sent { .. } => {}
            identify::Event::Pushed { .. } => {}
            identify::Event::Error { peer_id, error } => {
                warn!("Identify error with {}: {}", peer_id, error);
            }
        }
        Ok(())
    }

    /// Handle request-response events
    async fn handle_request_response_event(
        &mut self,
        event: request_response::Event<QuDagRequest, QuDagResponse>,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            request_response::Event::Message { peer, message } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let (tx, rx) = oneshot::channel();
                    self.event_tx.send(P2PEvent::RequestReceived {
                        peer_id: peer,
                        request,
                        channel: tx,
                    })?;
                    
                    // Wait for response and send it back
                    tokio::spawn(async move {
                        if let Ok(response) = rx.await {
                            let _ = channel.send(Ok(response));
                        }
                    });
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    if let Some(tx) = self
                        .pending_requests
                        .write()
                        .await
                        .remove(&request_id.to_string())
                    {
                        let _ = tx.send(response);
                    }
                }
            },
            request_response::Event::OutboundFailure {
                peer,
                request_id,
                error,
            } => {
                warn!(
                    "Request to {} failed (id: {}): {:?}",
                    peer, request_id, error
                );
                self.pending_requests
                    .write()
                    .await
                    .remove(&request_id.to_string());
            }
            request_response::Event::InboundFailure {
                peer,
                request_id,
                error,
            } => {
                warn!(
                    "Inbound request from {} failed (id: {}): {:?}",
                    peer, request_id, error
                );
            }
            _ => {}
        }
        Ok(())
    }

    /// Subscribe to a gossipsub topic
    pub async fn subscribe(&mut self, topic: &str) -> Result<(), Box<dyn Error>> {
        let topic = IdentTopic::new(topic);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        info!("Subscribed to topic: {}", topic);
        Ok(())
    }

    /// Unsubscribe from a gossipsub topic
    pub async fn unsubscribe(&mut self, topic: &str) -> Result<(), Box<dyn Error>> {
        let topic = IdentTopic::new(topic);
        self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic)?;
        info!("Unsubscribed from topic: {}", topic);
        Ok(())
    }

    /// Publish a message to a gossipsub topic
    pub async fn publish(
        &mut self,
        topic: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let topic = IdentTopic::new(topic);
        
        // Obfuscate traffic if configured
        let message_data = self.obfuscate_traffic(&data)?;
        
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message_data)?;
        
        debug!("Published message to topic: {}", topic);
        Ok(())
    }

    /// Send a request to a peer
    pub async fn send_request(
        &mut self,
        peer_id: LibP2PPeerId,
        request: QuDagRequest,
    ) -> Result<QuDagResponse, Box<dyn Error>> {
        let request_id = request.request_id.clone();
        let (tx, rx) = oneshot::channel();
        
        self.pending_requests
            .write()
            .await
            .insert(request_id.clone(), tx);
        
        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, request);
        
        // Wait for response with timeout
        match tokio::time::timeout(self.config.timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err("Response channel closed".into()),
            Err(_) => {
                self.pending_requests.write().await.remove(&request_id);
                Err("Request timeout".into())
            }
        }
    }

    /// Get the next network event
    pub async fn next_event(&mut self) -> Option<P2PEvent> {
        self.event_rx.recv().await
    }

    /// Get connected peers
    pub async fn connected_peers(&self) -> Vec<LibP2PPeerId> {
        self.connected_peers.read().await.iter().copied().collect()
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> LibP2PPeerId {
        self.local_peer_id
    }

    /// Get local listening addresses
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    /// Dial a peer
    pub async fn dial(&mut self, peer_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.dial(peer_addr)?;
        Ok(())
    }

    /// Obfuscates traffic using ChaCha20-Poly1305
    fn obfuscate_traffic(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        let mut encrypted = self
            .cipher
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
}

/// Build the transport layer with multiple protocol support
fn build_transport(
    local_key: &Keypair,
    config: &NetworkConfig,
) -> Result<Boxed<(LibP2PPeerId, StreamMuxerBox)>, Box<dyn Error>> {
    let noise_keys = noise::Config::new(local_key)?
        .into_authenticated();

    let yamux_config = yamux::Config::default();

    // Build base TCP transport
    let tcp = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));

    // Memory transport for testing
    let memory = MemoryTransport::default();

    // Combine transports
    let transport = tcp.or_transport(memory);

    // Add WebSocket support if enabled
    let transport = if config.enable_websocket {
        let ws = websocket::WsConfig::new(tcp::tokio::Transport::new(
            tcp::Config::default().nodelay(true),
        ));
        transport.or_transport(ws)
    } else {
        transport
    };

    // Apply multiplexing and encryption
    let transport = transport
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_keys)
        .multiplex(yamux_config)
        .timeout(Duration::from_secs(20))
        .boxed();

    Ok(transport)
}

/// Extract peer ID from multiaddr if present
fn extract_peer_id(addr: &Multiaddr) -> Option<LibP2PPeerId> {
    addr.iter().find_map(|p| match p {
        Protocol::P2p(peer_id) => Some(peer_id),
        _ => None,
    })
}

/// Type alias for stream muxer
type StreamMuxerBox = libp2p::core::muxing::StreamMuxerBox;

/// Type aliases for missing libp2p types in 0.53
type TransactionId = [u8; 12];
type Message = Vec<u8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config).await.unwrap();
        assert!(!node.local_peer_id().to_string().is_empty());
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
    async fn test_node_start() {
        let mut config = NetworkConfig::default();
        config.listen_addrs = vec!["/ip4/127.0.0.1/tcp/0".to_string()];
        config.enable_mdns = false; // Disable MDNS for tests
        
        let mut node = P2PNode::new(config).await.unwrap();
        node.start().await.unwrap();
        
        // Give it a moment to bind
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let listeners = node.listeners();
        assert!(!listeners.is_empty());
    }

    #[tokio::test]
    async fn test_pubsub() {
        let config = NetworkConfig::default();
        let mut node = P2PNode::new(config).await.unwrap();
        
        let topic = "test-topic";
        node.subscribe(topic).await.unwrap();
        
        let test_data = vec![1, 2, 3, 4, 5];
        node.publish(topic, test_data).await.unwrap();
    }
}