//! Utilities to interact with the Exercism CLI application.

use std::{env, fs, io};
use std::path::PathBuf;
use crate::core::{Credentials, Result};

/// Reads API credentials from the CLI config file and returns them.
pub fn get_cli_credentials() -> Result<Credentials> {
    let config_dir = get_cli_config_dir()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .or_else(|_| env::current_dir().map(|path| path.to_string_lossy().to_string()))?;

    let config_file_path: PathBuf = [config_dir, "user.json".to_string()].iter().collect();
    let config = fs::read_to_string(config_file_path)?;
    let config = CliConfig::from_string(config.as_str())?;

    Ok(Credentials::from_api_token(config.api_token))
}

fn get_cli_config_dir() -> Option<String> {
    match env::consts::OS {
        "windows" => os::windows::get_cli_config_dir(),
        _ => os::nix::get_cli_config_dir(),
    }
}

struct CliConfig {
    pub api_token: String,
}

impl CliConfig {
    fn from_string(config: &str) -> Result<Self> {
        let config: serde_json::Value = serde_json::from_str(config)?;

        Ok(Self {
            api_token: config["token"].to_string(),
        })
    }
}

mod os {
    pub mod windows {
        use std::env;
        use std::path::PathBuf;

        pub fn get_cli_config_dir() -> Option<String> {
            let path: PathBuf = [env::var_os("APPDATA")?, "exercism".into()].iter().collect();

            Some(path.to_str()?.to_string())
        }

        #[cfg(test)]
        mod tests {
            use std::env;
            use std::path::MAIN_SEPARATOR;
            use assert_matches::assert_matches;
            use serial_test::serial;
            use super::*;

            #[test]
            #[serial]
            fn test_config_dir_valid() {
                let app_data = "C:\\Users\\some_user\\AppData\\Roaming";
                env::set_var("APPDATA", app_data);
                let config_dir = get_cli_config_dir();

                assert_matches!(config_dir,
                    Some(dir) if dir == format!("{}{}{}", app_data, MAIN_SEPARATOR, "exercism"));
            }

            #[test]
            #[serial]
            fn test_config_dir_invalid() {
                env::remove_var("APPDATA");
                let config_dir = get_cli_config_dir();

                assert_matches!(config_dir, None);
            }
        }
    }

    pub mod nix {
        use std::env;
        use std::path::PathBuf;

        pub fn get_cli_config_dir() -> Option<String> {
            let mut path: PathBuf;

            if let Some(config_home) = env::var_os("EXERCISM_CONFIG_HOME") {
                path = config_home.into();
            } else {
                if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
                    path = config_home.into();
                } else {
                    path = env::var_os("HOME")?.into();
                    path.push(".config");
                }
                path.push("exercism");
            }

            Some(path.to_str()?.to_string())
        }

        #[cfg(test)]
        mod tests {
            use std::path::MAIN_SEPARATOR;
            use assert_matches::assert_matches;
            use serial_test::serial;
            use super::*;

            #[test]
            #[serial]
            fn test_config_dir_from_exercism_config_home() {
                let exercism_config_home = "/some/config/home";
                env::set_var("EXERCISM_CONFIG_HOME", exercism_config_home);
                let config_dir = get_cli_config_dir();

                assert_matches!(config_dir, Some(dir) if dir == exercism_config_home);
            }

            #[test]
            #[serial]
            fn test_config_dir_from_xdg_config_home() {
                let xdg_config_home = "/some/config/home";
                env::remove_var("EXERCISM_CONFIG_HOME");
                env::set_var("XDG_CONFIG_HOME", xdg_config_home);
                let config_dir = get_cli_config_dir();

                assert_matches!(config_dir,
                    Some(dir) if dir == format!("{}{}{}", xdg_config_home, MAIN_SEPARATOR, "exercism"));
            }

            #[test]
            #[serial]
            fn test_config_dir_from_home() {
                let home = "/some/home";
                env::remove_var("EXERCISM_CONFIG_HOME");
                env::remove_var("XDG_CONFIG_HOME");
                env::set_var("HOME", home);
                let config_dir = get_cli_config_dir();

                assert_matches!(config_dir,
                    Some(dir) if dir == format!("{}{}{}{}{}", home, MAIN_SEPARATOR, ".config", MAIN_SEPARATOR, "exercism"));
            }

            #[test]
            #[serial]
            fn test_config_dir_invalid() {
                env::remove_var("EXERCISM_CONFIG_HOME");
                env::remove_var("XDG_CONFIG_HOME");
                env::remove_var("HOME");
                let config_dir = get_cli_config_dir();

                assert_matches!(config_dir, None);
            }
        }
    }
}
