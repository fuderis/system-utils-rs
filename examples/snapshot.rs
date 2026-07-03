#![cfg(feature = "info")]
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
