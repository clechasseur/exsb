//! Helpers to deal with concurrent tasks.

use std::panic::resume_unwind;

use tokio::task::JoinSet;

use crate::Result;

/// Waits until all tasks in a [`JoinSet`] are completed, or until one of them fails.
///
/// If all tasks are successful (e.g. return [`Ok(())`]), the function will return [`Ok(())`].
/// Otherwise, the function will return [`Err`] with the error of the first failing task.
/// Any remaining task in the [`JoinSet`] will be left unchecked (and thus calcelled if the
/// [`JoinSet`] is later dropped).
pub async fn wait_for_all(join_set: &mut JoinSet<Result<()>>) -> Result<()> {
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(closure_result) => closure_result?,
            Err(err) => resume_unwind(err.into_panic()),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod wait_for_all {
        use std::sync::{Arc, Mutex};

        use anyhow::anyhow;
        use tokio::sync::mpsc;

        use super::*;

        #[tokio::test]
        async fn test_success() {
            let mut tasks = JoinSet::new();
            let i = Arc::new(Mutex::new(0));

            for _ in 0..3 {
                let i = i.clone();
                tasks.spawn(async move {
                    let mut i = i.lock().unwrap();
                    *i += 1;

                    Ok(())
                });
            }

            let wait_result = wait_for_all(&mut tasks).await;
            assert!(wait_result.is_ok());
            assert_eq!(3, *i.lock().unwrap());
        }

        #[tokio::test]
        async fn test_failure() {
            let mut tasks = JoinSet::new();
            let i = Arc::new(Mutex::new(0));

            for _ in 0..3 {
                let i = i.clone();
                tasks.spawn(async move {
                    let i = i.lock().unwrap();
                    assert_eq!(0, *i);

                    Err(anyhow!("error"))
                });
            }

            let wait_result = wait_for_all(&mut tasks).await;
            assert!(wait_result.is_err());
            assert_eq!(0, *i.lock().unwrap());
        }

        #[tokio::test]
        async fn test_first_failure() {
            let mut tasks = JoinSet::new();
            let mut txs = Vec::new();
            let i = Arc::new(Mutex::new(0));

            for _ in 0..1 {
                let (tx, mut rx) = mpsc::channel(1);
                txs.push(tx);
                let i = i.clone();
                tasks.spawn(async move {
                    let _ = rx.recv().await;

                    let mut i = i.lock().unwrap();
                    *i += 1;

                    Err(anyhow!("error"))
                });
            }

            txs.first().unwrap().send(()).await.unwrap();

            let wait_result = wait_for_all(&mut tasks).await;
            assert!(wait_result.is_err());
            assert_eq!(1, *i.lock().unwrap());
        }
    }
}
