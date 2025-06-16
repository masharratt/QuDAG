# QuDAG CLI

Command-line interface for the QuDAG quantum-resistant anonymous communication protocol.

## Installation

Build from source:
```bash
cargo build --release
cp target/release/qudag /usr/local/bin/
```

## Usage

### Node Management

Start a QuDAG node:
```bash
qudag start --port 8000 --data-dir ./node-data --peers peer1.example.com:8000 peer2.example.com:8000
```

Stop the running node:
```bash
qudag stop
```

Check node status:
```bash
qudag status
```

### Peer Management

List connected peers:
```bash
qudag peer list
```

Add a new peer:
```bash
qudag peer add peer3.example.com:8000
```

Remove a peer:
```bash
qudag peer remove peer3.example.com:8000
```

### Network Management

Display network statistics:
```bash
qudag network stats
```

Test network connectivity:
```bash
qudag network test
```

### DAG Visualization

Generate DAG visualization:
```bash
qudag dag --output dag.dot --format dot
```

## Command Reference

### `qudag start`

Starts a QuDAG node with the specified configuration.

Options:
- `--port <PORT>`: Port to listen on (default: 8000)
- `--data-dir <DIR>`: Data directory path (default: ./data)
- `--peers <ADDR>...`: Initial peer addresses to connect to

### `qudag stop`

Gracefully stops the running QuDAG node.

### `qudag status`

Displays the current status of the running node including:
- Connection status
- Number of connected peers
- DAG statistics
- Network uptime

### `qudag peer`

Peer management commands:

#### `qudag peer list`
Lists all currently connected peers with their addresses and connection status.

#### `qudag peer add <ADDRESS>`
Connects to a new peer at the specified address.

#### `qudag peer remove <ADDRESS>`
Disconnects from the specified peer.

### `qudag network`

Network diagnostic commands:

#### `qudag network stats`
Displays comprehensive network statistics including:
- Total peers
- Active connections
- Message throughput
- Bandwidth usage
- Average latency

#### `qudag network test`
Performs connectivity tests to all connected peers and reports results.

### `qudag dag`

DAG visualization and analysis:

Options:
- `--output <FILE>`: Output file path (default: dag_visualization.dot)
- `--format <FORMAT>`: Output format (default: dot)

Generates a visual representation of the current DAG state.

## Configuration

The CLI can be configured using a configuration file specified with the `--config` flag:

```bash
qudag --config config.toml start
```

Example configuration file (`config.toml`):
```toml
[node]
port = 8000
data_dir = "./node-data"
peers = ["peer1.example.com:8000", "peer2.example.com:8000"]

[network]
max_peers = 50
connection_timeout = 30

[logging]
level = "info"
```

## Examples

### Basic Node Operation

1. Start a node:
   ```bash
   qudag start --port 8000
   ```

2. In another terminal, check status:
   ```bash
   qudag status
   ```

3. Add some peers:
   ```bash
   qudag peer add 192.168.1.100:8000
   qudag peer add node.qudag.example.com:8000
   ```

4. Monitor network:
   ```bash
   qudag network stats
   qudag network test
   ```

### DAG Analysis

Generate and view DAG visualization:
```bash
qudag dag --output current_dag.dot
dot -Tpng current_dag.dot -o dag.png
open dag.png  # or xdg-open on Linux
```

## Troubleshooting

### Common Issues

1. **Port already in use**: Use a different port with `--port`
2. **Permission denied**: Ensure proper permissions for data directory
3. **Connection refused**: Check peer addresses and network connectivity

### Debug Mode

Enable verbose logging:
```bash
RUST_LOG=debug qudag start
```

### Log Files

Logs are written to:
- stdout/stderr (console output)
- `<data-dir>/logs/qudag.log` (file output)

## Security Considerations

- Keep your private keys secure
- Use encrypted connections between peers
- Regularly update to the latest version
- Monitor for suspicious network activity

## Contributing

See the main project repository for contribution guidelines.