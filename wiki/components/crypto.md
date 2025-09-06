# QuDAG Cryptographic Primitives

QuDAG implements a comprehensive suite of quantum-resistant cryptographic primitives designed to secure communications and data in the post-quantum era. All implementations follow NIST standards and include constant-time operations for side-channel resistance.

## Overview

The cryptographic module (`core/crypto`) provides:

- **Post-Quantum Key Encapsulation**: ML-KEM-768 (Kyber)
- **Post-Quantum Signatures**: ML-DSA (Dilithium) 
- **Code-Based Encryption**: HQC (128/192/256-bit security)
- **Quantum-Resistant Hashing**: BLAKE3
- **Content Authentication**: Quantum fingerprinting
- **Memory Security**: Automatic zeroization and constant-time operations

## Post-Quantum Key Encapsulation (ML-KEM-768)

ML-KEM-768 provides quantum-resistant key establishment based on the hardness of solving lattice problems.

### Usage

```rust
use qudag_crypto::MlKem768;

// Generate keypair
let (public_key, secret_key) = MlKem768::generate_keypair()?;

// Encapsulate (sender side)
let (ciphertext, shared_secret) = MlKem768::encapsulate(&public_key)?;

// Decapsulate (receiver side)  
let shared_secret_recv = MlKem768::decapsulate(&secret_key, &ciphertext)?;

assert_eq!(shared_secret, shared_secret_recv);
```

### Security Properties

| Property | Value | Notes |
|----------|-------|-------|
| **Security Level** | NIST Level 3 | ~192-bit classical security |
| **Public Key Size** | 1,184 bytes | Compact for post-quantum |
| **Secret Key Size** | 2,400 bytes | Includes public key |
| **Ciphertext Size** | 1,088 bytes | Encapsulated key |
| **Shared Secret** | 32 bytes | Standard AES-256 key size |

### Performance Characteristics

```
Operation        | Time      | Throughput
----------------|-----------|------------
Key Generation  | 1.94ms    | 516 ops/sec
Encapsulation   | 0.89ms    | 1,124 ops/sec  
Decapsulation   | 1.12ms    | 893 ops/sec
```

### Implementation Details

- **Algorithm**: ML-KEM-768 (FIPS 203 compliant)
- **Memory Safety**: Automatic secret key zeroization
- **Side-Channel Protection**: Constant-time operations
- **Hardware Acceleration**: AVX2/NEON SIMD when available

## Post-Quantum Digital Signatures (ML-DSA)

ML-DSA (formerly Dilithium) provides quantum-resistant digital signatures based on lattice problems.

### Usage

```rust
use qudag_crypto::MlDsa;

// Generate signing keypair
let keypair = MlDsa::generate_keypair()?;

// Sign a message
let message = b"Hello, quantum-resistant world!";
let signature = keypair.sign(message)?;

// Verify signature
let is_valid = keypair.verify(&signature, message)?;
assert!(is_valid);

// Public verification (without secret key)
let is_valid = MlDsa::verify(&keypair.public_key(), &signature, message)?;
assert!(is_valid);
```

### Security Properties

| Property | Value | Notes |
|----------|-------|-------|
| **Security Level** | NIST Level 3 | ~192-bit classical security |
| **Public Key Size** | 1,952 bytes | Verification key |
| **Secret Key Size** | 4,000 bytes | Signing key |
| **Signature Size** | 3,293 bytes | Variable length |
| **Hash Function** | SHAKE-256 | Part of ML-DSA spec |

### Performance Characteristics

```
Operation        | Time      | Throughput
----------------|-----------|------------
Key Generation  | 2.45ms    | 408 ops/sec
Signing         | 1.78ms    | 562 ops/sec
Verification    | 0.187ms   | 5,348 ops/sec
```

### Advanced Features

```rust
// Batch signature verification (more efficient)
let messages = vec![msg1, msg2, msg3];
let signatures = vec![sig1, sig2, sig3];
let public_keys = vec![pk1, pk2, pk3];

let all_valid = MlDsa::batch_verify(&public_keys, &signatures, &messages)?;

// Deterministic signing (same signature for same message)
let signature1 = keypair.sign_deterministic(message)?;
let signature2 = keypair.sign_deterministic(message)?;
assert_eq!(signature1, signature2);
```

