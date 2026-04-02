use super::{PackageManager, ServiceManager};
use anyhow::{Context, Result};
use std::process::Command;

pub struct Apt;

impl PackageManager for Apt {
    fn update(&self) -> Result<()> {
        println!("Updating package lists...");
        let status = Command::new("apt")
            .arg("update")
            .status()
            .context("Failed to execute apt update")?;

        if !status.success() {
            anyhow::bail!("apt update failed with status {}", status);
        }

        println!("Applying upgrades...");
        let status = Command::new("apt")
            .args(["upgrade", "-y"])
            .status()
            .context("Failed to execute apt upgrade")?;

        if !status.success() {
            anyhow::bail!("apt upgrade failed with status {}", status);
        }

        Ok(())
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        let mut cmd = Command::new("apt");
        cmd.arg("install").arg("-y").arg("--").args(packages);
        let status = cmd.status().context("Failed to execute apt install")?;

        if !status.success() {
            anyhow::bail!("apt install failed with status {}", status);
        }
        Ok(())
    }

    fn remove(&self, packages: &[String], purge: bool) -> Result<()> {
        let mut cmd = Command::new("apt");
        if purge {
            cmd.arg("purge");
        } else {
            cmd.arg("remove");
        }
        cmd.arg("-y").arg("--").args(packages);
        let status = cmd.status().context("Failed to execute apt remove/purge")?;

        if !status.success() {
            anyhow::bail!("apt remove failed with status {}", status);
        }
        Ok(())
    }

    fn search(&self, query: &str) -> Result<()> {
        let status = Command::new("apt")
            .arg("search")
            .arg("--")
            .arg(query)
            .status()
            .context("Failed to execute apt search")?;

        if !status.success() {
            anyhow::bail!("apt search failed with status {}", status);
        }
        Ok(())
    }

    fn list(&self) -> Result<()> {
        let status = Command::new("apt")
            .arg("list")
            .arg("--installed")
            .status()
            .context("Failed to execute apt list")?;

        if !status.success() {
            anyhow::bail!("apt list failed with status {}", status);
        }
        Ok(())
    }
}

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
            .arg("--")
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
            .arg("--")
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
            .arg("--")
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
            .arg("--")
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
            .arg("--")
            .arg(service)
            .status()
            .context("Failed to get status")?;
        Ok(())
    }
}
