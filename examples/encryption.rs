//! Encryption workflow example

use dotenvx::crypto::{decrypt, encrypt, Keypair};

fn main() {
    println!("=== dotenvx Encryption Workflow ===\n");

    // Generate keypair
    let keypair = Keypair::generate();
    println!("Generated keypair");
    println!("Public key:  {}", keypair.public_key());
    println!("Private key: {}\n", keypair.private_key());

    // Simulate multiple environment variables
    let secrets = vec![
        ("DATABASE_PASSWORD", "postgres_secret_123"),
        ("API_KEY", "sk_live_123456789"),
        ("JWT_SECRET", "super_secret_jwt_key"),
    ];

    println!("Encrypting secrets...");
    let mut encrypted_secrets = Vec::new();

    for (key, value) in &secrets {
        let encrypted = encrypt(value, &keypair.public_key()).expect("Failed to encrypt");
        encrypted_secrets.push((*key, encrypted));
        println!("  ✓ Encrypted {}", key);
    }

    println!("\nEncrypted values:");
    for (key, encrypted) in &encrypted_secrets {
        println!("  {}={}", key, &encrypted[..50]); // Show first 50 chars
    }

    println!("\nDecrypting secrets...");
    for (key, encrypted) in &encrypted_secrets {
        let decrypted = decrypt(encrypted, &keypair.private_key()).expect("Failed to decrypt");

        // Find original value
        let original = secrets
            .iter()
            .find(|(k, _)| *k == *key)
            .map(|(_, v)| v)
            .unwrap();

        assert_eq!(&decrypted, original);
        println!("  ✓ Decrypted and verified {}", key);
    }

    println!("\n=== All secrets encrypted and decrypted successfully! ===");
}
