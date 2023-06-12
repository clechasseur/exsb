//! Utilities to interact with the Exercism CLI application.

mod detail;

use crate::cli::detail::DefaultCliCredentialsHelper;
use crate::core::{Credentials, Result};

/// Reads API credentials from the CLI config file and returns them.
///
/// # Errors
///
/// - [`ConfigNotFound`]: CLI config file cannot be found, maybe CLI is not installed
/// - [`ConfigReadError`]: I/O error reading the config file
/// - [`ConfigParseError`]: Config file JSON could not be parsed
/// - [`ApiTokenNotFoundInConfig`]: Config file did not contain an API token
///
/// [`ConfigNotFound`]: crate::core::Error#variant.ConfigNotFound
/// [`ConfigReadError`]: crate::core::Error#variant.ConfigReadError
/// [`ConfigParseError`]: crate::core::Error#variant.ConfigParseError
/// [`ApiTokenNotFoundInConfig`]: crate::core::Error#variant.ApiTokenNotFoundInConfig
pub fn get_cli_credentials() -> Result<Credentials> {
    detail::get_cli_credentials(&DefaultCliCredentialsHelper{})
}
