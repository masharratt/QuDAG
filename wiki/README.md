# QuDAG Repository Wiki

Welcome to the comprehensive wiki for the QuDAG (Quantum-Resistant Distributed Anonymous Graph) project. This wiki serves as a central hub for understanding, developing, and contributing to the QuDAG ecosystem.

## 🌟 What is QuDAG?

QuDAG is a revolutionary **quantum-resistant distributed communication platform** designed for the quantum age. It enables **autonomous AI agent coordination**, **zero-person businesses**, and **secure decentralized communication** through a combination of:

- **Post-quantum cryptography** (ML-KEM-768, ML-DSA, HQC, BLAKE3)
- **DAG-based consensus** using QR-Avalanche algorithm
- **Anonymous onion routing** with traffic obfuscation
- **Dark addressing system** (`.dark` domains)
- **AI agent swarm coordination** via MCP integration
- **Resource exchange** using rUv tokens

## 📚 Wiki Navigation

### 🏗️ Architecture & Design
- [**System Architecture**](architecture/system-architecture.md) - Core system design and component interactions
- [**Protocol Specifications**](architecture/protocol-specs.md) - Detailed protocol documentation
- [**Consensus Algorithm**](architecture/consensus.md) - QR-Avalanche DAG consensus deep dive
- [**Security Model**](architecture/security-model.md) - Comprehensive security analysis

### 🔧 Core Components
- [**Cryptographic Primitives**](components/crypto.md) - Post-quantum cryptography implementation
- [**DAG Consensus**](components/dag.md) - Distributed consensus on directed acyclic graph
- [**P2P Networking**](components/network.md) - LibP2P-based networking with anonymous routing
- [**Protocol Coordination**](components/protocol.md) - Main protocol coordinator and state management
- [**Vault System**](components/vault.md) - Quantum-resistant password/secret management

### 🤖 AI & Automation
- [**MCP Integration**](ai/mcp-integration.md) - Model Context Protocol server implementation
- [**Agent Coordination**](ai/agent-coordination.md) - AI agent swarm management
- [**Resource Exchange**](ai/resource-exchange.md) - rUv token system for computational trading
- [**Zero-Person Businesses**](ai/autonomous-organizations.md) - Fully autonomous business operations

### 🌐 Network Features
- [**Dark Addressing**](network/dark-addressing.md) - Decentralized `.dark` domain system
- [**Anonymous Routing**](network/anonymous-routing.md) - Multi-hop onion routing implementation
- [**Peer Discovery**](network/peer-discovery.md) - Kademlia DHT-based peer discovery
- [**NAT Traversal**](network/nat-traversal.md) - STUN/TURN/UPnP hole punching

### 🛠️ Development
- [**Development Setup**](development/setup.md) - Getting started with QuDAG development
- [**Build System**](development/build-system.md) - Cargo workspace and build configuration
- [**Testing Strategy**](development/testing.md) - Unit, integration, and security testing
- [**Benchmarking**](development/benchmarking.md) - Performance testing and optimization

### 📖 User Guides
- [**Installation Guide**](guides/installation.md) - Installing QuDAG CLI and libraries
- [**Quick Start**](guides/quick-start.md) - Getting up and running quickly
- [**CLI Reference**](guides/cli-reference.md) - Complete command-line interface documentation
- [**API Documentation**](guides/api-docs.md) - JSON-RPC and Rust API reference

### 🚀 Deployment & Operations
- [**Testnet**](deployment/testnet.md) - Live testnet nodes and configuration
- [**Docker Deployment**](deployment/docker.md) - Containerized deployment options
- [**Production Deployment**](deployment/production.md) - Production-ready deployment guide
- [**Monitoring & Metrics**](deployment/monitoring.md) - Performance monitoring setup

### 🔐 Security
- [**Cryptographic Standards**](security/crypto-standards.md) - NIST post-quantum cryptography compliance
- [**Security Audits**](security/audits.md) - Security assessment and audit results
- [**Best Practices**](security/best-practices.md) - Security guidelines for developers and operators
- [**Threat Model**](security/threat-model.md) - Security assumptions and threat analysis

