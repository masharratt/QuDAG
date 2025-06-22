# Fly.io Deployment Research for QuDAG Testnet

## Executive Summary

This document provides comprehensive research on deploying Rust applications on Fly.io for the QuDAG testnet setup. Fly.io offers excellent support for distributed applications with built-in private networking, multi-region deployment capabilities, and persistent storage options suitable for DAG data.

## 1. Available Fly.io Regions

### Canadian Regions (Near Toronto)
- **Toronto, Canada** - Region code: `yyz` ✅ (Primary choice)
- **Montreal, Canada** - Region code: `yul` (Alternative)

### Complete Region Management
- View all 35 regions: `fly platform regions`
- Add regions: `flyctl regions add yyz yul`
- Toronto uses the IATA airport code system for identification

## 2. Deploying Rust Applications on Fly.io

### Quick Start
Fly.io includes a Rust scanner in flyctl that generates optimized Dockerfiles:
```bash
fly launch
```

### Dockerfile Configuration
Fly.io recommends using Cargo Chef for efficient builds:

```dockerfile
# Build stage
FROM rust:1.75-slim-buster AS builder
WORKDIR /app
COPY Cargo.lock Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY . .
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/qudag /app/qudag
EXPOSE 8080
CMD ["./qudag"]
```

### fly.toml Configuration
```toml
app = "qudag-testnet-node1"
primary_region = "yyz"

[build]
  dockerfile = "Dockerfile"

[env]
  NODE_ID = "node1"
  NETWORK_MODE = "testnet"

[[services]]
  internal_port = 8080
  protocol = "tcp"

  [[services.ports]]
    port = 80
    handlers = ["http"]
  
  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

[experimental]
  cmd = "./qudag"
```

## 3. Multi-Region Deployment Configuration

### Deployment Strategy
Deploy 4 nodes across different regions for resilience:
```bash
# Node 1 - Toronto
fly deploy --region yyz --app qudag-node1

# Node 2 - Montreal  
fly deploy --region yul --app qudag-node2

# Node 3 - Chicago
fly deploy --region ord --app qudag-node3

# Node 4 - New York
fly deploy --region ewr --app qudag-node4
```

### Region Configuration in fly.toml
```toml
primary_region = "yyz"
backup_regions = ["yul", "ord", "ewr"]
```

## 4. Networking Between Fly.io Instances

### Private Network (6PN)
- **Automatic Setup**: All apps in an organization are connected via WireGuard mesh (IPv6)
- **Zero Configuration**: Private networking is enabled by default
- **Internal Domains**: Each app gets `<appname>.internal` domain

### Inter-Node Communication
```rust
// Example: Connect to other nodes
let node2_addr = "qudag-node2.internal:8080";
let node3_addr = "qudag-node3.internal:8080";
let node4_addr = "qudag-node4.internal:8080";

// Region-specific addressing
let toronto_nodes = "yyz.qudag-testnet.internal";
```

### Service Binding
Bind services to the private network:
```rust
// In your Rust app
let addr = "[::]:8080"; // Binds to fly-local-6pn
```

## 5. Environment Variables and Secrets Management

### Environment Variables (fly.toml)
```toml
[env]
  NODE_ID = "node1"
  NETWORK_MODE = "testnet"
  DAG_SYNC_INTERVAL = "30"
  LOG_LEVEL = "info"
```

### Secrets Management
```bash
# Set individual secrets
fly secrets set DATABASE_URL="postgres://..." --app qudag-node1
fly secrets set API_KEY="secret_key_here" --app qudag-node1

# Set multiple secrets (stage for later deployment)
fly secrets set NODE_PRIVATE_KEY="..." --stage
fly secrets set PEER_AUTH_TOKEN="..." --stage
fly deploy # Deploy with new secrets

# List secrets (names only, values hidden)
fly secrets list
```

### Important Notes:
- Secrets are runtime environment variables (not available during Docker build)
- Setting secrets triggers a deployment unless `--stage` is used
- Secrets are encrypted at rest in Fly's vault

## 6. Persistent Storage Options for DAG Data

### Fly Volumes Overview
- **Type**: Local NVMe SSD storage
- **Performance**: 2000 IOPs, 8MiB/s bandwidth
- **Size**: 1GB default, up to 500GB maximum
- **Encryption**: Enabled by default

### Volume Creation for Each Node
```bash
# Create 10GB volumes for DAG storage
fly volumes create dag_data --size 10 --region yyz --app qudag-node1
fly volumes create dag_data --size 10 --region yul --app qudag-node2
fly volumes create dag_data --size 10 --region ord --app qudag-node3
fly volumes create dag_data --size 10 --region ewr --app qudag-node4
```

### Mount Configuration (fly.toml)
```toml
[mounts]
  destination = "/data"
  source = "dag_data"
```

### Data Resilience Strategy
Since Fly Volumes don't automatically replicate:
1. Implement application-level replication between nodes
2. Use daily snapshots (retained 5 days by default)
3. Regular backups to external storage
4. Consider using SQLite with LiteFS for distributed replication

## 7. Monitoring and Logging Setup

