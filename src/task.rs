//! Helpers to deal with concurrent tasks.

use std::panic::resume_unwind;

use tokio::task::JoinSet;

pub async fn wait_for_all(join_set: &mut JoinSet<crate::Result<()>>) -> crate::Result<()> {
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(closure_result) => closure_result?,
            Err(err) => resume_unwind(err.into_panic()),
        }
    }

    Ok(())
}