### 🤝 Contributing
- [**Contributing Guide**](contributing/guide.md) - How to contribute to QuDAG
- [**Code Style**](contributing/code-style.md) - Coding conventions and standards
- [**Issue Reporting**](contributing/issues.md) - Bug reports and feature requests
- [**Pull Request Process**](contributing/pull-requests.md) - Code review and merge process

### 📊 Performance & Benchmarks
- [**Performance Report**](performance/benchmarks.md) - Comprehensive performance analysis
- [**Optimization Guide**](performance/optimization.md) - Performance tuning recommendations
- [**Scalability Analysis**](performance/scalability.md) - Horizontal and vertical scaling characteristics

### 🔬 Research & Papers
- [**Research Overview**](research/overview.md) - Academic research and publications
- [**Technical Papers**](research/papers.md) - Whitepapers and technical documentation
- [**Protocol Evolution**](research/evolution.md) - Protocol development and future roadmap

## 🚦 Project Status

| Component | Status | Version | Notes |
|-----------|--------|---------|-------|
| **Cryptographic Core** | ✅ Production Ready | v0.4.3 | NIST-compliant post-quantum crypto |
| **P2P Networking** | ✅ Production Ready | v0.4.3 | LibP2P with anonymous routing |
| **DAG Consensus** | ✅ Production Ready | v0.4.3 | QR-Avalanche implementation |
| **Dark Addressing** | ✅ Production Ready | v0.4.3 | `.dark` domain resolution |
| **CLI Interface** | ✅ Production Ready | v0.4.3 | Full command suite |
| **MCP Server** | ✅ Production Ready | v0.4.3 | AI tool integration |
| **Exchange System** | ✅ Production Ready | v0.4.3 | rUv token trading |
| **Node Integration** | 🔄 In Progress | v0.5.0 | Final component integration |
| **State Persistence** | 🚧 In Development | v0.6.0 | Storage layer implementation |

## 🔗 Quick Links

### Essential Resources
- [**Main README**](../README.md) - Project overview and quick start
- [**Installation Instructions**](../INSTALL.md) - Detailed installation guide
- [**Architecture Documentation**](../docs/architecture/README.md) - System architecture details
- [**API Documentation**](https://docs.rs/qudag) - Rust API reference
- [**Live Testnet**](https://qudag-testnet-node1.fly.dev/health) - Global testnet status

### Development Resources
- [**Cargo Workspace**](../Cargo.toml) - Workspace configuration
- [**Core Modules**](../core/) - Core implementation modules
- [**CLI Tools**](../tools/cli/) - Command-line interface
- [**Benchmarks**](../benchmarks/) - Performance testing suite
- [**Examples**](../examples/) - Usage examples and demos

### Community & Support
- [**GitHub Repository**](https://github.com/ruvnet/QuDAG) - Source code and issues
- [**Contributing Guidelines**](../CONTRIBUTING.md) - How to contribute
- [**Security Policy**](../SECURITY.md) - Security reporting process
- [**License**](../LICENSE) - MIT/Apache-2.0 dual license

## 📝 How to Use This Wiki

1. **New to QuDAG?** Start with the [Quick Start Guide](guides/quick-start.md)
2. **Want to contribute?** Check the [Contributing Guide](contributing/guide.md)
3. **Need technical details?** Explore the [Architecture section](architecture/system-architecture.md)
4. **Looking for examples?** Visit the [User Guides](guides/installation.md)
5. **Performance questions?** See the [Performance section](performance/benchmarks.md)

## 🔄 Wiki Updates

This wiki is actively maintained and updated alongside the QuDAG codebase. Last updated: September 2024

For questions, suggestions, or corrections, please:
- Open an issue on [GitHub](https://github.com/ruvnet/QuDAG/issues)
- Start a discussion in [GitHub Discussions](https://github.com/ruvnet/QuDAG/discussions)
- Submit a pull request with improvements

---

**QuDAG**: The future of quantum-resistant autonomous communication and AI agent coordination.