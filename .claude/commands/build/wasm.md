# /build/wasm

## Purpose
Build WebAssembly (WASM) bindings for QuDAG with multiple targets, optimization levels, and feature configurations. Supports browser, Node.js, and bundler targets with size and performance optimization.

## Parameters
- `<target>`: Target platform - web|nodejs|bundler|all (default: all)
- `[features]`: Feature flags - crypto-only|dag|vault|full (default: crypto-only)
- `[optimization]`: Optimization level - debug|release|size (default: release)
- `[output-dir]`: Custom output directory (optional)

## Prerequisites
- [ ] Rust toolchain installed with wasm32-unknown-unknown target
- [ ] wasm-pack installed (`cargo install wasm-pack`)
- [ ] binaryen installed for wasm-opt (recommended)
- [ ] QuDAG WASM project at `/workspaces/QuDAG/qudag-wasm/`

## Execution Steps

### 1. Validation Phase
- Validate target parameter is supported
- Check wasm-pack installation
- Verify Rust wasm32 target availability
- Confirm project dependencies

### 2. Environment Setup
- Set WASM environment variables
  ```bash
  export WASM_BINDGEN_EXTERNREF=1
  export RUSTFLAGS="-C target-feature=+reference-types"
  ```
- Add wasm32 target if missing
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### 3. Build Execution
- Step 3.1: Clean previous builds
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  rm -rf pkg pkg-* target/wasm32-unknown-unknown
  ```
- Step 3.2: Build for specified target(s)
  - **Web target**:
    ```bash
    wasm-pack build --target web --out-dir pkg --release \
      --features crypto-only --quiet
    ```
  - **Node.js target**:
    ```bash
    wasm-pack build --target nodejs --out-dir pkg-node --release \
      --features crypto-only --quiet
    ```
  - **Bundler target**:
    ```bash
    wasm-pack build --target bundler --out-dir pkg-bundler --release \
      --features crypto-only --quiet
    ```
- Step 3.3: Size optimization with wasm-opt
  ```bash
  for dir in pkg*; do
    if [ -f "$dir/qudag_wasm_bg.wasm" ]; then
      wasm-opt -Os --enable-reference-types \
        "$dir/qudag_wasm_bg.wasm" -o "$dir/qudag_wasm_bg.wasm"
    fi
  done
  ```

### 4. Feature-Specific Builds
- **Crypto-only build** (minimal size):
  ```bash
  wasm-pack build --target web --out-dir pkg-crypto --release \
    --no-default-features --features crypto-only
  ```
- **Full feature build**:
  ```bash
  wasm-pack build --target web --out-dir pkg-full --release \
    --features full
  ```

### 5. Size Analysis
- Generate size report
  ```bash
  find pkg* -name "*.wasm" -exec ls -lh {} \; > wasm_sizes.txt
  wasm-objdump --section-headers pkg/qudag_wasm_bg.wasm > wasm_sections.txt
  ```

### 6. TypeScript Definitions
- Generate TypeScript bindings
- Validate type definitions
- Export type documentation

## Success Criteria
- [ ] All WASM builds complete without errors
- [ ] WASM files are under 2MB for crypto-only build
- [ ] TypeScript definitions are generated correctly
- [ ] No undefined symbols in WASM binaries
- [ ] All specified features are included in builds

## Error Handling
- **wasm-pack not found**: Install with `cargo install wasm-pack`
- **Target not found**: Run `rustup target add wasm32-unknown-unknown`
- **Build failures**: Check Rust dependencies compatibility with WASM
- **Size issues**: Use `wee_alloc` feature and wasm-opt optimization
- **Type errors**: Verify wasm-bindgen annotations in source code

## Output
- **Success**: WASM binaries in pkg directories with size report
- **Failure**: Build errors with specific error messages and solutions
- **Reports**: 
  - `/workspaces/QuDAG/qudag-wasm/wasm_sizes.txt`: Size analysis
  - `/workspaces/QuDAG/qudag-wasm/wasm_sections.txt`: Section analysis

## Example Usage
```bash
# Build all targets with crypto features
/build/wasm all crypto-only release

# Build web target only with full features
/build/wasm web full release

# Debug build for development
/build/wasm web crypto-only debug
```

### Example Output
```
Building QuDAG WASM bindings...
✓ Building for web target (crypto-only)... 1.2MB
✓ Building for Node.js target (crypto-only)... 1.1MB  
✓ Building for bundler target (crypto-only)... 1.0MB
✓ Optimizing with wasm-opt... -15% size reduction
✓ Generating TypeScript definitions... Done

Build Summary:
- Web build: pkg/qudag_wasm_bg.wasm (1.0MB)
- Node.js build: pkg-node/qudag_wasm_bg.wasm (950KB)
- TypeScript: pkg/qudag_wasm.d.ts
```

## Related Commands
- `/test/wasm`: Test WASM builds in different environments
- `/deploy/npm`: Publish WASM package to NPM
- `/dev/tools`: Development server with auto-rebuild

## Workflow Integration
This command is part of the WASM Development workflow and:
- Follows: Feature implementation in `/workspaces/QuDAG/qudag-wasm/src/`
- Precedes: `/test/wasm` for validation
- Can be run in parallel with: `/build/cargo` for native builds

## Agent Coordination
- **Primary Agent**: WASM Build Agent
- **Supporting Agents**: 
  - Crypto Agent: Validates cryptographic feature compilation
  - Performance Agent: Analyzes build sizes and optimization
  - Test Agent: Validates build outputs

## Notes
- WASM builds require pure Rust dependencies (no C bindings)
- Size optimization is critical for web deployment
- Different targets have different JavaScript API styles
- Reference types feature requires modern browser support
- Consider lazy loading for large WASM modules

---

## Advanced Build Configurations

### Custom Feature Combinations
```bash
# Vault + Crypto build
wasm-pack build --target web --features "crypto-only,vault"

# DAG + Network build (when available)
wasm-pack build --target web --features "dag,crypto-only"
```

### Debug Builds
```bash
# Debug with symbols for profiling
wasm-pack build --target web --dev --debug --out-dir pkg-debug

# Profile-guided optimization
wasm-pack build --target web --release --profiling
```

### Cross-Platform Considerations
- **Browser compatibility**: Check for reference types support
- **Node.js versions**: Ensure WASM imports work with target Node version
- **Bundler integration**: Test with Webpack, Rollup, Vite
- **Memory limits**: Consider WASM memory constraints in browsers