mod os;

use std::{env, fs, io};
use std::path::{Path, PathBuf};
use crate::core::{Credentials, Error, Result};

#[cfg(test)]
use mockall::automock;

pub fn get_cli_credentials<H: CliCredentialsHelper>(helper: &H) -> Result<Credentials> {
    let config_dir = helper.get_cli_config_dir()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .or_else(|_| env::current_dir().map(|path| path.to_string_lossy().to_string()))?;

    let config_file_path: PathBuf = [config_dir, "user.json".to_string()].iter().collect();
    match helper.read_file_to_string(config_file_path.as_path()) {
        Ok(config) => {
            let config = CliConfig::from_string(config.as_str())?;
            Ok(Credentials::from_api_token(config.api_token))
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(Error::ConfigNotFound),
        Err(err) => Err(Error::from(err)),
    }
}

#[cfg_attr(test, automock)]
pub trait CliCredentialsHelper {
    fn get_cli_config_dir(&self) -> Option<String>;
    fn read_file_to_string(&self, path: &Path) -> io::Result<String>;
}

pub struct DefaultCliCredentialsHelper;

impl CliCredentialsHelper for DefaultCliCredentialsHelper {
    fn get_cli_config_dir(&self) -> Option<String> {
        match env::consts::OS {
            "windows" => os::windows::get_cli_config_dir(),
            _ => os::nix::get_cli_config_dir(),
        }
    }

    fn read_file_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }
}

#[derive(Debug)]
pub struct CliConfig {
    pub api_token: String,
}

impl CliConfig {
    pub fn from_string(config: &str) -> Result<Self> {
        let config: serde_json::Value = serde_json::from_str(config)?;

        let token = config["token"].as_str().unwrap_or("").trim();
        match token.is_empty() {
            true => Err(Error::ApiTokenNotFoundInConfig),
            false => Ok(Self { api_token : token.to_string() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_cli_credentials {
        use assert_matches::assert_matches;
        use mockall::predicate::*;
        use super::*;

        #[test]
        fn test_valid_cli_credentials() {
            let mut mock = MockCliCredentialsHelper::new();
            let expected_config_dir = "/some/config/dir".to_string();
            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            let expected_json_file = "{\"token\": \"some_token\"}".to_string();
            mock.expect_get_cli_config_dir()
                .return_once(move || Some(expected_config_dir));
            mock.expect_read_file_to_string()
                .with(eq(expected_config_path))
                .return_once(move |_| Ok(expected_json_file));

            assert_matches!(get_cli_credentials(&mock),
                Ok(creds) if creds == Credentials::from_api_token("some_token".to_string()));
        }

        #[test]
        fn test_no_cli_config_dir() {
            let mut mock = MockCliCredentialsHelper::new();
            let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
            let expected_config_path: PathBuf = [current_dir, "user.json".to_string()].iter().collect();
            let expected_json_file = "{\"token\": \"some_token\"}".to_string();
            mock.expect_get_cli_config_dir()
                .return_once(|| None);
            mock.expect_read_file_to_string()
                .with(eq(expected_config_path))
                .return_once(move |_| Ok(expected_json_file));

            assert_matches!(get_cli_credentials(&mock),
                Ok(creds) if creds == Credentials::from_api_token("some_token".to_string()));
        }

        #[test]
        fn test_invalid_cli_config() {
            let mut mock = MockCliCredentialsHelper::new();
            let expected_config_dir = "/some/config/dir".to_string();
            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            let expected_json_file = "{invalid: json}".to_string();
            mock.expect_get_cli_config_dir()
                .return_once(move || Some(expected_config_dir));
            mock.expect_read_file_to_string()
                .with(eq(expected_config_path))
                .return_once(move |_| Ok(expected_json_file));

            assert_matches!(get_cli_credentials(&mock), Err(Error::ConfigParseError(_)));
        }

        #[test]
        fn test_cli_config_file_not_found() {
            let mut mock = MockCliCredentialsHelper::new();
            let expected_config_dir = "/some/config/dir".to_string();
            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            mock.expect_get_cli_config_dir()
                .return_once(move || Some(expected_config_dir));
            mock.expect_read_file_to_string()
                .with(eq(expected_config_path))
                .return_once(|_| Err(io::Error::from(io::ErrorKind::NotFound)));

            assert_matches!(get_cli_credentials(&mock), Err(Error::ConfigNotFound));
        }

        #[test]
        fn test_cli_config_file_inaccessible() {
            let mut mock = MockCliCredentialsHelper::new();
            let expected_config_dir = "/some/config/dir".to_string();
            let expected_config_path: PathBuf = ["/some/config/dir", "user.json"].iter().collect();
            mock.expect_get_cli_config_dir()
                .return_once(move || Some(expected_config_dir));
            mock.expect_read_file_to_string()
                .with(eq(expected_config_path))
                .return_once(|_| Err(io::Error::from(io::ErrorKind::PermissionDenied)));

            assert_matches!(get_cli_credentials(&mock), Err(Error::ConfigReadError(_)));
        }
    }

    mod cli_config {
        use assert_matches::assert_matches;
        use super::*;

        #[test]
        fn test_valid_cli_config() {
            let config_json = "{\"token\": \"some_token\"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Ok(cli_config) if cli_config.api_token == "some_token");
        }

        #[test]
        fn test_invalid_cli_config() {
            let config_json = "{invalid: json}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ConfigParseError(_)));
        }

        #[test]
        fn test_cli_config_with_missing_api_token() {
            let config_json = "{\"apibaseurl\": \"some_url\"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ApiTokenNotFoundInConfig));
        }

        #[test]
        fn test_cli_config_with_empty_token() {
            let config_json = "{\"token\": \"\"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ApiTokenNotFoundInConfig));
        }

        #[test]
        fn test_cli_config_with_blank_token() {
            let config_json = "{\"token\": \"   \"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ApiTokenNotFoundInConfig));
        }
    }
}
