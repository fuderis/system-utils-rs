use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub name: String,

    pub width: u32,
    pub height: u32,

    pub refresh_rate: Option<u32>,

    pub primary: bool,

    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UsbDeviceType {
    Storage,
    Audio,
    Camera,
    Network,
    Hub,
    Bluetooth,

    Serial,
    Arduino,

    XboxController,
    GameController,

    Composite,
    VendorSpecific,
    Receiver,

    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UsbInterfaceType {
    Hid,
    Audio,
    Video,
    CdcAcm,
    Storage,
    Hub,
    Bluetooth,
    Composite,
    VendorSpecific,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDeviceInfo {
    pub name: String,

    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,

    pub vendor_id: String,
    pub product_id: String,

    pub device_class: Option<u8>,
    pub device_sub_class: Option<u8>,
    pub device_protocol: Option<u8>,

    pub device_types: Vec<UsbDeviceType>,
    pub interface_types: Vec<UsbInterfaceType>,

    pub bus: u8,
    pub port: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum AudioDeviceType {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum AudioBackend {
    Hw,
    PlugHw,
    SysDefault,
    Front,
    Surround,
    Hdmi,
    UsbStream,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub kind: AudioDeviceType,
    pub default: bool,
    pub is_physical: bool,

    pub configs: Vec<AudioConfigInfo>,

    pub latency_ms: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfigInfo {
    pub channels: u16,
    pub sample_rate_min: u32,
    pub sample_rate_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEndpoint {
    pub id: String,
    pub name: String,
    pub kind: AudioDeviceType,

    pub configs: Vec<AudioConfigInfo>,

    pub default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceGroup {
    pub name: String,
    pub card_id: Option<String>,

    pub inputs: Vec<AudioEndpoint>,
    pub outputs: Vec<AudioEndpoint>,

    pub is_gpu_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub name: String,
}

/* TODO:
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum BluetoothDeviceType {
    Headphones,
    Earbuds,
    Speaker,
    Mouse,
    Keyboard,
    Phone,
    Gamepad,
    Watch,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDeviceInfo {
    pub name: String,

    pub device_type: Option<BluetoothDeviceType>,

    pub connected: bool,
    pub paired: bool,

    pub battery_percent: Option<u8>,
}
*/
