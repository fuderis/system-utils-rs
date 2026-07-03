#![allow(unused_imports)]
use crate::prelude::DynError;
use macron::{Display, Error, From};

/// The audio error
#[cfg(feature = "audio")]
#[derive(Debug, Display, Error, From)]
pub enum AudioError {
    #[display(fmt = "Audio devices not found")]
    DevicesNotFound,

    #[from(skip)]
    #[display(fmt = "Set volume failed: {0}")]
    SetVolume(DynError),

    #[from(skip)]
    #[display(fmt = "Get volume failed: {0}")]
    GetVolume(DynError),

    #[from(skip)]
    #[display(fmt = "Get mute status failed: {0}")]
    GetMute(DynError),

    #[from(skip)]
    #[display(fmt = "Get mute volume failed: {0}")]
    SetMute(DynError),

    #[display(fmt = "Unsupported operating system")]
    UnsupportedOS,
}

#[cfg(feature = "power")]
#[derive(Debug, Display, Error, From)]
pub enum PowerError {
    #[display(fmt = "Unsupported operating system")]
    UnsupportedOS,
}

#[cfg(feature = "theme")]
#[derive(Debug, Display, Error, From)]
pub enum ThemeError {
    #[cfg(target_os = "linux")]
    #[display(fmt = "Failed to execute gsettings: {0}")]
    GsettingsExecute(std::io::Error),

    #[cfg(target_os = "linux")]
    #[display(fmt = "gsettings exited with non-zero status")]
    GsettingsExitStatus,

    #[cfg(target_os = "macos")]
    #[display(fmt = "Failed to execute osascript: {0}")]
    OsascriptExecute(std::io::Error),

    #[cfg(target_os = "macos")]
    #[display(fmt = "osascript exited with non-zero status")]
    OsascriptExitStatus,

    #[display(fmt = "Unsupported operating system")]
    UnsupportedOS,
}
