//! Helpers for getting language track information from the Exercism API.

use mini_exercism::api;
use mini_exercism::api::v2::solution::Solution;
use mini_exercism::api::v2::tracks::StatusFilter::Joined;
use mini_exercism::api::v2::{exercises, tracks};

use crate::Result;

/// Returns the name of all language tracks joined by the user.
///
/// The names returned are the internal names, or "slug". See [`Track::name`](api::v2::Track::name).
pub async fn get_joined_tracks(client: &api::v2::Client) -> Result<Vec<String>> {
    let filters = tracks::Filters::builder().status(Joined).build();
    let tracks = client.get_tracks(Some(filters)).await?.tracks;

    Ok(tracks.into_iter().map(|track| track.name).collect())
}

/// Returns all solutions submitted by the user for the given track.
pub async fn get_solutions<T>(client: &api::v2::Client, track: T) -> Result<Vec<Solution>>
where
    T: AsRef<str>,
{
    let filters = exercises::Filters::builder()
        .include_solutions(true)
        .build();

    Ok(client
        .get_exercises(track.as_ref(), Some(filters))
        .await?
        .solutions)
}
