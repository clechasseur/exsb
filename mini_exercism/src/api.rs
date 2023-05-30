//! Types and functions to interact with the Exercism APIs.

use reqwest::{Client, IntoUrl, Method, RequestBuilder};
use crate::core::Credentials;

pub mod website;

/// Client class used to query the Exercism APIs.
pub struct ApiClient {
    client: Client,
    credentials: Option<Credentials>,
}

impl ApiClient {
    /// Creates an Exercism API client from the given reqwest::Client and credentials.
    /// If credentials are not specified, the Exercism API will be queried publicly.
    pub fn new(client: Client, credentials: Option<Credentials>) -> Self {
        Self { client, credentials }
    }

    /// Creates a RequestBuilder used to send a request to an Exercism API.
    /// Takes care of setting the authorization headers using the credentials
    /// provided to the constructor, if any.
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let builder = self.client.request(method, url);
        match &self.credentials {
            Some(creds) => builder.bearer_auth(creds.api_token()),
            None => builder,
        }
    }
}
