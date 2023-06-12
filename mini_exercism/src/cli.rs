//! Utilities to interact with the Exercism CLI application.

mod detail;

use std::{env, fs, io};
use std::path::PathBuf;
use crate::cli::detail::{CliConfig, get_cli_config_dir};
use crate::core::{Credentials, Error, Result};

/// Reads API credentials from the CLI config file and returns them.
pub fn get_cli_credentials() -> Result<Credentials> {
    let config_dir = get_cli_config_dir()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        .or_else(|_| env::current_dir().map(|path| path.to_string_lossy().to_string()))?;

    let config_file_path: PathBuf = [config_dir, "user.json".to_string()].iter().collect();
    match fs::read_to_string(config_file_path) {
        Ok(config) => {
            let config = CliConfig::from_string(config.as_str())?;
            Ok(Credentials::from_api_token(config.api_token))
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(Error::ConfigNotFound),
        Err(err) => Err(Error::from(err)),
    }
}
