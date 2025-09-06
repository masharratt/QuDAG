# QuDAG Quick Start Guide

Get up and running with QuDAG in minutes. This guide will walk you through installation, basic setup, and your first quantum-resistant communication.

## 🚀 Installation

### Option 1: Install from Crates.io (Recommended)

```bash
# Install the CLI tool
cargo install qudag-cli

# Verify installation
qudag --help

# Check version
qudag --version
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/ruvnet/QuDAG.git
cd QuDAG

# Build the workspace
cargo build --release

# Install CLI from local build
cargo install --path tools/cli

# Verify installation
qudag --help
```

### Option 3: Use with NPM/Node.js

```bash
# Use directly with npx
npx qudag@latest --help

# Or install globally
npm install -g qudag

# Or for programmatic use
npm install qudag
```

## 🎯 First Steps

### 1. Start Your First Node

```bash
# Start a QuDAG node on default port (8000)
qudag start --port 8000

# Start with custom configuration
qudag start --port 8001 --peers 10 --verbose
```

You should see output like:
```
🌐 QuDAG Node Starting...
✅ Cryptographic keys generated
✅ P2P network initialized  
✅ DAG consensus started
✅ Node running on port 8000
📊 Node ID: 12D3KooWABC123...
```

### 2. Create Your Dark Domain

QuDAG's unique feature is the `.dark` domain system - decentralized, quantum-resistant naming:

```bash
# Register your own .dark domain
qudag address register mynode.dark

# Register multiple domains
qudag address register secret-chat.dark
qudag address register my-business.dark

# List your registered domains
qudag address list
```

### 3. Generate Temporary Shadow Addresses

For ephemeral communication, create temporary addresses:

```bash
# Generate a shadow address with 1-hour TTL
qudag address shadow --ttl 3600

# Generate with custom expiration
qudag address shadow --ttl 86400  # 24 hours
```

Example output:
```
🕵️ Shadow Address Generated:
   Address: shadow-a1b2c3d4.dark
   TTL: 3600 seconds (1 hour)
   Expires: 2024-09-06T21:30:00Z
```

### 4. Create Quantum Fingerprints

Generate quantum-resistant content fingerprints:

```bash
# Create fingerprint for text content
qudag address fingerprint --data "Hello QuDAG World!"

# Create fingerprint for a file
qudag address fingerprint --file document.pdf
```

Example output:
```
🔐 Quantum Fingerprint Created:
   Algorithm: ML-DSA + BLAKE3
   Hash: blake3:1a2b3c4d5e6f...
   Signature: ml_dsa:9z8y7x6w...
   Verification: ✅ Valid
```

## 🌐 Connect to the Live Testnet

QuDAG has a global testnet with nodes in Toronto, Amsterdam, Singapore, and San Francisco:

```bash
# Connect to the global testnet
qudag start --bootstrap-peers /dns4/qudag-testnet-node1.fly.dev/tcp/4001

# Check testnet health
curl https://qudag-testnet-node1.fly.dev/health | jq

# View network stats
qudag network stats
```

### Testnet Nodes

