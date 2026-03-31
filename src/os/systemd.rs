use super::ServiceManager;
use anyhow::{Context, Result};
use std::process::Command;

pub struct Systemd;

impl ServiceManager for Systemd {
    fn list(&self) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .status()
            .context("Failed to execute systemctl list-units")?;

        if !status.success() {
            anyhow::bail!("systemctl failed with status {}", status);
        }
        Ok(())
    }

    fn up(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("enable")
            .arg("--now")
            .arg(service)
            .status()
            .context("Failed to start/enable service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to start service {}", service);
        }
        Ok(())
    }

    fn down(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("disable")
            .arg("--now")
            .arg(service)
            .status()
            .context("Failed to stop/disable service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to stop service {}", service);
        }
        Ok(())
    }

    fn restart(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("restart")
            .arg(service)
            .status()
            .context("Failed to restart service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to restart service {}", service);
        }
        Ok(())
    }

    fn reload(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("reload")
            .arg(service)
            .status()
            .context("Failed to reload service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to reload service {}", service);
        }
        Ok(())
    }

    fn status(&self, service: &str) -> Result<()> {
        // We use status to let it stream directly to the terminal stdout
        let _ = Command::new("systemctl")
            .arg("status")
            .arg(service)
            .status()
            .context("Failed to get status")?;
        Ok(())
    }
}
