//! Types and functions to interact with the Exercism APIs.

pub(crate) mod detail;

pub mod website;

use reqwest::{IntoUrl, Method};
use crate::core::Credentials;

/// Client class used to query an Exercism API.
///
/// ## Note
///
/// You do not usually have to create an instance of this class yourself;
pub struct Client {
    http_client: reqwest::Client,
    credentials: Option<Credentials>,
}

impl Client {
    /// Creates an Exercism API client from a default [reqwest::Client] and [credentials](Credentials).
    /// If credentials are not specified, the Exercism API will be queried publicly.
    pub fn with_default_http_client(credentials: Option<Credentials>) -> Self {
        Self::with_custom_http_client(reqwest::Client::new(), credentials)
    }

    /// Creates an Exercism API client from the given [reqwest::Client] and [credentials](Credentials).
    /// If credentials are not specified, the Exercism API will be queried publicly.
    pub fn with_custom_http_client(http_client: reqwest::Client, credentials: Option<Credentials>) -> Self {
        Self { http_client, credentials }
    }

    /// Accesses the credentials used to access the Exercism API.
    /// If [None], the Exercism API will be queried publicly.
    pub fn credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    /// Creates a [reqwest::RequestBuilder] used to send a request to an Exercism API.
    /// Takes care of setting the authorization headers using the credentials
    /// provided to the constructor, if any.
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> reqwest::RequestBuilder {
        let builder = self.http_client.request(method, url);
        match &self.credentials {
            Some(creds) => builder.bearer_auth(creds.api_token()),
            None => builder,
        }
    }

    /// Creates a [reqwest::RequestBuilder] used to send a GET request to an Exercism API.
    /// This is a shorthand for [request](Client::request) with `method` = [Method::GET].
    pub fn get<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.request(Method::GET, url)
    }
}
