//! Helpers related to the [`reqwest`] HTTP client library.

use anyhow::Context;

use crate::Result;

/// Creates an HTTP [`Client`](reqwest::Client) to contact the Exercism API.
///
/// This uses a [`ClientBuilder`](reqwest::ClientBuilder) to build the HTTP client, so that any
/// error can be caught instead of a resulting in a panic.
pub fn get_http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .build()
        .with_context(|| "failed to create HTTP client")
}
