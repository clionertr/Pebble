pub mod aes;
pub mod keystore;

use pebble_core::Result;
use zeroize::Zeroizing;

/// Service that manages encryption/decryption using a DEK from the OS keystore.
pub struct CryptoService {
    dek: Zeroizing<[u8; 32]>,
}

impl CryptoService {
    /// Initialize by loading (or creating) the DEK from the given file path.
    pub fn init(key_path: &std::path::Path) -> Result<Self> {
        let dek = keystore::KeyStore::get_or_create_dek(key_path)?;
        Ok(Self { dek })
    }

    /// Encrypt plaintext bytes.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        aes::encrypt(&self.dek, plaintext)
    }

    /// Decrypt ciphertext bytes.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        aes::decrypt(&self.dek, ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_service_init() {
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join("test.key");
        let service = CryptoService::init(&key_path);
        assert!(service.is_ok());
    }

    #[test]
    fn test_crypto_service_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join("test.key");
        let service = CryptoService::init(&key_path).unwrap();
        let plaintext = b"test credentials json";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
