use crate::crypto::decrypt;
use crate::parser::DotenvParser;
use crate::utils::error::Result;
use crate::utils::fs::read_file;
use std::path::Path;

pub fn get_command(key: Option<&str>, env_file: &Path, keys_file: Option<&Path>) -> Result<()> {
    let content = read_file(env_file)?;
    let mut parser = DotenvParser::new();
    parser.parse(&content)?;

    let private_key = find_private_key(env_file, keys_file);

    if let Some(key_name) = key {
        // Get specific key
        if let Some(value) = parser.variables().get(key_name) {
            let final_value =
                if let (true, Ok(key)) = (value.starts_with("encrypted:"), &private_key) {
                    decrypt(value, key).unwrap_or_else(|_| value.clone())
                } else {
                    value.clone()
                };
            println!("{}={}", key_name, final_value);
        } else {
            eprintln!("Key '{}' not found", key_name);
            std::process::exit(1);
        }
    } else {
        // Get all keys
        for (k, v) in parser.variables() {
            if k == "DOTENV_PUBLIC_KEY" {
                continue;
            }

            let final_value = if let (true, Ok(key)) = (v.starts_with("encrypted:"), &private_key) {
                decrypt(v, key).unwrap_or_else(|_| v.clone())
            } else {
                v.clone()
            };
            println!("{}={}", k, final_value);
        }
    }

    Ok(())
}

fn find_private_key(env_file: &Path, keys_file: Option<&Path>) -> Result<String> {
    use crate::utils::error::DotenvxError;

    if let Some(keys_path) = keys_file {
        if keys_path.exists() {
            let content = read_file(keys_path)?;
            if let Some(key) = extract_key(&content) {
                return Ok(key);
            }
        }
    }

    if let Some(parent) = env_file.parent() {
        let default_keys = parent.join(".env.keys");
        if default_keys.exists() {
            let content = read_file(&default_keys)?;
            if let Some(key) = extract_key(&content) {
                return Ok(key);
            }
        }
    }

    if let Ok(key) = std::env::var("DOTENV_PRIVATE_KEY") {
        return Ok(key);
    }

    Err(DotenvxError::MissingPrivateKey {
        key_name: "DOTENV_PRIVATE_KEY".to_string(),
    })
}

fn extract_key(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("DOTENV_PRIVATE_KEY=") {
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}
