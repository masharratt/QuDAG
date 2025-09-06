# QuDAG MCP Integration Guide

QuDAG includes a complete **Model Context Protocol (MCP)** server implementation, making it the ideal infrastructure for distributed AI agent systems. This guide covers setup, configuration, and usage of QuDAG's MCP capabilities.

## Overview

The MCP server in QuDAG provides:

- **Quantum-Resistant Security**: All operations secured with post-quantum cryptography
- **Comprehensive Tool Suite**: 6 built-in tools for vault, DAG, network, crypto, system, and config operations
- **Rich Resource Access**: 4 dynamic resources providing real-time system state
- **Multiple Transports**: stdio (for Claude Desktop), HTTP, and WebSocket support
- **AI-Ready Prompts**: 10 pre-built prompts for common QuDAG workflows
- **Real-time Updates**: Live resource subscriptions for dynamic data
- **JWT Authentication**: Secure authentication with configurable RBAC
- **Audit Logging**: Complete audit trail of all MCP operations

## Quick Start

### Start MCP Server

```bash
# Start with default HTTP transport
qudag mcp start

# Start with stdio transport (for Claude Desktop)
qudag mcp start --transport stdio

# Start with WebSocket support
qudag mcp start --transport ws --port 8080

# Start with custom configuration
qudag mcp start --config ~/.qudag/mcp.toml --port 3000
```

### Test Server Connectivity

```bash
# Test local MCP server
qudag mcp test --endpoint http://localhost:3000

# List available tools
qudag mcp tools

# List available resources
qudag mcp resources

# Get server info
curl http://localhost:3000/mcp | jq
```

## Available MCP Tools

QuDAG provides 6 comprehensive tools for AI integration:

### 1. Vault Tool (`vault`)

Quantum-resistant password and secret management.

**Operations**: `create`, `list`, `read`, `delete`, `search`, `generate`, `backup`, `restore`

```bash
# Example tool calls via MCP
{
  "tool": "vault",
  "operation": "create",
  "name": "api-key",
  "value": "secret-value",
  "category": "development"
}

{
  "tool": "vault",
  "operation": "search",
  "query": "api",
  "category": "development"
}
```

### 2. DAG Tool (`dag`)

DAG consensus operations and vertex management.

**Operations**: `query`, `add`, `validate`, `status`, `finalize`, `sync`

```bash
# Example tool calls
{
  "tool": "dag",
  "operation": "status"
}

{
  "tool": "dag",
  "operation": "add",
  "payload": "Hello DAG",
  "parents": ["vertex1", "vertex2"]
}
```

### 3. Network Tool (`network`)

P2P network management and peer operations.

**Operations**: `peers`, `connect`, `discover`, `status`, `stats`, `routes`

```bash
# Example tool calls
{
  "tool": "network",
  "operation": "peers"
}

{
  "tool": "network",
  "operation": "connect",
  "address": "/ip4/192.168.1.100/tcp/8000"
}
```

### 4. Crypto Tool (`crypto`)

Cryptographic operations and key management.

**Operations**: `keygen`, `sign`, `verify`, `encrypt`, `decrypt`, `hash`, `fingerprint`

```bash
# Example tool calls
{
  "tool": "crypto",
  "operation": "keygen",
  "algorithm": "ML-KEM-768"
}

{
  "tool": "crypto",
  "operation": "sign",
  "message": "Hello World",
  "key_id": "signing-key-1"
}
```

### 5. System Tool (`system`)

System information and monitoring.

**Operations**: `info`, `resources`, `processes`, `health`, `metrics`, `logs`

```bash
# Example tool calls
{
  "tool": "system",
  "operation": "health"
}

{
  "tool": "system",
  "operation": "metrics",
  "component": "network"
}
```

### 6. Config Tool (`config`)

Configuration management and validation.

**Operations**: `get`, `set`, `list`, `validate`, `export`, `import`, `reset`

```bash
# Example tool calls
{
  "tool": "config",
  "operation": "get",
  "key": "network.max_peers"
}

{
  "tool": "config",
  "operation": "set",
  "key": "consensus.finality_threshold",
  "value": 0.95
}
```

## Available MCP Resources

QuDAG exposes 4 dynamic resources for real-time system state:

### 1. Vault State (`qudag://vault/state`)

Current vault entries and metadata:

```json
{
  "total_entries": 15,
  "categories": ["development", "production", "personal"],
  "last_backup": "2024-09-06T12:00:00Z",
  "encryption": "AES-256-GCM + ML-KEM",
  "entries": [
    {
      "name": "github-token",
      "category": "development", 
      "created": "2024-09-01T10:00:00Z",
      "accessed": "2024-09-06T11:30:00Z"
    }
  ]
}
```

### 2. DAG Status (`qudag://dag/status`)

DAG consensus state and metrics:

```json
{
  "vertex_count": 1847,
  "finalized_count": 1823,
  "pending_count": 24,
  "tips": ["vertex_a1b2c3", "vertex_d4e5f6"],
  "consensus_state": "active",
  "finality_rate": 0.987,
  "last_finalized": "2024-09-06T12:15:00Z",
  "throughput": {
    "vertices_per_second": 152.3,
    "finality_latency_ms": 847
  }
}
```

