# Network Module API

The `qudag_network` module provides networking capabilities with anonymous routing for the QuDAG protocol.

## Core Types

### MessageHandler

Manages message sending and receiving with high-throughput queues.

```rust
pub struct MessageHandler {
    // private fields
}

impl MessageHandler {
    pub fn new() -> Self;
    pub async fn send(&self, msg: Message) -> Result<(), NetworkError>;
    pub async fn receive(&self) -> Result<Message, NetworkError>;
    pub fn get_stats(&self) -> Arc<RwLock<QueueStats>>;
}
```

### Message

Network message with routing information.

```rust
pub struct Message {
    // private fields
}

impl Message {
    pub fn new(content: Vec<u8>, destination: PeerId, route: Route) -> Self;
    pub fn encrypt(self) -> Self;
    pub fn decrypt(self) -> Result<Self, NetworkError>;
    pub fn content(&self) -> &[u8];
    pub fn route(&self) -> &Route;
    pub fn is_encrypted(&self) -> bool;
}
```

### Route

Defines message routing path and anonymity settings.

```rust
pub struct Route {
    // private fields
}

impl Route {
    pub fn new() -> Self;
    pub fn direct() -> Self;
    pub fn add_hop(mut self, peer: PeerId) -> Self;
    pub fn next_hop(&self) -> Option<&PeerId>;
    pub fn is_anonymous(&self) -> bool;
    pub fn reveals_sender(&self) -> bool;
}
```

### PeerId

Unique identifier for network nodes.

```rust
pub struct PeerId(Vec<u8>);

impl PeerId {
    pub fn random() -> Self;
}
```

## Core Traits

### NetworkNode

Base trait for network nodes.

```rust
pub trait NetworkNode: Send + Sync + 'static {
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn stop(&self) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn metrics(&self) -> Pin<Box<dyn Future<Output = NetworkMetrics> + Send>>;
    fn status(&self) -> ConnectionStatus;
}
```

### PeerDiscovery

Handles peer discovery and management.

```rust
pub trait PeerDiscovery: Send + Sync + 'static {
    fn add_peer(&self, peer: PeerId) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn remove_peer(&self, peer: &PeerId) -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
    fn get_peers(&self) -> Pin<Box<dyn Future<Output = Vec<PeerId>> + Send>>;
    fn find_peers(&self, service: &str) -> Pin<Box<dyn Future<Output = Vec<PeerId>> + Send>>;
}
```

### AnonymousRouting

Provides anonymous routing capabilities.

```rust
pub trait AnonymousRouting: Send + Sync + 'static {
    fn create_route(&self, destination: PeerId, hops: usize) 
        -> Pin<Box<dyn Future<Output = Result<Route, NetworkError>> + Send>>;
    fn next_hop(&self, route: &Route) -> Option<PeerId>;
    fn validate_route(&self, route: &Route) -> bool;
    fn update_routing_table(&self, routes: Vec<Route>) 
        -> Pin<Box<dyn Future<Output = Result<(), NetworkError>> + Send>>;
}
```

## Error Types

### NetworkError

```rust
pub enum NetworkError {
    InvalidRoute,
    MessageTooLarge,
    EncryptionError(String),
    ConnectionError(String),
    Internal(String),
}
```

## Example Usage

### Basic Message Handling

```rust
use qudag_network::{MessageHandler, Message, PeerId, Route};

#[tokio::main]
async fn main() -> Result<(), NetworkError> {
    // Create message handler
    let handler = MessageHandler::new();
    
    // Create and send a message
    let dest = PeerId::random();
    let route = Route::new().add_hop(PeerId::random());
    let msg = Message::new(b"Hello".to_vec(), dest, route).encrypt();
    
    handler.send(msg).await?;
    
    // Receive and process messages
    if let Ok(msg) = handler.receive().await {
        if msg.is_encrypted() {
            let decrypted = msg.decrypt()?;
            println!("Received: {:?}", decrypted.content());
        }
    }
    
    Ok(())
}
```

### Anonymous Routing

```rust
use qudag_network::{AnonymousRouting, PeerId, Route};

async fn setup_anonymous_route(
    router: &impl AnonymousRouting,
    dest: PeerId
) -> Result<Route, NetworkError> {
    // Create route with 3 hops for anonymity
    let route = router.create_route(dest, 3).await?;
    
    // Validate the route
    if !router.validate_route(&route) {
        return Err(NetworkError::InvalidRoute);
    }
    
    Ok(route)
}
```

### Peer Discovery

```rust
use qudag_network::{PeerDiscovery, PeerId};

async fn discover_service_peers(
    discovery: &impl PeerDiscovery,
    service: &str
) -> Vec<PeerId> {
    // Find peers providing specific service
    let peers = discovery.find_peers(service).await;
    
    // Add new peers to network
    for peer in &peers {
        if let Err(e) = discovery.add_peer(peer.clone()).await {
            eprintln!("Failed to add peer: {}", e);
        }
    }
    
    peers
}
```

## Best Practices

1. **Message Handling**
   - Always encrypt sensitive messages
   - Handle message size limits
   - Implement proper error handling
   - Monitor queue performance

2. **Anonymous Routing**
   - Use multiple hops for better anonymity
   - Validate routes before use
   - Implement route redundancy
   - Regular routing table updates

3. **Peer Management**
   - Regular peer discovery
   - Proper peer validation
   - Maintain optimal peer count
   - Handle peer disconnections

## Security Considerations

1. **Message Privacy**
   - All messages should be encrypted
   - Use anonymous routes for sensitive data
   - Clear message content after processing
   - Avoid logging sensitive data

2. **Network Security**
   - Validate peer connections
   - Monitor for malicious behavior
   - Implement rate limiting
   - Regular security audits

3. **Anonymity Protection**
   - Use sufficient routing hops
   - Avoid sender/receiver correlation
   - Implement mixing strategies
   - Regular anonymity analysis