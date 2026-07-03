pub mod mode;
pub use mode::PowerMode;

use crate::prelude::*;
use chrono::{DateTime, Utc};
use tokio::{process::Command, task::JoinHandle};

static POWER_TASK: State<Option<Arc<JoinHandle<()>>>> = State::default();
static POWER_STATUS: State<Option<ScheduledPowerTask>> = State::default();

#[derive(Debug, Clone, Copy)]
pub struct ScheduledPowerTask {
    pub mode: PowerMode,
    pub execute_at: DateTime<Utc>,
}

/// The system power manager
#[derive(Debug)]
pub struct PowerManager;

impl PowerManager {
    /// Helper method to schedule the power action
    async fn schedule_with<F>(
        mode: PowerMode,
        execute_at: Option<DateTime<Utc>>,
        callback: F,
    ) -> Result<()>
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        if let Some(timestamp) = execute_at {
            if timestamp <= Utc::now() {
                return Err(str!("Timestamp is in the past").into());
            }

            // cancel previous task
            if let Some(old) = POWER_TASK.get().await.as_ref() {
                old.abort();
            }

            POWER_TASK.set(None).await;

            POWER_STATUS
                .set(Some(ScheduledPowerTask {
                    mode,
                    execute_at: timestamp,
                }))
                .await;

            let duration = (timestamp - Utc::now()).to_std()?;

            let handle = tokio::spawn(async move {
                tokio::time::sleep(duration).await;

                POWER_STATUS.set(None).await;
                POWER_TASK.set(None).await;

                let _ = callback.await;
            });

            POWER_TASK.set(Some(arc!(handle))).await;
        } else {
            callback.await?;
        }

        Ok(())
    }

    /// Cancels the power action
    pub async fn cancel() -> bool {
        let had_task = POWER_STATUS.get().await.is_some();

        if let Some(task) = POWER_TASK.get().await.as_ref() {
            task.abort();
        }

        POWER_TASK.set(None).await;
        POWER_STATUS.set(None).await;

        had_task
    }

    /// Returns status about the active power action
    pub async fn status() -> Option<ScheduledPowerTask> {
        POWER_STATUS.get().await.as_ref().clone()
    }
}

impl PowerManager {
    /// Schedules the power action
    pub async fn schedule(mode: PowerMode, timestamp: DateTime<Utc>) -> Result<()> {
        use PowerMode::*;

        match mode {
            Shutdown => Self::shutdown(timestamp),
            Suspend => Self::suspend(timestamp),
            Reboot => Self::reboot(timestamp),
            Lock => Self::lock(timestamp),
            Logout => Self::logout(timestamp),
            Cancel => Self::cancel(timestamp),
            Status => Self::status(timestamp),
        }
    }

    /// Does shutdown the system in future
    pub async fn shutdown(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Shutdown, timestamp, async {
            Self::shutdown_now().await
        })
        .await
    }

    /// Does shutdown the system
    pub async fn shutdown_now() -> Result<()> {
        let (cmd, args): (&str, &[&str]) = {
            #[cfg(windows)]
            {
                ("shutdown", &["/s"])
            }
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            {
                ("shutdown", &["-h", "now"])
            }
        };

        Command::new(cmd).args(args).status().await?;
        Ok(())
    }

    /// Does reboot the system in future
    pub async fn reboot(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Reboot, timestamp, async {
            Self::reboot_now().await
        })
        .await
    }

    /// Does reboot the system
    pub async fn reboot_now() -> Result<()> {
        let (cmd, args): (&str, &[&str]) = {
            #[cfg(target_os = "linux")]
            {
                ("reboot", &[])
            }
            #[cfg(target_os = "macos")]
            {
                ("shutdown", &["-r", "now"])
            }
            #[cfg(windows)]
            {
                ("shutdown", &["/r"])
            }
        };

        Command::new(cmd).args(args).status().await?;
        Ok(())
    }

    /// Does suspend the system in future
    pub async fn suspend(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Suspend, timestamp, async {
            Self::suspend_now().await
        })
        .await
    }

    /// Does suspend the system
    pub async fn suspend_now() -> Result<()> {
        let (cmd, args): (&str, &[&str]) = {
            #[cfg(target_os = "linux")]
            {
                ("systemctl", &["suspend"])
            }
            #[cfg(target_os = "macos")]
            {
                ("pmset", &["sleepnow"])
            }
            #[cfg(windows)]
            {
                ("rundll32.exe", &["powrprof.dll,SetSuspendState", "0,1,0"])
            }
        };

        Command::new(cmd).args(args).status().await?;
        Ok(())
    }

    /// Does lock the system in future
    pub async fn lock(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Lock, timestamp, async { Self::lock_now().await }).await
    }

    /// Does lock the system
    pub async fn lock_now() -> Result<()> {
        let (cmd, args): (&str, &[&str]) = {
            #[cfg(target_os = "linux")]
            {
                ("loginctl", &["lock-session"])
            }
            #[cfg(target_os = "macos")]
            {
                ("open", &["-a", "loginwindow"])
            }
            #[cfg(windows)]
            {
                ("rundll32.exe", &["user32.dll,LockWorkStation"])
            }
        };

        Command::new(cmd).args(args).status().await?;
        Ok(())
    }

    /// Does logout the system in future
    pub async fn logout(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Logout, timestamp, async {
            Self::logout_now().await
        })
        .await
    }

    /// Does logout the system
    pub async fn logout_now() -> Result<()> {
        let (cmd, args): (&str, &[&str]) = {
            #[cfg(target_os = "linux")]
            {
                ("loginctl", &["terminate-session", "self"])
            }
            #[cfg(target_os = "macos")]
            {
                (
                    "osascript",
                    &["-e", "tell application \"System Events\" to log out"],
                )
            }
            #[cfg(windows)]
            {
                ("shutdown", &["/l"])
            }
        };

        Command::new(cmd).args(args).status().await?;
        Ok(())
    }
}
