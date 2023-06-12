//! Types and functions to interact with the Exercism APIs.

pub mod website;

use reqwest::{Client, IntoUrl, Method, RequestBuilder};
use crate::core::Credentials;

/// Client class used to query the Exercism APIs.
pub struct ApiClient {
    client: Client,
    credentials: Option<Credentials>,
}

impl ApiClient {
    /// Creates an Exercism API client from a default reqwest::Client and credentials.
    /// If credentials are not specified, the Exercism API will be queried publicly.
    pub fn with_default_client(credentials: Option<Credentials>) -> Self {
        Self::with_custom_client(Client::new(), credentials)
    }

    /// Creates an Exercism API client from the given reqwest::Client and credentials.
    /// If credentials are not specified, the Exercism API will be queried publicly.
    pub fn with_custom_client(client: Client, credentials: Option<Credentials>) -> Self {
        Self { client, credentials }
    }

    /// Accesses the credentials used to access the Exercism API.
    /// If `None`, the Exercism API will be queried publicly.
    pub fn credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    /// Creates a `RequestBuilder` used to send a request to an Exercism API.
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
