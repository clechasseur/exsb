//! Helpers to fetch Exercism credentials for our use.

use anyhow::Context;
use mini_exercism::cli::get_cli_credentials;
use mini_exercism::core::Credentials;

/// Returns API [`Credentials`] to use for our program.
///
/// If the user provided a `token` when the program is invoked, that token is used as credentials.
/// Otherwise, we attempt to fetch the credentials used by the Exercism CLI.
pub fn get_api_credentials(token: Option<&String>) -> crate::Result<Credentials> {
    match token {
        Some(token) => Ok(Credentials::from_api_token(token)),
        None => get_cli_credentials().with_context(|| "failed to get Exercism CLI credentials"),
    }
}
