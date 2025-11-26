use crate::crypto::decrypt;
use crate::parser::DotenvParser;
use crate::utils::error::{DotenvxError, Result};
use crate::utils::fs::read_file;
use std::collections::HashMap;
use std::path::Path;

/// Print environment variables in a format suitable for shell evaluation
///
/// # Arguments
///
/// * `env_files` - Paths to .env files to load
/// * `keys_file` - Optional path to .env.keys file
/// * `format` - Output format (bash, json, etc.)
///
/// # Returns
///
/// Result indicating success or failure
pub fn printenv_command(env_files: &[&Path], keys_file: Option<&Path>, format: &str) -> Result<()> {
    // Load and merge environment variables from all files
    let mut env_vars = HashMap::new();

    for env_file in env_files {
        let file_vars = load_env_file(env_file, keys_file)?;
        env_vars.extend(file_vars);
    }

    // Output based on format
    match format {
        "json" => print_json(&env_vars),
        "bash" | "sh" => print_bash(&env_vars),
        "fish" => print_fish(&env_vars),
        "powershell" | "ps1" => print_powershell(&env_vars),
        _ => print_bash(&env_vars), // Default to bash
    }

    Ok(())
}

fn load_env_file(env_file: &Path, keys_file: Option<&Path>) -> Result<HashMap<String, String>> {
    let content = read_file(env_file)?;
    let mut parser = DotenvParser::new();
    parser.parse_with_processing(&content)?;

    let mut variables = parser.variables().clone();

    // Find private key for decryption
    let private_key = find_private_key(env_file, keys_file);

    // Decrypt encrypted values
    if let Ok(private_key) = private_key {
        for (_key, value) in variables.iter_mut() {
            if value.starts_with("encrypted:") {
                match decrypt(value, &private_key) {
                    Ok(decrypted) => {
                        *value = decrypted;
                    }
                    Err(_) => {
                        // Continue with encrypted value
                    }
                }
            }
        }
    }

    // Remove DOTENV_PUBLIC_KEY from exported variables
    variables.remove("DOTENV_PUBLIC_KEY");

    Ok(variables)
}

fn find_private_key(env_file: &Path, keys_file: Option<&Path>) -> Result<String> {
    if let Some(keys_path) = keys_file {
        if keys_path.exists() {
            let content = read_file(keys_path)?;
            if let Some(key) = extract_key_from_content(&content, "DOTENV_PRIVATE_KEY") {
                return Ok(key);
            }
        }
    }

    if let Some(parent) = env_file.parent() {
        let default_keys = parent.join(".env.keys");
        if default_keys.exists() {
            let content = read_file(&default_keys)?;
            if let Some(key) = extract_key_from_content(&content, "DOTENV_PRIVATE_KEY") {
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

fn print_bash(env_vars: &HashMap<String, String>) {
    for (key, value) in env_vars {
        // Escape single quotes in the value by replacing ' with '\''
        let escaped_value = value.replace('\'', r"'\''");
        println!("export {}='{}'", key, escaped_value);
    }
}

fn print_fish(env_vars: &HashMap<String, String>) {
    for (key, value) in env_vars {
        // Fish shell uses different escaping
        let escaped_value = value.replace('\'', r"\'");
        println!("set -gx {} '{}'", key, escaped_value);
    }
}

fn print_powershell(env_vars: &HashMap<String, String>) {
    for (key, value) in env_vars {
        // PowerShell escaping: double quotes need to be escaped
        let escaped_value = value.replace('"', r#""`""#);
        println!("$env:{}=\"{}\"", key, escaped_value);
    }
}

fn print_json(env_vars: &HashMap<String, String>) {
    use std::collections::BTreeMap;
    // Sort keys for consistent output
    let sorted: BTreeMap<_, _> = env_vars.iter().collect();

    println!("{{");
    let count = sorted.len();
    for (i, (key, value)) in sorted.iter().enumerate() {
        let json_value = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
        if i < count - 1 {
            println!("  \"{}\": {},", key, json_value);
        } else {
            println!("  \"{}\": {}", key, json_value);
        }
    }
    println!("}}");
}
