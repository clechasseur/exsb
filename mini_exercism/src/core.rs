use std::io;
use thiserror::Error;

/// Struct storing the credentials used to access the Exercism API.
pub struct Credentials {
    api_token: String,
}

impl Credentials {
    /// Creates a new Exercism credentials wrapper from the given API token.
    pub fn from_api_token(api_token: String) -> Self {
        Self { api_token }
    }

    /// Accesses the Exercism API token.
    pub fn api_token(&self) -> &str {
        self.api_token.as_str()
    }
}

/// Result type used by the mini_exercism library when an error can occur.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type used by the mini_exercism library.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Could not read Exercism CLI config file: {0:?}")]
    ConfigReadError(#[from] io::Error),

    #[error("Failed to parse Exercism CLI config file: {0:?}")]
    ConfigParseError(#[from] serde_json::Error),
}
