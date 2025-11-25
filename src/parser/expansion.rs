use crate::utils::error::{DotenvxError, Result};
use regex::Regex;
use std::collections::HashMap;

/// Expand variables in a value string
///
/// Supports the following syntaxes:
/// - `${VAR}` - simple variable expansion
/// - `${VAR:-default}` - variable with default value
/// - `${VAR:+alternate}` - alternate value if variable exists
/// - `$VAR` - simple variable expansion (without braces)
///
/// # Arguments
///
/// * `value` - The value string to expand
/// * `env` - The environment variables map
///
/// # Returns
///
/// The expanded value string
pub fn expand_variables(value: &str, env: &HashMap<String, String>) -> Result<String> {
    let mut result = value.to_string();

    // Pattern for ${VAR:-default} or ${VAR:+alternate} or ${VAR}
    let re_braces = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)(:-|:\+)?([^}]*)\}")
        .map_err(|e| DotenvxError::VariableExpansion(e.to_string()))?;

    // Pattern for $VAR (without braces)
    let re_simple = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)")
        .map_err(|e| DotenvxError::VariableExpansion(e.to_string()))?;

    // First, handle ${VAR} patterns with operators
    result = re_braces
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let operator = caps.get(2).map(|m| m.as_str());
            let operand = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            match operator {
                Some(":-") => {
                    // ${VAR:-default} - use default if variable is unset or empty
                    env.get(var_name)
                        .filter(|v| !v.is_empty())
                        .map(|v| v.as_str())
                        .unwrap_or(operand)
                        .to_string()
                }
                Some(":+") => {
                    // ${VAR:+alternate} - use alternate if variable is set and non-empty
                    if env.get(var_name).filter(|v| !v.is_empty()).is_some() {
                        operand.to_string()
                    } else {
                        String::new()
                    }
                }
                None => {
                    // ${VAR} - simple expansion
                    env.get(var_name).cloned().unwrap_or_default()
                }
                _ => caps[0].to_string(),
            }
        })
        .to_string();

    // Then handle $VAR patterns (without braces)
    result = re_simple
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            env.get(var_name).cloned().unwrap_or_default()
        })
        .to_string();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_simple_expansion() {
        let env = make_env(&[("USER", "alice")]);
        let result = expand_variables("Hello $USER", &env).unwrap();
        assert_eq!(result, "Hello alice");
    }

    #[test]
    fn test_braces_expansion() {
        let env = make_env(&[("USER", "alice")]);
        let result = expand_variables("Hello ${USER}", &env).unwrap();
        assert_eq!(result, "Hello alice");
    }

    #[test]
    fn test_default_value() {
        let env = HashMap::new();
        let result = expand_variables("${USER:-guest}", &env).unwrap();
        assert_eq!(result, "guest");

        let env = make_env(&[("USER", "alice")]);
        let result = expand_variables("${USER:-guest}", &env).unwrap();
        assert_eq!(result, "alice");
    }

    #[test]
    fn test_default_value_empty() {
        let env = make_env(&[("USER", "")]);
        let result = expand_variables("${USER:-guest}", &env).unwrap();
        assert_eq!(result, "guest");
    }

    #[test]
    fn test_alternate_value() {
        let env = HashMap::new();
        let result = expand_variables("${USER:+present}", &env).unwrap();
        assert_eq!(result, "");

        let env = make_env(&[("USER", "alice")]);
        let result = expand_variables("${USER:+present}", &env).unwrap();
        assert_eq!(result, "present");
    }

    #[test]
    fn test_missing_variable() {
        let env = HashMap::new();
        let result = expand_variables("Hello $USER", &env).unwrap();
        assert_eq!(result, "Hello ");
    }

    #[test]
    fn test_multiple_expansions() {
        let env = make_env(&[("HOST", "localhost"), ("PORT", "3000")]);
        let result = expand_variables("http://$HOST:$PORT", &env).unwrap();
        assert_eq!(result, "http://localhost:3000");
    }

    #[test]
    fn test_nested_expansion() {
        let env = make_env(&[("USER", "alice"), ("SUFFIX", "123")]);
        let result = expand_variables("${USER:-guest}_${SUFFIX:-000}", &env).unwrap();
        assert_eq!(result, "alice_123");
    }

    #[test]
    fn test_no_expansion_needed() {
        let env = HashMap::new();
        let result = expand_variables("plain text", &env).unwrap();
        assert_eq!(result, "plain text");
    }
}
