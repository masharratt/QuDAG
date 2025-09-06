# QuDAG Development Setup Guide

Complete guide to setting up a development environment for QuDAG, covering everything from initial setup to advanced development workflows.

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows (with WSL2)
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 10GB free space for build artifacts
- **Network**: Unrestricted internet access for dependencies

### Required Tools

```bash
# Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Git version control
sudo apt install git        # Ubuntu/Debian
brew install git            # macOS
# Windows: Download from git-scm.com

# Build essentials
sudo apt install build-essential pkg-config libssl-dev    # Ubuntu/Debian
xcode-select --install                                     # macOS

# Optional but recommended
sudo apt install clang llvm                               # Better performance
```

### Rust Configuration

```bash
# Add required targets
rustup target add wasm32-unknown-unknown

# Install useful tools
cargo install cargo-watch          # Auto-rebuild on changes
cargo install cargo-audit          # Security vulnerability scanning  
cargo install cargo-benchcmp       # Benchmark comparison
cargo install cargo-expand         # Macro expansion
cargo install cargo-outdated       # Dependency update checking
cargo install flamegraph           # Performance profiling

# Configure cargo for optimal builds
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]  # Faster linking

[net]
git-fetch-with-cli = true

[registries.crates-io]
protocol = "sparse"
EOF
```

## Repository Setup

### Clone and Configure

```bash
# Clone your fork
git clone https://github.com/masharratt/QuDAG.git
cd QuDAG

# Add upstream remote (for staying up-to-date)
git remote add upstream https://github.com/ruvnet/QuDAG.git

# Verify remotes
git remote -v
# origin    https://github.com/masharratt/QuDAG.git (fetch)
# origin    https://github.com/masharratt/QuDAG.git (push)  
# upstream  https://github.com/ruvnet/QuDAG.git (fetch)
# upstream  https://github.com/ruvnet/QuDAG.git (push)

# Create development branch
git checkout -b development
git push -u origin development
```

### Workspace Overview

```bash
# Explore the workspace structure
ls -la
# core/         - Core implementation modules
# tools/        - CLI and simulation tools
# benchmarks/   - Performance testing
# examples/     - Usage examples
# docs/         - Documentation
# tests/        - Integration tests

# Check workspace configuration
cat Cargo.toml
```

## Build Configuration

### Development Build

```bash
# Full workspace build
cargo build

# Build specific components
cargo build -p qudag-crypto
cargo build -p qudag-network  
cargo build -p qudag-dag
cargo build -p qudag-cli

# Build with optimizations (for testing performance)
cargo build --release

# Build for WASM target
cargo build --target wasm32-unknown-unknown -p qudag-wasm
```

### Environment Variables

```bash
# Create development environment file
cat > .env.dev << 'EOF'
# Logging configuration
RUST_LOG=qudag=debug,qudag_crypto=trace
RUST_BACKTRACE=1

# Development settings
QUDAG_DEV_MODE=true
QUDAG_DATA_DIR=./dev-data
QUDAG_CONFIG_FILE=./dev-config.toml

# Performance settings
QUDAG_WORKER_THREADS=4
QUDAG_ENABLE_SIMD=true

# Network settings
QUDAG_DEFAULT_PORT=8000
QUDAG_MAX_PEERS=20
EOF

# Load development environment
source .env.dev
```

### Development Configuration

```bash
# Create development config
mkdir -p dev-data
cat > dev-config.toml << 'EOF'
[node]
data_dir = "./dev-data"
log_level = "debug"
enable_metrics = true

[network]
port = 8000
max_peers = 20
enable_mdns = true
bootstrap_peers = []

[consensus]
finality_threshold = 0.8  # Lower for faster dev testing
sample_size = 5           # Smaller network simulation

[crypto]
enable_simd = true
constant_time = true

[development]
auto_mine = true
dev_accounts = true
test_vectors = true
EOF
```

## Development Workflow

### Initial Setup Verification

```bash
# Run all tests to verify setup
cargo test --workspace

# Check for common issues
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Development Commands

```bash
# Watch mode - rebuilds on file changes
cargo watch -x 'build --workspace'

# Test specific module during development
cargo watch -x 'test -p qudag-crypto'

# Run with development configuration
cargo run -p qudag-cli -- start --config dev-config.toml

# Run benchmarks
cargo bench

