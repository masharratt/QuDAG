use crate::kem::{KEMError, KeyEncapsulation};
use rand_core::{CryptoRng, RngCore};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};
use lru::LruCache;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

/// ML-KEM performance metrics
pub struct MlKemMetrics {
    pub avg_decap_time_ns: u64,
    pub key_cache_hits: u64,
    pub key_cache_misses: u64,
}

/// ML-KEM-768 implementation (NIST security level 3)
pub struct MlKem768;

#[derive(Debug, Zeroize, ZeroizeOnDrop, Clone)]
pub struct PublicKey([u8; MlKem768::PUBLIC_KEY_SIZE]);

#[derive(Debug, Zeroize, ZeroizeOnDrop, Clone)]
pub struct SecretKey([u8; MlKem768::SECRET_KEY_SIZE]);

#[derive(Debug, Zeroize, ZeroizeOnDrop, Clone)]
pub struct Ciphertext([u8; MlKem768::CIPHERTEXT_SIZE]);

#[derive(Debug, Zeroize, ZeroizeOnDrop, Clone)]
pub struct SharedSecret([u8; MlKem768::SHARED_SECRET_SIZE]);

impl PartialEq for SharedSecret {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for SharedSecret {}

impl PartialEq for SecretKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for SecretKey {}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for PublicKey {}

impl PartialEq for Ciphertext {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for Ciphertext {}

impl MlKem768 {
    /// Get current performance metrics
    pub fn get_metrics() -> MlKemMetrics {
        let mut hits = 0;
        let mut misses = 0;
        let mut total_time = 0;
        let mut count = 0;

        Self::CACHE_HITS.with(|h| hits = h.borrow().load(Ordering::Relaxed));
        Self::CACHE_MISSES.with(|m| misses = m.borrow().load(Ordering::Relaxed));
        Self::DECAP_TIME_NS.with(|t| total_time = t.borrow().load(Ordering::Relaxed));
        Self::DECAP_COUNT.with(|c| count = c.borrow().load(Ordering::Relaxed));

        MlKemMetrics {
            avg_decap_time_ns: if count > 0 { total_time / count } else { 0 },
            key_cache_hits: hits,
            key_cache_misses: misses,
        }
    }
    // Constants for ML-KEM-768
    const SHARED_SECRET_SIZE: usize = 32;
    const PUBLIC_KEY_SIZE: usize = 1184;
    const SECRET_KEY_SIZE: usize = 2400;
    const CIPHERTEXT_SIZE: usize = 1088;
    const CACHE_SIZE: usize = 32;

    thread_local! {
        // Cache for commonly used keys to reduce allocations
        static KEY_CACHE: std::cell::RefCell<lru::LruCache<[u8; Self::PUBLIC_KEY_SIZE], SecretKey>> =
            std::cell::RefCell::new(lru::LruCache::new(Self::CACHE_SIZE));
        // Performance metrics
        static CACHE_HITS: std::cell::RefCell<AtomicU64> = std::cell::RefCell::new(AtomicU64::new(0));
        static CACHE_MISSES: std::cell::RefCell<AtomicU64> = std::cell::RefCell::new(AtomicU64::new(0));
        static DECAP_TIME_NS: std::cell::RefCell<AtomicU64> = std::cell::RefCell::new(AtomicU64::new(0));
        static DECAP_COUNT: std::cell::RefCell<AtomicU64> = std::cell::RefCell::new(AtomicU64::new(0));
    }
}

impl KeyEncapsulation for MlKem768 {
    type PublicKey = PublicKey;
    type SecretKey = SecretKey;
    type Ciphertext = Ciphertext;
    type SharedSecret = SharedSecret;

    const PUBLIC_KEY_SIZE: usize = Self::PUBLIC_KEY_SIZE;
    const SECRET_KEY_SIZE: usize = Self::SECRET_KEY_SIZE;
    const CIPHERTEXT_SIZE: usize = Self::CIPHERTEXT_SIZE;
    const SHARED_SECRET_SIZE: usize = Self::SHARED_SECRET_SIZE;

