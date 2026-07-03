use crate::prelude::*;
use tokio::process::Command;

#[cfg(target_os = "linux")]
const OS_MAX_VOLUME: u32 = 200;
#[cfg(not(target_os = "linux"))]
const OS_MAX_VOLUME: u32 = 100;

/// The audio volume manager
#[derive(Debug)]
pub struct AudioControl;

impl AudioControl {
    /// Returns the audio volume [0-100]% (max 200% on linux)
    pub async fn get_volume() -> Result<u32> {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("pactl")
                .args(["get-sink-volume", "@DEFAULT_SINK@"])
                .output()
                .await
                .map_err(|e| AudioError::GetVolume(e.into()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let re = re!(r"(\d+)%");

            if let Some(caps) = re.captures(&stdout)
                && let Some(vol_str) = caps.get(1)
            {
                Ok(vol_str.as_str().parse()?)
            } else {
                Err(AudioError::GetVolume(AudioError::DevicesNotFound.into()).into())
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("osascript")
                .args(["-e", "output volume of (get volume settings)"])
                .output()
                .await
                .map_err(|e| AudioError::GetVolume(e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim().parse::<u32>()?)
        }

        #[cfg(target_os = "windows")]
        {
            let script = "$w=(New-Object -ComObject MMDeviceEnumerator).GetDefaultAudioEndpoint(0,0).AudioEndpointVolume; \
                          [int]($w.GetMasterVolumeLevelScalar() * 100)";

            let output = Command::new("powershell")
                .args(["-Command", &script])
                .output()
                .await
                .map_err(|e| AudioError::GetVolume(e.into()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .trim()
                .parse::<u32>()
                .map_err(|_| AudioError::GetVolume(AudioError::DevicesNotFound.into()))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(AudioError::UnsupportedOS.into())
        }
    }

    /// Sets the audio volume [0..100]% (max 200% on linux)
    pub async fn set_volume(vol: u32) -> Result<()> {
        let vol = vol.min(OS_MAX_VOLUME);

        #[cfg(target_os = "linux")]
        {
            Command::new("pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", &str!("{vol}%")])
                .status()
                .await
                .map_err(|e| AudioError::SetVolume(e.into()))?;

            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("osascript")
                .args(["-e", &str!("set volume output volume {vol}")])
                .status()
                .await
                .map_err(|e| AudioError::SetVolume(e.into()))?;

            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            let normalized_vol = (vol as f32) / 100.0;
            let script = str!(
                "$w=(New-Object -ComObject MMDeviceEnumerator).GetDefaultAudioEndpoint(0,0).AudioEndpointVolume; \
                 $w.SetMasterVolumeLevelScalar({normalized_vol}, $null)"
            );

            Command::new("powershell")
                .args(["-Command", &script])
                .status()
                .await
                .map_err(|e| AudioError::SetVolume(e.into()))?;

            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(AudioError::UnsupportedOS.into())
        }
    }

    /// Sets the audio volume by delta [-100..100]%
    pub async fn set_volume_delta(delta: i32) -> Result<u32> {
        let current_vol = Self::get_volume().await?.clamp(0, OS_MAX_VOLUME);
        let calculated = (current_vol as i32) + delta;
        let target_vol = calculated.clamp(0, OS_MAX_VOLUME as i32) as u32;

        if target_vol != current_vol {
            Self::set_volume(target_vol).await?;
        }

        Ok(target_vol)
    }

    /// Increases the audio volume
    pub async fn increase_volume(add: u32) -> Result<u32> {
        Self::set_volume_delta(add as i32).await
    }

    /// Decreases the audio volume
    pub async fn decrease_volume(reduce: u32) -> Result<u32> {
        Self::set_volume_delta(-(reduce as i32)).await
    }

    /// Returns true if audio is muted
    pub async fn is_muted() -> Result<bool> {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("pactl")
                .args(["get-sink-mute", "@DEFAULT_SINK@"])
                .output()
                .await
                .map_err(|e| AudioError::GetMute(e.into()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.contains("yes"))
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("osascript")
                .args(["-e", "output muted of (get volume settings)"])
                .output()
                .await
                .map_err(|e| AudioError::GetMute(e.into()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim() == "true")
        }

        #[cfg(target_os = "windows")]
        {
            let script = "$w=(New-Object -ComObject MMDeviceEnumerator).GetDefaultAudioEndpoint(0,0).AudioEndpointVolume; $w.Mute";
            let output = Command::new("powershell")
                .args(["-Command", &script])
                .output()
                .await
                .map_err(|e| AudioError::GetMute(e.into()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim() == "True")
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(AudioError::UnsupportedOS.into())
        }
    }

    /// Mutes/unmutes the audio volume
    pub async fn set_mute(mute: bool) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mute_arg = if mute { "1" } else { "0" };
            Command::new("pactl")
                .args(["set-sink-mute", "@DEFAULT_SINK@", mute_arg])
                .status()
                .await
                .map_err(|e| AudioError::SetMute(e.into()))?;

            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            let mute_arg = if mute { "true" } else { "false" };
            Command::new("osascript")
                .args(["-e", &str!("set volume muted {mute_arg}")])
                .status()
                .await
                .map_err(|e| AudioError::SetMute(e.into()))?;

            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            let mute_arg = if mute { "$true" } else { "$false" };
            let script = str!(
                "$w=(New-Object -ComObject MMDeviceEnumerator).GetDefaultAudioEndpoint(0,0).AudioEndpointVolume; \
                $w.SetMute({mute_arg}, $null)"
            );

            Command::new("powershell")
                .args(["-Command", &script])
                .status()
                .await
                .map_err(|e| AudioError::SetMute(e.into()))?;

            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(AudioError::UnsupportedOS.into())
        }
    }
}
