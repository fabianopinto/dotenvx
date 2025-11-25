use crate::crypto::keypair::Keypair;
use crate::utils::error::{DotenvxError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use secp256k1::{ecdh::SharedSecret, PublicKey};
use sha2::Sha256;

const ENCRYPTED_PREFIX: &str = "encrypted:";
const AES_KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

/// Encrypt a value using ECIES (Elliptic Curve Integrated Encryption Scheme)
///
/// # Arguments
///
/// * `plaintext` - The value to encrypt
/// * `public_key_hex` - The 66-character hex-encoded public key
///
/// # Returns
///
/// The encrypted value prefixed with "encrypted:" and base64-encoded
///
/// # Example
///
/// ```
/// use dotenvx::crypto::{Keypair, encrypt};
///
/// let keypair = Keypair::generate();
/// let encrypted = encrypt("Hello, World!", &keypair.public_key()).unwrap();
/// assert!(encrypted.starts_with("encrypted:"));
/// ```
pub fn encrypt(plaintext: &str, public_key_hex: &str) -> Result<String> {
    let keypair = Keypair::from_public_key(public_key_hex)?;
    let recipient_public_key = keypair.public_key_raw();

    // Generate ephemeral keypair
    let ephemeral_keypair = Keypair::generate();
    let ephemeral_secret = ephemeral_keypair.secret_key();
    let ephemeral_public = ephemeral_keypair.public_key_raw();

    // Compute shared secret using ECDH
    let shared_secret = SharedSecret::new(recipient_public_key, ephemeral_secret);

    // Derive AES key using HKDF
    let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_ref());
    let mut aes_key = [0u8; AES_KEY_SIZE];
    hkdf.expand(b"dotenvx-ecies-aes", &mut aes_key)
        .map_err(|e| DotenvxError::EncryptionFailed(format!("HKDF expand failed: {}", e)))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt using AES-256-GCM
    let cipher = Aes256Gcm::new(&aes_key.into());
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| DotenvxError::EncryptionFailed(format!("AES encryption failed: {}", e)))?;

    // Combine: ephemeral_public_key (33 bytes) || nonce (12 bytes) || ciphertext
    let ephemeral_public_bytes = ephemeral_public.serialize();
    let mut combined = Vec::new();
    combined.extend_from_slice(&ephemeral_public_bytes);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    // Base64 encode and add prefix
    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(&combined);
    Ok(format!("{}{}", ENCRYPTED_PREFIX, encoded))
}

/// Decrypt a value using ECIES
///
/// # Arguments
///
/// * `encrypted` - The encrypted value (with "encrypted:" prefix)
/// * `private_key_hex` - The 64-character hex-encoded private key
///
/// # Returns
///
/// The decrypted plaintext value
///
/// # Example
///
/// ```
/// use dotenvx::crypto::{Keypair, encrypt, decrypt};
///
/// let keypair = Keypair::generate();
/// let plaintext = "Hello, World!";
/// let encrypted = encrypt(plaintext, &keypair.public_key()).unwrap();
/// let decrypted = decrypt(&encrypted, &keypair.private_key()).unwrap();
/// assert_eq!(decrypted, plaintext);
/// ```
pub fn decrypt(encrypted: &str, private_key_hex: &str) -> Result<String> {
    // Check for encrypted prefix
    if !encrypted.starts_with(ENCRYPTED_PREFIX) {
        return Ok(encrypted.to_string());
    }

    let encoded = &encrypted[ENCRYPTED_PREFIX.len()..];
    use base64::{engine::general_purpose, Engine as _};
    let combined = general_purpose::STANDARD.decode(encoded).map_err(|_| {
        DotenvxError::MalformedEncryptedData {
            key: "unknown".to_string(),
        }
    })?;

    // Parse: ephemeral_public_key (33 bytes) || nonce (12 bytes) || ciphertext
    if combined.len() < 33 + NONCE_SIZE {
        return Err(DotenvxError::MalformedEncryptedData {
            key: "unknown".to_string(),
        });
    }

    let ephemeral_public_bytes = &combined[..33];
    let nonce_bytes = &combined[33..33 + NONCE_SIZE];
    let ciphertext = &combined[33 + NONCE_SIZE..];

    // Parse ephemeral public key
    let ephemeral_public = PublicKey::from_slice(ephemeral_public_bytes).map_err(|e| {
        DotenvxError::MalformedEncryptedData {
            key: format!("invalid ephemeral public key: {}", e),
        }
    })?;

    // Get recipient's private key
    let keypair = Keypair::from_private_key(private_key_hex)?;
    let recipient_secret = keypair.secret_key();

    // Compute shared secret using ECDH
    let shared_secret = SharedSecret::new(&ephemeral_public, recipient_secret);

    // Derive AES key using HKDF
    let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_ref());
    let mut aes_key = [0u8; AES_KEY_SIZE];
    hkdf.expand(b"dotenvx-ecies-aes", &mut aes_key)
        .map_err(|_| DotenvxError::DecryptionFailed {
            key: "unknown".to_string(),
            private_key_name: "provided".to_string(),
        })?;

    // Decrypt using AES-256-GCM
    let cipher = Aes256Gcm::new(&aes_key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext_bytes =
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| DotenvxError::DecryptionFailed {
                key: "unknown".to_string(),
                private_key_name: "provided".to_string(),
            })?;

    // Convert to string
    let plaintext =
        String::from_utf8(plaintext_bytes).map_err(|e| DotenvxError::DecryptionFailed {
            key: "unknown".to_string(),
            private_key_name: format!("invalid UTF-8: {}", e),
        })?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let keypair = Keypair::generate();
        let plaintext = "Hello, World!";

        let encrypted = encrypt(plaintext, &keypair.public_key()).unwrap();
        assert!(encrypted.starts_with(ENCRYPTED_PREFIX));

        let decrypted = decrypt(&encrypted, &keypair.private_key()).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_multiple_times_different_output() {
        let keypair = Keypair::generate();
        let plaintext = "test";

        let encrypted1 = encrypt(plaintext, &keypair.public_key()).unwrap();
        let encrypted2 = encrypt(plaintext, &keypair.public_key()).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(
            decrypt(&encrypted1, &keypair.private_key()).unwrap(),
            plaintext
        );
        assert_eq!(
            decrypt(&encrypted2, &keypair.private_key()).unwrap(),
            plaintext
        );
    }

    #[test]
    fn test_decrypt_without_prefix() {
        let plaintext = "not_encrypted";
        let keypair = Keypair::generate();

        let result = decrypt(plaintext, &keypair.private_key()).unwrap();
        assert_eq!(result, plaintext);
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let keypair = Keypair::generate();
        let result = decrypt("encrypted:!!!invalid!!!", &keypair.private_key());
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        let plaintext = "secret";

        let encrypted = encrypt(plaintext, &keypair1.public_key()).unwrap();
        let result = decrypt(&encrypted, &keypair2.private_key());
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let keypair = Keypair::generate();
        let plaintext = "";

        let encrypted = encrypt(plaintext, &keypair.public_key()).unwrap();
        let decrypted = decrypt(&encrypted, &keypair.private_key()).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_long_string() {
        let keypair = Keypair::generate();
        let plaintext = "a".repeat(10000);

        let encrypted = encrypt(&plaintext, &keypair.public_key()).unwrap();
        let decrypted = decrypt(&encrypted, &keypair.private_key()).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_unicode() {
        let keypair = Keypair::generate();
        let plaintext = "Hello, 世界! 🌍";

        let encrypted = encrypt(plaintext, &keypair.public_key()).unwrap();
        let decrypted = decrypt(&encrypted, &keypair.private_key()).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
