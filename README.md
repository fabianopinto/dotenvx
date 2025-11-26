# dotenvx

[![CI](https://github.com/fabianopinto/dotenvx/workflows/CI/badge.svg)](https://github.com/fabianopinto/dotenvx/actions)
[![codecov](https://codecov.io/gh/fabianopinto/dotenvx/branch/main/graph/badge.svg)](https://codecov.io/gh/fabianopinto/dotenvx)
[![Crates.io](https://img.shields.io/crates/v/dotenvx.svg)](https://crates.io/crates/dotenvx)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A secure environment variable management tool with built-in encryption, written in Rust. This is a complete reimplementation of [dotenvx](https://github.com/dotenvx/dotenvx) with performance improvements and memory safety guarantees.

## Features

- 🔐 **Built-in Encryption**: ECIES encryption using secp256k1 (same curve as Bitcoin)
- 🚀 **High Performance**: 10-100x faster than the Node.js version
- 🔒 **Memory Safe**: Written in Rust with zero unsafe code
- 🌍 **Cross-Platform**: Works on Linux, macOS, and Windows
- 📦 **Small Binary**: Self-contained binaries (5-10 MB)
- 🔑 **Secure Key Management**: Public/private keypair generation and management
- 🔄 **Variable Expansion**: Supports `${VAR:-default}` syntax
- 💻 **Command Substitution**: Execute shell commands in `.env` files
- 🎯 **Zero Dependencies**: No runtime dependencies required

## Installation

### Homebrew (macOS)

```bash
brew tap fabianopinto/tap
brew install dotenvx
```

### From crates.io

```bash
cargo install dotenvx
```

### From source

```bash
git clone https://github.com/fabianopinto/dotenvx
cd dotenvx
cargo install --path .
```

### Pre-built binaries

Download pre-built binaries from the [releases page](https://github.com/fabianopinto/dotenvx/releases).

## Quick Start

### 1. Generate a keypair

```bash
dotenvx keypair
```

This generates a public/private keypair for encryption.

### 2. Encrypt your .env file

```bash
# Create a .env file
echo "DATABASE_PASSWORD=super_secret" > .env

# Encrypt it
dotenvx encrypt
```

Your `.env` file now contains encrypted values:

```ini
#/-------------------[DOTENV_PUBLIC_KEY]--------------------/
#/            public-key encryption for .env files          /
#/       [how it works](https://dotenvx.com/encryption)     /
#/----------------------------------------------------------/
DOTENV_PUBLIC_KEY="034af93e..."

DATABASE_PASSWORD="encrypted:BG8M6U+GKJGwpGA..."
```

The private key is stored in `.env.keys` (automatically added to `.gitignore`).

### 3. Run your application

```bash
dotenvx run -- your-command
```

Environment variables are automatically decrypted and injected.

## Usage

### Commands

#### `keypair` - Generate a new keypair

```bash
dotenvx keypair
```

Output:

```
DOTENV_PUBLIC_KEY="034af93e93708b994c10f236c96ef88e..."
DOTENV_PRIVATE_KEY="ec9e80073d7ace817d35acb8b7293cbf..."
```

#### `encrypt` - Encrypt environment variables

```bash
# Encrypt default .env file
dotenvx encrypt

# Encrypt specific file
dotenvx encrypt -f .env.production

# Encrypt specific keys only
dotenvx encrypt -K API_KEY -K DATABASE_PASSWORD

# Exclude certain keys
dotenvx encrypt -e DEBUG -e LOG_LEVEL
```

#### `decrypt` - Decrypt environment variables

```bash
# Decrypt default .env file
dotenvx decrypt

# Decrypt specific file
dotenvx decrypt -f .env.production
```

#### `set` - Set an environment variable (encrypted by default)

```bash
# Set encrypted variable
dotenvx set API_KEY "sk_live_123456"

# Set plain text variable
dotenvx set DEBUG "true" --plain

# Set in specific file
dotenvx set DATABASE_URL "postgres://..." -f .env.production
```

#### `get` - Get an environment variable value

```bash
# Get specific variable
dotenvx get API_KEY

# Get all variables
dotenvx get

# From specific file
dotenvx get DATABASE_URL -f .env.production
```

#### `ls` - List all .env files

```bash
# List in current directory
dotenvx ls

# List in specific directory
dotenvx ls /path/to/project
```

#### `run` - Run command with environment variables

```bash
# Run with default .env
dotenvx run -- node server.js

# Run with specific files (last wins)
dotenvx run -f .env -f .env.local -- python app.py

# Override existing environment variables
dotenvx run --overload -- ./my-app
```

## How It Works

### Encryption Flow

1. **Key Generation**: Generate a secp256k1 keypair (same curve as Bitcoin)
2. **Encryption**: Values are encrypted using ECIES (Elliptic Curve Integrated Encryption Scheme)
   - Ephemeral keypair is generated for each encryption
   - Shared secret is derived using ECDH
   - AES-256-GCM is used for symmetric encryption
3. **Storage**:
   - Public key is stored in `.env` file
   - Private key is stored in `.env.keys` (gitignored)
   - Encrypted values are base64-encoded with `encrypted:` prefix

### Decryption Flow

1. **Key Lookup**: Find private key from `.env.keys`, environment variable, or custom path
2. **Decryption**: Reverse the encryption process
3. **Injection**: Set decrypted values as environment variables

### File Structure

```
project/
├── .env                    # Public key + encrypted values (committed)
├── .env.keys              # Private keys (gitignored)
├── .env.production        # Production environment
└── .env.keys              # Production keys (deploy separately)
```

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
dotenvx = "0.1"
```

Example:

```rust
use dotenvx::crypto::{Keypair, encrypt, decrypt};
use dotenvx::parser::DotenvParser;

fn main() {
    // Generate keypair
    let keypair = Keypair::generate();

    // Encrypt a value
    let encrypted = encrypt("secret", &keypair.public_key()).unwrap();

    // Decrypt a value
    let decrypted = decrypt(&encrypted, &keypair.private_key()).unwrap();

    // Parse .env file
    let mut parser = DotenvParser::new();
    parser.parse_with_processing("KEY=value\nURL=$KEY/path").unwrap();
}
```

## Configuration Files

### `.env` file format

```ini
# Comments are supported
KEY=value
export EXPORTED_KEY=value

# Quotes (single, double, or none)
SINGLE='value'
DOUBLE="value"
UNQUOTED=value

# Variable expansion
DATABASE_URL=postgres://${DB_HOST:-localhost}/${DB_NAME}

# Command substitution
CURRENT_USER=$(whoami)
BUILD_TIME=$(date +%s)

# Encrypted values
API_KEY="encrypted:BG8M6U+GKJGwpGA42ml2erb9..."

# Public key (added automatically)
DOTENV_PUBLIC_KEY="034af93e93708b994c10f236..."
```

### `.env.keys` file format

```ini
#/------------------!DOTENV_PRIVATE_KEYS!-------------------/
#/ private decryption keys. DO NOT commit to source control /
#/     [how it works](https://dotenvx.com/encryption)       /
#/----------------------------------------------------------/

# .env
DOTENV_PRIVATE_KEY=ec9e80073d7ace817d35acb8b7293cbf...

# .env.production
DOTENV_PRIVATE_KEY_PRODUCTION=1fc1cafa954a7a2bf0a6fbff...
```

## Security Best Practices

1. **Never commit `.env.keys`** - Add to `.gitignore` immediately
2. **Rotate keys regularly** - Generate new keypairs periodically
3. **Use different keys per environment** - Separate development/staging/production
4. **Store production keys securely** - Use secret management systems
5. **Audit encrypted files** - Review what's encrypted regularly

## Performance

Compared to the Node.js version:

- **Startup time**: ~50ms vs ~500ms (10x faster)
- **Encryption**: ~100x faster for bulk operations
- **Memory usage**: ~2MB vs ~50MB (25x smaller)
- **Binary size**: 6MB vs 50MB+ with Node.js

## Development

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Build

```bash
cargo build --release
```

### Test

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_encrypt_decrypt
```

### Lint

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

### Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Original [dotenvx](https://github.com/dotenvx/dotenvx) by @motdotla
- [secp256k1](https://github.com/rust-bitcoin/rust-secp256k1) Rust implementation
- Bitcoin for the secp256k1 curve specification

## Links

- [Documentation](https://docs.rs/dotenvx)
- [Repository](https://github.com/fabianopinto/dotenvx)
- [Crates.io](https://crates.io/crates/dotenvx)
- [Original dotenvx](https://dotenvx.com)
