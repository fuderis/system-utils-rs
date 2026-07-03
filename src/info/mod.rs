use crate::prelude::*;

use sysinfo::System;

/// The basic system info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
}

impl SystemInfo {
    /// Creates a new system info instance
    pub fn new() -> Self {
        Self {
            hostname: Self::hostname(),
            os_name: Self::os_name(),
            os_version: Self::os_version(),
            kernel_version: Self::kernel_version(),
        }
    }

    pub fn hostname() -> String {
        System::host_name().unwrap_or_default()
    }

    pub fn os_name() -> String {
        System::name().unwrap_or_default()
    }

    pub fn os_version() -> String {
        System::long_os_version().unwrap_or_default()
    }

    pub fn kernel_version() -> String {
        System::kernel_version().unwrap_or_default()
    }
}

impl std::fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== SYSTEM INFORMATION ===")?;
        writeln!(f, "Hostname      : {}", self.hostname)?;
        writeln!(f, "OS            : {}", self.os_name)?;
        writeln!(f, "Version       : {}", self.os_version)?;
        writeln!(f, "Kernel        : {}", self.kernel_version)?;

        Ok(())
    }
}
