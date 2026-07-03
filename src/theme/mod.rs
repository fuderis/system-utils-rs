pub mod style;
pub use style::ThemeStyle;

use crate::prelude::*;
use ThemeStyle::*;

/// The system theme switcher
#[derive(Debug)]
pub struct SystemTheme;

impl SystemTheme {
    #[cfg(target_os = "linux")]
    pub async fn switch(style: ThemeStyle) -> Result<()> {
        use tokio::process::Command;

        let schema = match style {
            Dark => "prefer-dark",
            Light => "prefer-light",
        };

        let status = Command::new("gsettings")
            .args(&["set", "org.gnome.desktop.interface", "color-scheme", schema])
            .status()
            .await
            .map_err(ThemeError::GsettingsExecute)?;

        if !status.success() {
            return Err(ThemeError::GsettingsExitStatus.into());
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub async fn switch(style: ThemeStyle) -> Result<()> {
        use tokio::process::Command;

        let script = match style {
            ThemeStyle::Dark => {
                r#"tell application "System Events" to tell appearance preferences to set dark mode to true"#
            }
            ThemeStyle::Light => {
                r#"tell application "System Events" to tell appearance preferences to set dark mode to false"#
            }
        };

        Command::new("osascript")
            .args(["-e", script])
            .status()
            .await
            .map_err(ThemeError::OsascriptExecute)?;

        if !status.success() {
            return Err(ThemeError::OsascriptExitStatus);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn switch(style: ThemeStyle) -> Result<(), Error> {
        use winreg::RegKey;
        use winreg::enums::*;

        tokio::task::spawn_blocking(move || {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
            let key = hkcu.open_subkey_with_flags(path, KEY_SET_VALUE)?;

            let val = if style.is_dark() { 0u32 } else { 1u32 };
            key.set_value("AppsUseLightTheme", &val)?;
            key.set_value("SystemUsesLightTheme", &val)?;
            Ok::<(), std::io::Error>(())
        })
        .await
        .map_err(|e| ThemeError::TaskJoin(e.into()))?
        .map_err(ThemeError::Registry)?;

        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    async fn switch(_dark: bool) -> Result<()> {
        Err(ThemeError::UnsupportedOS)
    }
}
