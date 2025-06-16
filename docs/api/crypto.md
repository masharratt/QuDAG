# Cryptography Module API

The `qudag_crypto` module provides quantum-resistant cryptographic primitives for the QuDAG protocol. It implements ML-KEM (Key Encapsulation Mechanism), ML-DSA (Digital Signature Algorithm), BLAKE3 hashing, and quantum fingerprinting.

## Key Encapsulation (ML-KEM)

### MlKem768

ML-KEM-768 implementation providing NIST security level 3 (equivalent to AES-256).

```rust
pub struct MlKem768;

impl MlKem768 {
    pub const PUBLIC_KEY_SIZE: usize = 1184;
    pub const SECRET_KEY_SIZE: usize = 2400;
    pub const CIPHERTEXT_SIZE: usize = 1088;
    pub const SHARED_SECRET_SIZE: usize = 32;
}
```

### ML-KEM Key Types

```rust
pub struct PublicKey([u8; MlKem768::PUBLIC_KEY_SIZE]);
pub struct SecretKey([u8; MlKem768::SECRET_KEY_SIZE]);
pub struct Ciphertext([u8; MlKem768::CIPHERTEXT_SIZE]);
pub struct SharedSecret([u8; MlKem768::SHARED_SECRET_SIZE]);
```

### KeyEncapsulation Trait

```rust
pub trait KeyEncapsulation {
    type PublicKey;
    type SecretKey;
    type Ciphertext;
    type SharedSecret;
    type Error;

    fn keygen(&self) -> Result<(Self::PublicKey, Self::SecretKey), Self::Error>;
    fn encap(&self, pk: &Self::PublicKey) -> Result<(Self::Ciphertext, Self::SharedSecret), Self::Error>;
    fn decap(&self, sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::SharedSecret, Self::Error>;
}
```

### Performance Metrics

```rust
pub struct MlKemMetrics {
    pub avg_decap_time_ns: u64,
    pub key_cache_hits: u64,
    pub key_cache_misses: u64,
}
```

## Digital Signatures (ML-DSA)

### MlDsaKeyPair

A key pair for quantum-resistant digital signatures using ML-DSA.

```rust
pub struct MlDsaKeyPair {
    // private fields
}

impl MlDsaKeyPair {
    pub fn generate() -> Result<Self, MlDsaError>;
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, MlDsaError>;
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), MlDsaError>;
}
```

### MlDsaPublicKey

The public portion of an ML-DSA key pair used for signature verification.

```rust
pub struct MlDsaPublicKey {
    // private fields
}
```

## Hash Functions

### BLAKE3 Implementation

```rust
pub trait HashFunction {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn hash_with_key(&self, data: &[u8], key: &[u8]) -> Vec<u8>;
    fn verify_hash(&self, data: &[u8], hash: &[u8]) -> bool;
}
```

## Quantum Fingerprinting

### QuantumFingerprint

Data fingerprinting using ML-DSA for authentication.

```rust
pub struct QuantumFingerprint {
    // Implementation details
}

impl QuantumFingerprint {
    pub fn new() -> Self;
    pub fn fingerprint(&self, data: &[u8]) -> Result<Vec<u8>, FingerprintError>;
    pub fn verify(&self, data: &[u8], fingerprint: &[u8]) -> Result<bool, FingerprintError>;
}
```

## Error Types

### KEMError

```rust
pub enum KEMError {
    KeyGenError,
    EncapsulationError,
    DecapsulationError,
    InvalidKey,
    InvalidParameters,
    OperationFailed,
    InternalError,
}
```

### MlDsaError

```rust
pub enum MlDsaError {
    InvalidKeyFormat(String),
    SigningFailed(String),
    VerificationFailed(String),
}
```

### CryptoError

Main error type for the crypto module:

```rust
pub enum CryptoError {
    KemError(KEMError),
    SignatureError(SignatureError),
    HashError(String),
    FingerprintError(String),
}
```

## Example Usage

### ML-KEM Key Encapsulation

```rust
use qudag_crypto::{MlKem768, KeyEncapsulation};

// Initialize ML-KEM
let kem = MlKem768;

// Generate key pair
let (public_key, secret_key) = kem.keygen()?;

// Encapsulation (sender side)
let (ciphertext, shared_secret1) = kem.encap(&public_key)?;

// Decapsulation (receiver side)
let shared_secret2 = kem.decap(&secret_key, &ciphertext)?;

// Both sides now have the same shared secret
assert_eq!(shared_secret1, shared_secret2);
```

### ML-DSA Digital Signatures

```rust
use qudag_crypto::{MlDsaKeyPair, MlDsaError};

// Generate a new key pair
let keypair = MlDsaKeyPair::generate()?;

// Sign a message
let message = b"Hello, quantum-resistant world!";
let signature = keypair.sign(message)?;

// Verify the signature
keypair.verify(message, &signature)?;
```

### Quantum Fingerprinting

```rust
use qudag_crypto::QuantumFingerprint;

// Create fingerprint generator
let fingerprinter = QuantumFingerprint::new();

// Generate fingerprint for data
let data = b"Important protocol data";
let fingerprint = fingerprinter.fingerprint(data)?;

// Verify fingerprint later
let is_valid = fingerprinter.verify(data, &fingerprint)?;
assert!(is_valid);
```

### BLAKE3 Hashing

```rust
use qudag_crypto::HashFunction;

// Create BLAKE3 hasher
let hasher = Blake3::new();

// Hash data
let data = b"Data to hash";
let hash = hasher.hash(data);

// Hash with key for authentication
let key = b"secret key for authenticated hashing";
let auth_hash = hasher.hash_with_key(data, key);

// Verify hash
let is_valid = hasher.verify_hash(data, &hash);
assert!(is_valid);
```

