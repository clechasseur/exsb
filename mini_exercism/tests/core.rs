use mini_exercism::core::Credentials;

#[test]
fn test_credentials_from_api_token() {
    let api_token = "some_token";
    let credentials = Credentials::from_api_token(api_token.to_string());

    assert_eq!(credentials.api_token(), api_token);
}