# Profile performance (requires flamegraph)
cargo flamegraph --bin qudag-cli -- start --config dev-config.toml
```

## IDE Configuration

### VS Code Setup

```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension serayuzgur.crates
code --install-extension tamasfe.even-better-toml

# Create VS Code workspace settings
mkdir -p .vscode
cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "rust-analyzer.cargo.features": "all",
    "files.watcherExclude": {
        "**/target/**": true
    },
    "search.exclude": {
        "**/target": true,
        "**/Cargo.lock": true
    }
}
EOF

# Create launch configuration for debugging
cat > .vscode/launch.json << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug QuDAG CLI",
            "cargo": {
                "args": ["build", "--bin=qudag-cli"],
                "filter": {
                    "name": "qudag-cli",
                    "kind": "bin"
                }
            },
            "args": ["start", "--config", "dev-config.toml"],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug"
            }
        },
        {
            "type": "lldb",
            "request": "launch", 
            "name": "Debug Tests",
            "cargo": {
                "args": ["test", "--no-run", "--bin=qudag-cli"],
                "filter": {
                    "name": "qudag-cli",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
EOF
```

### IntelliJ IDEA / CLion Setup

```bash
# Install Rust plugin through IDE
# File → Settings → Plugins → Rust

# Configure Rust settings:
# File → Settings → Languages & Frameworks → Rust
# - Toolchain location: ~/.cargo/bin
# - Standard library: [Auto-detected]
# - Use cargo check: Enabled
# - Use clippy: Enabled
```

## Testing Setup

### Unit Tests

```bash
# Run all unit tests
cargo test --workspace

# Run tests for specific crate
cargo test -p qudag-crypto
cargo test -p qudag-network  
cargo test -p qudag-dag

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_ml_kem_keygen

# Run tests in release mode (faster)
cargo test --release
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run network integration tests (requires 2+ nodes)
./scripts/test-network.sh

# Run consensus integration tests
cargo test --test dag_consensus_integration

# Stress testing
cargo test --release stress_test_consensus
```

### Property Testing

```bash
# Install proptest for property-based testing
# Already included in dev-dependencies

# Run property tests
cargo test prop_test_

# Generate and save test cases
PROPTEST_CASES=10000 cargo test prop_test_crypto_roundtrip
```

## Benchmarking Setup

### Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run crypto benchmarks only
cargo bench --bench crypto_benchmarks

# Compare benchmark results
cargo benchcmp baseline.txt current.txt

# Profile benchmarks
cargo bench --bench crypto_benchmarks -- --profile-time=5
```

### Custom Benchmark Creation

```rust
// benchmarks/benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_crypto::MlKem768;

fn benchmark_ml_kem_keygen(c: &mut Criterion) {
    c.bench_function("ml_kem_keygen", |b| {
        b.iter(|| {
            let _ = black_box(MlKem768::generate_keypair().unwrap());
        });
    });
}

criterion_group!(benches, benchmark_ml_kem_keygen);
criterion_main!(benches);
```

## Debugging Configuration

### Logging Setup

```bash
# Configure structured logging
export RUST_LOG=qudag=debug,qudag_crypto=trace,qudag_network=debug
export RUST_LOG_STYLE=always

# JSON logging for analysis
export RUST_LOG_FORMAT=json

# Enable backtraces
export RUST_BACKTRACE=full

# Performance debugging
export RUST_LOG="qudag::performance=trace"
```

### Memory Debugging

```bash
# Install valgrind (Linux only)
sudo apt install valgrind

# Run with memory checking
valgrind --tool=memcheck --leak-check=full \
  ./target/debug/qudag-cli start

# Address sanitizer (requires nightly Rust)
rustup toolchain install nightly
RUSTFLAGS="-Z sanitizer=address" \
  cargo +nightly build --target x86_64-unknown-linux-gnu

# Memory profiling with heaptrack
heaptrack ./target/debug/qudag-cli start
heaptrack_gui heaptrack.qudag-cli.*.gz
```

## Documentation Development

### Generate Documentation

```bash
# Generate docs for all crates
cargo doc --workspace --no-deps

# Generate docs with private items
cargo doc --workspace --document-private-items

# Open docs in browser
cargo doc --workspace --open

# Check documentation coverage
cargo doc --workspace 2>&1 | grep "missing docs"
```

### Documentation Testing

```bash
# Test code examples in documentation
cargo test --doc

# Test specific crate documentation
cargo test --doc -p qudag-crypto

# Generate documentation with doctests
cargo doc --workspace --no-deps --document-private-items
```

## Continuous Integration Locally

### Pre-commit Hooks

```bash
# Install pre-commit tool
pip install pre-commit

# Create pre-commit configuration
cat > .pre-commit-config.yaml << 'EOF'
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --
        language: system
        types: [rust]
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --workspace -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: cargo-test
        name: cargo test
        entry: cargo test --workspace
        language: system
        types: [rust]  
        pass_filenames: false
EOF

# Install hooks
pre-commit install
```

### Local CI Simulation

```bash
# Create CI simulation script
cat > scripts/ci-local.sh << 'EOF'
#!/bin/bash
set -euo pipefail

echo "🔍 Running local CI simulation..."

echo "📋 Checking code formatting..."
cargo fmt --all -- --check

echo "🔧 Running clippy..."
cargo clippy --workspace -- -D warnings

echo "🧪 Running tests..."
cargo test --workspace

echo "🔐 Security audit..."
cargo audit

echo "📊 Running benchmarks..."
cargo bench --bench crypto_benchmarks

echo "📖 Checking documentation..."
cargo doc --workspace --no-deps

echo "✅ All checks passed!"
EOF

chmod +x scripts/ci-local.sh

# Run local CI
./scripts/ci-local.sh
```

## Performance Analysis

### Profiling Tools

```bash
# CPU profiling with perf (Linux)
sudo perf record --call-graph=dwarf ./target/release/qudag-cli start
sudo perf report

# Heap profiling with massif
valgrind --tool=massif ./target/debug/qudag-cli start
ms_print massif.out.* > memory-profile.txt

# Custom profiling with criterion
cargo bench --bench network_benchmarks -- --profile-time=10
```

### Performance Monitoring

```rust
// Add to main.rs for development
#[cfg(debug_assertions)]
fn setup_performance_monitoring() {
    use std::time::Instant;
    use tracing::info;
    
    let start = Instant::now();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let uptime = start.elapsed();
            info!("System uptime: {:?}", uptime);
            
            // Log memory usage
            if let Ok(usage) = memory_usage() {
                info!("Memory usage: {} MB", usage / 1024 / 1024);
            }
        }
    });
}
```

## Troubleshooting

### Common Issues

**Build Errors:**
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for platform-specific issues
rustup show
```

**Linker Errors:**
```bash
# Install system dependencies
sudo apt install pkg-config libssl-dev  # Ubuntu/Debian
brew install openssl pkg-config         # macOS
```

**WASM Build Issues:**
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

**Performance Issues:**
```bash
# Enable optimizations
export RUSTFLAGS="-C target-cpu=native"

# Use faster linker
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

### Debug Environment

```bash
# Create debug script
cat > debug-env.sh << 'EOF'
#!/bin/bash
echo "=== QuDAG Debug Environment ==="
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Git status: $(git status --porcelain | wc -l) modified files"
echo "Build target: $(rustc --print target-list | grep $(rustc -Vv | grep host | cut -d' ' -f2))"
echo "Available memory: $(free -h | grep Mem | awk '{print $7}')" 2>/dev/null || echo "N/A"
echo "CPU cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 'N/A')"
echo "=== Environment Variables ==="
env | grep -E "(RUST|QUDAG|CARGO)" | sort
EOF

chmod +x debug-env.sh
./debug-env.sh
```

## Contributing Workflow

### Branch Management

```bash
# Stay up-to-date with upstream
git fetch upstream
git checkout main
git merge upstream/main
git push origin main

# Create feature branch
git checkout -b feature/new-cryptographic-primitive
git push -u origin feature/new-cryptographic-primitive

# Regular development cycle
git add .
git commit -m "feat: implement new cryptographic primitive"
git push origin feature/new-cryptographic-primitive
```

### Code Quality Checklist

```bash
# Before committing:
cargo fmt --all                    # Format code
cargo clippy --workspace           # Lint code
cargo test --workspace             # Run tests
cargo doc --workspace              # Check docs
cargo audit                        # Security check

# Before pull request:
./scripts/ci-local.sh              # Full CI simulation
cargo bench                        # Performance regression check
```

This development setup provides a comprehensive environment for contributing to QuDAG, with proper tooling, testing, and quality assurance workflows.