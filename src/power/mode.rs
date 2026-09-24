use crate::prelude::*;

/// The power action type
#[derive(Debug, Clone, Copy, Display, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[display(rename_all = "lowercase")]
pub enum PowerMode {
    Shutdown,
    Suspend,
    Reboot,
    Lock,
    Logout,
}
