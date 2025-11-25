use crate::utils::error::{DotenvxError, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Find all `.env` files in the given directory and its subdirectories
///
/// # Arguments
///
/// * `dir` - The directory to search in
///
/// # Returns
///
/// A vector of paths to `.env` files
pub fn find_env_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut env_files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.starts_with(".env") {
                env_files.push(path.to_path_buf());
            }
        }
    }

    Ok(env_files)
}

/// Read the contents of a file
///
/// # Arguments
///
/// * `path` - The path to the file
///
/// # Returns
///
/// The contents of the file as a string
pub fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DotenvxError::MissingEnvFile {
                path: path.display().to_string(),
            }
        } else {
            DotenvxError::Io(e)
        }
    })
}

/// Write contents to a file
///
/// # Arguments
///
/// * `path` - The path to the file
/// * `contents` - The contents to write
pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    std::fs::write(path, contents).map_err(DotenvxError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_env_files() {
        let temp = TempDir::new().unwrap();
        let temp_path = temp.path();

        // Create some test files
        std::fs::write(temp_path.join(".env"), "TEST=value").unwrap();
        std::fs::write(temp_path.join(".env.local"), "LOCAL=value").unwrap();
        std::fs::write(temp_path.join("not_env.txt"), "not an env file").unwrap();

        let env_files = find_env_files(temp_path).unwrap();
        assert_eq!(env_files.len(), 2);
    }

    #[test]
    fn test_read_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join(".env");
        std::fs::write(&file_path, "TEST=value").unwrap();

        let contents = read_file(&file_path).unwrap();
        assert_eq!(contents, "TEST=value");
    }

    #[test]
    fn test_read_missing_file() {
        let result = read_file(Path::new("/nonexistent/file"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DotenvxError::MissingEnvFile { .. }
        ));
    }
}
