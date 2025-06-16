# Cryptography Module API

The `qudag_crypto` module provides quantum-resistant cryptographic primitives for the QuDAG protocol.

## Key Types

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

## Error Types

### MlDsaError

```rust
pub enum MlDsaError {
    InvalidKeyFormat(String),
    SigningFailed(String),
    VerificationFailed(String),
}
```

## Example Usage

### Generating Keys and Signing

```rust
use qudag_crypto::{MlDsaKeyPair, MlDsaError};

// Generate a new key pair
let keypair = MlDsaKeyPair::generate()?;

// Sign a message
let message = b"Hello, world!";
let signature = keypair.sign(message)?;

// Verify the signature
keypair.verify(message, &signature)?;
```

### Error Handling

```rust
use qudag_crypto::MlDsaError;

fn handle_signature(keypair: &MlDsaKeyPair, message: &[u8]) -> Result<(), MlDsaError> {
    let signature = keypair.sign(message)?;
    
    match keypair.verify(message, &signature) {
        Ok(()) => println!("Signature valid"),
        Err(MlDsaError::VerificationFailed(e)) => {
            eprintln!("Signature verification failed: {}", e);
            return Err(e.into());
        }
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

## Security Considerations

1. **Memory Management**
   - All secret key material is automatically zeroized when dropped
   - Memory is securely cleared after sensitive operations
   - Avoid logging or printing sensitive key material

2. **Constant-time Operations**
   - All cryptographic operations are implemented to be constant-time
   - No branch conditions based on secret data
   - Protected against timing side-channel attacks

3. **Key Generation**
   - Always use the provided `generate()` method for key generation
   - Never reuse key pairs across different contexts
   - Properly secure private keys at rest

## Configuration

The cryptographic parameters are preset to provide quantum security level equivalent to AES-256. No additional configuration is required for standard usage.