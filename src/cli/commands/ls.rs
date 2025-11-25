use crate::utils::fs::find_env_files;
use crate::utils::Result;
use std::path::Path;

pub fn ls_command(directory: &Path) -> Result<()> {
    let env_files = find_env_files(directory)?;

    if env_files.is_empty() {
        println!("No .env files found in {}", directory.display());
    } else {
        println!("Found {} .env file(s):", env_files.len());
        for file in env_files {
            println!("  {}", file.display());
        }
    }

    Ok(())
}
