pub const WEBSITE_API_BASE_URL: &str = "https://exercism.org/api/v2";

pub fn website_api_url(url: &str) -> String {
    format!("{}/{}", WEBSITE_API_BASE_URL, url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_api_url() {
        let url = website_api_url("tracks");

        assert_eq!(url, format!("{}/tracks", WEBSITE_API_BASE_URL));
    }
}
