use derive_builder::Builder;
use reqwest::{Client, IntoUrl, Method, RequestBuilder};
use crate::core::Credentials;

#[derive(Builder)]
#[builder(build_fn(error = "crate::core::Error"))]
pub struct ApiClient {
    #[builder(default)]
    http_client: Client,

    #[builder(setter(custom))]
    api_base_url: String,

    #[builder(default, setter(strip_option))]
    credentials: Option<Credentials>,
}

impl ApiClient {
    pub fn build() -> ApiClientBuilder {
        ApiClientBuilder::default()
    }

    pub fn request<'a, U: Into<&'a str>>(&self, method: Method, url: U) -> RequestBuilder {
        let builder = self.http_client.request(method, self.api_url(url));
        match &self.credentials {
            Some(creds) => builder.bearer_auth(creds.api_token()),
            None => builder,
        }
    }

    pub fn get<'a, U: Into<&'a str>>(&self, url: U) -> reqwest::RequestBuilder {
        self.request(Method::GET, url)
    }

    fn api_url<'a, U: Into<&'a str>>(&self, url: U) -> String {
        format!("{}{}", self.api_base_url, url.into())
    }
}

impl ApiClientBuilder {
    pub fn api_base_url<'a, U: Into<&'a str>>(&mut self, url: U) -> &mut Self {
        let url_s = url.into();
        let api_base_url = if url_s.ends_with('/') {
            url_s.to_string()
        } else {
            format!("{}/", url_s)
        };

        self.api_base_url = Some(api_base_url);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const API_TOKEN: &str = "some_api_token";
    const TEST_HEADER: &str = "x-mini_exercism-test";

    const ANONYMOUS_API_PATH: &str = "anonymous";
    const AUTHENTICATED_API_PATH: &str = "authenticated";
    const TEST_API_PATH: &str = "test";
}
