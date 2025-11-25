use crate::services::encrypt_file;
use crate::utils::Result;
use std::path::{Path, PathBuf};

pub fn encrypt_command(
    env_files: &[PathBuf],
    keys_file: Option<&Path>,
    keys: Option<&[String]>,
    exclude_keys: Option<&[String]>,
    _stdout: bool,
) -> Result<()> {
    let files = if env_files.is_empty() {
        vec![PathBuf::from(".env")]
    } else {
        env_files.to_vec()
    };

    for env_file in files {
        encrypt_file(&env_file, keys_file, keys, exclude_keys)?;
    }

    Ok(())
}
