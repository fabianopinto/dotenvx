use thiserror::Error;

/// Result type alias for dotenvx operations
pub type Result<T> = std::result::Result<T, DotenvxError>;

/// Error types for dotenvx operations
#[derive(Error, Debug)]
pub enum DotenvxError {
    #[error("missing .env file: {path}")]
    MissingEnvFile { path: String },

    #[error("missing key: {key}")]
    MissingKey { key: String },

    #[error("missing private key: {key_name}")]
    MissingPrivateKey { key_name: String },

    #[error("decryption failed for key '{key}' using '{private_key_name}'")]
    DecryptionFailed {
        key: String,
        private_key_name: String,
    },

    #[error("malformed encrypted data for key: {key}")]
    MalformedEncryptedData { key: String },

    #[error("invalid public key format: {0}")]
    InvalidPublicKey(String),

    #[error("invalid private key format: {0}")]
    InvalidPrivateKey(String),

    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("invalid .env file format at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("command execution failed: {0}")]
    CommandFailed(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] secp256k1::Error),

    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("variable expansion error: {0}")]
    VariableExpansion(String),

    #[error("command substitution error: {0}")]
    CommandSubstitution(String),

    #[error("{0}")]
    Other(String),
}

impl DotenvxError {
    /// Returns the error code for this error
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingEnvFile { .. } => "MISSING_ENV_FILE",
            Self::MissingKey { .. } => "MISSING_KEY",
            Self::MissingPrivateKey { .. } => "MISSING_PRIVATE_KEY",
            Self::DecryptionFailed { .. } => "DECRYPTION_FAILED",
            Self::MalformedEncryptedData { .. } => "MALFORMED_ENCRYPTED_DATA",
            Self::InvalidPublicKey(_) => "INVALID_PUBLIC_KEY",
            Self::InvalidPrivateKey(_) => "INVALID_PRIVATE_KEY",
            Self::EncryptionFailed(_) => "ENCRYPTION_FAILED",
            Self::ParseError { .. } => "PARSE_ERROR",
            Self::CommandFailed(_) => "COMMAND_FAILED",
            Self::Io(_) => "IO_ERROR",
            Self::Secp256k1(_) => "CRYPTO_ERROR",
            Self::Base64Decode(_) => "BASE64_DECODE_ERROR",
            Self::HexDecode(_) => "HEX_DECODE_ERROR",
            Self::Utf8(_) => "UTF8_ERROR",
            Self::Regex(_) => "REGEX_ERROR",
            Self::VariableExpansion(_) => "VARIABLE_EXPANSION_ERROR",
            Self::CommandSubstitution(_) => "COMMAND_SUBSTITUTION_ERROR",
            Self::Other(_) => "UNKNOWN_ERROR",
        }
    }
}
