use anyhow::Context;
use mini_exercism::cli::get_cli_credentials;
use mini_exercism::core::Credentials;

pub fn get_api_credentials(token: Option<&String>) -> crate::Result<Credentials> {
    match token {
        Some(token) => Ok(Credentials::from_api_token(token)),
        None => get_cli_credentials().with_context(|| "failed to get Exercism CLI credentials"),
    }
}