### Error Handling

```rust
use qudag_crypto::{CryptoError, KEMError, MlDsaError};

fn handle_crypto_operations() -> Result<(), CryptoError> {
    let kem = MlKem768;
    
    match kem.keygen() {
        Ok((pk, sk)) => {
            // Use keys
            let (ct, ss) = kem.encap(&pk)?;
            let ss2 = kem.decap(&sk, &ct)?;
            assert_eq!(ss, ss2);
        }
        Err(KEMError::KeyGenError) => {
            eprintln!("Key generation failed - check entropy source");
            return Err(CryptoError::KemError(KEMError::KeyGenError));
        }
        Err(e) => return Err(CryptoError::KemError(e)),
    }
    
    Ok(())
}
```

### Performance Monitoring

```rust
use qudag_crypto::{MlKem768, MlKemMetrics};

fn monitor_kem_performance() {
    let kem = MlKem768;
    let (pk, sk) = kem.keygen().unwrap();
    
    // Perform operations and collect metrics
    let start = std::time::Instant::now();
    let (ct, _) = kem.encap(&pk).unwrap();
    let _ = kem.decap(&sk, &ct).unwrap();
    let elapsed = start.elapsed();
    
    println!("KEM operation took: {:?}", elapsed);
    
    // Access metrics if available
    let metrics = kem.get_metrics(); // hypothetical method
    println!("Cache hits: {}", metrics.key_cache_hits);
    println!("Average decap time: {}ns", metrics.avg_decap_time_ns);
}
```

## Security Considerations

### 1. Memory Management

All cryptographic types implement `ZeroizeOnDrop` to ensure sensitive data is securely cleared:

- **Secret Keys**: Automatically zeroized when dropped
- **Shared Secrets**: Memory cleared after use
- **Intermediate Values**: Temporary crypto values are zeroized
- **Error Handling**: No sensitive data leaked in error messages

```rust
use zeroize::Zeroize;

// Secret data is automatically cleared
{
    let secret_key = kem.keygen()?.1;
    // ... use secret key
} // secret_key memory is zeroized here
```

### 2. Constant-Time Operations

All implementations use constant-time algorithms to prevent timing attacks:

- **ML-KEM Operations**: Decapsulation runs in constant time
- **ML-DSA Signatures**: Signing and verification are constant-time
- **Comparisons**: All secret comparisons use `subtle::ConstantTimeEq`
- **Side-Channel Resistance**: No conditional branches on secret data

### 3. Post-Quantum Security

The implemented algorithms provide security against both classical and quantum attacks:

- **ML-KEM-768**: NIST security level 3 (equivalent to AES-256)
- **ML-DSA**: Quantum-resistant digital signatures
- **BLAKE3**: Quantum-resistant cryptographic hashing
- **Forward Secrecy**: Each session uses fresh key material

### 4. Key Management

Best practices for key handling:

```rust
// Good: Generate fresh keys for each session
let (pk, sk) = kem.keygen()?;

// Bad: Reusing keys across contexts
// Don't do this - generate fresh keys instead

// Good: Clear sensitive data explicitly if needed
let mut sensitive_data = get_sensitive_data();
sensitive_data.zeroize();
```

### 5. Random Number Generation

All key generation uses cryptographically secure randomness:

- **Entropy Source**: Uses system entropy for key generation
- **CSPRNG**: Cryptographically secure pseudo-random generation
- **Seed Security**: No predictable or weak entropy sources

### 6. Algorithm Parameters

Current cryptographic parameters provide quantum security:

```rust
// ML-KEM-768 parameters (security level 3)
MlKem768::PUBLIC_KEY_SIZE   // 1184 bytes
MlKem768::SECRET_KEY_SIZE   // 2400 bytes  
MlKem768::CIPHERTEXT_SIZE   // 1088 bytes
MlKem768::SHARED_SECRET_SIZE // 32 bytes

// Security equivalent to AES-256 against quantum attacks
```

### 7. Error Handling Security

Error messages are designed to prevent information leakage:

```rust
// Good: Generic error messages
match crypto_operation() {
    Err(CryptoError::KemError(_)) => {
        // Log internally but don't expose details
        log::warn!("Cryptographic operation failed");
        return Err("Operation failed");
    }
}
```

### 8. Performance vs Security

The implementation prioritizes security over performance:

- **Constant-Time**: All operations run in constant time
- **Memory Clearing**: Extra cycles spent clearing sensitive memory
- **Side-Channel Resistance**: Additional protections against attacks
- **Quantum Security**: Larger key sizes for post-quantum resistance

## Configuration

### Default Security Parameters

The cryptographic parameters are preset for maximum security:

```rust
// Default configuration provides:
// - NIST security level 3 (equivalent to AES-256)
// - Quantum resistance for both encryption and signatures
// - Side-channel attack resistance
// - Forward secrecy for session keys
```

### Performance Tuning

For performance-critical applications, monitor metrics:

```rust
// Enable performance monitoring
let metrics = kem.get_metrics();
if metrics.avg_decap_time_ns > PERFORMANCE_THRESHOLD {
    // Consider caching or optimization
}
```

### Integration Guidelines

When integrating with the QuDAG protocol:

1. **Use ML-KEM** for key establishment between nodes
2. **Use ML-DSA** for message authentication and non-repudiation  
3. **Use BLAKE3** for general hashing and data integrity
4. **Use Quantum Fingerprinting** for data authentication in DAG
5. **Generate fresh keys** for each session or communication channel