mod credentials {
    use mini_exercism::core::Credentials;

    #[test]
    fn test_eq() {
        let api_token = "some_token";
        let credentials = Credentials::from_api_token(api_token);
        let expected = Credentials::from_api_token(api_token);

        assert_eq!(credentials, expected);
    }

    #[test]
    fn test_ne() {
        let api_token = "some_token";
        let credentials = Credentials::from_api_token(api_token);
        let not_expected = Credentials::from_api_token("some_other_token");

        assert_ne!(credentials, not_expected);
    }
}

mod error {
    use std::io;
    use assert_matches::assert_matches;
    use mini_exercism::core::Error;

    #[test]
    fn test_config_read_error_from() {
        let error: Error = io::Error::from(io::ErrorKind::NotFound).into();

        assert_matches!(error, Error::ConfigReadError(_));
    }

    #[test]
    fn test_config_parse_error_from() {
        let invalid_json = "{hello: world}";
        let error: Error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err().into();

        assert_matches!(error, Error::ConfigParseError(_));
    }
}
