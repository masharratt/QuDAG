# Protocol Module API

The `qudag_protocol` module serves as the main coordinator for the QuDAG protocol, integrating the cryptographic, DAG consensus, and networking components.

## Core Types

### Coordinator

The main protocol coordinator.

```rust
pub struct Coordinator {
    // private fields
}

impl Coordinator {
    pub async fn new(config: ProtocolConfig) -> Result<Self>;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    pub async fn broadcast_message(&self, message: Vec<u8>) -> Result<()>;
    pub async fn state(&self) -> ProtocolState;
    pub fn is_initialized(&self) -> bool;
}
```

### ProtocolConfig

Configuration parameters for the protocol.

```rust
pub struct ProtocolConfig {
    pub network_port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
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
```

### ProtocolState

Protocol state machine states.

```rust
pub enum ProtocolState {
    Initialized,
    Running,
    Stopped,
    Error,
}
```

## Getting Started

### Basic Setup

```rust
use qudag_protocol::{Coordinator, ProtocolConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Create default configuration
    let config = ProtocolConfig::default();
    
    // Initialize protocol coordinator
    let mut coordinator = Coordinator::new(config).await?;
    
    // Start the protocol
    coordinator.start().await?;
    
    // Broadcast a message
    coordinator.broadcast_message(b"Hello QuDAG!".to_vec()).await?;
    
    // Stop the protocol
    coordinator.stop().await?;
    
    Ok(())
}
```

### Custom Configuration

```rust
use qudag_protocol::ProtocolConfig;

let config = ProtocolConfig {
    network_port: 9000,
    bootstrap_nodes: vec![
        "node1.example.com:9000".to_string(),
        "node2.example.com:9000".to_string(),
    ],
    max_peers: 100,
    validation_timeout: 10000,
};
```

## Error Handling

The protocol uses the `anyhow` crate for error handling. Here's how to handle common errors:

```rust
use qudag_protocol::{Coordinator, ProtocolConfig};
use anyhow::{Result, Context};

async fn run_protocol() -> Result<()> {
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config)
        .await
        .context("Failed to initialize protocol")?;
        
    // Start protocol with error context
    coordinator.start()
        .await
        .context("Failed to start protocol")?;
        
    // Handle message broadcast errors
    match coordinator.broadcast_message(vec![1, 2, 3]).await {
        Ok(()) => println!("Message broadcast successful"),
        Err(e) => {
            eprintln!("Broadcast failed: {}", e);
            coordinator.stop().await?;
            return Err(e);
        }
    }
    
    Ok(())
}
```

## Security Guidelines

1. **Message Validation**
   - All messages are cryptographically signed
   - Signatures are verified before processing
   - Message validation has configurable timeout
   - Invalid messages are rejected

2. **State Management**
   - Protocol state transitions are atomic
   - State is monitored for consistency
   - Error states trigger automatic cleanup

3. **Resource Management**
   - Memory is properly cleaned after use
   - Network connections are properly closed
   - Resources are released on shutdown

## Configuration

### Network Settings

- `network_port`: Port for P2P communication (default: 8000)
- `max_peers`: Maximum number of peer connections (default: 50)
- `bootstrap_nodes`: Initial nodes for network discovery

### Timing Parameters

- `validation_timeout`: Message validation timeout in ms (default: 5000)

### Recommended Values

```rust
// High security configuration
let secure_config = ProtocolConfig {
    max_peers: 30,                    // Smaller peer set for better control
    validation_timeout: 10000,        // Longer validation time
    ..Default::default()
};

// High performance configuration
let performance_config = ProtocolConfig {
    max_peers: 100,                   // Larger peer set for better connectivity
    validation_timeout: 3000,         // Shorter validation time
    ..Default::default()
};
```

## Component Integration

### Cryptographic Operations

```rust
// Inside message validation
async fn validate_message(message: &[u8], keypair: &KeyPair) -> Result<()> {
    // Timeout for validation
    let validation_future = async {
        // Verify message signature
        keypair.verify(message)
            .context("Failed to verify message signature")?;
        
        // Additional validation...
        Ok(())
    };
    
    match timeout(Duration::from_millis(5000), validation_future).await {
        Ok(result) => result,
        Err(_) => Err(anyhow!("Message validation timed out")),
    }
}
```

### Network Communication

```rust
// Message broadcasting
async fn broadcast_message(&self, message: Vec<u8>) -> Result<()> {
    // Sign message
    let signature = self.crypto_manager.as_ref()
        .context("Crypto manager not initialized")?
        .sign(&message)?;
        
    // Add to DAG
    self.dag_manager.as_ref()
        .context("DAG manager not initialized")?
        .add_message(&message, &signature)?;
        
    // Broadcast through network
    self.network_manager.as_ref()
        .context("Network manager not initialized")?
        .broadcast(message)
        .await?;
        
    Ok(())
}
```

## Best Practices

1. **Initialization**
   - Always check `is_initialized()` before operations
   - Use custom configuration for production
   - Initialize components in correct order

2. **State Management**
   - Monitor protocol state transitions
   - Handle state changes atomically
   - Clean up resources in all exit paths

3. **Error Handling**
   - Use context for error messages
   - Implement proper cleanup on errors
   - Log errors appropriately

4. **Resource Management**
   - Close connections properly
   - Clean up temporary resources
   - Monitor resource usage