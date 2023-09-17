use anyhow::Context;

pub fn get_http_client() -> crate::Result<reqwest::Client> {
    reqwest::Client::builder()
        .build()
        .with_context(|| "Failed to create HTTP client")
}
