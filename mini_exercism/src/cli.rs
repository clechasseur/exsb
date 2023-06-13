//! Utilities to interact with the Exercism CLI application.

mod detail;

use std::{env, io};
use std::path::PathBuf;
use mockall_double::double;
use crate::cli::detail::CliConfig;
use crate::core::{Credentials, Error, Result};

#[double]
use crate::cli::detail::helpers;

/// Reads API credentials from the CLI config file and returns them.
///
/// # Errors
///
/// - [`ConfigNotFound`]: CLI config file cannot be found, maybe CLI is not installed
/// - [`ConfigReadError`]: I/O error reading the config file
/// - [`ConfigParseError`]: Config file JSON could not be parsed
/// - [`ApiTokenNotFoundInConfig`]: Config file did not contain an API token
///
/// [`ConfigNotFound`]: crate::core::Error#variant.ConfigNotFound
/// [`ConfigReadError`]: crate::core::Error#variant.ConfigReadError
/// [`ConfigParseError`]: crate::core::Error#variant.ConfigParseError
/// [`ApiTokenNotFoundInConfig`]: crate::core::Error#variant.ApiTokenNotFoundInConfig
pub fn get_cli_credentials() -> Result<Credentials> {
    let config_dir = helpers::get_cli_config_dir()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .or_else(|_| env::current_dir().map(|path| path.to_string_lossy().to_string()))?;

    let config_file_path: PathBuf = [config_dir, "user.json".to_string()].iter().collect();
    match helpers::read_to_string(config_file_path.as_path()) {
        Ok(config) => {
            let config = CliConfig::from_string(config.as_str())?;
            Ok(Credentials::from_api_token(config.api_token))
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(Error::ConfigNotFound),
        Err(err) => Err(Error::from(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_cli_credentials {
        use assert_matches::assert_matches;
        use mockall::predicate::*;
        use serial_test::serial;
        use super::*;

        #[test]
        #[serial]
        fn test_valid() {
            let expected_config_dir = "/some/config/dir".to_string();
            let gccd_ctx = helpers::get_cli_config_dir_context();
            gccd_ctx.expect()
                .return_once(move || Some(expected_config_dir));

            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            let expected_json_file = "{\"token\": \"some_token\"}".to_string();
            let rts_ctx = helpers::read_to_string_context();
            rts_ctx.expect()
                .with(eq(expected_config_path))
                .return_once(move |_| Ok(expected_json_file));

            assert_matches!(get_cli_credentials(),
                Ok(creds) if creds == Credentials::from_api_token("some_token".to_string()));
        }

        #[test]
        #[serial]
        fn test_no_config_dir() {
            let gccd_ctx = helpers::get_cli_config_dir_context();
            gccd_ctx.expect()
                .return_once(|| None);

            let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
            let expected_config_path: PathBuf = [current_dir, "user.json".to_string()].iter().collect();
            let expected_json_file = "{\"token\": \"some_token\"}".to_string();
            let rts_ctx = helpers::read_to_string_context();
            rts_ctx.expect()
                .with(eq(expected_config_path))
                .return_once(move |_| Ok(expected_json_file));

            assert_matches!(get_cli_credentials(),
                Ok(creds) if creds == Credentials::from_api_token("some_token".to_string()));
        }

        #[test]
        #[serial]
        fn test_invalid_config() {
            let expected_config_dir = "/some/config/dir".to_string();
            let gccd_ctx = helpers::get_cli_config_dir_context();
            gccd_ctx.expect()
                .return_once(move || Some(expected_config_dir));

            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            let expected_json_file = "{invalid: json}".to_string();
            let rts_ctx = helpers::read_to_string_context();
            rts_ctx.expect()
                .with(eq(expected_config_path))
                .return_once(move |_| Ok(expected_json_file));

            assert_matches!(get_cli_credentials(), Err(Error::ConfigParseError(_)));
        }

        #[test]
        #[serial]
        fn test_config_file_not_found() {
            let expected_config_dir = "/some/config/dir".to_string();
            let gccd_ctx = helpers::get_cli_config_dir_context();
            gccd_ctx.expect()
                .return_once(move || Some(expected_config_dir));

            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            let rts_ctx = helpers::read_to_string_context();
            rts_ctx.expect()
                .with(eq(expected_config_path))
                .return_once(|_| Err(io::Error::from(io::ErrorKind::NotFound)));

            assert_matches!(get_cli_credentials(), Err(Error::ConfigNotFound));
        }

        #[test]
        #[serial]
        fn test_config_file_inaccessible() {
            let expected_config_dir = "/some/config/dir".to_string();
            let gccd_ctx = helpers::get_cli_config_dir_context();
            gccd_ctx.expect()
                .return_once(move || Some(expected_config_dir));

            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            let rts_ctx = helpers::read_to_string_context();
            rts_ctx.expect()
                .with(eq(expected_config_path))
                .return_once(|_| Err(io::Error::from(io::ErrorKind::PermissionDenied)));

            assert_matches!(get_cli_credentials(), Err(Error::ConfigReadError(_)));
        }
    }
}
