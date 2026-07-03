pub mod structs;
pub use structs::*;

use crate::prelude::*;

/// The devices snapshot
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DevicesList {
    pub monitors: Vec<MonitorInfo>,
    pub audio: Vec<AudioDeviceInfo>,
    pub cameras: Vec<CameraInfo>,
    pub usb: Vec<UsbDeviceInfo>,
}

impl DevicesList {
    pub fn new() -> Self {
        Self {
            monitors: Self::monitors(),
            audio: Self::audio(),
            cameras: Self::cameras(),
            usb: Self::usb(),
        }
    }
}

impl DevicesList {
    pub fn monitors() -> Vec<MonitorInfo> {
        use display_info::DisplayInfo;

        let mut result = Vec::new();

        if let Ok(displays) = DisplayInfo::all() {
            let count = displays.len();

            for d in displays {
                result.push(MonitorInfo {
                    name: d.name,
                    width: d.width,
                    height: d.height,

                    refresh_rate: None,

                    primary: if count == 1 { true } else { d.is_primary },
                    x: d.x,
                    y: d.y,
                });
            }
        }

        result
    }

    pub fn audio() -> Vec<AudioDeviceInfo> {
        use cpal::traits::{DeviceTrait, HostTrait};
        use std::collections::HashMap;

        let host = cpal::default_host();
        let mut raw = Vec::new();

        let default_input_id = host
            .default_input_device()
            .and_then(|d| d.id().ok())
            .map(|id| id.to_string());

        let default_output_id = host
            .default_output_device()
            .and_then(|d| d.id().ok())
            .map(|id| id.to_string());

        // ===================== PHYSICAL CHECK =====================
        fn is_physical_device(name: &str, id: &str) -> bool {
            let n = name.to_lowercase();
            let i = id.to_lowercase();

            let noise = [
                "monitor", "null", "dummy", "loopback", "pulse", "pipewire", "jack", "oss",
            ];

            if noise.iter().any(|x| n.contains(x) || i.contains(x)) {
                return false;
            }

            // ALSA processing plugins
            let plugins = [
                "plugin",
                "rate converter",
                "speex",
                "ffmpeg",
                "upmix",
                "downmix",
                "discard",
                "spatialization",
                "resampler",
                "usbstream",
            ];

            if plugins.iter().any(|x| n.contains(x) || i.contains(x)) {
                return false;
            }

            true
        }

        // ===================== GROUP KEY =====================
        fn device_group_key(name: &str) -> String {
            let mut s = name.to_lowercase();

            let patterns = [", analog", ", alt analog", ", usb audio", "input", "output"];

            for p in patterns {
                s = s.replace(p, "");
            }

            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        }

        // ===================== ENDPOINT SCORE =====================
        fn endpoint_score(id: &str) -> u8 {
            let i = id.to_lowercase();

            if i.contains("hw:") {
                3
            } else if i.contains("front:") {
                2
            } else if i.contains("sysdefault") {
                1
            } else {
                0
            }
        }

        fn collect_configs<I>(cfgs: I) -> Vec<AudioConfigInfo>
        where
            I: Iterator<Item = cpal::SupportedStreamConfigRange>,
        {
            let mut result = Vec::new();

            for cfg in cfgs {
                let min = cfg.min_sample_rate();
                let max = cfg.max_sample_rate();

                if min < 4_000 {
                    continue;
                }

                if max > 768_000 {
                    continue;
                }

                result.push(AudioConfigInfo {
                    channels: cfg.channels(),
                    sample_rate_min: min,
                    sample_rate_max: max,
                });
            }

            result.sort_by(|a, b| {
                b.sample_rate_max
                    .cmp(&a.sample_rate_max)
                    .then(b.channels.cmp(&a.channels))
            });

            result.dedup_by(|a, b| {
                a.channels == b.channels
                    && a.sample_rate_min == b.sample_rate_min
                    && a.sample_rate_max == b.sample_rate_max
            });

            result
        }

        // ===================== INPUT =====================
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                let desc = match device.description() {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let name = desc.name();

                let id = device
                    .id()
                    .ok()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| name.to_string());

                if !is_physical_device(name, &id) {
                    continue;
                }

                let configs = match device.supported_input_configs() {
                    Ok(cfgs) => collect_configs(cfgs),
                    Err(_) => Vec::new(),
                };

                raw.push(AudioDeviceInfo {
                    default: default_input_id.as_ref().map(|d| d == &id).unwrap_or(false),

                    id,
                    name: name.to_string(),

                    kind: AudioDeviceType::Input,
                    is_physical: true,

                    configs,

                    latency_ms: None,
                });
            }
        }

        // ===================== OUTPUT =====================
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                let desc = match device.description() {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let name = desc.name();

                let id = device
                    .id()
                    .ok()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| name.to_string());

                if !is_physical_device(name, &id) {
                    continue;
                }

                let configs = match device.supported_output_configs() {
                    Ok(cfgs) => collect_configs(cfgs),
                    Err(_) => Vec::new(),
                };

                raw.push(AudioDeviceInfo {
                    default: default_output_id
                        .as_ref()
                        .map(|d| d == &id)
                        .unwrap_or(false),

                    id,
                    name: name.to_string(),

                    kind: AudioDeviceType::Output,
                    is_physical: true,

                    configs,

                    latency_ms: None,
                });
            }
        }

        // ===================== GROUP + MERGE =====================
        let mut grouped: HashMap<String, AudioDeviceInfo> = HashMap::new();

        for d in raw {
            let key = device_group_key(&d.name);

            let entry = grouped.entry(key).or_insert_with(|| d.clone());

            // default merge
            entry.default |= d.default;

            // physical detection
            entry.is_physical |= is_physical_device(&d.name, &d.id);

            // configs merge
            entry.configs.extend(d.configs);

            entry.configs.sort_by(|a, b| {
                b.sample_rate_max
                    .cmp(&a.sample_rate_max)
                    .then(b.channels.cmp(&a.channels))
            });

            entry.configs.dedup_by(|a, b| {
                a.channels == b.channels
                    && a.sample_rate_min == b.sample_rate_min
                    && a.sample_rate_max == b.sample_rate_max
            });

            // best endpoint selection
            if endpoint_score(&d.id) > endpoint_score(&entry.id) {
                entry.id = d.id.clone();
            }
        }

        let mut devices: Vec<_> = grouped.into_values().collect();

        devices.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| b.is_physical.cmp(&a.is_physical))
                .then_with(|| a.name.cmp(&b.name))
        });

        devices
    }

    pub fn cameras() -> Vec<CameraInfo> {
        use nokhwa::query;

        let mut result = Vec::new();

        if let Ok(devices) = query(nokhwa::utils::ApiBackend::Auto) {
            for d in devices {
                result.push(CameraInfo {
                    name: d.human_name(),
                });
            }
        }

        result
    }

    pub fn usb() -> Vec<UsbDeviceInfo> {
        use rusb::{Context, UsbContext};
        use std::time::Duration;

        let mut result = Vec::new();

        let context = match Context::new() {
            Ok(c) => c,
            Err(_) => return result,
        };

        let devices = match context.devices() {
            Ok(d) => d,
            Err(_) => return result,
        };

        fn classify(class: u8) -> UsbDeviceType {
            match class {
                0x01 => UsbDeviceType::Audio,
                0x02 => UsbDeviceType::Serial,
                0x08 => UsbDeviceType::Storage,
                0x09 => UsbDeviceType::Hub,
                0x0e => UsbDeviceType::Camera,
                0xe0 => UsbDeviceType::Bluetooth,
                0xef => UsbDeviceType::Composite,
                0xff => UsbDeviceType::VendorSpecific,
                _ => UsbDeviceType::Unknown,
            }
        }

        fn classify_interface(class: u8, _subclass: u8, _protocol: u8) -> UsbInterfaceType {
            match class {
                0x01 => UsbInterfaceType::Audio,
                0x02 => UsbInterfaceType::CdcAcm,
                0x03 => UsbInterfaceType::Hid,
                0x08 => UsbInterfaceType::Storage,
                0x09 => UsbInterfaceType::Hub,
                0x0e => UsbInterfaceType::Video,
                0xe0 => UsbInterfaceType::Bluetooth,
                0xef => UsbInterfaceType::Composite,
                0xff => UsbInterfaceType::VendorSpecific,
                _ => UsbInterfaceType::Unknown,
            }
        }

        fn add_type(types: &mut Vec<UsbDeviceType>, t: UsbDeviceType) {
            if t != UsbDeviceType::Unknown && !types.contains(&t) {
                types.push(t);
            }
        }

        fn add_interface(interfaces: &mut Vec<UsbInterfaceType>, t: UsbInterfaceType) {
            if t != UsbInterfaceType::Unknown && !interfaces.contains(&t) {
                interfaces.push(t);
            }
        }

        fn score_type(t: &UsbDeviceType) -> u8 {
            match t {
                UsbDeviceType::Arduino => 100,
                UsbDeviceType::XboxController => 100,

                UsbDeviceType::GameController => 90,

                UsbDeviceType::Storage => 80,
                UsbDeviceType::Serial => 80,

                UsbDeviceType::Audio => 70,
                UsbDeviceType::Camera => 70,

                UsbDeviceType::Bluetooth => 60,
                UsbDeviceType::Network => 60,
                UsbDeviceType::Receiver => 60,

                UsbDeviceType::Composite => 20,
                UsbDeviceType::VendorSpecific => 10,

                UsbDeviceType::Hub => 5,

                UsbDeviceType::Unknown => 0,
            }
        }

        fn classify_vendor(
            vendor_id: u16,
            product_id: u16,
            manufacturer: &Option<String>,
            product: &Option<String>,
        ) -> Option<UsbDeviceType> {
            match vendor_id {
                0x2341 => Some(UsbDeviceType::Arduino),

                0x045e => match product_id {
                    0x02d1 | 0x02dd | 0x02ea | 0x028e => Some(UsbDeviceType::XboxController),
                    _ => Some(UsbDeviceType::GameController),
                },

                0x0a12 => Some(UsbDeviceType::Bluetooth),

                0x0403 | 0x10c4 | 0x067b => Some(UsbDeviceType::Serial),

                0x046d | 0x4842 => Some(UsbDeviceType::Receiver),

                _ => {
                    let text = format!(
                        "{} {}",
                        manufacturer.clone().unwrap_or_default(),
                        product.clone().unwrap_or_default()
                    )
                    .to_lowercase();

                    if text.contains("arduino") {
                        return Some(UsbDeviceType::Arduino);
                    }

                    if text.contains("xbox") {
                        return Some(UsbDeviceType::XboxController);
                    }

                    if text.contains("serial")
                        || text.contains("cdc")
                        || text.contains("ch340")
                        || text.contains("cp210")
                        || text.contains("ftdi")
                    {
                        return Some(UsbDeviceType::Serial);
                    }

                    if text.contains("gamepad")
                        || text.contains("controller")
                        || text.contains("joystick")
                    {
                        return Some(UsbDeviceType::GameController);
                    }

                    None
                }
            }
        }

        fn interface_score(t: &UsbInterfaceType) -> u8 {
            match t {
                UsbInterfaceType::Hid => 90,
                UsbInterfaceType::CdcAcm => 80,
                UsbInterfaceType::Audio => 70,
                UsbInterfaceType::Video => 60,
                UsbInterfaceType::Storage => 50,
                UsbInterfaceType::Bluetooth => 40,
                UsbInterfaceType::VendorSpecific => 10,
                _ => 0,
            }
        }

        for device in devices.iter() {
            let desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(_) => continue,
            };

            if desc.vendor_id() == 0x0000 || desc.product_id() == 0x0000 {
                continue;
            }

            let handle = device.open().ok();

            let lang = handle.as_ref().and_then(|h| {
                h.read_languages(Duration::from_millis(50))
                    .ok()
                    .and_then(|l| l.first().copied())
            });

            let manufacturer = handle.as_ref().and_then(|h| {
                lang.and_then(|l| {
                    h.read_manufacturer_string(l, &desc, Duration::from_millis(50))
                        .ok()
                })
            });

            let product = handle.as_ref().and_then(|h| {
                lang.and_then(|l| {
                    h.read_product_string(l, &desc, Duration::from_millis(50))
                        .ok()
                })
            });

            let serial_number = handle.as_ref().and_then(|h| {
                lang.and_then(|l| {
                    h.read_serial_number_string(l, &desc, Duration::from_millis(50))
                        .ok()
                })
            });

            let name = product.clone().unwrap_or_else(|| {
                format!("USB {:04x}:{:04x}", desc.vendor_id(), desc.product_id())
            });

            let mut device_types = Vec::new();
            let mut interface_types = Vec::new();

            add_type(&mut device_types, classify(desc.class_code()));

            if let Some(t) =
                classify_vendor(desc.vendor_id(), desc.product_id(), &manufacturer, &product)
            {
                add_type(&mut device_types, t);
            }

            if let Ok(config) = device.active_config_descriptor() {
                for interface in config.interfaces() {
                    for iface in interface.descriptors() {
                        add_type(&mut device_types, classify(iface.class_code()));

                        add_interface(
                            &mut interface_types,
                            classify_interface(
                                iface.class_code(),
                                iface.sub_class_code(),
                                iface.protocol_code(),
                            ),
                        );
                    }
                }
            }

            if device_types.is_empty() {
                device_types.push(UsbDeviceType::Unknown);
            }

            if device_types.contains(&UsbDeviceType::XboxController) {
                device_types.retain(|t| *t != UsbDeviceType::VendorSpecific);
            }

            if device_types.contains(&UsbDeviceType::Arduino) {
                device_types.retain(|t| *t != UsbDeviceType::Composite);
            }

            interface_types.sort_by(|a, b| interface_score(b).cmp(&interface_score(a)));

            result.push(UsbDeviceInfo {
                name,

                manufacturer,
                product,
                serial_number,

                vendor_id: format!("{:04x}", desc.vendor_id()),
                product_id: format!("{:04x}", desc.product_id()),

                device_class: Some(desc.class_code()),
                device_sub_class: Some(desc.sub_class_code()),
                device_protocol: Some(desc.protocol_code()),

                device_types,
                interface_types,

                bus: device.bus_number(),
                port: device.port_number(),
            });
        }

        result.sort_by(|a, b| {
            let a_score = a.device_types.iter().map(score_type).max().unwrap_or(0);
            let b_score = b.device_types.iter().map(score_type).max().unwrap_or(0);

            b_score.cmp(&a_score).then(a.name.cmp(&b.name))
        });

        result
    }
}

