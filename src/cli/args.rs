use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dotenvx")]
#[command(author, version, about = "A secure environment variable management tool with built-in encryption", long_about = None)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new keypair
    Keypair {
        /// Format to output (hex, pem)
        #[arg(short, long, default_value = "hex")]
        format: String,
    },

    /// Encrypt environment variables in .env files
    Encrypt {
        /// Path(s) to .env file(s)
        #[arg(short = 'f', long = "env-file")]
        env_files: Vec<PathBuf>,

        /// Path to .env.keys file
        #[arg(short = 'k', long = "env-keys-file")]
        keys_file: Option<PathBuf>,

        /// Specific keys to encrypt
        #[arg(short = 'K', long = "key")]
        keys: Option<Vec<String>>,

        /// Keys to exclude from encryption
        #[arg(short = 'e', long = "exclude-key")]
        exclude_keys: Option<Vec<String>>,

        /// Output to stdout instead of modifying file
        #[arg(long)]
        stdout: bool,
    },

    /// Decrypt environment variables in .env files
    Decrypt {
        /// Path(s) to .env file(s)
        #[arg(short = 'f', long = "env-file")]
        env_files: Vec<PathBuf>,

        /// Path to .env.keys file
        #[arg(short = 'k', long = "env-keys-file")]
        keys_file: Option<PathBuf>,
    },

    /// Set an environment variable (encrypted by default)
    Set {
        /// Variable name
        key: String,

        /// Variable value
        value: String,

        /// Path to .env file
        #[arg(short = 'f', long = "env-file", default_value = ".env")]
        env_file: PathBuf,

        /// Path to .env.keys file
        #[arg(short = 'k', long = "env-keys-file")]
        keys_file: Option<PathBuf>,

        /// Store as plain text (don't encrypt)
        #[arg(short = 'p', long)]
        plain: bool,
    },

    /// Get an environment variable value
    Get {
        /// Variable name (if not provided, shows all)
        key: Option<String>,

        /// Path to .env file
        #[arg(short = 'f', long = "env-file", default_value = ".env")]
        env_file: PathBuf,

        /// Path to .env.keys file
        #[arg(short = 'k', long = "env-keys-file")]
        keys_file: Option<PathBuf>,
    },

    /// List all .env files in the directory tree
    Ls {
        /// Directory to search (defaults to current)
        #[arg(default_value = ".")]
        directory: PathBuf,
    },

    /// Run a command with environment variables loaded
    Run {
        /// Inline environment variables (KEY=value)
        #[arg(short = 'e', long = "env")]
        env: Vec<String>,

        /// Path(s) to .env file(s)
        #[arg(short = 'f', long = "env-file")]
        env_files: Vec<PathBuf>,

        /// Path to .env.keys file
        #[arg(short = 'k', long = "env-keys-file")]
        keys_file: Option<PathBuf>,

        /// Override existing environment variables
        #[arg(short = 'o', long)]
        overload: bool,

        /// Command to run
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
}
