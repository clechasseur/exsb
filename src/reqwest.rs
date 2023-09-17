pub fn get_http_client() -> crate::Result<reqwest::Client> {
    Ok(reqwest::Client::builder().build()?)
}
