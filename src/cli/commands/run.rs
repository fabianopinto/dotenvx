use crate::services::run_command as run_service;
use crate::utils::Result;
use std::path::{Path, PathBuf};

pub async fn run_command(
    env_files: &[PathBuf],
    keys_file: Option<&Path>,
    overload: bool,
    command: &[String],
) -> Result<i32> {
    if command.is_empty() {
        eprintln!("No command specified");
        std::process::exit(1);
    }

    let files = if env_files.is_empty() {
        vec![PathBuf::from(".env")]
    } else {
        env_files.to_vec()
    };

    // Filter to only existing files
    let existing_files: Vec<&Path> = files
        .iter()
        .filter(|f| f.exists())
        .map(|p| p.as_path())
        .collect();

    let cmd = &command[0];
    let args: Vec<String> = command[1..].to_vec();

    run_service(&existing_files, keys_file, cmd, &args, overload).await
}
