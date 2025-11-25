use crate::utils::error::{DotenvxError, Result};
use regex::Regex;
use std::process::Command;

/// Substitute command outputs in a value string
///
/// Supports `$(command)` syntax to execute shell commands and replace with their output.
///
/// # Arguments
///
/// * `value` - The value string containing command substitutions
///
/// # Returns
///
/// The value with commands substituted with their outputs
///
/// # Example
///
/// ```no_run
/// use dotenvx::parser::substitute_commands;
///
/// let result = substitute_commands("User: $(whoami)").unwrap();
/// println!("{}", result);
/// ```
pub fn substitute_commands(value: &str) -> Result<String> {
    let re = Regex::new(r"\$\(([^)]+)\)")
        .map_err(|e| DotenvxError::CommandSubstitution(e.to_string()))?;

    let mut result = value.to_string();

    // Collect all matches first to avoid iterator invalidation
    let matches: Vec<_> = re.captures_iter(value).collect();

    // Process matches in reverse order to maintain correct indices
    for caps in matches.iter().rev() {
        let full_match = caps.get(0).unwrap();
        let command_str = &caps[1];

        // Execute the command
        let output = execute_command(command_str)?;

        // Replace in result string
        let start = full_match.start();
        let end = full_match.end();
        result.replace_range(start..end, &output);
    }

    Ok(result)
}

/// Execute a shell command and return its output
fn execute_command(command_str: &str) -> Result<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command_str])
            .output()
            .map_err(|e| {
                DotenvxError::CommandSubstitution(format!("Failed to execute command: {}", e))
            })?
    } else {
        Command::new("sh")
            .args(["-c", command_str])
            .output()
            .map_err(|e| {
                DotenvxError::CommandSubstitution(format!("Failed to execute command: {}", e))
            })?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DotenvxError::CommandSubstitution(format!(
            "Command failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| DotenvxError::CommandSubstitution(format!("Invalid UTF-8 output: {}", e)))?;

    // Trim trailing newline
    Ok(stdout.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_substitution() {
        let result = substitute_commands("Hello $(echo world)").unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_multiple_substitutions() {
        let result = substitute_commands("$(echo hello) $(echo world)").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_no_substitution() {
        let result = substitute_commands("plain text").unwrap();
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_expr_substitution() {
        let result = substitute_commands("Result: $(expr 2 + 2)").unwrap();
        assert_eq!(result, "Result: 4");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_pwd_substitution() {
        let result = substitute_commands("Dir: $(pwd)").unwrap();
        assert!(result.starts_with("Dir: /"));
    }

    #[test]
    fn test_invalid_command() {
        let result = substitute_commands("$(nonexistent_command_xyz)");
        assert!(result.is_err());
    }
}
