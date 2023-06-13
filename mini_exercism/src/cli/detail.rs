mod os;

use crate::core::{Error, Result};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub mod helpers {
    use std::{env, fs, io};
    use std::path::Path;
    use crate::cli::detail::os;

    // Note: the methods in this module are indeed used (see cli.rs),
    // but apparently rustc gets confused because of the automock shenanigans.

    #[allow(dead_code)]
    pub fn get_cli_config_dir() -> Option<String> {
        match env::consts::OS {
            "windows" => os::windows::get_cli_config_dir(),
            _ => os::nix::get_cli_config_dir(),
        }
    }

    #[allow(dead_code)]
    pub fn read_to_string(path: &Path) -> io::Result<String> {
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

    mod cli_config {
        use assert_matches::assert_matches;
        use super::*;

        #[test]
        fn test_valid() {
            let config_json = "{\"token\": \"some_token\"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Ok(cli_config) if cli_config.api_token == "some_token");
        }

        #[test]
        fn test_invalid_json() {
            let config_json = "{invalid: json}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ConfigParseError(_)));
        }

        #[test]
        fn test_with_missing_api_token() {
            let config_json = "{\"apibaseurl\": \"some_url\"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ApiTokenNotFoundInConfig));
        }

        #[test]
        fn test_with_empty_token() {
            let config_json = "{\"token\": \"\"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ApiTokenNotFoundInConfig));
        }

        #[test]
        fn test_with_blank_token() {
            let config_json = "{\"token\": \"   \"}";
            let config = CliConfig::from_string(config_json);

            assert_matches!(config, Err(Error::ApiTokenNotFoundInConfig));
        }
    }
}
