use crate::crypto::decrypt;
use crate::parser::DotenvParser;
use crate::utils::error::{DotenvxError, Result};
use crate::utils::fs::{read_file, write_file};
use std::path::Path;
use tracing::{debug, info};

/// Decrypt values in a .env file
///
/// # Arguments
///
/// * `env_file` - Path to the .env file
/// * `keys_file` - Optional path to the .env.keys file
///
/// # Returns
///
/// Success message
pub fn decrypt_file(env_file: &Path, keys_file: Option<&Path>) -> Result<()> {
    info!("Decrypting file: {}", env_file.display());

    let content = read_file(env_file)?;
    let mut parser = DotenvParser::new();
    parser.parse(&content)?;

    // Find the private key
    let private_key = find_private_key(env_file, keys_file, "DOTENV_PRIVATE_KEY")?;

    // Build decrypted content
    let mut output = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            output.push_str(line);
            output.push('\n');
            continue;
        }

        // Skip DOTENV_PUBLIC_KEY lines
        if trimmed.starts_with("DOTENV_PUBLIC_KEY") {
            continue;
        }

        let (export_prefix, line_content) = if let Some(stripped) = trimmed.strip_prefix("export ")
        {
            ("export ", stripped)
        } else {
            ("", trimmed)
        };

        if let Some(eq_pos) = line_content.find('=') {
            let key = line_content[..eq_pos].trim();
            let value_part = line_content[eq_pos + 1..].trim();
            let value = parse_value(value_part);

            if value.starts_with("encrypted:") {
                match decrypt(&value, &private_key) {
                    Ok(decrypted) => {
                        output.push_str(&format!("{}{}=\"{}\"\n", export_prefix, key, decrypted));
                        debug!("Decrypted key: {}", key);
                    }
                    Err(_) => {
                        return Err(DotenvxError::DecryptionFailed {
                            key: key.to_string(),
                            private_key_name: "DOTENV_PRIVATE_KEY".to_string(),
                        });
                    }
                }
            } else {
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    write_file(env_file, &output)?;
    info!("✔ decrypted {}", env_file.display());
    Ok(())
}

fn parse_value(value: &str) -> String {
    let value = value.trim();
    if ((value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\'')))
        && value.len() >= 2
    {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}

fn find_private_key(env_file: &Path, keys_file: Option<&Path>, key_name: &str) -> Result<String> {
    if let Some(keys_path) = keys_file {
        if keys_path.exists() {
            let content = read_file(keys_path)?;
            if let Some(key) = extract_key_from_content(&content, key_name) {
                return Ok(key);
            }
        }
    }

    if let Some(parent) = env_file.parent() {
        let default_keys = parent.join(".env.keys");
        if default_keys.exists() {
            let content = read_file(&default_keys)?;
            if let Some(key) = extract_key_from_content(&content, key_name) {
                return Ok(key);
            }
        }
    }

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
