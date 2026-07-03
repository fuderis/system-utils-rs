use crate::prelude::*;

use tokio::time::{Duration, Instant};

/// The system monitor
#[derive(Default, Clone)]
pub struct SystemMonitor {
    #[cfg(feature = "info")]
    info: Option<Arc<crate::SystemInfo>>,

    #[cfg(feature = "metrics")]
    metrics: Option<Arc<crate::SystemMetrics>>,
    #[cfg(feature = "metrics")]
    metrics_instant: Option<Instant>,

    #[cfg(feature = "devices")]
    devices: Option<Arc<crate::DevicesList>>,
    #[cfg(feature = "devices")]
    devices_instant: Option<Instant>,
}

impl SystemMonitor {
    /// Returns the basic system info
    #[cfg(feature = "info")]
    pub fn info(&mut self) -> Arc<crate::SystemInfo> {
        if let Some(info) = self.info.clone() {
            info
        } else {
            let info = arc!(crate::SystemInfo::new());
            self.info.replace(info.clone());
            info
        }
    }

    /// Returns the system metrics
    #[cfg(feature = "metrics")]
    pub fn metrics(&mut self) -> Arc<crate::SystemMetrics> {
        if let Some(metrics) = self.metrics.clone() {
            metrics
        } else {
            self.refresh_metrics()
        }
    }

    #[cfg(feature = "metrics")]
    pub fn refresh_metrics(&mut self) -> Arc<crate::SystemMetrics> {
        let metrics = arc!(crate::SystemMetrics::new());

        self.metrics.replace(metrics.clone());
        self.metrics_instant.replace(Instant::now());

        metrics
    }

    #[cfg(feature = "metrics")]
    pub fn refresh_metrics_with_interval(
        &mut self,
        interval: Duration,
    ) -> Arc<crate::SystemMetrics> {
        if self.metrics.is_none()
            || self
                .metrics_instant
                .map(|last| last.elapsed() >= interval)
                .unwrap_or(false)
        {
            self.refresh_metrics()
        } else {
            self.metrics.clone().unwrap()
        }
    }

    /// Returns the devices list
    #[cfg(feature = "devices")]
    pub fn devices(&mut self) -> Arc<crate::DevicesList> {
        if let Some(devices) = self.devices.clone() {
            devices
        } else {
            self.refresh_devices()
        }
    }

    #[cfg(feature = "devices")]
    pub fn refresh_devices(&mut self) -> Arc<crate::DevicesList> {
        let devices = arc!(crate::DevicesList::new());

        self.devices.replace(devices.clone());
        self.devices_instant.replace(Instant::now());

        devices
    }

    #[cfg(feature = "devices")]
    pub fn refresh_devices_with_interval(&mut self, interval: Duration) -> Arc<crate::DevicesList> {
        if self.devices.is_none()
            || self
                .devices_instant
                .map(|last| last.elapsed() >= interval)
                .unwrap_or(false)
        {
            self.refresh_devices()
        } else {
            self.devices.clone().unwrap()
        }
    }
}