### Built-in Monitoring
- **Metrics**: Free Prometheus metrics (currently)
- **Dashboard**: Access at https://fly-metrics.net
- **Grafana**: Pre-configured dashboards included

### Accessing Metrics
```bash
# View metrics in dashboard
fly dashboard --app qudag-node1

# Prometheus endpoint (per organization)
curl https://api.fly.io/prometheus/personal
```

### Custom Metrics
Export Prometheus-formatted metrics from your Rust app:
```rust
// Using prometheus-rust
use prometheus::{Encoder, TextEncoder, Counter, Gauge};

// Define metrics
lazy_static! {
    static ref DAG_NODES: Gauge = register_gauge!("qudag_dag_nodes_total", "Total DAG nodes").unwrap();
    static ref TRANSACTIONS: Counter = register_counter!("qudag_transactions_total", "Total transactions").unwrap();
}

// Expose metrics endpoint
async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

### Logging
```bash
# View logs
fly logs --app qudag-node1

# Stream logs
fly logs --app qudag-node1 --tail
```

## 8. Cost Estimates for Running 4 Nodes

### VM Costs (per node)
- **Shared CPU (1x) + 256MB RAM**: ~$1.94/month
- **Shared CPU (1x) + 512MB RAM**: ~$3.88/month
- **Shared CPU (2x) + 1GB RAM**: ~$7.76/month

### Storage Costs
- **10GB Volume per node**: $1.50/month
- **Total for 4 nodes**: $6.00/month

### Bandwidth Costs
- **Inbound**: Free
- **Outbound**: $0.02/GB (North America)
- **Estimated 50GB/month**: $1.00

### Total Monthly Cost Estimate
For 4 nodes with moderate resources:
- **Basic Setup** (256MB RAM each): ~$19.76/month
  - VMs: 4 × $1.94 = $7.76
  - Storage: 4 × $1.50 = $6.00
  - Bandwidth: ~$1.00
  - **Total**: ~$14.76/month

- **Recommended Setup** (1GB RAM each): ~$43.04/month
  - VMs: 4 × $7.76 = $31.04
  - Storage: 4 × $1.50 = $6.00
  - Bandwidth: ~$1.00
  - **Total**: ~$38.04/month

### Cost Optimization
- Usage under $5/month often waived for new accounts
- Reserved instances offer 40% discount for annual commitments
- Scale machines up/down based on load

## Deployment Checklist

1. **Prepare Rust Application**
   - [ ] Create optimized Dockerfile with Cargo Chef
   - [ ] Configure for 6PN networking
   - [ ] Implement metrics endpoints
   - [ ] Add health check endpoints

2. **Initialize Fly Apps**
   ```bash
   fly launch --name qudag-node1 --region yyz --no-deploy
   fly launch --name qudag-node2 --region yul --no-deploy
   fly launch --name qudag-node3 --region ord --no-deploy
   fly launch --name qudag-node4 --region ewr --no-deploy
   ```

3. **Create Volumes**
   ```bash
   fly volumes create dag_data --size 10 --app qudag-node1
   fly volumes create dag_data --size 10 --app qudag-node2
   fly volumes create dag_data --size 10 --app qudag-node3
   fly volumes create dag_data --size 10 --app qudag-node4
   ```

4. **Set Secrets**
   ```bash
   for app in qudag-node1 qudag-node2 qudag-node3 qudag-node4; do
     fly secrets set NODE_PRIVATE_KEY="..." --app $app --stage
     fly secrets set PEER_AUTH_TOKEN="..." --app $app --stage
   done
   ```

5. **Deploy Applications**
   ```bash
   fly deploy --app qudag-node1
   fly deploy --app qudag-node2
   fly deploy --app qudag-node3
   fly deploy --app qudag-node4
   ```

6. **Verify Deployment**
   ```bash
   fly status --app qudag-node1
   fly logs --app qudag-node1
   curl https://qudag-node1.fly.dev/health
   ```

## Best Practices for QuDAG on Fly.io

1. **Data Persistence**
   - Implement application-level replication between nodes
   - Regular backups to external storage (S3, etc.)
   - Monitor volume usage and expand as needed

2. **Networking**
   - Use internal `.internal` domains for inter-node communication
   - Implement retry logic for network operations
   - Consider using gRPC for efficient binary protocol

3. **Security**
   - Use secrets for all sensitive configuration
   - Enable TLS for external endpoints
   - Implement mutual TLS for node-to-node communication

4. **Monitoring**
   - Export custom Prometheus metrics
   - Set up alerts for critical metrics
   - Use distributed tracing for debugging

5. **Scaling**
   - Start with minimal resources and scale up
   - Use fly autoscale for automatic scaling
   - Monitor resource usage and adjust

## Conclusion

Fly.io provides an excellent platform for deploying the QuDAG testnet with:
- Native support for distributed applications
- Built-in private networking with zero configuration
- Persistent storage suitable for DAG data
- Comprehensive monitoring and metrics
- Reasonable pricing for a 4-node testnet (~$38/month)

The platform's focus on edge computing and global distribution aligns well with QuDAG's distributed architecture, making it an ideal choice for the testnet deployment.