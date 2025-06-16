use libfuzzer_sys::fuzz_target;
use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }

    // Fuzz ML-KEM
    if let Ok(key_bytes) = data[..32].try_into() {
        let _ = MLKem::derive_key(&key_bytes);
    }

    // Fuzz ML-DSA
    if data.len() >= 64 {
        if let Ok(signature) = MLDsa::from_bytes(&data[..64]) {
            let _ = MLDsa::verify_signature(&signature);
        }
    }

    // Fuzz HQC
    if data.len() >= 128 {
        let _ = HQC::decrypt(&data[..128]);
    }
});