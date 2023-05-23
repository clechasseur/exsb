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
#[error(transparent)]
pub struct Error(ErrorImpl);

impl<T> From<T> for Error
where
    T: Into<ErrorImpl>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Error)]
enum ErrorImpl {
    #[error("Could not read Exercism CLI config file: {0:?}")]
    ConfigReadError(#[from] io::Error),

    #[error("Failed to parse Exercism CLI config file: {0:?}")]
    ConfigParseError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_io_error() {
        let error: Error = io::Error::from(io::ErrorKind::NotFound).into();

        assert_matches!(error.0, ErrorImpl::ConfigReadError(_));
    }

    #[test]
    fn test_error_from_json_error() {
        let invalid_json = "{hello: world}";
        let error: Error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err().into();

        assert_matches!(error.0, ErrorImpl::ConfigParseError(_));
    }
}