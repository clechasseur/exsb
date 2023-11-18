//! Helpers related to the use of the [`mini_exercism`] library.

pub mod credentials;
pub mod tracks;

use mini_exercism::api;
use mini_exercism::core::Credentials;

/// Creates a [`Client`](api::v1::Client) for the v1 Exercism API.
pub fn get_v1_client(http_client: &reqwest::Client, credentials: &Credentials) -> api::v1::Client {
    api::v1::Client::builder()
        .http_client(http_client.clone())
        .credentials(credentials.clone())
        .build()
}

/// Creates a [`Client`](api::v2::Client) for the v2 Exercism API.
pub fn get_v2_client(http_client: &reqwest::Client, credentials: &Credentials) -> api::v2::Client {
    api::v2::Client::builder()
        .http_client(http_client.clone())
        .credentials(credentials.clone())
        .build()
}
