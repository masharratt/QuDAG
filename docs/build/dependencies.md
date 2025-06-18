# QuDAG Build Dependencies and Requirements

This document outlines the dependencies, build requirements, and system specifications needed to build and run QuDAG with production cryptographic implementations.

## System Requirements

### Minimum Requirements

| Component | Requirement | Notes |
|-----------|-------------|-------|
| **OS** | Linux, macOS, Windows | WSL2 recommended for Windows |
| **CPU** | x86_64 or ARM64 | AVX2/NEON acceleration recommended |
| **RAM** | 4 GB | 8 GB recommended for compilation |
| **Disk** | 2 GB free space | For dependencies and build artifacts |
| **Rust** | 1.70.0+ | MSRV (Minimum Supported Rust Version) |

### Recommended Requirements

| Component | Recommendation | Benefits |
|-----------|----------------|----------|
| **CPU** | Multi-core (4+ cores) | Parallel compilation |
| **RAM** | 16 GB+ | Faster linking and concurrent builds |
| **Disk** | SSD storage | Faster I/O during compilation |
| **CPU Features** | AVX2 (x86) / NEON (ARM) | Hardware crypto acceleration |

## Rust Toolchain

### Version Requirements

```bash
# Install Rust 1.70.0 or later
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should be >= 1.70.0
cargo --version

# Update if needed
rustup update
```

### Required Components

```bash
# Install required Rust components
rustup component add clippy       # Linting
rustup component add rustfmt      # Code formatting
rustup target add x86_64-unknown-linux-musl  # Static linking (optional)
```

### Compilation Flags

For optimal performance with production crypto:

```bash
# Set environment variables for builds
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
export CARGO_TARGET_DIR="./target"

# Alternative: Create .cargo/config.toml
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[build]
rustflags = ["-C", "target-cpu=native", "-C", "opt-level=3"]

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
EOF
```

## Core Dependencies

### Cryptographic Libraries

#### ML-KEM (Key Encapsulation)

```toml
[dependencies]
ml-kem = "0.2"
```

**Description:** NIST-compliant ML-KEM implementation  
**Features:** 
- ML-KEM-768 (NIST Level 3 security)
- Constant-time operations
- Hardware acceleration support

**System Dependencies:**
- No additional system libraries required
- Pure Rust implementation

#### ML-DSA (Digital Signatures)

```toml
[dependencies]
pqcrypto-dilithium = "0.5"
pqcrypto-traits = "0.3"
```

**Description:** ML-DSA (Dilithium) post-quantum signatures  
**Features:**
- Multiple parameter sets (Dilithium-2, 3, 5)
- Rejection sampling for security
- Optimized NTT operations

**System Dependencies:**
- No additional system libraries required
- C-compatible FFI bindings

#### BLAKE3 Hashing

```toml
[dependencies]
blake3 = "1.3"
```

**Description:** Fast, secure cryptographic hash function  
**Features:**
- SIMD acceleration (AVX2, SSE4.1, NEON)
- Parallelizable tree hashing
- Multiple output lengths

**System Dependencies:**
- Automatically detects CPU features
- Falls back to portable implementation

#### SHA3/SHAKE

```toml
[dependencies]
sha3 = "0.10"
```

**Description:** SHA-3 and SHAKE extendable-output functions  
**Features:**
- Used in ML-DSA for domain separation
- Keccak-based permutation
- Variable output lengths

### Security Libraries

#### Memory Protection

```toml
[dependencies]
zeroize = "1.5"
subtle = "2.4"
```

**zeroize Features:**
- Automatic secret memory clearing
- `ZeroizeOnDrop` trait implementation
- Compiler-resistant memory clearing

**subtle Features:**
- Constant-time operations
- Side-channel resistant comparisons
- Timing attack prevention

#### Randomness

```toml
[dependencies]
rand = "0.8"
rand_core = "0.6"
rand_chacha = "0.3"
```

**Features:**
- Cryptographically secure random generation
- Deterministic randomness for testing
- Cross-platform entropy sources

### Performance Dependencies

#### Serialization

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

#### Error Handling

```toml
[dependencies]
thiserror = "1.0"
anyhow = "1.0"  # For application errors
```

#### Async Runtime (Optional)

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
```

## Development Dependencies

### Testing

```toml
[dev-dependencies]
criterion = "0.5"         # Benchmarking
proptest = "1.0"          # Property-based testing
hex-literal = "0.4"       # Hex constants in tests
tempfile = "3.0"          # Temporary files
```

### Fuzzing

```toml
[dev-dependencies]
arbitrary = "1.0"         # Arbitrary data generation
libfuzzer-sys = "0.4"     # LibFuzzer integration
```

### Documentation

```toml
[dev-dependencies]
doc-comment = "0.3"       # Documentation tests
```

## Build Configuration

### Cargo.toml Features

```toml
[features]
default = ["std", "production"]

# Standard library support
std = [
    "rand/std",
    "zeroize/std",
    "blake3/std"
]

# Production cryptography
production = [
    "ml-kem",
    "pqcrypto-dilithium"
]

# No standard library (embedded)
no-std = [
    "rand/no-std",
    "zeroize/no-std"
]

# Security testing
security-tests = []

# Performance optimizations
optimized = [
    "blake3/simd",
    "ml-kem/avx2"
]

