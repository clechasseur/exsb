pub mod tracks;

use mini_exercism::api;
use mini_exercism::core::Credentials;
use crate::reqwest::get_http_client;

pub fn get_v1_client(credentials: &Credentials) -> crate::Result<api::v1::Client> {
    Ok(api::v1::Client::builder()
        .http_client(get_http_client()?)
        .credentials(credentials.clone())
        .build())
}

pub fn get_v2_client(credentials: &Credentials) -> crate::Result<api::v2::Client> {
    Ok(api::v2::Client::builder()
        .http_client(get_http_client()?)
        .credentials(credentials.clone())
        .build())
}
