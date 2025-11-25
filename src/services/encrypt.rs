use crate::crypto::{encrypt, Keypair};
use crate::parser::DotenvParser;
use crate::utils::error::{DotenvxError, Result};
use crate::utils::fs::{read_file, write_file};
use std::path::Path;
use tracing::{debug, info};

/// Encrypt values in a .env file
///
/// # Arguments
///
/// * `env_file` - Path to the .env file
/// * `keys_file` - Optional path to the .env.keys file
/// * `specific_keys` - Optional list of specific keys to encrypt
/// * `exclude_keys` - Optional list of keys to exclude from encryption
///
/// # Returns
///
/// The public key used for encryption
pub fn encrypt_file(
    env_file: &Path,
    keys_file: Option<&Path>,
    specific_keys: Option<&[String]>,
    exclude_keys: Option<&[String]>,
) -> Result<String> {
    info!("Encrypting file: {}", env_file.display());

    // Read the .env file
    let content = read_file(env_file)?;

    // Parse the file
    let mut parser = DotenvParser::new();
    parser.parse(&content)?;
    let variables = parser.variables().clone();

    // Check if already has a public key
    let keypair = if variables.contains_key("DOTENV_PUBLIC_KEY") {
        debug!("Found existing DOTENV_PUBLIC_KEY");

        // Try to find the corresponding private key
        let private_key = find_private_key(env_file, keys_file, "DOTENV_PRIVATE_KEY")?;
        Keypair::from_private_key(&private_key)?
    } else {
        debug!("Generating new keypair");
        Keypair::generate()
    };
    let public_key = keypair.public_key();

    // Build the encrypted content
    let mut output = String::new();

    // Add header
    output.push_str("#/-------------------[DOTENV_PUBLIC_KEY]--------------------/\n");
    output.push_str("#/            public-key encryption for .env files          /\n");
    output.push_str("#/       [how it works](https://dotenvx.com/encryption)     /\n");
    output.push_str("#/----------------------------------------------------------/\n");
    output.push_str(&format!("DOTENV_PUBLIC_KEY=\"{}\"\n\n", public_key));

    // Process each line
    for line in content.lines() {
        let trimmed = line.trim();

        // Keep comments and empty lines as-is
        if trimmed.is_empty() || trimmed.starts_with('#') {
            output.push_str(line);
            output.push('\n');
            continue;
        }

        // Skip if it's the DOTENV_PUBLIC_KEY line
        if trimmed.starts_with("DOTENV_PUBLIC_KEY=") || trimmed.starts_with("DOTENV_PUBLIC_KEY \"")
        {
            continue;
        }

        // Handle export prefix
        let (export_prefix, line_content) = if let Some(stripped) = trimmed.strip_prefix("export ")
        {
            ("export ", stripped)
        } else {
            ("", trimmed)
        };

        // Parse key=value
        if let Some(eq_pos) = line_content.find('=') {
            let key = line_content[..eq_pos].trim();
            let value_part = line_content[eq_pos + 1..].trim();

            // Determine if we should encrypt this key
            let should_encrypt = should_encrypt_key(key, specific_keys, exclude_keys);

            if should_encrypt && !value_part.starts_with("encrypted:") {
                // Parse the value (remove quotes if present)
                let value = parse_value(value_part);

                // Encrypt the value
                let encrypted = encrypt(&value, &public_key)?;
                output.push_str(&format!("{}{}=\"{}\"\n", export_prefix, key, encrypted));
                debug!("Encrypted key: {}", key);
            } else {
                // Keep as-is
                output.push_str(line);
                output.push('\n');
            }
        } else {
            // Keep invalid lines as-is
            output.push_str(line);
            output.push('\n');
        }
    }

    // Write the encrypted content
    write_file(env_file, &output)?;

    // Write the keys file if needed
    write_keys_file(
        env_file,
        keys_file,
        "DOTENV_PRIVATE_KEY",
        &keypair.private_key(),
    )?;

    info!("✔ encrypted {}", env_file.display());
    Ok(public_key)
}

