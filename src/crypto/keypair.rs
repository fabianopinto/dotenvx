use crate::utils::error::{DotenvxError, Result};
use secp256k1::{PublicKey, Secp256k1, SecretKey};

/// A keypair containing a public and private key for ECIES encryption
pub struct Keypair {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl Keypair {
    /// Generate a new random keypair
    ///
    /// # Example
    ///
    /// ```
    /// use dotenvx::crypto::Keypair;
    ///
    /// let keypair = Keypair::generate();
    /// println!("Public key: {}", keypair.public_key());
    /// println!("Private key: {}", keypair.private_key());
    /// ```
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        Self {
            secret_key,
            public_key,
        }
    }

    /// Create a keypair from a hex-encoded public key
    ///
    /// # Arguments
    ///
    /// * `public_key_hex` - A 66-character hex-encoded public key (with 02/03 prefix)
    pub fn from_public_key(public_key_hex: &str) -> Result<Self> {
        let public_key = PublicKey::from_slice(&hex::decode(public_key_hex)?)
            .map_err(|e| DotenvxError::InvalidPublicKey(format!("{}", e)))?;

        // Create a dummy secret key (we only have the public key)
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let (secret_key, _) = secp.generate_keypair(&mut rng);

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// Create a keypair from a hex-encoded private key
    ///
    /// # Arguments
    ///
    /// * `private_key_hex` - A 64-character hex-encoded private key
    pub fn from_private_key(private_key_hex: &str) -> Result<Self> {
        let secret_key = SecretKey::from_slice(&hex::decode(private_key_hex)?)
            .map_err(|e| DotenvxError::InvalidPrivateKey(format!("{}", e)))?;

        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// Get the hex-encoded public key (66 characters with 02/03 prefix)
    pub fn public_key(&self) -> String {
        hex::encode(self.public_key.serialize())
    }

    /// Get the hex-encoded private key (64 characters)
    pub fn private_key(&self) -> String {
        hex::encode(self.secret_key.secret_bytes())
    }

    /// Get the raw public key for cryptographic operations
    pub fn public_key_raw(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the raw secret key for cryptographic operations
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = Keypair::generate();
        assert_eq!(keypair.public_key().len(), 66);
        assert_eq!(keypair.private_key().len(), 64);
    }

    #[test]
    fn test_from_private_key() {
        let keypair1 = Keypair::generate();
        let private_key = keypair1.private_key();
        let public_key = keypair1.public_key();

        let keypair2 = Keypair::from_private_key(&private_key).unwrap();
        assert_eq!(keypair2.public_key(), public_key);
        assert_eq!(keypair2.private_key(), private_key);
    }

    #[test]
    fn test_from_public_key() {
        let keypair1 = Keypair::generate();
        let public_key = keypair1.public_key();

        let keypair2 = Keypair::from_public_key(&public_key).unwrap();
        assert_eq!(keypair2.public_key(), public_key);
    }

    #[test]
    fn test_invalid_private_key() {
        let result = Keypair::from_private_key("invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_public_key() {
        let result = Keypair::from_public_key("invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_keypair_consistency() {
        for _ in 0..10 {
            let keypair = Keypair::generate();
            let public_key = keypair.public_key();
            let private_key = keypair.private_key();

            // Recreate from private key
            let recreated = Keypair::from_private_key(&private_key).unwrap();
            assert_eq!(recreated.public_key(), public_key);
        }
    }
}