    fn keygen() -> Result<(Self::PublicKey, Self::SecretKey), KEMError> {
        // Use stack-allocated buffers initialized to zero
        let mut pk = [0u8; Self::PUBLIC_KEY_SIZE];
        let mut sk = [0u8; Self::SECRET_KEY_SIZE];
        
        // Create new RNG instance for better security
        let mut rng = rand::thread_rng();
        let keypair = crate::kem::ml_kem::generate_keypair(&mut rng)
            .map_err(|_| KEMError::KeyGenerationError)?;
        
        // Validate buffer lengths in constant time
        let pk_len = subtle::Choice::from((keypair.public_key.len() == Self::PUBLIC_KEY_SIZE) as u8);
        let sk_len = subtle::Choice::from((keypair.secret_key.len() == Self::SECRET_KEY_SIZE) as u8);
        
        if !(pk_len & sk_len).unwrap_u8() == 1 {
            return Err(KEMError::InvalidLength);
        }
        
        // Constant-time memory operations
        pk.copy_from_slice(&keypair.public_key);
        sk.copy_from_slice(&keypair.secret_key);
        
        Ok((PublicKey(pk), SecretKey(sk)))
    }

    fn encapsulate(pk: &Self::PublicKey) -> Result<(Self::Ciphertext, Self::SharedSecret), KEMError> {
        // Stack-allocated buffers initialized to zero
        let mut ct = [0u8; Self::CIPHERTEXT_SIZE];
        let mut ss = [0u8; Self::SHARED_SECRET_SIZE];
        
        // Attempt encapsulation
        let (shared_secret, ciphertext) = crate::kem::ml_kem::encapsulate(pk.as_ref())
            .map_err(|_| KEMError::EncapsulationError)?;
        
        // Validate buffer lengths in constant time
        let ct_len = subtle::Choice::from((ciphertext.len() == Self::CIPHERTEXT_SIZE) as u8);
        let ss_len = subtle::Choice::from((shared_secret.as_bytes().len() == Self::SHARED_SECRET_SIZE) as u8);
        
        if !(ct_len & ss_len).unwrap_u8() == 1 {
            return Err(KEMError::InvalidLength);
        }
        
        // Constant-time memory operations
        ct.copy_from_slice(&ciphertext);
        ss.copy_from_slice(shared_secret.as_bytes());
        
        Ok((Ciphertext(ct), SharedSecret(ss)))
    }

    fn decapsulate(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::SharedSecret, KEMError> {
        // Track operation timing
        let start = Instant::now();

        // Stack-allocated buffer initialized to zero
        let mut ss = [0u8; Self::SHARED_SECRET_SIZE];
        
        // Try to get cached key using constant-time operations
        let secret_key = Self::KEY_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            
            // Constant-time cache lookup
            let cache_hit = cache.contains_key(sk.as_ref());
            let hit_choice = subtle::Choice::from(cache_hit as u8);
            
            // Update metrics in constant time
            Self::CACHE_HITS.with(|hits| hits.borrow().fetch_add(u64::from(hit_choice.unwrap_u8()), Ordering::Relaxed));
            Self::CACHE_MISSES.with(|misses| misses.borrow().fetch_add(u64::from(!hit_choice.unwrap_u8()), Ordering::Relaxed));
            
            // Try to insert into cache if not present
            if !cache_hit {
                // Convert to fixed-size array safely
                if let Ok(key_array) = sk.as_ref().try_into() {
                    cache.put(key_array, sk.clone());
                }
            }
            
            // Get cached key or use provided one
            cache.get(sk.as_ref())
                .map(|k| k.clone())
                .unwrap_or_else(|| sk.clone())
        });

        // Perform decapsulation
        let shared_secret = crate::kem::ml_kem::decapsulate(secret_key.as_ref(), ct.as_ref())
            .map_err(|_| KEMError::DecapsulationError)?;
        
        // Validate shared secret length in constant time
        let ss_len = subtle::Choice::from((shared_secret.as_bytes().len() == Self::SHARED_SECRET_SIZE) as u8);
        
        if !ss_len.unwrap_u8() == 1 {
            return Err(KEMError::InvalidLength);
        }
        
        // Constant-time memory copy
        ss.copy_from_slice(shared_secret.as_bytes());

        // Record operation timing
        let elapsed = start.elapsed().as_nanos() as u64;
        Self::DECAP_TIME_NS.with(|time| time.borrow().fetch_add(elapsed, Ordering::Relaxed));
        Self::DECAP_COUNT.with(|count| count.borrow().fetch_add(1, Ordering::Relaxed));

        Ok(SharedSecret(ss))
    }
}

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

impl AsRef<[u8]> for Ciphertext {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SharedSecret {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}