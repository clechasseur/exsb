use std::env;
use std::path::PathBuf;

pub fn get_cli_config_dir() -> Option<String> {
    // Based on:
    // https://github.com/exercism/cli/blob/9e1285b62502f3f5a4a896a44e540ee1bee5c1bf/config/config.go#L62-L72

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
    use super::*;

    mod get_cli_config_dir {
        use std::path::MAIN_SEPARATOR;
        use serial_test::serial;
        use super::*;

        #[test]
        #[serial]
        fn test_from_exercism_config_home() {
            let exercism_config_home = "/some/config/home";
            env::set_var("EXERCISM_CONFIG_HOME", exercism_config_home);
            let config_dir = get_cli_config_dir();

            assert_eq!(config_dir, Some(exercism_config_home.to_string()));
        }

        #[test]
        #[serial]
        fn test_from_xdg_config_home() {
            let xdg_config_home = "/some/config/home";
            env::remove_var("EXERCISM_CONFIG_HOME");
            env::set_var("XDG_CONFIG_HOME", xdg_config_home);
            let config_dir = get_cli_config_dir();

            assert_eq!(config_dir, Some(format!("{}{}{}", xdg_config_home, MAIN_SEPARATOR, "exercism")));
        }

        #[test]
        #[serial]
        fn test_from_home() {
            let home = "/some/home";
            env::remove_var("EXERCISM_CONFIG_HOME");
            env::remove_var("XDG_CONFIG_HOME");
            env::set_var("HOME", home);
            let config_dir = get_cli_config_dir();

            assert_eq!(config_dir, Some(format!("{}{}{}{}{}", home, MAIN_SEPARATOR, ".config", MAIN_SEPARATOR, "exercism")));
        }

        #[test]
        #[serial]
        fn test_invalid() {
            env::remove_var("EXERCISM_CONFIG_HOME");
            env::remove_var("XDG_CONFIG_HOME");
            env::remove_var("HOME");
            let config_dir = get_cli_config_dir();

            assert_eq!(config_dir, None);
        }
    }
}
