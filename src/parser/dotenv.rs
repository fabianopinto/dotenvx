use crate::parser::{expand_variables, substitute_commands};
use crate::utils::error::{DotenvxError, Result};
use std::collections::HashMap;

/// Parser for .env files
#[derive(Debug, Default)]
pub struct DotenvParser {
    variables: HashMap<String, String>,
}

impl DotenvParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a .env file content
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the .env file
    ///
    /// # Returns
    ///
    /// A Result containing the parsed variables
    pub fn parse(&mut self, content: &str) -> Result<&HashMap<String, String>> {
        for (line_num, line) in content.lines().enumerate() {
            self.parse_line(line, line_num + 1)?;
        }
        Ok(&self.variables)
    }

    /// Parse a single line
    fn parse_line(&mut self, line: &str, line_num: usize) -> Result<()> {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        // Handle export prefix
        let line_content = if let Some(stripped) = trimmed.strip_prefix("export ") {
            stripped
        } else {
            trimmed
        };

        // Find the = separator
        let Some(eq_pos) = line_content.find('=') else {
            return Err(DotenvxError::ParseError {
                line: line_num,
                message: "Missing '=' in variable assignment".to_string(),
            });
        };

        let key = line_content[..eq_pos].trim();
        let value_part = line_content[eq_pos + 1..].trim();

        // Validate key
        if key.is_empty() {
            return Err(DotenvxError::ParseError {
                line: line_num,
                message: "Empty variable name".to_string(),
            });
        }

        // Parse value (handle quotes)
        let value = self.parse_value(value_part)?;

        self.variables.insert(key.to_string(), value);
        Ok(())
    }

    /// Parse a value, handling quotes and escapes
    fn parse_value(&self, value: &str) -> Result<String> {
        if value.is_empty() {
            return Ok(String::new());
        }

        let value = value.trim();

        // Handle single quotes (no expansion)
        if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
            return Ok(value[1..value.len() - 1].to_string());
        }

        // Handle double quotes (with expansion)
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            let inner = &value[1..value.len() - 1];
            return Ok(self.unescape(inner));
        }

        // Handle backticks (command substitution)
        if value.starts_with('`') && value.ends_with('`') && value.len() >= 2 {
            let command = &value[1..value.len() - 1];
            return substitute_commands(&format!("$({})", command));
        }

        // No quotes - return as is
        Ok(value.to_string())
    }

    /// Unescape special characters in a string
    fn unescape(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '\'' => result.push('\''),
                        _ => {
                            result.push('\\');
                            result.push(next_ch);
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Get the parsed variables
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Expand all variables in the parsed values
    pub fn expand(&mut self) -> Result<()> {
        let keys: Vec<String> = self.variables.keys().cloned().collect();

        for key in keys {
            let value = self.variables.get(&key).unwrap().clone();
            let expanded = expand_variables(&value, &self.variables)?;
            self.variables.insert(key, expanded);
        }

        Ok(())
    }

    /// Perform command substitution on all values
    pub fn substitute(&mut self) -> Result<()> {
        let keys: Vec<String> = self.variables.keys().cloned().collect();

        for key in keys {
            let value = self.variables.get(&key).unwrap().clone();
            if value.contains("$(") {
                let substituted = substitute_commands(&value)?;
                self.variables.insert(key, substituted);
            }
        }

        Ok(())
    }

    /// Parse with full processing (expansion and substitution)
    pub fn parse_with_processing(&mut self, content: &str) -> Result<&HashMap<String, String>> {
        self.parse(content)?;
        self.substitute()?;
        self.expand()?;
        Ok(&self.variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut parser = DotenvParser::new();
        let content = "KEY=value";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "value");
    }

    #[test]
    fn test_parse_with_spaces() {
        let mut parser = DotenvParser::new();
        let content = "  KEY  =  value  ";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "value");
    }

    #[test]
    fn test_parse_double_quotes() {
        let mut parser = DotenvParser::new();
        let content = r#"KEY="value with spaces""#;
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "value with spaces");
    }

    #[test]
    fn test_parse_single_quotes() {
        let mut parser = DotenvParser::new();
        let content = "KEY='value with spaces'";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "value with spaces");
    }

    #[test]
    fn test_parse_comment() {
        let mut parser = DotenvParser::new();
        let content = "# This is a comment\nKEY=value";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars.get("KEY").unwrap(), "value");
    }

    #[test]
    fn test_parse_export() {
        let mut parser = DotenvParser::new();
        let content = "export KEY=value";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "value");
    }

    #[test]
    fn test_parse_empty_value() {
        let mut parser = DotenvParser::new();
        let content = "KEY=";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "");
    }

    #[test]
    fn test_parse_escape_sequences() {
        let mut parser = DotenvParser::new();
        let content = r#"KEY="line1\nline2\ttab""#;
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.get("KEY").unwrap(), "line1\nline2\ttab");
    }

    #[test]
    fn test_parse_multiline() {
        let mut parser = DotenvParser::new();
        let content = "KEY1=value1\nKEY2=value2\n\nKEY3=value3";
        let vars = parser.parse(content).unwrap();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars.get("KEY1").unwrap(), "value1");
        assert_eq!(vars.get("KEY2").unwrap(), "value2");
        assert_eq!(vars.get("KEY3").unwrap(), "value3");
    }

    #[test]
    fn test_parse_invalid_no_equals() {
        let mut parser = DotenvParser::new();
        let content = "INVALID";
        let result = parser.parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_expand_variables() {
        let mut parser = DotenvParser::new();
        let content = "HOST=localhost\nURL=http://$HOST:3000";
        parser.parse(content).unwrap();
        parser.expand().unwrap();
        assert_eq!(
            parser.variables().get("URL").unwrap(),
            "http://localhost:3000"
        );
    }

    #[test]
    fn test_expand_with_default() {
        let mut parser = DotenvParser::new();
        let content = "URL=${HOST:-localhost}:3000";
        parser.parse(content).unwrap();
        parser.expand().unwrap();
        assert_eq!(parser.variables().get("URL").unwrap(), "localhost:3000");
    }

    #[test]
    fn test_command_substitution() {
        let mut parser = DotenvParser::new();
        let content = "RESULT=$(echo test)";
        parser.parse(content).unwrap();
        parser.substitute().unwrap();
        assert_eq!(parser.variables().get("RESULT").unwrap(), "test");
    }

    #[test]
    fn test_parse_with_processing() {
        let mut parser = DotenvParser::new();
        let content = "BASE=$(echo /tmp)\nPATH=$BASE/subdir";
        parser.parse_with_processing(content).unwrap();
        assert_eq!(parser.variables().get("PATH").unwrap(), "/tmp/subdir");
    }
}