## Code-Based Encryption (HQC)

HQC provides quantum-resistant encryption based on error-correcting codes, offering multiple security levels.

### Usage

```rust
use qudag_crypto::{Hqc128, Hqc192, Hqc256};

// HQC-128 (fastest)
let (public_key, secret_key) = Hqc128::generate_keypair()?;
let plaintext = b"Secret message";
let ciphertext = Hqc128::encrypt(&public_key, plaintext)?;
let decrypted = Hqc128::decrypt(&secret_key, &ciphertext)?;
assert_eq!(plaintext, &decrypted[..]);

// HQC-256 (highest security)
let (public_key, secret_key) = Hqc256::generate_keypair()?;
let ciphertext = Hqc256::encrypt(&public_key, plaintext)?;
let decrypted = Hqc256::decrypt(&secret_key, &ciphertext)?;
```

### Security Levels

| Variant | Classical Security | Quantum Security | Public Key | Secret Key | Ciphertext Expansion |
|---------|-------------------|------------------|------------|------------|----------------------|
| **HQC-128** | 128-bit | 64-bit | 2,249 bytes | 2,289 bytes | ~3x |
| **HQC-192** | 192-bit | 96-bit | 4,522 bytes | 4,562 bytes | ~3x |
| **HQC-256** | 256-bit | 128-bit | 7,245 bytes | 7,285 bytes | ~3x |

### Use Cases

```rust
// Encrypt large files efficiently
use qudag_crypto::{Hqc256, ChaCha20Poly1305};

// Hybrid encryption: HQC for key, ChaCha20 for data
let (hqc_pk, hqc_sk) = Hqc256::generate_keypair()?;
let data_key = ChaCha20Poly1305::generate_key();

// Encrypt the data with ChaCha20
let encrypted_data = ChaCha20Poly1305::encrypt(&data_key, large_file_data)?;

// Encrypt the data key with HQC
let encrypted_key = Hqc256::encrypt(&hqc_pk, &data_key)?;

// Send both encrypted_key and encrypted_data
```

## Quantum-Resistant Hashing (BLAKE3)

BLAKE3 provides fast, secure hashing resistant to quantum cryptanalysis.

### Usage

```rust
use qudag_crypto::Blake3Hash;

// Simple hashing
let message = b"Hello BLAKE3";
let hash = Blake3Hash::hash(message);
println!("Hash: {}", hex::encode(&hash));

// Streaming hash for large data
let mut hasher = Blake3Hash::new();
hasher.update(b"chunk1");
hasher.update(b"chunk2");
hasher.update(b"chunk3");
let final_hash = hasher.finalize();

// Keyed hashing (HMAC equivalent)
let key = Blake3Hash::generate_key();
let keyed_hash = Blake3Hash::keyed_hash(&key, message);

// Key derivation
let derived_key = Blake3Hash::derive_key("context", &key, 32)?;
```

### Performance Characteristics

```
Operation       | Time      | Throughput
----------------|-----------|------------------
Hash (1KB)      | 0.043ms   | 23,256 ops/sec
Hash (1MB)      | 0.89ms    | 1,124 MB/sec
Stream Update   | 0.015ms   | Variable
Key Derivation  | 0.052ms   | 19,230 ops/sec
```

### Security Properties

- **Output Size**: 256 bits (32 bytes) by default
- **Variable Output**: 1 to 2^64 bytes
- **Collision Resistance**: 2^128 operations
- **Preimage Resistance**: 2^256 operations  
- **Quantum Resistance**: Grover's algorithm → 2^128 security

## Quantum Fingerprinting

Quantum fingerprinting combines BLAKE3 hashing with ML-DSA signatures for content authentication.

### Usage

```rust
use qudag_crypto::QuantumFingerprint;

// Create fingerprint for content
let content = b"Important document content";
let keypair = MlDsa::generate_keypair()?;
let fingerprint = QuantumFingerprint::create(&keypair, content)?;

// Verify fingerprint
let is_authentic = fingerprint.verify(content)?;
assert!(is_authentic);

// Fingerprint contains both hash and signature
println!("Hash: {}", hex::encode(&fingerprint.hash));
println!("Signature: {}", hex::encode(&fingerprint.signature));
```