impl std::fmt::Display for DevicesList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /* ================= MONITORS ================= */
        if !self.monitors.is_empty() {
            writeln!(f)?;
            writeln!(f, "=== MONITORS ===")?;

            for (i, m) in self.monitors.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }

                writeln!(f, "{}", m.name)?;
                writeln!(f, "  Resolution : {}x{}", m.width, m.height)?;

                if let Some(rr) = m.refresh_rate {
                    writeln!(f, "  Refresh    : {} Hz", rr)?;
                }

                writeln!(f, "  Position   : {}x{}", m.x, m.y)?;
                writeln!(f, "  Primary    : {}", if m.primary { "Yes" } else { "No" })?;
            }
        }

        /* ================= AUDIO ================= */
        if !self.audio.is_empty() {
            writeln!(f)?;
            writeln!(f, "=== AUDIO DEVICES ===")?;

            use std::collections::HashMap;

            fn extract_card(id: &str) -> Option<String> {
                id.to_lowercase()
                    .split(',')
                    .find_map(|p| p.strip_prefix("card=").map(|v| v.to_string()))
            }

            fn is_hdmi(d: &AudioDeviceInfo) -> bool {
                let n = d.name.to_lowercase();
                let id = d.id.to_lowercase();

                n.contains("hdmi") || id.contains("hdmi")
            }

            fn endpoint_channels(e: &AudioEndpoint) -> u16 {
                e.configs.iter().map(|c| c.channels).max().unwrap_or(0)
            }

            let mut groups: HashMap<String, AudioDeviceGroup> = HashMap::new();

            for d in &self.audio {
                let card = extract_card(&d.id);

                let key = if is_hdmi(&d) {
                    format!("gpu:{}", card.clone().unwrap_or_else(|| "unknown".into()))
                } else {
                    card.clone().unwrap_or_else(|| d.name.clone())
                };

                let entry = groups.entry(key).or_insert(AudioDeviceGroup {
                    name: d.name.clone(),
                    card_id: card.clone(),
                    inputs: vec![],
                    outputs: vec![],
                    is_gpu_audio: is_hdmi(&d),
                });

                let ep = AudioEndpoint {
                    id: d.id.clone(),
                    name: d.name.clone(),
                    kind: d.kind,
                    configs: d.configs.clone(),
                    default: d.default,
                };

                match d.kind {
                    AudioDeviceType::Input => entry.inputs.push(ep),
                    AudioDeviceType::Output => entry.outputs.push(ep),
                }
            }

            fn pick_best(mut v: Vec<AudioEndpoint>) -> Option<AudioEndpoint> {
                v.sort_by(|a, b| {
                    b.default
                        .cmp(&a.default)
                        .then(endpoint_channels(b).cmp(&endpoint_channels(a)))
                });

                v.into_iter().next()
            }

            fn collapse_hdmi(v: &[AudioEndpoint]) -> Vec<AudioEndpoint> {
                let mut best: Option<AudioEndpoint> = None;

                for e in v {
                    if !e.name.to_lowercase().contains("hdmi") {
                        continue;
                    }

                    match &best {
                        None => best = Some(e.clone()),
                        Some(cur) => {
                            let score = |x: &AudioEndpoint| {
                                (x.default, x.name.contains("HDMI 0"), endpoint_channels(x))
                            };

                            if score(e) > score(cur) {
                                best = Some(e.clone());
                            }
                        }
                    }
                }

                best.into_iter().collect()
            }

            let mut groups: Vec<_> = groups.into_values().collect();

            groups.sort_by(|a, b| a.name.cmp(&b.name));

            for (i, g) in groups.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }

                writeln!(f, "{}", g.name)?;

                if let Some(c) = &g.card_id {
                    writeln!(f, "  Card      : {}", c)?;
                } else {
                    writeln!(f, "  Card      : None")?;
                }

                if g.is_gpu_audio {
                    writeln!(f, "  Type      : GPU Audio (HDMI)")?;

                    for h in collapse_hdmi(&g.outputs) {
                        writeln!(f, "  HDMI      : {}", h.name)?;
                    }
                }

                if let Some(inp) = pick_best(g.inputs.clone()) {
                    writeln!(f, "  Input     : {}", inp.name)?;
                }

                if let Some(out) = pick_best(g.outputs.clone()) {
                    writeln!(f, "  Output    : {}", out.name)?;
                }

                writeln!(f, "  Endpoints :")?;

                for e in g.inputs.iter().chain(g.outputs.iter()) {
                    writeln!(f, "  • {}", e.name)?;
                    writeln!(f, "      Id          : {}", e.id)?;
                    writeln!(f, "      Type        : {:?}", e.kind)?;

                    if !e.configs.is_empty() {
                        writeln!(
                            f,
                            "      Configs     : {}",
                            &e.configs
                                .iter()
                                .map(|cfg| if cfg.sample_rate_min == cfg.sample_rate_max {
                                    format!(
                                        "{} ch {}",
                                        cfg.channels,
                                        format_sample_rate(cfg.sample_rate_max)
                                    )
                                } else {
                                    format!(
                                        "{} ch {} - {}",
                                        cfg.channels,
                                        format_sample_rate(cfg.sample_rate_min),
                                        format_sample_rate(cfg.sample_rate_max)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        )?;
                    }

                    writeln!(
                        f,
                        "      Default     : {}",
                        if e.default { "Yes" } else { "No" }
                    )?;
                }
            }
        }

        /* ================= CAMERAS ================= */
        if !self.cameras.is_empty() {
            writeln!(f)?;
            writeln!(f, "=== CAMERAS ===")?;

            for (i, c) in self.cameras.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }

                writeln!(f, "Camera {}", i + 1)?;
                writeln!(f, "  Name : {}", c.name)?;
            }
        }

        /* ================= USB ================= */
        if !self.usb.is_empty() {
            writeln!(f)?;
            writeln!(f, "=== USB DEVICES ===")?;

            for (i, u) in self.usb.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }

                if let Some(p) = &u.product {
                    writeln!(f, "{}", p)?;
                } else {
                    writeln!(f, "{}", u.name)?;
                }

                if let Some(m) = &u.manufacturer {
                    writeln!(f, "  Manufacturer : {}", m)?;
                }

                if let Some(s) = &u.serial_number {
                    writeln!(f, "  Serial       : {}", s)?;
                }

                if !u.device_types.is_empty() {
                    let types = u
                        .device_types
                        .iter()
                        .map(|t| format!("{:?}", t))
                        .collect::<Vec<_>>()
                        .join(", ");

                    writeln!(f, "  Types        : {}", types)?;
                }

                if !u.interface_types.is_empty() {
                    let interfaces = u
                        .interface_types
                        .iter()
                        .map(|t| format!("{:?}", t))
                        .collect::<Vec<_>>()
                        .join(", ");

                    writeln!(f, "  Interfaces   : {}", interfaces)?;
                }

                writeln!(f, "  VID:PID      : {}:{}", u.vendor_id, u.product_id)?;
                writeln!(f, "  Bus          : {}", u.bus)?;
                writeln!(f, "  Port         : {}", u.port)?;
            }
        }

        Ok(())
    }
}

fn format_sample_rate(rate: u32) -> String {
    let rate = rate as f64;

    if rate >= 1_000_000.0 {
        format!("{:.2} MHz", rate / 1_000_000.0)
    } else if rate >= 1_000.0 {
        if (rate % 1000.0).abs() < f64::EPSILON {
            format!("{:.0} kHz", rate / 1000.0)
        } else {
            format!("{:.1} kHz", rate / 1000.0)
        }
    } else {
        format!("{:.0} Hz", rate)
    }
}
