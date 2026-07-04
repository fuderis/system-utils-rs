pub mod meta;
pub use meta::MediaMetadata;

use crate::prelude::*;
use tokio::process::Command;

/// The media control manager
/// (for Linux requires `playerctl` to be installed)
#[derive(Debug)]
pub struct MediaControl;

impl MediaControl {
    /// Toggles play/pause.
    pub async fn play_pause() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            playerctl(&["play-pause"]).await?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            osascript(r#"tell application "Music" to playpause"#).await?;
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            powershell(r#"(New-Object -ComObject WScript.Shell).SendKeys([char]179)"#).await?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Starts playback.
    pub async fn play() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            playerctl(&["play"]).await?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            osascript(r#"tell application "Music" to play"#).await?;
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            // Play/Pause only
            powershell(r#"(New-Object -ComObject WScript.Shell).SendKeys([char]179)"#).await?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Pauses playback.
    pub async fn pause() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            playerctl(&["pause"]).await?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            osascript(r#"tell application "Music" to pause"#).await?;
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            powershell(r#"(New-Object -ComObject WScript.Shell).SendKeys([char]179)"#).await?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Stops playback.
    pub async fn stop() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            playerctl(&["stop"]).await?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            osascript(r#"tell application "Music" to stop"#).await?;
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            powershell(r#"(New-Object -ComObject WScript.Shell).SendKeys([char]178)"#).await?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Plays the next track.
    pub async fn next_track() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            playerctl(&["next"]).await?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            osascript(r#"tell application "Music" to next track"#).await?;
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            powershell(r#"(New-Object -ComObject WScript.Shell).SendKeys([char]176)"#).await?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Plays the previous track.
    pub async fn previous_track() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            playerctl(&["previous"]).await?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            osascript(r#"tell application "Music" to previous track"#).await?;
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            powershell(r#"(New-Object -ComObject WScript.Shell).SendKeys([char]177)"#).await?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Seeks forward by the specified number of seconds.
    pub async fn seek_forward(seconds: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let offset = format!("{seconds}+");
            playerctl(&["position", &offset]).await?;
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = seconds;
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Seeks backward by the specified number of seconds.
    pub async fn seek_backward(seconds: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let offset = format!("{seconds}-");
            playerctl(&["position", &offset]).await?;
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = seconds;
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Returns the current playback position.
    pub async fn position() -> Result<Duration> {
        #[cfg(target_os = "linux")]
        {
            let output = playerctl(&["position"]).await?;
            let seconds: f64 = output.parse()?;

            Ok(Duration::from_secs_f64(seconds))
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Returns the media duration.
    pub async fn duration() -> Result<Duration> {
        #[cfg(target_os = "linux")]
        {
            let output = playerctl(&["metadata", "mpris:length"]).await?;
            let micros: u64 = output.parse()?;

            Ok(Duration::from_micros(micros))
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }

    /// Returns media metadata.
    pub async fn metadata() -> Result<MediaMetadata> {
        #[cfg(target_os = "linux")]
        {
            let metadata = playerctl(&[
                "metadata",
                "--format",
                concat!(
                    "{{title}}\n",
                    "{{artist}}\n",
                    "{{album}}\n",
                    "{{mpris:length}}\n",
                    "{{status}}\n",
                    "{{mpris:artUrl}}"
                ),
            ])
            .await?;

            let position = playerctl(&["position"]).await?;

            let mut lines = metadata.lines();

            let title = lines.next().unwrap_or_default().to_owned();
            let artist = lines.next().unwrap_or_default().to_owned();
            let album = lines.next().unwrap_or_default().to_owned();

            let duration = Duration::from_micros(lines.next().unwrap_or("0").parse::<u64>()?);

            let playing = matches!(lines.next().unwrap_or_default(), "Playing");

            let artwork_url = match lines.next().unwrap_or_default() {
                "" => None,
                url => Some(url.to_owned()),
            };

            let position = Duration::from_secs_f64(position.parse::<f64>()?);

            Ok(MediaMetadata {
                title,
                artist,
                album,
                duration,
                position,
                playing,
                artwork_url,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::UnsupportedOS.into())
        }
    }
}

#[cfg(target_os = "linux")]
async fn playerctl(args: &[&str]) -> Result<String> {
    let output = Command::new("playerctl").args(args).output().await?;

    if !output.status.success() {
        return Err(MediaError::PlayerNotFound.into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(target_os = "macos")]
async fn osascript(script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(target_os = "windows")]
async fn powershell(script: &str) -> Result<String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
