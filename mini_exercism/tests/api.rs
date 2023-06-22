pub mod api_helpers;

mod client {
    use reqwest::{Method, StatusCode};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{bearer_token, header_exists, method, path};
    use mini_exercism::api::Client;
    use mini_exercism::core::Credentials;
    use crate::api_helpers::not;

    const API_TOKEN: &str = "some_api_token";
    const TEST_HEADER: &str = "x-mini_exercism-test";

    const ANONYMOUS_API_PATH: &str = "/anonymous";
    const AUTHENTICATED_API_PATH: &str = "/authenticated";
    const TEST_API_PATH: &str = "/test";

    async fn setup_route<T: Into<String>>(mock_server: &MockServer, route: T, anonymous: bool, test: bool) {
        let mut mock = Mock::given(method("GET"))
            .and(path(route));

        if anonymous {
            mock = mock.and(not(header_exists("authorization")));
        } else {
            mock = mock.and(bearer_token(API_TOKEN));
        }

        if test {
            mock = mock.and(header_exists(TEST_HEADER));
        } else {
            mock = mock.and(not(header_exists(TEST_HEADER)));
        }

        mock.respond_with(ResponseTemplate::new(StatusCode::OK))
            .mount(mock_server)
            .await;
    }

    async fn setup_mock_server() -> MockServer {
        let mock_server = MockServer::start().await;

        setup_route(&mock_server, ANONYMOUS_API_PATH, true, false).await;
        setup_route(&mock_server, format!("{}{}", ANONYMOUS_API_PATH, TEST_API_PATH), true, true).await;
        setup_route(&mock_server, AUTHENTICATED_API_PATH, false, false).await;
        setup_route(&mock_server, format!("{}{}", AUTHENTICATED_API_PATH, TEST_API_PATH), false, true).await;

        mock_server
    }

    fn expected_status_code(working_uri: &str, actual_uri: &str) -> StatusCode {
        if working_uri == actual_uri {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    }

    fn get_uri_to_test(mock_server: &MockServer, anonymous: bool, test: bool) -> String {
        match (anonymous, test) {
            (true, false) => format!("{}{}", mock_server.uri(), ANONYMOUS_API_PATH),
            (true, true) => format!("{}{}{}", mock_server.uri(), ANONYMOUS_API_PATH, TEST_API_PATH),
            (false, false) => format!("{}{}", mock_server.uri(), AUTHENTICATED_API_PATH),
            (false, true) => format!("{}{}{}", mock_server.uri(), AUTHENTICATED_API_PATH, TEST_API_PATH),
        }
    }

    async fn test_routes(mock_server: &MockServer, client: &Client, anonymous: bool, test: bool) {
        let working_uri = get_uri_to_test(mock_server, anonymous, test);

        let uris = vec![
            get_uri_to_test(mock_server, true, false),
            get_uri_to_test(mock_server, true, true),
            get_uri_to_test(mock_server, false, false),
            get_uri_to_test(mock_server, false, false),
        ];

        for uri in &uris {
            let request_status = client.request(Method::GET, uri)
                .send()
                .await
                .unwrap()
                .status();
            let get_status = client.get(uri)
                .send()
                .await
                .unwrap()
                .status();

            let expected_status = expected_status_code(working_uri.as_str(), uri);
            assert_eq!(request_status, expected_status);
            assert_eq!(get_status, expected_status);
        }
    }

    fn create_test_http_client() -> reqwest::Client {
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(TEST_HEADER, reqwest::header::HeaderValue::from_static("any_value_will_do"));

        reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap()
    }

    fn create_anonymous_credentials() -> Option<Credentials> {
        None
    }

    fn create_authenticated_credentials() -> Option<Credentials> {
        Some(Credentials::from_api_token(API_TOKEN))
    }

    fn create_anonymous_client() -> Client {
        Client::with_default_http_client(create_anonymous_credentials())
    }

    fn create_anonymous_test_client() -> Client {
        Client::with_custom_http_client(create_test_http_client(), create_anonymous_credentials())
    }

    fn create_authenticated_client() -> Client {
        Client::with_default_http_client(create_authenticated_credentials())
    }

    fn create_authenticated_test_client() -> Client {
        Client::with_custom_http_client(create_test_http_client(), create_authenticated_credentials())
    }

    #[tokio::test]
    async fn with_default_client_and_no_credentials() {
        let mock_server = setup_mock_server().await;
        let client = create_anonymous_client();

        test_routes(&mock_server, &client, true, false).await;
    }

    #[tokio::test]
    async fn with_default_client_and_credentials() {
        let mock_server = setup_mock_server().await;
        let client = create_authenticated_client();

        test_routes(&mock_server, &client, false, false).await;
    }

    #[tokio::test]
    async fn with_test_client_and_no_credentials() {
        let mock_server = setup_mock_server().await;
        let client = create_anonymous_test_client();

        test_routes(&mock_server, &client, true, true).await;
    }

    #[tokio::test]
    async fn with_test_client_and_credentials() {
        let mock_server = setup_mock_server().await;
        let client = create_authenticated_test_client();

        test_routes(&mock_server, &client, false, true).await;
    }
}
