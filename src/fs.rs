//! Filesystem-related helpers.

use std::path::Path;

use anyhow::Context;
use tokio::fs;

pub async fn delete_directory_content(directory_path: &Path) -> crate::Result<()> {
    let mut destination_entries = fs::read_dir(directory_path).await.with_context(|| {
        format!("failed to get content of directory {}", directory_path.display())
    })?;

    loop {
        let entry = destination_entries.next_entry().await.with_context(|| {
            format!("failed to get next entry of directory {}", directory_path.display())
        })?;

        if let Some(entry) = entry {
            let entry_type = entry.file_type().await.with_context(|| {
                format!("failed to fetch file type for entry {}", entry.path().display())
            })?;

            if entry_type.is_dir() {
                fs::remove_dir_all(entry.path()).await.with_context(|| {
                    format!("failed to remove directory {}", entry.path().display())
                })?;
            } else {
                fs::remove_file(entry.path())
                    .await
                    .with_context(|| format!("failed to remove file {}", entry.path().display()))?;
            }
        } else {
            break;
        }
    }

    Ok(())
}
