[![github]](https://github.com/fuderis/system-utils-rs)&ensp;
[![crates-io]](https://crates.io/crates/system-utils)&ensp;
[![docs-rs]](https://docs.rs/system-utils)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# System Utilities

Cross-platform utilities for interacting with operating system features such as audio control, power management, system information, hardware monitoring, and device enumeration.

## Features:

* Audio management
* Power actions with optional timers
* System information
* Real-time system metrics monitoring
* Hardware device enumeration
* Cross-platform API (Windows, Linux, macOS where supported)

# Installation

```bash
cargo add system-utils --features full
```

## Examples:

### System Monitoring [feature `monitor`]

`SystemMonitor` caches system information, metrics, and device lists internally.
Calling `refresh_*_with_interval()` updates the cached data only if the specified interval has elapsed,
reducing unnecessary operating system queries.

```rust
use system_utils::SystemMonitor;
use tokio::time::Duration;

fn main() {
    let mut monitor = SystemMonitor::default();

    // static system information (cached forever)
    let info = monitor.info();
    println!("{info}");

    // collect current system metrics
    let metrics = monitor.refresh_metrics_with_interval(Duration::from_secs(10));
    println!("{metrics}");

    // enumerate connected devices
    let devices = monitor.refresh_devices_with_interval(Duration::from_secs(60));
    println!("{devices}")
}
```

### Basic System Info [feature `info`]

```rust
use system_utils::SystemInfo;

fn main() {
    // create a snapshot of the current system information
    let info = SystemInfo::new();
    println!("{info}");
}
```

### System Metrics [feature `metrics`]

`SystemMetrics::new()` collects a complete snapshot of the current system state, including CPU, GPU, memory, disks, network interfaces,
proxy configuration, battery information (when available), and system uptime.
The returned snapshot is immutable and represents the system state at the moment it was created.

```rust
use system_utils::SystemMetrics;

fn main() {
    // collect a snapshot of the current system metrics
    let metrics = SystemMetrics::new();
    println!("{metrics}");
}
```

### Devices List [feature `devices`]

`DevicesList::new()` creates a snapshot of the currently connected hardware devices, including monitors, audio devices, cameras, and USB devices.
The snapshot is immutable and reflects the hardware state at the time it was created.

```rust
use system_utils::DevicesList;

fn main() {
    // enumerate connected hardware devices
    let devices = DevicesList::new();
    println!("{devices}");
}
```

### Audio Control [feature `audio`]

`SystemAudio` provides a cross-platform API for controlling the system master volume.
It supports getting and setting the volume, increasing or decreasing it by a delta, and muting or unmuting the default audio output device.

```rust
use system_utils::AudioControl;

#[tokio::main]
async fn main() -> Result<()> {
    // get the current master volume
    let volume = AudioControl::get_volume().await?;
    println!("Current volume: {}%", volume);

    // increase the volume by 10%
    let volume = AudioControl::increase_volume(10).await?;
    println!("Volume: {}%", volume);

    // decrease the volume by 5%
    let volume = AudioControl::decrease_volume(5).await?;
    println!("Volume: {}%", volume);

    // set an exact volume level
    AudioControl::set_volume(50).await?;

    // mute / unmute
    if !AudioControl::is_muted().await? {
        AudioControl::set_mute(true).await?;
        println!("Audio muted.");
    }

    AudioControl::set_mute(false).await?;
    println!("Audio restored.");

    Ok(())
}
```

### Power Management [feature `power`]

`SystemPower` provides cross-platform power management, including shutdown, reboot, suspend, lock, and logout operations.
Every action can be executed immediately or scheduled for a future `DateTime<Utc>`.
Only one scheduled power action can be active at a time; scheduling a new action automatically replaces the previous one.

```rust
use chrono::{Duration, Utc};
use system_utils::PowerManager;

#[tokio::main]
async fn main() -> Result<()> {
    // schedule a shutdown in 5 minutes
    let execute_at = Utc::now() + Duration::minutes(5);

    PowerManager::shutdown(Some(execute_at)).await?;

    // check the scheduled task
    if let Some(task) = PowerManager::status().await {
        println!(
            "{:?} scheduled for {}",
            task.mode,
            task.execute_at
        );
    }

    // cancel the scheduled action
    if PowerManager::cancel().await {
        println!("Scheduled power action cancelled.");
    }

    // execute an action immediately
    // PowerManager::lock_now().await?;
    // PowerManager::logout_now().await?;
    // PowerManager::reboot_now().await?;
    // PowerManager::shutdown_now().await?;
    // PowerManager::suspend_now().await?;

    Ok(())
}
```

### System Theme Switcher [feature `theme`]

A cross-platform utility for switching the system theme between Light and Dark modes
on Linux, macOS and Windows through native operating system APIs.

```rust
use system_utils::{ThemeStyle, SystemTheme};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SystemTheme::switch(ThemeStyle::Dark).await?;
    println!("Dark theme enabled.");

    Ok(())
}
```

## License & Feedback:

> Distributed under the [MIT](https://github.com/fuderis/system-utils-rs/blob/main/LICENSE.md) license.

You can contact me via [GitHub](https://github.com/fuderis) or send a message to my [E-Mail](mailto:synapdrake@ya.ru).
This library is actively evolving, and your suggestions and feedback are always welcome!
