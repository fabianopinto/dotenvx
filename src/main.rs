use clap::Parser;
use dotenvx::cli::args::{Cli, Commands};
use dotenvx::cli::commands::*;
use dotenvx::utils::logger::init_logging;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        Some("debug")
    } else if cli.quiet {
        Some("error")
    } else {
        None
    };
    init_logging(log_level, cli.verbose);

    let result = match cli.command {
        Commands::Keypair { format } => keypair_command(&format),

        Commands::Encrypt {
            env_files,
            keys_file,
            keys,
            exclude_keys,
            stdout,
        } => encrypt_command(
            &env_files,
            keys_file.as_deref(),
            keys.as_deref(),
            exclude_keys.as_deref(),
            stdout,
        ),

        Commands::Decrypt {
            env_files,
            keys_file,
        } => decrypt_command(&env_files, keys_file.as_deref()),

        Commands::Set {
            key,
            value,
            env_file,
            keys_file,
            plain,
        } => set_command(&key, &value, &env_file, keys_file.as_deref(), plain),

        Commands::Get {
            key,
            env_file,
            keys_file,
        } => get_command(key.as_deref(), &env_file, keys_file.as_deref()),

        Commands::Ls { directory } => ls_command(&directory),

        Commands::Run {
            env: _,
            env_files,
            keys_file,
            overload,
            command,
        } => {
            let exit_code = run_command(&env_files, keys_file.as_deref(), overload, &command)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                });
            std::process::exit(exit_code);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
