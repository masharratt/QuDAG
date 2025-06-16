use super::{AsymmetricEncryption, EncryptionError};
use crate::hqc::{self, SecurityParameter};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Wrapper for HQC public key
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct PublicKey(pub Vec<u8>);

/// Wrapper for HQC secret key  
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey(pub Vec<u8>);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EncryptionError> {
        Ok(PublicKey(bytes.to_vec()))
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// HQC-256 implementation
pub struct Hqc256;

impl Hqc256 {
    pub const PUBLIC_KEY_SIZE: usize = 7245;
    pub const SECRET_KEY_SIZE: usize = 7285;
    pub const CIPHERTEXT_SIZE: usize = 14469;
}

impl AsymmetricEncryption for Hqc256 {
    type PublicKey = PublicKey;
    type SecretKey = SecretKey;
    
    const PUBLIC_KEY_SIZE: usize = 7245;
    const SECRET_KEY_SIZE: usize = 7285;
    const CIPHERTEXT_SIZE: usize = 14469;

    fn keygen() -> Result<(Self::PublicKey, Self::SecretKey), EncryptionError> {
        let (pk, sk) = hqc::Hqc256::keygen()
            .map_err(|_| EncryptionError::EncryptionError)?;
        
        Ok((
            PublicKey(pk.as_bytes()),
            SecretKey(sk.as_bytes())
        ))
    }

    fn encrypt(pk: &Self::PublicKey, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let hqc_pk = hqc::PublicKey::from_bytes(&pk.0)
            .map_err(|_| EncryptionError::EncryptionError)?;
        
        hqc::Hqc256::encrypt(&hqc_pk, data)
            .map_err(|_| EncryptionError::EncryptionError)
    }

    fn decrypt(sk: &Self::SecretKey, ct: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let params = hqc::Parameters::new(SecurityParameter::Hqc256);
        let ct_len = params.ciphertext_len() / 2;
        
        if ct.len() < ct_len * 2 {
            return Err(EncryptionError::DecryptionError);
        }
        
        let u = ct[..ct_len].to_vec();
        let v = ct[ct_len..ct_len * 2].to_vec();
        let hqc_ct = hqc::Ciphertext { u, v, params: params.clone() };
        
        let hqc_sk = hqc::SecretKey {
            x: sk.0[..ct_len].to_vec(),
            y: sk.0[ct_len..].to_vec(),
            params
        };
        
        let hqc_instance = hqc::Hqc::new(SecurityParameter::Hqc256);
        hqc_instance.decrypt(&hqc_ct, &hqc_sk)
            .map_err(|_| EncryptionError::DecryptionError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hqc_256_keygen() {
        let (pk, sk) = Hqc256::keygen().unwrap();
        assert!(pk.as_ref().len() > 0);
        assert!(sk.as_ref().len() > 0);
    }

    #[test]
    fn test_hqc_256_encrypt_decrypt() {
        let (pk, sk) = Hqc256::keygen().unwrap();
        let data = b"test data for HQC256";
        
        let ct = Hqc256::encrypt(&pk, data).unwrap();
        let pt = Hqc256::decrypt(&sk, &ct).unwrap();
        
        // Verify the message was properly encoded/decoded
        assert!(pt.len() >= data.len());
        assert_eq!(&pt[..data.len()], data);
    }
}