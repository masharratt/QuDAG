# QuDAG Protocol 🌐

> The Darkest of Darknets - Built for the Quantum Age

QuDAG is the next evolution in anonymous communication, engineered specifically for the quantum internet era. By combining post-quantum cryptography with advanced DAG consensus, it creates a new foundation for private messaging infrastructure.

**Key Highlights:**
- 🔒 Post-quantum cryptography using ML-KEM-768 & ML-DSA with BLAKE3
- ⚡ High-performance asynchronous DAG with QR-Avalanche consensus
- 🕵️ Anonymous onion routing with ChaCha20Poly1305 traffic obfuscation
- 🛡️ Memory-safe Rust implementation with zero unsafe code
- 🔗 LibP2P-based networking with Kademlia DHT peer discovery
- 📊 Real-time performance metrics and benchmarking

## Use Cases

### 🔐 Secure Communication
- End-to-end encrypted messaging
- Secure file transfer
- Private group communication
- Data streaming

### 🌐 Network Infrastructure
- P2P message routing
- Distributed content storage
- Secure relay networks
- Anonymous networking
- Dark addressing with quantum fingerprints

### 🛡️ Privacy Applications
- Anonymous messaging
- Private data transfer
- Secure group coordination
- Metadata protection

## Core Features

### 🔐 Quantum-Resistant Cryptography
- **ML-KEM-768**: NIST Level 3 key encapsulation mechanism
- **ML-DSA**: Post-quantum digital signatures with constant-time operations
- **BLAKE3**: Quantum-resistant cryptographic hashing
- **Quantum Fingerprinting**: Data authentication using ML-DSA
- **Memory Security**: Automatic zeroization with `ZeroizeOnDrop`
- **Side-Channel Resistance**: Constant-time implementations

### 📊 DAG Architecture
- **Asynchronous Processing**: Non-blocking message handling
- **QR-Avalanche Consensus**: Byzantine fault-tolerant quantum-resistant consensus
- **Conflict Resolution**: Automatic detection and resolution
- **Tip Selection**: Optimal parent selection algorithm
- **Performance Metrics**: Real-time throughput and latency monitoring
- **State Management**: Atomic state transitions

### 🌐 Network Layer
- **P2P Networking**: LibP2P-based node implementation
- **Anonymous Routing**: Onion routing with multiple hops
- **Traffic Obfuscation**: ChaCha20Poly1305-based disguising
- **Peer Discovery**: Kademlia DHT for decentralized peer management
- **Transport Security**: Quantum-resistant TLS with ML-KEM
- **Connection Management**: Secure handshakes and session management

### 🌐 Dark Addressing
- Quantum-resistant `.dark` domains (eg: mysite.dark)
- Stealth `.shadow` addresses for enhanced privacy
- Human-readable aliases with quantum fingerprints
- Decentralized address resolution

## How It Works

### Network Architecture
```
Peer A ←→ [Multiple Encrypted Paths] ←→ Peer B
   ↑         ↑         ↑        ↑         ↑
   └─ ML-KEM-768 Encrypted Segments ─────┘
```

### Core Components
- Quantum-resistant keypair generation
- Message encryption and signing
- DAG-based message ordering
- P2P network communication

### Messaging Flow
1. Message split into shards
2. Each shard encrypted with ML-KEM-768
3. Shards routed through different paths
4. Reassembly at destination

## Quick Start

```bash
# Clone & Build
git clone https://github.com/ruvnet/QuDAG
cd QuDAG
cargo build --release

# Run Tests
cargo test --all-features --workspace

# Start Node
./target/release/qudag start --port 8000
```

## CLI Usage

```bash
# Node Management
qudag start --port 8000
qudag stop
qudag status

# Peer Operations
qudag peer list
qudag peer add <address>
qudag peer remove <address>

# Network Management
qudag network stats
qudag network test

# Dark Addressing
qudag address register mysite.dark
qudag address resolve mysite.dark
qudag shadow generate --ttl 3600
qudag fingerprint create --data "content"
```

## Architecture

```
core/
├── crypto/    # Post-quantum cryptography
│   ├── ml_kem.rs      # ML-KEM-768 encryption
│   ├── ml_dsa.rs      # ML-DSA-65 signatures
│   └── fingerprint.rs # Quantum fingerprints
├── dag/       # Consensus implementation
├── network/   # P2P networking with dark addressing
│   ├── dark_resolver.rs  # .dark domain resolution
│   ├── shadow_address.rs # .shadow stealth addresses
│   └── dns.rs           # ruv.io DNS integration
└── protocol/  # Core protocol logic

tools/
├── cli/       # Command-line interface
└── simulator/ # Network simulation
```

## Development

```bash
# Run Tests
cargo test

# Run Specific Tests
cargo test -p qudag-crypto
cargo test -p qudag-network

# Run Benchmarks
cargo bench
```

## Benchmarks

### Current Performance
```
Cryptographic Operations (ms)
├── Key Generation:     ~2
├── Encryption:         ~1
├── Decryption:         ~1
├── Fingerprint Gen:    ~0.235
└── Signature Verify:   ~0.187

Network Operations (ms)
├── Peer Discovery:     ~500
├── Path Setup:         ~200
├── Message Relay:      ~50
├── Dark Domain Reg:    ~0.045
├── Domain Resolution:  ~0.128
└── Shadow Address:     ~0.079

Memory Usage (MB)
├── Base:              ~50
├── Active:            ~100
└── Peak:              ~200
```

## Security Features

- Post-quantum cryptographic primitives
- Constant-time operations
- Memory zeroization
- Side-channel protections
- Secure key management

## Resources

- [Documentation](https://docs.qudag.io)
- [Research Papers](https://github.com/ruvnet/QuDAG/tree/main/research)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

## License

Licensed under either:
- Apache License 2.0
- MIT License

---

Created by [rUv](https://github.com/ruvnet)

[GitHub](https://github.com/ruvnet/QuDAG) • [Documentation](https://docs.qudag.io) • [Research](https://github.com/ruvnet/QuDAG/tree/main/research)