# Benchmarking
bench = []
```

### Profile Configuration

```toml
[profile.dev]
opt-level = 0
debug = true
overflow-checks = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
debug = false
overflow-checks = false

[profile.bench]
inherits = "release"
debug = true

[profile.test]
opt-level = 1
debug = true
```

## Platform-Specific Requirements

### Linux

#### Ubuntu/Debian

```bash
# Install build essentials
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# For development tools
sudo apt install git curl gdb valgrind

# Optional: Cross-compilation targets
sudo apt install gcc-multilib
```

#### CentOS/RHEL/Fedora

```bash
# Install development tools
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel pkg-config

# Or for CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel pkg-config
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install Homebrew (optional, for additional tools)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install development tools
brew install git curl
```

### Windows

#### Using WSL2 (Recommended)

```bash
# Enable WSL2 and install Ubuntu
wsl --install

# Follow Linux instructions within WSL2
```

#### Native Windows

```powershell
# Install Visual Studio Build Tools or Visual Studio Community
# Download from: https://visualstudio.microsoft.com/downloads/

# Install Git for Windows
# Download from: https://git-scm.com/download/win

# Install Rust
# Follow instructions at: https://rustup.rs/
```

## Build Commands

### Standard Build

```bash
# Clean build
cargo clean
cargo build --release

# With all features
cargo build --release --all-features

# For specific target
cargo build --release --target x86_64-unknown-linux-musl
```

### Development Build

```bash
# Fast development build
cargo build

# With specific features
cargo build --features "production,security-tests"

# Check without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test --all-features

# Run specific test suites
cargo test -p qudag-crypto
cargo test -p qudag-network
cargo test -p qudag-dag

# Run security tests
cargo test --features security-tests

# Run with specific test threads
cargo test -- --test-threads=1
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run crypto benchmarks
cargo bench --features bench -p qudag-crypto

# Save benchmark results
cargo bench --features bench | tee benchmark-results.txt
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Fix automatically
cargo clippy --fix
```

## Performance Optimization

### CPU-Specific Builds

```bash
# For the current CPU
RUSTFLAGS="-C target-cpu=native" cargo build --release

# For specific CPU features
RUSTFLAGS="-C target-feature=+avx2,+aes" cargo build --release

# For older CPUs (compatibility)
RUSTFLAGS="-C target-cpu=x86-64" cargo build --release
```

### Memory Optimization

```bash
# Reduce memory usage during compilation
export CARGO_BUILD_JOBS=1

# Use less memory for linking
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Profile memory usage
cargo build --timings
```

### Cross-Compilation

```bash
# Add targets
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

# Cross-compile
cargo build --target aarch64-unknown-linux-gnu --release
```

## Verification

### Build Verification

```bash
# Verify all components build
cargo build --all-targets --all-features

# Verify examples build
cargo build --examples

# Verify benchmarks build
cargo bench --no-run
```

### Runtime Verification

```bash
# Run crypto examples
cargo run --example ml_kem_example
cargo run --example ml_dsa_example
cargo run --example hqc_example
cargo run --example fingerprint_example

# Run CLI
cargo run -p qudag-cli -- --help
```

### Security Verification

```bash
# Run security tests
cargo test --features security-tests

# Check for vulnerabilities
cargo audit

# Check dependencies
cargo tree
cargo outdated  # Requires cargo-outdated
```

## Troubleshooting

### Common Build Issues

#### Issue: "Cannot find ml-kem crate"

**Solution:**
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

#### Issue: "Linking failed with undefined symbols"

**Solution:**
```bash
# Install missing system libraries
sudo apt install build-essential pkg-config

# Or use static linking
cargo build --target x86_64-unknown-linux-musl
```

#### Issue: "Out of memory during compilation"

**Solution:**
```bash
# Reduce parallel jobs
export CARGO_BUILD_JOBS=1

# Use more efficient linker
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Build incrementally
cargo build -p qudag-crypto
cargo build -p qudag-network
cargo build -p qudag-dag
```

#### Issue: "Failed to download dependencies"

**Solution:**
```bash
# Configure proxy if needed
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080

# Use alternate registry
export CARGO_REGISTRY_INDEX=https://github.com/rust-lang/crates.io-index
```

### Performance Issues

#### Issue: Slow cryptographic operations

**Solution:**
```bash
# Build with native CPU features
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Enable SIMD optimizations
cargo build --release --features optimized
```

#### Issue: Large binary size

**Solution:**
```bash
# Strip debug symbols
cargo build --release
strip target/release/qudag

# Use dynamic linking
cargo build --release --features dynamic

# Enable LTO
# (Already enabled in release profile)
```

## Continuous Integration

### GitHub Actions

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Format check
      run: cargo fmt -- --check
    
    - name: Lint
      run: cargo clippy -- -D warnings
    
    - name: Build
      run: cargo build --all-features
    
    - name: Test
      run: cargo test --all-features
    
    - name: Benchmark
      run: cargo bench --no-run
```

### Docker Build

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build application
RUN cargo build --release --all-features

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/qudag /usr/local/bin/

ENTRYPOINT ["qudag"]
```

---

This comprehensive dependency and build documentation ensures that developers can successfully build and deploy QuDAG with all production cryptographic implementations across various platforms and configurations.