pub mod mode;
pub use mode::PowerMode;

use crate::prelude::*;

use atoman::{Command, JoinHandle, time};
use chrono::{DateTime, Utc};

static POWER_STATE: State<Option<PowerState>> = State::default();

#[derive(Debug, Clone)]
pub struct PowerState {
    task: Arc<JoinHandle<()>>,
    status: ScheduledPowerTask,
}

#[derive(Debug, Clone, Copy)]
pub struct ScheduledPowerTask {
    pub mode: PowerMode,
    pub execute_at: DateTime<Utc>,
}

/// System power manager.
#[derive(Debug)]
pub struct PowerManager;

impl PowerManager {
    /// Helper method to schedule the power action.
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
            Self::cancel().await;

            let mut guard = POWER_STATE.lock().await;

            // spawn new task
            let duration = (timestamp - Utc::now()).to_std()?;
            let handle = atoman::spawn(async move {
                time::sleep(duration).await;

                POWER_STATE.lock().await.take();
                let _ = callback.await;
            });

            // update state
            guard.replace(PowerState {
                task: arc!(handle),
                status: ScheduledPowerTask {
                    mode,
                    execute_at: timestamp,
                },
            });
        } else {
            callback.await?;
        }

        Ok(())
    }

    /// Cancels power action (returns the canceled `PowerMode`).
    pub async fn cancel() -> Option<PowerMode> {
        // cancel active task without lock's
        if let Some(state) = POWER_STATE.get().as_ref() {
            state.task.abort();
        }

        // remove task from state
        let mut guard = POWER_STATE.lock().await;
        guard.take().map(|s| s.status.mode)
    }

    /// Returns active power action (fast check).
    #[inline]
    pub fn check() -> Option<ScheduledPowerTask> {
        (*POWER_STATE.get()).as_ref().map(|s| s.status.clone())
    }

    /// Returns active power action.
    pub async fn status() -> Option<ScheduledPowerTask> {
        (*POWER_STATE.get_locked().await)
            .as_ref()
            .map(|s| s.status.clone())
    }
}

impl PowerManager {
    /// Schedules power action.
    pub async fn schedule(mode: PowerMode, timestamp: Option<DateTime<Utc>>) -> Result<()> {
        use PowerMode::*;

        match mode {
            Shutdown => Self::shutdown(timestamp).await,
            Suspend => Self::suspend(timestamp).await,
            Reboot => Self::reboot(timestamp).await,
            Lock => Self::lock(timestamp).await,
            Logout => Self::logout(timestamp).await,
        }
    }

    /// Does shutdown the system in future.
    pub async fn shutdown(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Shutdown, timestamp, async {
            Self::shutdown_now().await
        })
        .await
    }

    /// Does shutdown the system.
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

    /// Does reboot the system in future.
    pub async fn reboot(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Reboot, timestamp, async {
            Self::reboot_now().await
        })
        .await
    }

    /// Does reboot the system.
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

    /// Does suspend the system in future.
    pub async fn suspend(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Suspend, timestamp, async {
            Self::suspend_now().await
        })
        .await
    }

    /// Does suspend the system.
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

    /// Does lock the system in future.
    pub async fn lock(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Lock, timestamp, async { Self::lock_now().await }).await
    }

    /// Does lock the system.
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

    /// Does logout the system in future.
    pub async fn logout(timestamp: Option<DateTime<Utc>>) -> Result<()> {
        Self::schedule_with(PowerMode::Logout, timestamp, async {
            Self::logout_now().await
        })
        .await
    }

    /// Does logout the system.
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
