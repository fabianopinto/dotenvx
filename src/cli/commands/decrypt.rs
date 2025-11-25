use crate::services::decrypt_file;
use crate::utils::Result;
use std::path::{Path, PathBuf};

pub fn decrypt_command(env_files: &[PathBuf], keys_file: Option<&Path>) -> Result<()> {
    let files = if env_files.is_empty() {
        vec![PathBuf::from(".env")]
    } else {
        env_files.to_vec()
    };

    for env_file in files {
        decrypt_file(&env_file, keys_file)?;
    }

    Ok(())
}
