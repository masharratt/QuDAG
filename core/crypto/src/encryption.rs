use crate::error::CryptoError;
use zeroize::Zeroize;

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Encryption failed")]
    EncryptionError,
    #[error("Decryption failed")]
    DecryptionError,
    #[error(transparent)]
    CryptoError(#[from] CryptoError),
}

pub trait AsymmetricEncryption: Sized {
    type PublicKey: AsRef<[u8]> + Zeroize;
    type SecretKey: AsRef<[u8]> + Zeroize;

    const PUBLIC_KEY_SIZE: usize;
    const SECRET_KEY_SIZE: usize;
    const CIPHERTEXT_SIZE: usize;

    /// Generate a new key pair
    fn keygen() -> Result<(Self::PublicKey, Self::SecretKey), EncryptionError>;

    /// Encrypt a message using a public key
    fn encrypt(pk: &Self::PublicKey, message: &[u8]) -> Result<Vec<u8>, EncryptionError>;

    /// Decrypt a ciphertext using a secret key
    fn decrypt(sk: &Self::SecretKey, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError>;
}