### Structure

```rust
pub struct QuantumFingerprint {
    pub hash: Blake3Hash,           // BLAKE3 content hash
    pub signature: MlDsaSignature,  // ML-DSA signature of hash
    pub timestamp: u64,             // Creation timestamp
    pub algorithm: String,          // "BLAKE3+ML-DSA" 
}
```

### Use Cases

```rust
// File integrity verification
let file_content = std::fs::read("important.pdf")?;
let fingerprint = QuantumFingerprint::create(&keypair, &file_content)?;

// Store fingerprint separately
std::fs::write("important.pdf.fingerprint", bincode::serialize(&fingerprint)?)?;

// Later verification
let stored_fingerprint: QuantumFingerprint = 
    bincode::deserialize(&std::fs::read("important.pdf.fingerprint")?)?;
let current_content = std::fs::read("important.pdf")?;
let is_unchanged = stored_fingerprint.verify(&current_content)?;
```

## Memory Security

All cryptographic implementations include automatic memory protection:

### Automatic Zeroization

```rust
use zeroize::Zeroize;

// Secret keys automatically zero on drop
{
    let secret_key = MlKem768::generate_keypair()?.1;
    // ... use secret key ...
} // secret_key memory automatically zeroed here

// Manual zeroization when needed
let mut sensitive_data = vec![1, 2, 3, 4];
sensitive_data.zeroize(); // Memory cleared
```

### Constant-Time Operations

All implementations use constant-time operations to prevent timing side-channel attacks:

```rust
// These operations take the same time regardless of input values
let is_equal = subtle::ConstantTimeEq::ct_eq(&hash1, &hash2);
let choice = subtle::Choice::from(condition as u8);
let result = subtle::ConditionallySelectable::conditional_select(&a, &b, choice);
```

## Configuration and Customization

### Algorithm Selection

```rust
// Choose crypto algorithms at runtime
use qudag_crypto::{CryptoConfig, CryptoProvider};

let config = CryptoConfig {
    kem_algorithm: "ML-KEM-768",
    signature_algorithm: "ML-DSA",
    hash_algorithm: "BLAKE3",
    symmetric_cipher: "ChaCha20Poly1305",
};

let crypto = CryptoProvider::new(config)?;
```

### Performance Tuning

```rust
// Enable hardware acceleration
let config = CryptoConfig {
    enable_simd: true,
    enable_parallel: true,
    thread_pool_size: 4,
    batch_size: 100,
};
```

## Integration Examples

### Network Transport Security

```rust
// Secure channel establishment
use qudag_crypto::{MlKem768, ChaCha20Poly1305};

// Key exchange
let (kem_pk, kem_sk) = MlKem768::generate_keypair()?;
let (ciphertext, shared_secret) = MlKem768::encapsulate(&kem_pk)?;

// Derive encryption keys
let key_material = Blake3Hash::derive_key("QuDAG-v1", &shared_secret, 64)?;
let encrypt_key = &key_material[..32];
let mac_key = &key_material[32..];

// Secure communication
let cipher = ChaCha20Poly1305::new(encrypt_key);
let encrypted_message = cipher.encrypt(b"Hello, secure world!")?;
```

### DAG Vertex Signing

```rust
// Sign DAG vertices
use qudag_crypto::{MlDsa, Blake3Hash};

let signing_key = MlDsa::generate_keypair()?;

// Create vertex data
let vertex_data = VertexData {
    id: VertexId::new(),
    parents: vec![parent1_id, parent2_id],
    payload: message_data,
    timestamp: SystemTime::now(),
};

// Sign vertex
let vertex_hash = Blake3Hash::hash(&bincode::serialize(&vertex_data)?);
let signature = signing_key.sign(&vertex_hash)?;

let signed_vertex = SignedVertex {
    data: vertex_data,
    signature,
    public_key: signing_key.public_key(),
};
```

This cryptographic foundation ensures QuDAG remains secure even in the presence of quantum computers, while maintaining high performance for real-time communication.