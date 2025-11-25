use crate::crypto::encrypt;
use crate::parser::DotenvParser;
use crate::utils::error::{DotenvxError, Result};
use crate::utils::fs::{read_file, write_file};
use std::path::Path;

pub fn set_command(
    key: &str,
    value: &str,
    env_file: &Path,
    keys_file: Option<&Path>,
    plain: bool,
) -> Result<()> {
    // Read existing file or create empty content
    let content = if env_file.exists() {
        read_file(env_file)?
    } else {
        String::new()
    };

    let mut parser = DotenvParser::new();
    if !content.is_empty() {
        parser.parse(&content)?;
    }

    // Get or create keypair for encryption
    let (final_value, public_key) = if plain {
        (value.to_string(), None)
    } else {
        // Find or generate keypair
        let public_key = if let Some(existing_key) = parser.variables().get("DOTENV_PUBLIC_KEY") {
            existing_key.clone()
        } else {
            let keypair = crate::crypto::Keypair::generate();
            let priv_key = keypair.private_key();

            // Save private key
            save_private_key(env_file, keys_file, &priv_key)?;

            keypair.public_key()
        };

        let encrypted = encrypt(value, &public_key)?;
        (encrypted, Some(public_key))
    };

    // Build new content
    let mut output = String::new();
    let mut key_found = false;

    // Add public key header if encrypting
    if let Some(ref pub_key) = public_key {
        if !content.contains("DOTENV_PUBLIC_KEY") {
            output.push_str("#/-------------------[DOTENV_PUBLIC_KEY]--------------------/\n");
            output.push_str("#/            public-key encryption for .env files          /\n");
            output.push_str("#/       [how it works](https://dotenvx.com/encryption)     /\n");
            output.push_str("#/----------------------------------------------------------/\n");
            output.push_str(&format!("DOTENV_PUBLIC_KEY=\"{}\"\n\n", pub_key));
        }
    }

    // Process existing content
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(&format!("{}=", key))
            || trimmed.starts_with(&format!("export {}=", key))
        {
            // Replace this line
            output.push_str(&format!("{}=\"{}\"\n", key, final_value));
            key_found = true;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    // Add key if not found
    if !key_found {
        output.push_str(&format!("{}=\"{}\"\n", key, final_value));
    }

    write_file(env_file, &output)?;
    println!("✔ set {} in {}", key, env_file.display());

    Ok(())
}

fn save_private_key(env_file: &Path, keys_file: Option<&Path>, private_key: &str) -> Result<()> {
    let keys_path = if let Some(path) = keys_file {
        path.to_path_buf()
    } else {
        env_file
            .parent()
            .ok_or_else(|| DotenvxError::Other("Invalid env file path".to_string()))?
            .join(".env.keys")
    };

    let mut content = if keys_path.exists() {
        read_file(&keys_path)?
    } else {
        let mut c = String::new();
        c.push_str("#/------------------!DOTENV_PRIVATE_KEYS!-------------------/\n");
        c.push_str("#/ private decryption keys. DO NOT commit to source control /\n");
        c.push_str("#/     [how it works](https://dotenvx.com/encryption)       /\n");
        c.push_str("#/----------------------------------------------------------/\n\n");
        c
    };

    if !content.contains("DOTENV_PRIVATE_KEY=") {
        content.push_str(&format!("DOTENV_PRIVATE_KEY={}\n", private_key));
        write_file(&keys_path, &content)?;
    }

    Ok(())
}
