mod structs;
pub use structs::*;

use crate::prelude::*;

use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::sync::Mutex as StdMutex;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};

static SYSTEM: State<Arc<StdMutex<System>>> = State::new(|| {
    std_arc_mutex!(System::new_with_specifics(
        RefreshKind::everything().with_cpu(CpuRefreshKind::everything())
    ))
});

/// The system metrics
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub proxy: ProxyInfo,
    pub battery: Option<BatteryInfo>,
    pub uptime_sec: u64,
}

impl SystemMetrics {
    /// Creates the system metrics snapshot
    pub fn new() -> Self {
        (**SYSTEM.dirty_get()).lock().unwrap().refresh_all();

        Self {
            cpu: Self::cpu(),
            gpus: Self::gpus(),
            memory: Self::memory(),
            disks: Self::disks(),
            networks: Self::networks(),
            proxy: Self::proxy(),
            battery: Self::battery(),
            uptime_sec: Self::uptime(),
        }
    }

    fn system() -> Arc<StdMutex<System>> {
        (*SYSTEM.dirty_get()).clone()
    }
}

impl SystemMetrics {
    pub fn uptime() -> u64 {
        System::uptime()
    }

    pub fn cpu() -> CpuInfo {
        let system = Self::system();
        let mut system = system.lock().unwrap();
        system.refresh_cpu_all();

        let cpus = system.cpus();

        let usage = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };

        let cores = cpus
            .iter()
            .map(|cpu| CpuCoreInfo {
                name: cpu.name().to_string(),
                frequency: cpu.frequency(),
                usage: cpu.cpu_usage(),
            })
            .collect();

        let first = cpus.first();

        let mut temperature = None;
        #[cfg(target_os = "linux")]
        {
            for path in [
                "/sys/class/thermal/thermal_zone0/temp",
                "/sys/class/hwmon/hwmon0/temp1_input",
                "/sys/class/hwmon/hwmon1/temp1_input",
            ] {
                if let Ok(v) = std::fs::read_to_string(path) {
                    if let Ok(val) = v.trim().parse::<f32>() {
                        temperature = Some(val / 1000.0);
                        break;
                    }
                }
            }
        };

        let load = System::load_average();
        let mut load_average = None;
        #[cfg(unix)]
        {
            load_average.replace(LoadAverage {
                one: load.one,
                five: load.five,
                fifteen: load.fifteen,
            });
        }

        CpuInfo {
            brand: first.map(|c| c.brand().to_string()).unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),

            usage,
            frequency: first.map(|c| c.frequency()).unwrap_or_default(),
            temperature,

            cores,
            physical_cores: System::physical_core_count().unwrap_or(0),
            logical_cores: cpus.len(),

