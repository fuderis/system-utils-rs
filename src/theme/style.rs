use crate::prelude::*;

/// The system theme mode
#[derive(Debug, Display, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
#[display(rename = "lowercase")]
pub enum ThemeStyle {
    Light,
    Dark,
}

impl ThemeStyle {
    pub fn is_light(&self) -> bool {
        matches!(self, Self::Light)
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }
}
