use pebble_core::{PebbleError, Result};
use rand::RngCore;
use tracing::info;
use zeroize::Zeroizing;
use std::path::Path;

pub struct KeyStore;

impl KeyStore {
    /// Get or create the Data Encryption Key from the given file.
    pub fn get_or_create_dek(key_path: &Path) -> Result<Zeroizing<[u8; 32]>> {
        if key_path.exists() {
            let secret = std::fs::read(key_path)
                .map_err(|e| PebbleError::Auth(format!("Failed to read DEK file: {e}")))?;
            let secret = Zeroizing::new(secret);
            if secret.len() != 32 {
                return Err(PebbleError::Auth(format!(
                    "Invalid DEK length in file: expected 32, got {}",
                    secret.len()
                )));
            }
            let mut key = Zeroizing::new([0u8; 32]);
            key.copy_from_slice(&secret);
            Ok(key)
        } else {
            info!("No DEK file found, generating new one at {:?}", key_path);
            let mut key = Zeroizing::new([0u8; 32]);
            rand::thread_rng().fill_bytes(&mut *key);
            
            if let Some(parent) = key_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| PebbleError::Auth(format!("Failed to create DEK directory: {e}")))?;
            }
            
            std::fs::write(key_path, *key)
                .map_err(|e| PebbleError::Auth(format!("Failed to write DEK file: {e}")))?;
                
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(key_path)
                    .map_err(|e| PebbleError::Auth(format!("Failed to get DEK metadata: {e}")))?
                    .permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(key_path, perms)
                    .map_err(|e| PebbleError::Auth(format!("Failed to set DEK permissions: {e}")))?;
            }
            
            Ok(key)
        }
    }
}
