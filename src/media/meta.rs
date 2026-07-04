use crate::prelude::*;

/// The media metadata
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,

    pub duration: Duration,
    pub position: Duration,

    pub playing: bool,

    pub artwork_url: Option<String>,
}
