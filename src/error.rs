#![allow(unused_imports)]
use crate::prelude::DynError;
use macron::{Display, Error, From};

/// The error instance
#[derive(Debug, Display, Error, From)]
pub enum Error {
    Msg(String),

    #[cfg(feature = "audio")]
    AudioError(#[source] Box<AudioError>),

    #[cfg(feature = "power")]
    PowerError(#[source] Box<PowerError>),
}

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
