# /build/cargo

## Purpose
Build Rust binaries and libraries for QuDAG with cross-compilation support, optimization profiles, and multiple target architectures. Supports native builds, static linking, and release preparation.

## Parameters
- `<target>`: Target architecture - native|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-pc-windows-gnu|x86_64-apple-darwin|aarch64-apple-darwin|all (default: native)
- `[profile]`: Build profile - dev|release|bench|test (default: release)
- `[features]`: Feature flags - default|crypto|dag|network|vault|full (default: default)
- `[workspace]`: Workspace member to build - all|qudag|qudag-crypto|qudag-dag|qudag-network|qudag-vault (default: all)

## Prerequisites
- [ ] Rust toolchain installed (latest stable)
- [ ] Cross-compilation targets installed via rustup
- [ ] System dependencies for crypto libraries (libssl, libclang)
- [ ] Docker available for cross-compilation (if using)

## Execution Steps

### 1. Validation Phase
- Validate target architecture support
- Check Rust toolchain version
- Verify cross-compilation requirements
- Confirm workspace structure

### 2. Environment Setup
- Install required targets
  ```bash
  rustup target add x86_64-unknown-linux-gnu
  rustup target add aarch64-unknown-linux-gnu
  rustup target add x86_64-pc-windows-gnu
  rustup target add x86_64-apple-darwin
  rustup target add aarch64-apple-darwin
  ```
- Set environment variables
  ```bash
  export RUSTFLAGS="-C target-cpu=native"
  export CARGO_TARGET_DIR="/workspaces/QuDAG/target"
  ```

### 3. Dependency Verification
- Update Cargo.lock file
  ```bash
  cd /workspaces/QuDAG
  cargo update
  ```
- Check for security advisories
  ```bash
  cargo audit
  ```

### 4. Build Execution
- Step 4.1: Clean previous builds
  ```bash
  cargo clean
  ```
- Step 4.2: Build for target architecture
  - **Native build**:
    ```bash
    cargo build --release --workspace
    ```
  - **Linux x86_64**:
    ```bash
    cargo build --release --target x86_64-unknown-linux-gnu --workspace
    ```
  - **Linux ARM64**:
    ```bash
    cargo build --release --target aarch64-unknown-linux-gnu --workspace
    ```
  - **Windows x86_64**:
    ```bash
    cargo build --release --target x86_64-pc-windows-gnu --workspace
    ```
  - **macOS x86_64**:
    ```bash
    cargo build --release --target x86_64-apple-darwin --workspace
    ```
  - **macOS ARM64**:
    ```bash
    cargo build --release --target aarch64-apple-darwin --workspace
    ```

### 5. Feature-Specific Builds
- **Crypto-only build**:
  ```bash
  cargo build --release -p qudag-crypto --no-default-features --features ml-kem,ml-dsa
  ```
- **Network build**:
  ```bash
  cargo build --release -p qudag-network --features dark-addressing,onion-routing
  ```
- **Vault build**:
  ```bash
  cargo build --release -p qudag-vault --features distributed,encryption
  ```

### 6. Static Linking
- Build with static linking for distribution
  ```bash
  RUSTFLAGS="-C target-feature=+crt-static" \
  cargo build --release --target x86_64-unknown-linux-gnu
  ```

### 7. Optimization Profiles
- **Size-optimized build**:
  ```bash
  cargo build --release --config 'profile.release.opt-level="z"' \
    --config 'profile.release.lto=true' \
    --config 'profile.release.codegen-units=1' \
    --config 'profile.release.strip=true'
  ```
- **Performance-optimized build**:
  ```bash
  cargo build --release --config 'profile.release.opt-level=3' \
    --config 'profile.release.lto="fat"'
  ```

### 8. Binary Verification
- Verify binary integrity
  ```bash
  find target -name "qudag*" -type f -executable -exec file {} \;
  find target -name "*.so" -exec file {} \;
  ```

## Success Criteria
- [ ] All workspace members build successfully
- [ ] No compilation warnings with -D warnings
- [ ] Binaries run without runtime errors
- [ ] Cross-compiled binaries are valid for target architecture
- [ ] Static analysis passes (clippy, audit)
- [ ] Build artifacts are under 50MB per binary

## Error Handling
- **Compilation errors**: Check dependency versions and feature flags
- **Linker errors**: Verify system dependencies and cross-compilation setup
- **Target not found**: Install with `rustup target add <target>`
- **Cross-compilation failures**: Use Docker containers for consistent builds
- **Size issues**: Enable LTO and strip symbols for release builds
- **Performance issues**: Profile with `cargo flamegraph` or `perf`

## Output
- **Success**: Compiled binaries in `/workspaces/QuDAG/target/[target]/release/`
- **Failure**: Compilation errors with specific error messages
- **Reports**: 
  - Build log with timing information
  - Binary size analysis
  - Static analysis results

## Example Usage
```bash
# Build all targets for Linux x86_64
/build/cargo x86_64-unknown-linux-gnu release full all

# Build crypto library only
/build/cargo native release crypto qudag-crypto

# Cross-compile for all platforms
/build/cargo all release default all
```

### Example Output
```
Building QuDAG workspace...
✓ qudag-crypto (2.1s) -> 15.2MB
✓ qudag-dag (1.8s) -> 12.8MB
✓ qudag-network (3.2s) -> 18.4MB
✓ qudag-vault (2.5s) -> 14.1MB
✓ qudag-cli (1.2s) -> 8.9MB

Cross-compilation results:
- Linux x86_64: 5 binaries, 69.4MB total
- Linux ARM64: 5 binaries, 72.1MB total
- macOS x86_64: 5 binaries, 71.2MB total
- macOS ARM64: 5 binaries, 68.8MB total
```

## Related Commands
- `/build/wasm`: Build WebAssembly binaries
- `/test/cargo`: Run Rust test suite
- `/deploy/crates`: Publish to crates.io

## Workflow Integration
This command is part of the Release Preparation workflow and:
- Follows: Code implementation and testing
- Precedes: `/deploy/github` for release distribution
- Can be run in parallel with: `/build/wasm` for web builds

## Agent Coordination
- **Primary Agent**: Build Agent
- **Supporting Agents**: 
  - Crypto Agent: Validates cryptographic library builds
  - Network Agent: Ensures network features compile correctly
  - Performance Agent: Analyzes build performance and sizes

## Notes
- Cross-compilation may require Docker for consistent results
- Static linking increases binary size but improves portability
- LTO can significantly reduce binary size but increases build time
- Some crypto dependencies may not support all targets
- Consider using GitHub Actions for consistent cross-compilation

---

## Advanced Build Configurations

### Custom Profiles
```toml
# Add to Cargo.toml
[profile.dist]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = "symbols"
```

### Docker Cross-Compilation
```bash
# Using cross for easier cross-compilation
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

### Feature Flag Optimization
```bash
# Minimal build for specific use case
cargo build --release --no-default-features \
  --features "crypto,serde" -p qudag-crypto

# Full build with all features
cargo build --release --all-features --workspace
```

### Benchmarking Builds
```bash
# Build with benchmark support
cargo build --release --features "bench" --workspace

# Profile-guided optimization
cargo pgo build --release
```

### Security Hardening
```bash
# Build with security features
RUSTFLAGS="-C relro-level=full -C strip=symbols" \
cargo build --release --target x86_64-unknown-linux-gnu
```