fn should_encrypt_key(
    key: &str,
    specific_keys: Option<&[String]>,
    exclude_keys: Option<&[String]>,
) -> bool {
    // Don't encrypt DOTENV_PUBLIC_KEY
    if key == "DOTENV_PUBLIC_KEY" {
        return false;
    }

    // If specific keys are provided, only encrypt those
    if let Some(keys) = specific_keys {
        return keys.contains(&key.to_string());
    }

    // If exclude keys are provided, don't encrypt those
    if let Some(keys) = exclude_keys {
        return !keys.contains(&key.to_string());
    }

    // By default, encrypt everything
    true
}

fn parse_value(value: &str) -> String {
    let value = value.trim();

    // Remove quotes if present
    if ((value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\'')))
        && value.len() >= 2
    {
        return value[1..value.len() - 1].to_string();
    }

    value.to_string()
}

fn find_private_key(env_file: &Path, keys_file: Option<&Path>, key_name: &str) -> Result<String> {
    // Try the provided keys file first
    if let Some(keys_path) = keys_file {
        if keys_path.exists() {
            let content = read_file(keys_path)?;
            if let Some(key) = extract_key_from_content(&content, key_name) {
                return Ok(key);
            }
        }
    }

    // Try .env.keys in the same directory
    if let Some(parent) = env_file.parent() {
        let default_keys = parent.join(".env.keys");
        if default_keys.exists() {
            let content = read_file(&default_keys)?;
            if let Some(key) = extract_key_from_content(&content, key_name) {
                return Ok(key);
            }
        }
    }

    // Try environment variable
    if let Ok(key) = std::env::var(key_name) {
        return Ok(key);
    }

    Err(DotenvxError::MissingPrivateKey {
        key_name: key_name.to_string(),
    })
}

fn extract_key_from_content(content: &str, key_name: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}=", key_name)) {
            let value = &trimmed[key_name.len() + 1..];
            return Some(parse_value(value));
        }
    }
    None
}

fn write_keys_file(
    env_file: &Path,
    keys_file: Option<&Path>,
    key_name: &str,
    private_key: &str,
) -> Result<()> {
    let keys_path = if let Some(path) = keys_file {
        path.to_path_buf()
    } else {
        env_file
            .parent()
            .ok_or_else(|| DotenvxError::Other("Invalid env file path".to_string()))?
            .join(".env.keys")
    };

    // Read existing content if file exists
    let mut content = if keys_path.exists() {
        read_file(&keys_path)?
    } else {
        String::new()
    };

    // Check if key already exists
    let key_line = format!("{}={}", key_name, private_key);

    if !content.contains(&format!("{}=", key_name)) {
        // Add header if file is empty
        if content.is_empty() {
            content.push_str("#/------------------!DOTENV_PRIVATE_KEYS!-------------------/\n");
            content.push_str("#/ private decryption keys. DO NOT commit to source control /\n");
            content.push_str("#/     [how it works](https://dotenvx.com/encryption)       /\n");
            content.push_str("#/----------------------------------------------------------/\n\n");
            content.push_str(&format!(
                "# {}\n",
                env_file.file_name().unwrap().to_string_lossy()
            ));
        }

        content.push_str(&format!("{}\n", key_line));
        write_file(&keys_path, &content)?;
        info!("✔ key saved to {}", keys_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_file() {
        let temp = TempDir::new().unwrap();
        let env_file = temp.path().join(".env");

        write_file(&env_file, "SECRET=my_secret_value").unwrap();

        let public_key = encrypt_file(&env_file, None, None, None).unwrap();
        assert_eq!(public_key.len(), 66);

        let content = read_file(&env_file).unwrap();
        assert!(content.contains("DOTENV_PUBLIC_KEY"));
        assert!(content.contains("encrypted:"));
        assert!(!content.contains("my_secret_value"));
    }

    #[test]
    fn test_encrypt_specific_keys() {
        let temp = TempDir::new().unwrap();
        let env_file = temp.path().join(".env");

        write_file(&env_file, "KEY1=value1\nKEY2=value2").unwrap();

        let keys = vec!["KEY1".to_string()];
        encrypt_file(&env_file, None, Some(&keys), None).unwrap();

        let content = read_file(&env_file).unwrap();
        assert!(content.contains("KEY1=\"encrypted:"));
        assert!(content.contains("KEY2=value2"));
    }
}