### 3. Network Info (`qudag://network/info`)

Peer connections and network statistics:

```json
{
  "peer_count": 12,
  "connection_state": "healthy",
  "bandwidth": {
    "inbound": "1.2 MB/s",
    "outbound": "0.8 MB/s"
  },
  "peers": [
    {
      "id": "12D3KooWABC123",
      "address": "/ip4/203.0.113.1/tcp/8000",
      "latency": 45,
      "connected_since": "2024-09-06T10:30:00Z"
    }
  ],
  "dark_domains": ["mynode.dark", "service.dark"]
}
```

### 4. System Status (`qudag://system/status`)

System health and performance metrics:

```json
{
  "uptime": 86400,
  "cpu_usage": 15.2,
  "memory_usage": 67.8,
  "disk_usage": 23.1,
  "network_io": {
    "bytes_sent": 1048576000,
    "bytes_received": 2097152000
  },
  "health": "healthy",
  "alerts": [],
  "version": "0.4.3"
}
```

## Integration Examples

### Claude Desktop Integration

Add to your `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "qudag": {
      "command": "qudag",
      "args": ["mcp", "start", "--transport", "stdio"],
      "alwaysAllow": [
        "vault",
        "dag", 
        "network",
        "crypto",
        "system",
        "config"
      ],
      "description": "QuDAG MCP Server",
      "timeout": 600
    }
  }
}
```

### Testnet MCP Configuration

Connect to the live QuDAG testnet MCP server:

```json
{
  "mcpServers": {
    "qudag-testnet": {
      "command": "node",
      "args": [
        "/path/to/mcp-http-proxy.js",
        "https://qudag-testnet-node1.fly.dev"
      ],
      "alwaysAllow": [
        "qudag_crypto",
        "qudag_vault", 
        "qudag_dag",
        "qudag_network",
        "qudag_exchange"
      ],
      "description": "QuDAG Testnet MCP Server (Toronto Node)",
      "timeout": 600
    }
  }
}
```

**HTTP Proxy Script** (`mcp-http-proxy.js`):

```javascript
// Bridge HTTP MCP to stdio transport for Claude Desktop
const readline = require('readline');
const baseUrl = process.argv[2] || 'https://qudag-testnet-node1.fly.dev';

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

async function makeRequest(endpoint, method = 'GET', body = null) {
  const url = `${baseUrl}/mcp${endpoint}`;
  const options = { method, headers: { 'Content-Type': 'application/json' } };
  if (body) options.body = JSON.stringify(body);
  
  const response = await fetch(url, options);
  return response.json();
}

rl.on('line', async (line) => {
  try {
    const request = JSON.parse(line);
    let result;
    
    switch (request.method) {
      case 'initialize':
        const discovery = await makeRequest('');
        result = {
          protocolVersion: '2024-11-05',
          capabilities: discovery.mcp?.capabilities || {},
          serverInfo: discovery.mcp?.serverInfo || {}
        };
        break;
        
      case 'tools/list':
        result = await makeRequest('/tools');
        break;
        
      case 'tools/call':
        result = await makeRequest('/tools/execute', 'POST', request.params);
        break;
        
      case 'resources/list':
        result = await makeRequest('/resources');
        break;
        
      case 'resources/read':
        result = await makeRequest(`/resources/read?uri=${encodeURIComponent(request.params.uri)}`);
        break;
        
      default:
        throw new Error(`Unknown method: ${request.method}`);
    }
    
    const response = { jsonrpc: '2.0', id: request.id, result };
    console.log(JSON.stringify(response));
    
  } catch (error) {
    const response = { 
      jsonrpc: '2.0', 
      id: request.id, 
      error: { code: -32603, message: error.message } 
    };
    console.log(JSON.stringify(response));
  }
});
```

### VS Code Extension

```typescript
import { MCPClient } from 'qudag-mcp-client';

class QuDAGExtension {
  private client: MCPClient;
  
  async activate() {
    this.client = new MCPClient('http://localhost:3000');
    await this.client.connect();
    
    // Use vault tool
    const passwords = await this.client.callTool('vault', {
      operation: 'list',
      category: 'development'
    });
    
    // Subscribe to network updates
    this.client.subscribe('qudag://network/info', (data) => {
      this.updateStatusBar(`Peers: ${data.peer_count}`);
    });
  }
  
  async createSecureNote(content: string) {
    // Use crypto tool to create quantum fingerprint
    const fingerprint = await this.client.callTool('crypto', {
      operation: 'fingerprint',
      data: content
    });
    
    // Store in vault
    await this.client.callTool('vault', {
      operation: 'create',
      name: `note-${Date.now()}`,
      value: content,
      metadata: { fingerprint: fingerprint.hash }
    });
  }
}
```

### Python Integration

