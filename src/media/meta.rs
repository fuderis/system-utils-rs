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

impl std::fmt::Display for MediaMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== MEDIA METADATA ===")?;
        writeln!(f, "Title         : {}", self.title)?;
        writeln!(f, "Artist        : {}", self.artist)?;
        writeln!(f, "Album         : {}", self.album)?;
        writeln!(f, "Duration      : {:.2?}", self.duration)?;
        writeln!(f, "Position      : {:.2?}", self.position)?;
        writeln!(
            f,
            "Playing       : {}",
            if self.playing { "Yes" } else { "No" }
        )?;
        writeln!(
            f,
            "Artwork URL   : {}",
            self.artwork_url.as_deref().unwrap_or("None")
        )?;

        Ok(())
    }
}
