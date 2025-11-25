use crate::crypto::decrypt;
use crate::parser::DotenvParser;
use crate::utils::error::{DotenvxError, Result};
use crate::utils::fs::read_file;
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;
use tracing::{debug, info};

/// Run a command with environment variables loaded from .env files
///
/// # Arguments
///
/// * `env_files` - Paths to .env files to load
/// * `keys_file` - Optional path to .env.keys file
/// * `command` - The command to execute
/// * `args` - Arguments for the command
/// * `overload` - Whether to override existing environment variables
///
/// # Returns
///
/// The exit code of the command
pub async fn run_command(
    env_files: &[&Path],
    keys_file: Option<&Path>,
    command: &str,
    args: &[String],
    overload: bool,
) -> Result<i32> {
    info!("Running command: {} {:?}", command, args);

    // Load and merge environment variables from all files
    let mut env_vars = HashMap::new();

    for env_file in env_files {
        debug!("Loading env file: {}", env_file.display());
        let file_vars = load_env_file(env_file, keys_file)?;
        env_vars.extend(file_vars);
    }

    // Merge with existing environment if not overloading
    if !overload {
        for (key, value) in std::env::vars() {
            env_vars.entry(key).or_insert(value);
        }
    }

    debug!("Loaded {} environment variables", env_vars.len());

    // Execute the command
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.envs(&env_vars);

    let status = cmd
        .status()
        .await
        .map_err(|e| DotenvxError::CommandFailed(format!("Failed to execute command: {}", e)))?;

    let exit_code = status.code().unwrap_or(-1);
    info!("Command exited with code: {}", exit_code);

    Ok(exit_code)
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
        for (key, value) in variables.iter_mut() {
            if value.starts_with("encrypted:") {
                match decrypt(value, &private_key) {
                    Ok(decrypted) => {
                        *value = decrypted;
                        debug!("Decrypted key: {}", key);
                    }
                    Err(e) => {
                        debug!("Failed to decrypt {}: {:?}", key, e);
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