```python
from qudag_mcp import MCPClient
import asyncio

class QuDAGAgent:
    def __init__(self):
        self.client = MCPClient("http://localhost:3000")
    
    async def connect(self):
        await self.client.connect()
    
    async def get_network_status(self):
        # Use network tool
        peers = await self.client.call_tool("network", {
            "operation": "peers"
        })
        
        # Get network resource
        network_info = await self.client.get_resource("qudag://network/info")
        
        return {
            "peer_count": len(peers["peers"]),
            "bandwidth": network_info["bandwidth"]
        }
    
    async def create_dag_message(self, content):
        # Sign message with crypto tool
        signature = await self.client.call_tool("crypto", {
            "operation": "sign",
            "message": content,
            "algorithm": "ML-DSA"
        })
        
        # Add to DAG
        vertex = await self.client.call_tool("dag", {
            "operation": "add",
            "payload": content,
            "signature": signature["signature"]
        })
        
        return vertex["vertex_id"]
    
    async def monitor_system(self):
        # Subscribe to system status updates
        async for update in self.client.subscribe("qudag://system/status"):
            if update["cpu_usage"] > 80:
                print(f"High CPU usage: {update['cpu_usage']}%")
            
            if update["memory_usage"] > 90:
                print(f"High memory usage: {update['memory_usage']}%")

# Usage
async def main():
    agent = QuDAGAgent()
    await agent.connect()
    
    # Get network status
    status = await agent.get_network_status()
    print(f"Connected to {status['peer_count']} peers")
    
    # Create message in DAG
    vertex_id = await agent.create_dag_message("Hello from Python!")
    print(f"Created vertex: {vertex_id}")
    
    # Start monitoring
    await agent.monitor_system()

if __name__ == "__main__":
    asyncio.run(main())
```

## Configuration

### MCP Server Configuration

Create `~/.qudag/mcp.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000
transport = "http"
timeout = 30

[security]
enable_auth = true
jwt_secret = "your-secret-key"
allowed_origins = ["http://localhost:3000"]

[tools]
vault = true
dag = true
network = true
crypto = true
system = true
config = true

[resources]
update_interval = 5  # seconds
cache_ttl = 300      # seconds

[logging]
level = "info"
audit_log = true
log_file = "/var/log/qudag/mcp.log"
```

### Environment Variables

```bash
# MCP server configuration
export QUDAG_MCP_PORT=3000
export QUDAG_MCP_HOST=127.0.0.1
export QUDAG_MCP_TRANSPORT=http

# Security configuration
export QUDAG_MCP_JWT_SECRET=your-secret-key
export QUDAG_MCP_ENABLE_AUTH=true

# Resource configuration
export QUDAG_MCP_UPDATE_INTERVAL=5
export QUDAG_MCP_CACHE_TTL=300

# Logging configuration
export QUDAG_MCP_LOG_LEVEL=info
export QUDAG_MCP_AUDIT_LOG=true
```

## Advanced Usage

### Custom Tool Development

```rust
use qudag_mcp::{Tool, ToolResult, ToolParams};

#[async_trait]
impl Tool for CustomAnalyticsTool {
    fn name(&self) -> &str {
        "analytics"
    }
    
    fn description(&self) -> &str {
        "Custom analytics and reporting tool"
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult> {
        match params.operation.as_str() {
            "generate_report" => {
                let report = self.generate_network_report().await?;
                Ok(ToolResult::success(report))
            }
            "analyze_performance" => {
                let analysis = self.analyze_dag_performance().await?;
                Ok(ToolResult::success(analysis))
            }
            _ => Err(ToolError::UnknownOperation)
        }
    }
}
```

### Resource Streaming

```rust
use qudag_mcp::{Resource, ResourceStream};

impl Resource for RealTimeMetrics {
    fn uri(&self) -> &str {
        "qudag://metrics/realtime"
    }
    
    fn stream(&self) -> ResourceStream {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let metrics = collect_realtime_metrics().await;
                if tx.send(metrics).await.is_err() {
                    break;
                }
            }
        });
        
        ResourceStream::new(rx)
    }
}
```

## Troubleshooting

### Common Issues

**MCP server won't start:**
```bash
# Check port availability
netstat -ln | grep 3000

# Check configuration
qudag mcp config show

# Start with debug logging
QUDAG_MCP_LOG_LEVEL=debug qudag mcp start
```

**Tool calls failing:**
```bash
# Test individual tools
qudag mcp test --tool vault --operation list

# Check authentication
curl -H "Authorization: Bearer your-jwt" http://localhost:3000/mcp/tools
```

**Resource subscriptions not updating:**
```bash
# Check resource configuration
qudag mcp resources --verbose

# Verify update intervals
qudag mcp config get resources.update_interval
```

### Performance Tuning

```toml
[performance]
worker_threads = 4
request_timeout = 30
max_concurrent_requests = 100
resource_cache_size = 1000

[optimization]
enable_compression = true
batch_tool_calls = true
async_resource_updates = true
```

This MCP integration makes QuDAG an ideal platform for AI-powered applications, providing secure, quantum-resistant access to distributed communication and consensus capabilities.