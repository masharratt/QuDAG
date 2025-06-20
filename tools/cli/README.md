# QuDAG CLI

Command-line interface for the QuDAG quantum-resistant distributed protocol.

## Installation

```bash
cargo install qudag-cli
```

## Quick Start

```bash
# Start a QuDAG node
qudag-cli start --port 8000

# Create your own darknet domain
qudag-cli address register mynode.dark

# Connect to peers
qudag-cli peer add /ip4/192.168.1.100/tcp/8000/p2p/12D3KooW...

# Check node status
qudag-cli status
```

## Commands

### Node Management

```bash
# Start a node
qudag-cli start [--port 8000] [--rpc-port 9090]

# Stop the node
qudag-cli stop

# Get node status
qudag-cli status

# Restart node
qudag-cli restart
```

### Peer Operations

```bash
# List connected peers
qudag-cli peer list

# Add a peer
qudag-cli peer add <multiaddr>

# Remove a peer
qudag-cli peer remove <peer_id>

# Ban a peer
qudag-cli peer ban <peer_id>

# Test connectivity
qudag-cli network test
```

### Dark Addressing

```bash
# Register a .dark domain
qudag-cli address register mydomain.dark

# Resolve a domain
qudag-cli address resolve somedomain.dark

# Create temporary shadow address
qudag-cli address shadow --ttl 3600

# Generate quantum fingerprint
qudag-cli address fingerprint --data "Hello World"

# List your registered domains
qudag-cli address list
```

### Network Operations

```bash
# Show network statistics
qudag-cli network stats

# Test peer connectivity
qudag-cli network test

# Monitor network events
qudag-cli logs --follow
```

### Configuration

```bash
# Show current configuration
qudag-cli config show

# Set configuration value
qudag-cli config set key value

# Generate systemd service
qudag-cli systemd --output /etc/systemd/system/
```

## Examples

### Setting Up Your First Node

```bash
# 1. Start your node
qudag-cli start --port 8000

# 2. Register your identity
qudag-cli address register mynode.dark

# 3. Connect to the network (use bootstrap peers)
qudag-cli peer add /ip4/bootstrap.qudag.io/tcp/8000/p2p/12D3KooW...

# 4. Check status
qudag-cli status
```

### Creating a Private Network

```bash
# Node 1
qudag-cli start --port 8001
qudag-cli address register node1.dark

# Node 2
qudag-cli start --port 8002
qudag-cli address register node2.dark
qudag-cli peer add /ip4/127.0.0.1/tcp/8001/p2p/...

# Node 3
qudag-cli start --port 8003
qudag-cli address register node3.dark
qudag-cli peer add /ip4/127.0.0.1/tcp/8001/p2p/...
qudag-cli peer add /ip4/127.0.0.1/tcp/8002/p2p/...
```

### Dark Domain System

```bash
# Register domains for different services
qudag-cli address register chat.dark
qudag-cli address register files.dark
qudag-cli address register api.dark

# Create temporary addresses for ephemeral communication
qudag-cli address shadow --ttl 3600  # 1 hour
qudag-cli address shadow --ttl 86400 # 24 hours

# Resolve any .dark domain to find peers
qudag-cli address resolve chat.dark
qudag-cli address resolve files.dark
```

## Configuration File

QuDAG CLI uses a configuration file at `~/.qudag/config.toml`:

```toml
[node]
port = 8000
rpc_port = 9090
data_dir = "~/.qudag/data"
log_level = "info"

[network]
max_peers = 50
bootstrap_peers = [
    "/ip4/bootstrap1.qudag.io/tcp/8000/p2p/12D3KooW...",
    "/ip4/bootstrap2.qudag.io/tcp/8000/p2p/12D3KooW..."
]

[dark_addressing]
enable = true
ttl_default = 3600

[security]
enable_encryption = true
quantum_resistant = true
```

## Output Formats

Many commands support different output formats:

```bash
# JSON output
qudag-cli status --output json

# Table output (default)
qudag-cli peer list --output table

# Raw output for scripting
qudag-cli peer list --output raw
```

## Logging

```bash
# View logs
qudag-cli logs

# Follow logs in real-time
qudag-cli logs --follow

# Filter by level
qudag-cli logs --level error

# Save logs to file
qudag-cli logs --output /var/log/qudag.log
```

## Systemd Integration

Generate systemd service files:

```bash
# Generate service file
qudag-cli systemd --output /etc/systemd/system/

# Enable and start
sudo systemctl enable qudag
sudo systemctl start qudag
sudo systemctl status qudag
```

## Environment Variables

- `QUDAG_CONFIG` - Path to configuration file
- `QUDAG_DATA_DIR` - Data directory override
- `QUDAG_LOG_LEVEL` - Log level (trace, debug, info, warn, error)
- `QUDAG_PORT` - Default port override
- `QUDAG_RPC_PORT` - RPC port override

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Network error
- `4` - Permission error
- `5` - Not found error

## Shell Completion

Generate shell completion scripts:

```bash
# Bash
qudag-cli completions bash > /etc/bash_completion.d/qudag-cli

# Zsh
qudag-cli completions zsh > ~/.zsh/completions/_qudag-cli

# Fish
qudag-cli completions fish > ~/.config/fish/completions/qudag-cli.fish
```

## Security Considerations

- All communication is quantum-resistant encrypted
- Private keys are stored securely in `~/.qudag/keys/`
- Configuration supports file permissions verification
- Network traffic uses onion routing for anonymity

## Troubleshooting

### Common Issues

**Node won't start**
```bash
# Check if port is in use
netstat -ln | grep :8000

# Check logs
qudag-cli logs --level error
```

**Can't connect to peers**
```bash
# Test network connectivity
qudag-cli network test

# Check firewall settings
sudo ufw status
```

**Permission errors**
```bash
# Check data directory permissions
ls -la ~/.qudag/

# Fix permissions
chmod 700 ~/.qudag/
```

## Documentation

- [API Documentation](https://docs.rs/qudag-cli)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)
- [User Guide](https://github.com/ruvnet/QuDAG/blob/main/docs/cli/README.md)

## License

Licensed under either MIT or Apache-2.0 at your option.