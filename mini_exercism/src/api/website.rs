//! Types and functions to interact with the Exercism website API.

mod detail;

use derive_builder::Builder;
use serde::Deserialize;
use strum_macros::{AsRefStr, Display};
use crate::api;
use crate::api::website::detail::DEFAULT_WEBSITE_API_BASE_URL;
use crate::core::Result;

/// Client for the Exercism website API. This API is undocumented and
/// is mostly used by the website itself to fetch information.
pub struct Client {
    api_client: api::Client,
    api_base_url: String,
}

impl Client {
    /// Creates a new client for the Exercism website API.
    ///
    /// # Arguments
    ///
    /// - `api_client` - The [Exercism API client](api::Client) used to perform requests.
    pub fn new(api_client: api::Client) -> Self {
        Self::with_custom_api_base_url(api_client, DEFAULT_WEBSITE_API_BASE_URL)
    }

    /// Creates a new client for the Exercism website API using the provided API base URL.
    /// This is meant to be used for testing purposes.
    ///
    /// # Arguments
    ///
    /// - `api_client` - The [Exercism API client](api::Client) used to perform requests.
    /// - `api_base_url` - Base URL of the Exercism website API. Must not end with `/`.
    pub fn with_custom_api_base_url<T: Into<String>>(api_client: api::Client, api_base_url: T) -> Self {
        Self {
            api_client,
            api_base_url: api_base_url.into(),
        }
    }

    /// Returns a list of Exercism tracks.
    /// - If the request is performed anonymously (see [api::Client::credentials]),
    ///   will return a list of all tracks supported on the website.
    /// - If the request is performed with credentials, tracks that the user has joined will be
    ///   identified by the `is_joined` field.
    ///
    /// # Arguments
    ///
    /// TODO
    ///
    /// # Errors
    ///
    /// - [`ApiError`]: Error while fetching track information from API
    ///
    /// [`ApiError`]: crate::core::Error#variant.ApiError
    pub async fn get_tracks(&self, status_filter: TrackStatusFilter) -> Result<Tracks> {
        Ok(self.api_client.get(self.api_url("tracks"))
            .query(&[("status", status_filter.as_ref())])
            .send()
            .await?
            .json::<Tracks>()
            .await?)
    }

    fn api_url(&self, url: &str) -> String {
        format!("{}/{}", self.api_base_url, url)
    }
}

/// Filters that can be applied when fetching language tracks from the Exercism website API.
#[derive(Debug, Default, Builder)]
#[builder(derive(Debug), default, build_fn(name = "fallible_build", error = "crate::core::Error"))]
pub struct TrackFilters {
    /// Criteria used to filter language tracks.
    /// Applied to both track `name`s (e.g. slugs) and `title`s.
    #[builder(setter(into, strip_option))]
    pub criteria: Option<String>,

    /// List of `tags` that must be attached to the language track.
    #[builder(setter(each(name = "tag", into)))]
    pub tags: Vec<String>,

    /// Language track's status filter.
    #[builder(setter(strip_option))]
    pub status: Option<TrackStatusFilter>,
}

impl TrackFiltersBuilder {
    pub fn build(&self) -> TrackFilters {
        self.fallible_build().unwrap()
    }
}

/// Possible status filter of Exercism language tracks.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Display, AsRefStr)]
pub enum TrackStatusFilter {
    /// Return all language tracks.
    #[default]
    #[strum(serialize = "all")]
    All,

    /// Return only language tracks joined by the user.
    #[strum(serialize = "joined")]
    Joined,

    /// Return only language tracks *not* joined by the user.
    #[strum(serialize = "unjoined")]
    Unjoined,
}

/// Struct used to return Exercism language tracks from the website API.
#[derive(Debug, Deserialize)]
pub struct Tracks {
    /// List of Exercism language tracks. Usually sorted alphabetically by track name.
    pub tracks: Vec<Track>,
}

/// Struct representing a single language track returned by the Exercism website API.
#[derive(Debug, Deserialize)]
pub struct Track {
    /// Name of the language track.
    /// This is an internal name, like `common-lisp`. Also called `slug`.
    #[serde(rename = "slug")]
    pub name: String,

    /// Language track title.
    /// This is a textual representation of the track name, like `Common Lisp`.
    pub title: String,

    /// Total number of concepts taught by the track.
    pub num_concepts: usize,

    /// Total number of exercises available in the track.
    pub num_exercises: usize,

    /// URL of this language track on the Exercism website.
    pub web_url: String,

    /// URL of the icon representing this language track on the Exercism website.
    pub icon_url: String,

    /// List of tags attached to this language track.
    /// Can contain many information, like `Object-oriented`, `Linux`, etc.
    pub tags: Vec<String>,

    /// Struct containing some links pertaining to the language track.
    pub links: TrackLinks,

    /// Whether this track has been joined by the user.
    /// Will be set to `false` for anonymous queries or unjoined tracks.
    #[serde(default)]
    pub is_joined: bool,

    /// Number of concepts learnt by the user in this track.
    /// Will be set to `0` for anonymous queries or unjoined tracks.
    #[serde(default)]
    pub num_learnt_concepts: usize,

    /// Number of exercises completed by the user in this track.
    /// Will be set to `0` for anonymous queries or unjoined tracks.
    #[serde(default)]
    pub num_completed_exercises: usize,
}

/// Struct containing links pertaining to an Exercism language track returned by the website API.
#[derive(Debug, Deserialize)]
pub struct TrackLinks {
    /// URL of the language track's exercises on the Exercism website.
    pub exercises: String,

    /// URL of the language track's concepts on the Exercism website.
    pub concepts: String,
}