            load_average,
        }
    }

    pub fn gpus() -> Vec<GpuInfo> {
        use std::collections::HashSet;

        let instance = wgpu::Instance::default();

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for adapter in instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
        {
            let info = adapter.get_info();

            let name = info
                .name
                .split('(')
                .next()
                .unwrap_or(&info.name)
                .trim()
                .to_string();

            if !seen.insert(name.clone()) {
                continue;
            }

            let mut gpu = GpuInfo {
                name,
                usage: None,
                temperature: None,
                vram_total: None,
                vram_used: None,
            };

            #[cfg(target_os = "linux")]
            {
                let (temp, vram_total, vram_used, usage) = Self::linux_gpu_metrics();

                gpu.temperature = temp;
                gpu.vram_total = vram_total;
                gpu.vram_used = vram_used;
                gpu.usage = usage;
            }

            result.push(gpu);
        }

        result
    }

    #[cfg(target_os = "linux")]
    #[cfg(target_os = "linux")]
    fn linux_gpu_metrics() -> (Option<f32>, Option<u64>, Option<u64>, Option<f32>) {
        use std::fs;

        let mut temperature = None;
        let mut vram_total = None;
        let mut vram_used = None;
        let mut usage = None;

        let drm_dir = "/sys/class/drm";

        if let Ok(entries) = fs::read_dir(drm_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                if !name.starts_with("card") {
                    continue;
                }

                let device = path.join("device");

                // VRAM (AMD)
                let total_path = device.join("mem_info_vram_total");
                let used_path = device.join("mem_info_vram_used");

                if let (Ok(t), Ok(u)) = (
                    fs::read_to_string(&total_path),
                    fs::read_to_string(&used_path),
                ) {
                    vram_total = t.trim().parse::<u64>().ok();
                    vram_used = u.trim().parse::<u64>().ok();
                }

                // Temperature
                let hwmon_dir = device.join("hwmon");

                if let Ok(hwmons) = fs::read_dir(hwmon_dir) {
                    for hw in hwmons.flatten() {
                        let temp_file = hw.path().join("temp1_input");

                        if let Ok(temp) = fs::read_to_string(temp_file) {
                            temperature = temp.trim().parse::<f32>().ok().map(|v| v / 1000.0);
                            break;
                        }
                    }
                }

                // GPU usage (AMD)
                let busy = device.join("gpu_busy_percent");
                if let Ok(val) = fs::read_to_string(busy) {
                    usage = val.trim().parse::<f32>().ok();
                }

                break;
            }
        }

        (temperature, vram_total, vram_used, usage)
    }

    pub fn temperatures() -> Vec<TemperatureInfo> {
        let components = sysinfo::Components::new_with_refreshed_list();

        components
            .iter()
            .map(|c| TemperatureInfo {
                component: c.label().to_string(),
                temperature: c.temperature().unwrap_or(0.0),
            })
            .collect()
    }

    pub fn memory() -> MemoryInfo {
        let system = Self::system();
        let mut system = system.lock().unwrap();
        system.refresh_memory();

        let total = system.total_memory();
        let used = system.used_memory();

        MemoryInfo {
            total,
            used,
            free: total.saturating_sub(used),

            swap_total: system.total_swap(),
            swap_used: system.used_swap(),
        }
    }

    pub fn disks() -> Vec<DiskInfo> {
        let disks = Disks::new_with_refreshed_list();

        disks
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();

                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),

                    filesystem: disk.file_system().to_string_lossy().to_string(),

                    total_space: total,
                    available_space: available,
                    used_space: total.saturating_sub(available),
                }
            })
            .collect()
    }

    pub fn networks() -> Vec<NetworkInfo> {
        let mut result = Vec::new();

        let interfaces = match NetworkInterface::show() {
            Ok(v) => v,
            Err(_) => return result,
        };

        let networks = Networks::new_with_refreshed_list();

        for iface in interfaces {
            let mut ipv4 = Vec::new();
            let mut ipv6 = Vec::new();

            for addr in &iface.addr {
                let text = addr.ip().to_string();

                if addr.ip().is_ipv4() {
                    ipv4.push(text);
                } else {
                    ipv6.push(text);
                }
            }

            let (received, transmitted) = networks
                .get(&iface.name)
                .map(|n| (n.received(), n.transmitted()))
                .unwrap_or((0, 0));

            result.push(NetworkInfo {
                interface_type: interface_type(&iface.name),
                name: iface.name,

                status: if ipv4.is_empty() && ipv6.is_empty() {
                    "Down".to_string()
                } else {
                    "Up".to_string()
                },
                ipv4,
                ipv6,
                received,
                transmitted,
            });
        }

        result
    }

    pub fn proxy() -> ProxyInfo {
        let http_proxy = std::env::var("HTTP_PROXY")
            .or_else(|_| std::env::var("http_proxy"))
            .ok();

        let https_proxy = std::env::var("HTTPS_PROXY")
            .or_else(|_| std::env::var("https_proxy"))
            .ok();

        let all_proxy = std::env::var("ALL_PROXY")
            .or_else(|_| std::env::var("all_proxy"))
            .ok();

        let no_proxy = std::env::var("NO_PROXY")
            .or_else(|_| std::env::var("no_proxy"))
            .ok();

        let enabled = http_proxy.is_some() || https_proxy.is_some() || all_proxy.is_some();

        ProxyInfo {
            enabled,

            http_proxy,
            https_proxy,
            all_proxy,
            no_proxy,
        }
    }

    pub fn battery() -> Option<BatteryInfo> {
        let manager = battery::Manager::new().ok()?;

        let mut batteries = manager.batteries().ok()?;

        let battery = batteries.next()?.ok()?;

        Some(BatteryInfo {
            present: true,
            percentage: battery.state_of_charge().value * 100.0,
            state: format!("{:?}", battery.state()),

            energy_now: battery.energy().value,
            energy_full: battery.energy_full().value,

            cycles: battery.cycle_count(),

            time_to_empty: battery.time_to_empty().map(|v| v.value as u64),
            time_to_full: battery.time_to_full().map(|v| v.value as u64),
        })
    }
}

