use super::PackageManager;
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
        cmd.arg("install").arg("-y").args(packages);
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
        cmd.arg("-y").args(packages);
        let status = cmd.status().context("Failed to execute apt remove/purge")?;

        if !status.success() {
            anyhow::bail!("apt remove failed with status {}", status);
        }
        Ok(())
    }

    fn search(&self, query: &str) -> Result<()> {
        let status = Command::new("apt")
            .arg("search")
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
