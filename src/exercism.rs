pub mod tracks;

use mini_exercism::api;
use mini_exercism::core::Credentials;

pub fn get_v1_client(http_client: &reqwest::Client, credentials: &Credentials) -> api::v1::Client {
    api::v1::Client::builder()
        .http_client(http_client.clone())
        .credentials(credentials.clone())
        .build()
}

pub fn get_v2_client(http_client: &reqwest::Client, credentials: &Credentials) -> api::v2::Client {
    api::v2::Client::builder()
        .http_client(http_client.clone())
        .credentials(credentials.clone())
        .build()
}