| Location | Domain | Status |
|----------|--------|--------|
| Toronto | [qudag-testnet-node1.fly.dev](https://qudag-testnet-node1.fly.dev/health) | ✅ Healthy |
| Amsterdam | [qudag-testnet-node2.fly.dev](https://qudag-testnet-node2.fly.dev/health) | ✅ Healthy |
| Singapore | [qudag-testnet-node3.fly.dev](https://qudag-testnet-node3.fly.dev/health) | ✅ Healthy |
| San Francisco | [qudag-testnet-node4.fly.dev](https://qudag-testnet-node4.fly.dev/health) | ✅ Healthy |

## 🤖 AI Integration with MCP

QuDAG includes a native MCP (Model Context Protocol) server for AI integration:

```bash
# Start MCP server for AI tools
qudag mcp start

# Start with HTTP transport (for web integration)
qudag mcp start --transport http --port 3000

# Start with WebSocket support
qudag mcp start --transport ws --port 8080
```

### Connect to Claude Desktop

Add to your `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "qudag": {
      "command": "qudag",
      "args": ["mcp", "start", "--transport", "stdio"]
    }
  }
}
```

## 💱 Resource Exchange with rUv Tokens

QuDAG includes a native token system for resource trading:

```bash
# Create exchange accounts
qudag exchange create-account --name alice
qudag exchange create-account --name bob

# Mint tokens for testing
qudag exchange mint --account alice --amount 10000

# Check balance
qudag exchange balance --account alice

# Transfer tokens
qudag exchange transfer --from alice --to bob --amount 1000

# Check fee rates
qudag exchange fee-status --examples
```

## 🔐 Quantum-Resistant Vault

Manage passwords and secrets with quantum-resistant encryption:

```bash
# Initialize vault (if not done automatically)
qudag vault config init

# Generate secure passwords
qudag vault generate --length 16
qudag vault generate --length 32 --special-chars

# Store password (interactive)
qudag vault add --name "github-token"

# List stored passwords
qudag vault list

# Retrieve a password
qudag vault get --name "github-token"
```

## 🌍 Network Operations

### Peer Management

```bash
# List connected peers
qudag peer list

# Connect to a specific peer
qudag peer add /ip4/192.168.1.100/tcp/8000

# Remove a peer
qudag peer remove 12D3KooWABC123...

# Network statistics
qudag network stats
```

### Dark Domain Operations

```bash
# Resolve any .dark domain
qudag address resolve mynode.dark
qudag address resolve friend.dark

# Test domain resolution
qudag network test

# Check node status
qudag status
```

## 📊 Monitoring and Logs

```bash
# View real-time logs
qudag logs --follow

# Check node health
qudag status

# Performance metrics
qudag network stats --format json

# System information
qudag system info
```

## 🔧 Configuration

### Basic Configuration

Create a configuration file at `~/.qudag/config.toml`:

```toml
[node]
port = 8000
max_peers = 50
enable_mdns = true

[crypto]
algorithm = "ML-KEM-768"
constant_time = true

[network]
circuit_length = 5
connection_timeout = "30s"

[consensus]
finality_threshold = 0.99
timeout_ms = 1000

[vault]
encryption = "AES-256-GCM"
key_derivation = "Argon2"
```

### Environment Variables

```bash
# Set log level
export QUDAG_LOG=debug

# Custom data directory
export QUDAG_DATA_DIR=/custom/path

# Network configuration
export QUDAG_PORT=8001
export QUDAG_MAX_PEERS=100
```

## 🚦 Common Operations

### Development Workflow

```bash
# 1. Start development node
qudag start --port 8000 --verbose

# 2. In another terminal, register your domain
qudag address register dev-node.dark

# 3. Generate test data
qudag address fingerprint --data "test message"

# 4. Check everything is working
qudag status
qudag network stats
```

### Production Deployment

```bash
# 1. Start production node with specific configuration
qudag start --config /etc/qudag/prod.toml --daemon

# 2. Register production domain
qudag address register production.dark

# 3. Connect to bootstrap peers
qudag peer add /dns4/bootstrap-node.example.com/tcp/8000

# 4. Monitor health
qudag status --format json
```

## 🛠️ Troubleshooting

### Common Issues

**Node won't start:**
```bash
# Check port availability
netstat -ln | grep 8000

# Try different port
qudag start --port 8001
```

**Connection issues:**
```bash
# Test network connectivity
qudag network test

# Check firewall settings
sudo ufw status

# Try with different bootstrap peers
qudag start --bootstrap-peers /ip4/1.2.3.4/tcp/8000
```

**Peer discovery problems:**
```bash
# Enable mDNS discovery
qudag start --enable-mdns

# Check NAT/firewall configuration
qudag network diagnose
```

### Getting Help

- **Logs**: Use `qudag logs --level debug` for detailed information
- **Status**: Check `qudag status` for node health
- **Network**: Use `qudag network stats` for connectivity info
- **Documentation**: Visit the [full documentation](../README.md)
- **Issues**: Report problems on [GitHub](https://github.com/ruvnet/QuDAG/issues)

## 🎉 Next Steps

Now that you have QuDAG running:

1. **Explore the CLI**: Check out the [CLI Reference](cli-reference.md)
2. **Learn about Architecture**: Read the [System Architecture](../architecture/system-architecture.md)
3. **Set up Development**: Follow the [Development Setup](../development/setup.md)
4. **Join the Community**: Contribute to [GitHub](https://github.com/ruvnet/QuDAG)
5. **Build Applications**: Use the [API Documentation](api-docs.md)

Welcome to the quantum-resistant future of decentralized communication! 🌐