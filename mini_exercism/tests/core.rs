mod error {
    use std::collections::HashMap;
    use std::io;
    use assert_matches::assert_matches;
    use derive_builder::Builder;
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

    #[test]
    fn test_api_error_from() {
        // There's no way to create a reqwest::Error outside of the reqwest crate, so we'll
        // have to trigger an actual error to test this.
        let map_with_non_string_keys: HashMap<_, _> = [(true, "hello"), (false, "world")].into();
        let client = reqwest::Client::new();
        let reqwest_error = client.get("/test").json(&map_with_non_string_keys).build().unwrap_err();
        let error: Error = reqwest_error.into();

        assert_matches!(error, Error::ApiError(_));
    }

    #[test]
    fn test_uninitialized_field_error_from() {
        #[derive(Debug, Builder)]
        #[builder(build_fn(error = "mini_exercism::core::Error"))]
        struct WithMandatoryField {
            #[allow(dead_code)]
            mandatory_field: String,
        }

        let error: Error = WithMandatoryFieldBuilder::default().build().unwrap_err();

        assert_matches!(error, Error::ApiClientUninitializedField(field_name) if field_name == "mandatory_field");
    }
}
