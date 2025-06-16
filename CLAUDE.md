# QuDAG Protocol - Claude Code Development Guide

## Project Overview

QuDAG (Quantum DAG) is a quantum-resistant DAG-based anonymous communication protocol implemented in Rust using Test-Driven Development (TDD) methodology.

## Architecture

The project follows a modular workspace architecture:

- `core/crypto`: Quantum-resistant cryptographic primitives (ML-KEM, ML-DSA, HQC)
- `core/dag`: DAG consensus implementation with QR-Avalanche algorithm
- `core/network`: P2P networking layer with anonymous routing
- `core/protocol`: Main protocol implementation and coordination
- `tools/cli`: Command-line interface for node operation
- `tools/simulator`: Network simulation for testing and validation
- `benchmarks`: Performance benchmarks and regression testing

## Development Principles

### 1. Test-Driven Development (TDD)

- **RED**: Write failing tests first
- **GREEN**: Implement minimal code to pass tests
- **REFACTOR**: Improve code while keeping tests green

### 2. Security-First Approach

- All cryptographic operations must be constant-time
- Memory must be securely cleared after use
- Side-channel resistance is mandatory

### 3. Performance Awareness

- Profile all critical paths
- Benchmark against performance targets
- Monitor for regressions

### 4. Documentation-Driven Design

- Update documentation with each feature
- Include security considerations
- Provide usage examples

## Testing Strategy

### Unit Tests
- Individual function and struct testing
- Property-based testing with proptest
- Cryptographic primitive validation
- Edge case coverage

### Integration Tests
- Multi-component interaction testing
- Protocol flow validation
- Network behavior testing
- Error condition handling

### Security Tests
- Timing attack resistance
- Side-channel analysis
- Cryptographic compliance
- Adversarial input handling

### Performance Tests
- Throughput benchmarking
- Latency measurement
- Scalability testing
- Resource usage monitoring

## Claude Code Commands

### Primary Development Commands

- `/tdd-cycle <module> <feature>`: Execute complete TDD cycle for a feature
- `/security-audit`: Comprehensive security analysis and testing
- `/performance-benchmark`: Run all benchmarks and generate reports
- `/integration-test`: Execute full integration test suite
- `/deploy-validate`: Validate deployment configuration and test

### Development Workflow Commands

- `/create-test <path> <description>`: Generate test skeleton for new feature
- `/implement-feature <test-path>`: Implement feature to pass specified tests
- `/refactor-optimize <module>`: Refactor module while maintaining test coverage
- `/review-security <module>`: Security-focused code review
- `/update-docs <module>`: Update documentation for module changes

### Specialized Commands

- `/crypto-validate <algorithm>`: Validate cryptographic implementation
- `/network-simulate <scenario>`: Run network simulation scenarios
- `/dag-visualize <state>`: Generate DAG state visualization
- `/fuzz-test <target>`: Execute fuzzing campaign against target

## Multi-Agent Coordination

### Agent Roles

1. **Crypto Agent**: Handles all cryptographic implementations and validations
2. **Network Agent**: Manages P2P networking and communication protocols
3. **Consensus Agent**: Implements and tests DAG consensus mechanisms
4. **Security Agent**: Performs security analysis and vulnerability assessment
5. **Performance Agent**: Monitors and optimizes system performance
6. **Integration Agent**: Coordinates component integration and system testing

### Coordination Protocols

- Use shared context files in `.claude/contexts/` for inter-agent communication
- Maintain test status in `.claude/test-status.json`
- Log all agent activities in `.claude/agent-logs/`

## Code Quality Standards

### Rust Best Practices

- Use `#![deny(unsafe_code)]` except where explicitly needed
- Implement comprehensive error handling with `thiserror`
- Use `tracing` for structured logging
- Follow Rust API guidelines

### Security Requirements

- All crypto operations use constant-time implementations
- Secrets are zeroized on drop
- No debug prints of sensitive data
- Memory allocations are minimized for crypto operations

### Performance Requirements

- Sub-second consensus finality (99th percentile)
- 10,000+ messages/second throughput per node
- Linear scalability with node count
- <100MB memory usage for base node

## Testing Requirements

### Coverage Targets

- Unit test coverage: >90%
- Integration test coverage: >80%
- Security test coverage: 100% of crypto operations
- Performance benchmarks: All critical paths

### Test Categories

- **Functional**: Correctness of implementation
- **Property**: Invariant validation with property-based testing
- **Adversarial**: Resistance to malicious inputs
- **Performance**: Throughput, latency, and resource usage
- **Compatibility**: Interoperability with other implementations

## Deployment Guidelines

### Environment Configuration

- Development: Local testing with simulator
- Staging: Multi-node testnet deployment
- Production: Mainnet with monitoring and alerting

### Security Considerations

- Container image scanning
- Supply chain verification
- Runtime security monitoring
- Incident response procedures

## Troubleshooting

### Common Issues

- Build failures: Check Rust version and dependencies
- Test failures: Verify test data and mock configurations
- Network issues: Check firewall and NAT configurations
- Performance degradation: Profile and check for resource exhaustion

### Debug Commands

- `/debug-network`: Diagnose networking issues
- `/debug-consensus`: Analyze consensus state
- `/debug-performance`: Profile performance bottlenecks
- `/debug-security`: Check security configurations

## Contribution Guidelines

### Code Submission Process

1. Create feature branch from `develop`
2. Implement using TDD methodology
3. Ensure all tests pass and coverage targets met
4. Submit pull request with comprehensive description
5. Address review feedback and security audit results

### Review Criteria

- Code follows TDD principles
- Security requirements are met
- Performance targets are achieved
- Documentation is updated
- Tests provide adequate coverage

## Quick Start Commands

```bash
# Set up the project
./qudag.sh

# Navigate to project
cd qudag-protocol

# Run initial build and tests
cargo build
cargo test

# Run specific module tests
cargo test -p qudag-crypto
cargo test -p qudag-dag
cargo test -p qudag-network

# Run benchmarks
cargo bench

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check
```

## Development Workflow Example

```bash
# Start TDD cycle for a new cryptographic feature
/tdd-cycle crypto ml_kem_implementation

# Run security audit after implementation
/security-audit crypto

# Benchmark performance
/performance-benchmark crypto

# Create integration tests
/create-test tests/integration/crypto_integration_tests.rs "ML-KEM integration with protocol"

# Run full test suite
cargo test --all-features --workspace
```

## Important Notes

- Always run `cargo fmt` before committing
- Use `cargo clippy` to catch common mistakes
- Run security audit on cryptographic changes
- Benchmark performance-critical code paths
- Update documentation with API changes

---

For detailed technical specifications, see `docs/architecture/` directory.
For security considerations, see `docs/security/` directory.
For performance benchmarks, see `benchmarks/` directory.