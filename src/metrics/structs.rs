use crate::prelude::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub brand: String,
    pub architecture: String,

    pub usage: f32,
    pub frequency: u64,
    pub temperature: Option<f32>,

    pub cores: Vec<CpuCoreInfo>,
    pub physical_cores: usize,
    pub logical_cores: usize,

    pub load_average: Option<LoadAverage>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub name: String,

    pub usage: f32,
    pub frequency: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,

    pub usage: Option<f32>,
    pub temperature: Option<f32>,

    pub vram_total: Option<u64>,
    pub vram_used: Option<u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureInfo {
    pub component: String,
    pub temperature: f32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,

    pub filesystem: String,

    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub name: String,

    pub interface_type: String,

    pub status: String,

    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,

    pub received: u64,
    pub transmitted: u64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub enabled: bool,

    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub all_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub present: bool,
    pub percentage: f32,
    pub state: String,

    pub energy_now: f32,
    pub energy_full: f32,

    pub cycles: Option<u32>,

    pub time_to_empty: Option<u64>,
    pub time_to_full: Option<u64>,
}
