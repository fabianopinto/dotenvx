//! # dotenvx
//!
//! A secure environment variable management tool with built-in encryption using ECIES.
//!
//! ## Features
//!
//! - **Runtime Environment Injection**: Load `.env` files at runtime
//! - **Built-in Encryption**: ECIES encryption using secp256k1
//! - **Key Management**: Secure public/private key handling
//! - **Variable Expansion**: Support for `${VAR:-default}` syntax
//! - **Command Substitution**: Execute shell commands in `.env` files
//!
//! ## Example
//!
//! ```rust,no_run
//! use dotenvx::crypto::Keypair;
//!
//! let keypair = Keypair::generate();
//! println!("Public key: {}", keypair.public_key());
//! ```

pub mod cli;
pub mod crypto;
pub mod parser;
pub mod services;
pub mod utils;

pub use crypto::{decrypt, encrypt, Keypair};
pub use parser::DotenvParser;
pub use utils::error::{DotenvxError, Result};
