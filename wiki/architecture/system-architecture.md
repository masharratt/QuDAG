# QuDAG System Architecture

## Overview

QuDAG implements a modular, quantum-resistant distributed communication protocol designed for high-performance, anonymous communication and autonomous AI agent coordination. The system is built using a Rust workspace architecture with clear separation of concerns.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    QuDAG Ecosystem                      │
├─────────────────────────────────────────────────────────┤
│  Applications & Interfaces                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │  CLI Tools  │ │  MCP Server │ │   WASM/JS   │      │
│  │             │ │             │ │   Bindings  │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│  Protocol Coordination Layer                            │
│  ┌─────────────────────────────────────────────────────┐ │
│  │           Protocol Coordinator                      │ │
│  │  ├── Message Validation  ├── State Management      │ │
│  │  ├── Component Bridge    ├── Error Handling        │ │
│  │  └── Metrics Collection  └── Resource Management   │ │
│  └─────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Core Modules                                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │   Crypto    │ │     DAG     │ │   Network   │      │
│  │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │      │
│  │ │ ML-KEM  │ │ │ │QR-Aval. │ │ │ │ Onion   │ │      │
│  │ │ ML-DSA  │ │ │ │ Vertex  │ │ │ │ Routing │ │      │
│  │ │ BLAKE3  │ │ │ │ Tips    │ │ │ │ LibP2P  │ │      │
│  │ │ HQC     │ │ │ │ State   │ │ │ │ Dark    │ │      │
│  │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
│  ┌─────────────┐ ┌─────────────┐                      │
│  │   Vault     │ │  Exchange   │                      │
│  │ ┌─────────┐ │ │ ┌─────────┐ │                      │
│  │ │ AES-GCM │ │ │ │   rUv   │ │                      │
│  │ │ Argon2  │ │ │ │  Fees   │ │                      │
│  │ │ Backup  │ │ │ │ Agents  │ │                      │
│  │ └─────────┘ │ │ └─────────┘ │                      │
│  └─────────────┘ └─────────────┘                      │
├─────────────────────────────────────────────────────────┤
│  Runtime & Infrastructure                               │
│  ├── Tokio Async Runtime    ├── Tracing Logging        │
│  ├── Serde Serialization   ├── Metrics Collection      │
│  ├── Memory Management      ├── Error Propagation      │
│  └── Zero Unsafe Code      └── Constant-Time Crypto    │
└─────────────────────────────────────────────────────────┘
```

## Core Module Architecture

### 1. Cryptographic Module (`core/crypto`)

**Purpose**: Quantum-resistant cryptographic primitives and operations

**Key Components**:
- **ML-KEM-768**: NIST-standardized key encapsulation mechanism
- **ML-DSA**: Digital signatures with constant-time operations  
- **HQC**: Code-based encryption (128/192/256-bit security levels)
- **BLAKE3**: Fast, quantum-resistant hashing
- **Quantum Fingerprinting**: Content authentication using ML-DSA

**Architecture**:
```rust
// Core traits and interfaces
pub trait QuantumResistantEncryption {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

pub trait PostQuantumSignature {
    fn sign(&self, message: &[u8]) -> Result<Signature>;
    fn verify(&self, signature: &Signature, message: &[u8]) -> bool;
}
```

**Security Properties**:
- Constant-time implementations to prevent side-channel attacks
- Automatic memory zeroization for cryptographic material
- NIST compliance for post-quantum security
- Hardware acceleration where available (AVX2, NEON)

### 2. DAG Consensus Module (`core/dag`)

**Purpose**: Distributed consensus using quantum-resistant Avalanche protocol on DAG structure

**Key Components**:
- **QR-Avalanche Consensus**: Byzantine fault-tolerant consensus adapted for quantum resistance
- **Vertex Management**: DAG vertex creation, validation, and storage
- **Tip Selection**: Optimal parent selection algorithm for DAG growth
- **State Synchronization**: Consensus state management and synchronization

**Architecture**:
```rust
pub struct Dag {
    vertices: HashMap<VertexId, Vertex>,
    tips: HashSet<VertexId>,
    consensus: QRAvalanche,
    state: DagState,
}

pub struct Vertex {
    id: VertexId,
    parents: Vec<VertexId>,
    payload: Vec<u8>,
    timestamp: u64,
    signature: MlDsaSignature,
}
```

**Consensus Properties**:
- **Safety**: Byzantine fault tolerance (< 1/3 adversarial nodes)
- **Liveness**: Progress guarantee under network conditions
- **Finality**: Probabilistic finality with high confidence (>99.9%)
- **Performance**: 10,000+ vertices/second theoretical throughput

### 3. Network Module (`core/network`)

**Purpose**: Anonymous P2P networking with quantum-resistant transport security

**Key Components**:
- **Dark Resolver**: Decentralized `.dark` domain system
- **Onion Router**: Multi-hop anonymous routing with ML-KEM encryption
- **Connection Manager**: Secure P2P connection handling
- **Peer Discovery**: Kademlia DHT-based peer discovery
- **NAT Traversal**: STUN/TURN/UPnP hole punching

**Architecture**:
```rust
pub struct NetworkManager {
    swarm: Swarm<QuDAGBehaviour>,
    dark_resolver: DarkResolver,
    onion_router: OnionRouter,
    connection_pool: ConnectionPool,
}

pub struct OnionCircuit {
    hops: Vec<PeerId>,
    encryption_layers: Vec<MlKemEncryption>,
    circuit_id: CircuitId,
}
```

**Network Properties**:
- **Anonymity**: Multi-hop routing prevents traffic analysis
- **Security**: ML-KEM transport encryption with forward secrecy
- **Performance**: Adaptive routing based on latency and bandwidth
- **Scalability**: O(log n) routing with Kademlia DHT

### 4. Protocol Coordination (`core/protocol`)

**Purpose**: Main protocol coordinator managing component interactions

**Key Components**:
- **Node Manager**: Node lifecycle and configuration management
- **Message Validator**: Cross-module message validation
- **State Coordinator**: Distributed state management
- **Performance Monitor**: Real-time metrics collection

**Architecture**:
```rust
pub struct ProtocolCoordinator {
    crypto: Arc<CryptoManager>,
    dag: Arc<DagConsensus>,
    network: Arc<NetworkManager>,
    state: Arc<ProtocolState>,
}
```

## Data Flow Architecture

### Message Processing Pipeline

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Create    │───▶│   Crypto    │───▶│    DAG      │───▶│   Network   │
│   Message   │    │   Sign      │    │   Vertex    │    │   Route     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Validate   │    │  ML-DSA     │    │  Parent     │    │  Onion      │
│  Content    │    │  Signature  │    │  Selection  │    │  Encryption │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### Consensus Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Receive   │───▶│   Validate  │───▶│  Consensus  │───▶│  Finalize   │
│   Vertex    │    │   Vertex    │    │   Query     │    │   State     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Signature  │    │   Parents   │    │  Network    │    │   Update    │
│  Check      │    │   Check     │    │   Vote      │    │   DAG       │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

## Integration Patterns

### Inter-Module Communication

The system uses async message passing and shared state for component coordination:

```rust
// Protocol coordinator manages component interactions
impl ProtocolCoordinator {
    pub async fn process_message(&self, message: Message) -> Result<()> {
        // 1. Cryptographic validation
        let verified = self.crypto.verify_message(&message).await?;
        
        // 2. DAG integration
        let vertex = self.dag.create_vertex(verified).await?;
        
        // 3. Network propagation
        self.network.broadcast_vertex(vertex).await?;
        
        Ok(())
    }
}
```

### Event-Driven Architecture

Components communicate through an event system for loose coupling:

```rust
pub enum ProtocolEvent {
    MessageReceived(Message),
    VertexFinalized(VertexId),
    PeerConnected(PeerId),
    ConsensusAchieved(StateHash),
}

pub trait EventHandler {
    async fn handle_event(&self, event: ProtocolEvent) -> Result<()>;
}
```

## Performance Architecture

### Async/Await Concurrency

- **Tokio Runtime**: Efficient async I/O and task scheduling
- **Parallel Processing**: Concurrent vertex validation and consensus
- **Non-blocking Operations**: All I/O operations are non-blocking
- **Resource Pooling**: Connection and memory pool management

### Memory Management

- **Zero-Copy Serialization**: Efficient data handling with `serde`
- **Memory Pools**: Pre-allocated buffers for crypto operations
- **Automatic Cleanup**: Cryptographic material zeroization
- **Memory Limits**: Configurable memory usage bounds

### Optimization Strategies

1. **SIMD Acceleration**: Hardware-accelerated cryptographic operations
2. **Batch Processing**: Group operations for efficiency
3. **Adaptive Algorithms**: Dynamic optimization based on network conditions
4. **Cache-Friendly**: Data structures optimized for CPU cache usage

## Security Architecture

### Defense in Depth

1. **Memory Safety**: Rust's ownership system prevents memory vulnerabilities
2. **Cryptographic Security**: Post-quantum algorithms for future-proofing
3. **Network Security**: Anonymous routing and traffic obfuscation
4. **Protocol Security**: Byzantine fault tolerance and replay protection

### Threat Model

**Protected Against**:
- Quantum computer attacks on cryptographic primitives
- Traffic analysis and correlation attacks
- Byzantine node behavior (< 1/3 adversarial)
- Memory safety vulnerabilities
- Side-channel attacks on cryptographic operations

**Assumptions**:
- Majority of network nodes are honest
- Cryptographic primitives remain secure
- System clock synchronization within reasonable bounds
- Network eventually delivers messages

## Extensibility Architecture

### Plugin System

The modular design allows for component replacement and extension:

```rust
pub trait CryptoProvider {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// Multiple implementations can be plugged in
impl CryptoProvider for MlKemProvider { /* ... */ }
impl CryptoProvider for HqcProvider { /* ... */ }
```

### Configuration System

Runtime configuration allows deployment flexibility:

```toml
[crypto]
algorithm = "ML-KEM-768"
constant_time = true

[network]
max_peers = 50
circuit_length = 5

[consensus]
finality_threshold = 0.99
timeout_ms = 1000
```

This architecture provides a solid foundation for quantum-resistant distributed communication while maintaining high performance, security, and extensibility.