impl std::fmt::Display for SystemMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ram_percent = if self.memory.total > 0 {
            self.memory.used as f64 * 100.0 / self.memory.total as f64
        } else {
            0.0
        };

        let swap_percent = if self.memory.swap_total > 0 {
            self.memory.swap_used as f64 * 100.0 / self.memory.swap_total as f64
        } else {
            0.0
        };

        /* ================= System ================= */
        writeln!(f, "=== System ===")?;
        writeln!(f, "Uptime        : {}", format_uptime(self.uptime_sec))?;

        /* ================= CPU ================= */
        writeln!(f)?;
        writeln!(f, "=== CPU ===")?;

        writeln!(f, "Model         : {}", self.cpu.brand)?;
        writeln!(f, "Architecture  : {}", self.cpu.architecture)?;

        writeln!(f, "Usage         : {:.1}%", self.cpu.usage)?;
        if let Some(load) = &self.cpu.load_average {
            writeln!(
                f,
                "Load Average  : {:.2} / {:.2} / {:.2}",
                load.one, load.five, load.fifteen
            )?;
        }

        writeln!(
            f,
            "Frequency     : {}",
            format_frequency(self.cpu.frequency)
        )?;
        if let Some(temp) = self.cpu.temperature {
            writeln!(f, "Temperature   : {:.1}°C", temp)?;
        }

        writeln!(
            f,
            "Cores         : {} physical / {} logical",
            self.cpu.physical_cores, self.cpu.logical_cores
        )?;

        if !self.cpu.cores.is_empty() {
            writeln!(f, "Per Core      :")?;
            for core in &self.cpu.cores {
                writeln!(
                    f,
                    "  {:<6} {:>5.1}% @ {}",
                    core.name,
                    core.usage,
                    format_frequency(core.frequency)
                )?;
            }
        }

        /* ================= GPU ================= */
        writeln!(f)?;

        for (i, gpu) in self.gpus.iter().enumerate() {
            if i > 0 {
                writeln!(f)?
            };
            writeln!(f, "=== GPU #{i} ===")?;
            writeln!(f, "Model         : {}", gpu.name)?;

            if let Some(u) = gpu.usage {
                writeln!(f, "Usage         : {:.1}%", u)?;
            }

            if let Some(temp) = gpu.temperature {
                writeln!(f, "Temperature   : {:.1}°C", temp)?;
            }

            if let (Some(u), Some(t)) = (gpu.vram_used, gpu.vram_total) {
                writeln!(
                    f,
                    "VRAM          : {} / {} ({:.1}%)",
                    format_bytes(u),
                    format_bytes(t),
                    if t == 0 {
                        0.0
                    } else {
                        u as f32 * 100.0 / t as f32
                    }
                )?;

                writeln!(f, "Free VRAM     : {}", format_bytes(t - u),)?;
            }
        }

        /* ================= MEMORY ================= */
        writeln!(f)?;
        writeln!(f, "=== MEMORY ===")?;

        writeln!(f, "Usage         : {:.1}%", ram_percent)?;
        writeln!(
            f,
            "RAM           : {} / {}",
            format_bytes(self.memory.used),
            format_bytes(self.memory.total)
        )?;
        writeln!(f, "Free RAM      : {}", format_bytes(self.memory.free))?;
        writeln!(
            f,
            "Swap          : {} / {} ({:.1}%)",
            format_bytes(self.memory.swap_used),
            format_bytes(self.memory.swap_total),
            swap_percent,
        )?;

        /* ================= DISKS ================= */
        writeln!(f)?;
        writeln!(f, "=== DISKS ===")?;

        for (i, disk) in self.disks.iter().enumerate() {
            let usage_percent = if disk.total_space > 0 {
                disk.used_space as f64 * 100.0 / disk.total_space as f64
            } else {
                0.0
            };

            if i > 0 {
                writeln!(f)?
            };
            writeln!(f, "{}", disk.name)?;
            writeln!(f, "  Mount       : {}", disk.mount_point)?;
            writeln!(f, "  Filesystem  : {}", disk.filesystem)?;
            writeln!(f, "  Usage       : {:.1}%", usage_percent)?;
            writeln!(
                f,
                "  Used        : {} / {}",
                format_bytes(disk.used_space),
                format_bytes(disk.total_space)
            )?;
            writeln!(f, "  Free        : {}", format_bytes(disk.available_space))?;
        }

        /* ================= NETWORK ================= */
        writeln!(f)?;
        writeln!(f, "=== NETWORKS ===")?;

        for (i, net) in self.networks.iter().enumerate() {
            if i > 0 {
                writeln!(f)?
            };
            writeln!(f, "{}", net.name)?;
            writeln!(f, "  Type        : {}", net.interface_type)?;
            writeln!(f, "  Status      : {}", net.status)?;

            if !net.ipv4.is_empty() {
                writeln!(f, "  IPv4        : {}", net.ipv4.join(", "))?;
            }

            if !net.ipv6.is_empty() {
                writeln!(f, "  IPv6        : {}", net.ipv6.join(", "))?;
            }

            writeln!(f, "  RX          : {}", format_bytes(net.received))?;
            writeln!(f, "  TX          : {}", format_bytes(net.transmitted))?;
        }

        /* ================= PROXY ================= */
        writeln!(f)?;
        writeln!(f, "=== PROXY ===")?;

        writeln!(
            f,
            "Enabled       : {}",
            if self.proxy.enabled { "Yes" } else { "No" }
        )?;

        if let Some(p) = &self.proxy.http_proxy {
            writeln!(f, "HTTP Proxy    : {p}")?;
        }
        if let Some(p) = &self.proxy.https_proxy {
            writeln!(f, "HTTPS Proxy   : {p}")?;
        }
        if let Some(p) = &self.proxy.all_proxy {
            writeln!(f, "ALL Proxy     : {p}")?;
        }
        if let Some(p) = &self.proxy.no_proxy {
            writeln!(f, "NO Proxy      : {p}")?;
        }

        /* ================= BATTERY ================= */
        if let Some(b) = &self.battery {
            writeln!(f)?;
            writeln!(f, "=== BATTERY ===")?;
            writeln!(f, "Charge       : {:.1}%", b.percentage)?;
            writeln!(f, "State        : {}", b.state)?;
            writeln!(
                f,
                "Energy       : {:.2} Wh / {:.2} Wh",
                b.energy_now, b.energy_full
            )?;
            if let Some(cycles) = b.cycles {
                writeln!(f, "Cycles       : {}", cycles)?;
            }
        }

        Ok(())
    }
}

fn interface_type(name: &str) -> String {
    let lower = name.to_lowercase();

    if lower == "lo" {
        "Loopback".into()
    } else if lower.starts_with("wl") || lower.starts_with("wlan") || lower.starts_with("wifi") {
        "Wi-Fi".into()
    } else if lower.starts_with("en") || lower.starts_with("eth") {
        "Ethernet".into()
    } else if lower.starts_with("tun")
        || lower.starts_with("tap")
        || lower.starts_with("wg")
        || lower.starts_with("tailscale")
    {
        "VPN".into()
    } else {
        "Unknown".into()
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;

    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

fn format_frequency(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.2} GHz", mhz as f64 / 1000.0)
    } else {
        format!("{mhz} MHz")
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    format!("{days}d {hours}h {minutes}m")
}
