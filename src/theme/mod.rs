pub mod style;
pub use style::ThemeStyle;

use crate::prelude::*;

use ThemeStyle::*;
use atoman::Command;

/// System theme switcher.
#[derive(Debug)]
pub struct SystemTheme;

impl SystemTheme {
    /// Switches system theme.
    #[cfg(target_os = "linux")]
    pub async fn switch(style: ThemeStyle) -> Result<()> {
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

    /// Switches system theme.
    #[cfg(target_os = "macos")]
    pub async fn switch(style: ThemeStyle) -> Result<()> {
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

    /// Switches system theme.
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

    /// Switches system theme.
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    async fn switch(_dark: bool) -> Result<()> {
        Err(ThemeError::UnsupportedOS)
    }

    // Returns current system theme.
    #[cfg(target_os = "linux")]
    pub async fn current() -> Result<ThemeStyle> {
        let output = Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
            .await
            .map_err(ThemeError::GsettingsExecute)?;

        if !output.status.success() {
            return Err(ThemeError::GsettingsExitStatus.into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("prefer-dark") {
            Ok(ThemeStyle::Dark)
        } else {
            Ok(ThemeStyle::Light)
        }
    }

    // Returns current system theme.
    #[cfg(target_os = "macos")]
    pub async fn current() -> Result<ThemeStyle> {
        let script =
            r#"tell application "System Events" to tell appearance preferences to get dark mode"#;

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .await
            .map_err(ThemeError::OsascriptExecute)?;

        if !output.status.success() {
            return Err(ThemeError::OsascriptExitStatus.into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_lowercase();
        if stdout == "true" {
            Ok(ThemeStyle::Dark)
        } else {
            Ok(ThemeStyle::Light)
        }
    }

    // Returns current system theme.
    #[cfg(target_os = "windows")]
    pub async fn current() -> Result<ThemeStyle> {
        use winreg::RegKey;
        use winreg::enums::*;

        tokio::task::spawn_blocking(|| {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
            let key = hkcu.open_subkey_with_flags(path, KEY_READ)?;

            // reading value of AppsUseLightTheme (0 = Dark, 1 = Light)
            let val: u32 = key.get_value("AppsUseLightTheme")?;
            if val == 0 {
                Ok(ThemeStyle::Dark)
            } else {
                Ok(ThemeStyle::Light)
            }
        })
        .await
        .map_err(|e| ThemeError::TaskJoin(e.into()))?
        .map_err(ThemeError::Registry)
    }

    // Returns current system theme.
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub async fn current() -> Result<ThemeStyle> {
        Err(ThemeError::UnsupportedOS.into())
    }
}
