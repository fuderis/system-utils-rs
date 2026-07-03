#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod error;
mod prelude;

#[cfg(feature = "monitor")]
pub mod monitor;
#[cfg(feature = "monitor")]
pub use monitor::SystemMonitor;

#[cfg(feature = "info")]
pub mod info;
#[cfg(feature = "info")]
pub use info::SystemInfo;

#[cfg(feature = "metrics")]
pub mod metrics;
#[cfg(feature = "metrics")]
pub use metrics::SystemMetrics;

#[cfg(feature = "devices")]
pub mod devices;
#[cfg(feature = "devices")]
pub use devices::DevicesList;

#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "audio")]
pub use audio::AudioControl;

#[cfg(feature = "power")]
pub mod power;
#[cfg(feature = "power")]
pub use power::PowerManager;
