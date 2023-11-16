use std::borrow::Cow;
use std::path::Path;

use anyhow::Context;
use tracing::instrument;

#[instrument(skip(tracks))]
pub async fn create_track_directories(
    output_path: &Path,
    tracks: &Vec<Cow<'_, str>>,
) -> crate::Result<()> {
    for track in tracks {
        let mut destination_path = output_path.to_path_buf();
        destination_path.push(track.as_ref());

        tokio::fs::create_dir_all(&destination_path)
            .await
            .with_context(|| format!("failed to create directory for track {}", track))?;
    }

    Ok(())
}
