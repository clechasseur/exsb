pub mod args;

use crate::commands::backup::args::BackupArgs;
use crate::credentials::get_api_credentials;

pub fn backup_solutions(args: &BackupArgs) -> crate::Result<()> {
    println!("Args: {:?}", args);

    match get_api_credentials(args.token.as_ref()) {
        Ok(credentials) => println!("Credentials: {:?}", credentials),
        Err(err) => println!("No credentials, error: {:?}", err),
    }

    Ok(())
}
