use super::PackageManager;
use anyhow::{Context, Result};
use std::process::Command;

pub struct Dnf;

impl PackageManager for Dnf {
    fn update(&self) -> Result<()> {
        println!("Updating package lists and applying upgrades...");
        let status = Command::new("dnf")
            .args(["upgrade", "-y"])
            .status()
            .context("Failed to execute dnf upgrade")?;

        if !status.success() {
            anyhow::bail!("dnf upgrade failed with status {}", status);
        }

        Ok(())
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        let mut cmd = Command::new("dnf");
        cmd.arg("install").arg("-y").args(packages);
        let status = cmd.status().context("Failed to execute dnf install")?;

        if !status.success() {
            anyhow::bail!("dnf install failed with status {}", status);
        }
        Ok(())
    }

    fn remove(&self, packages: &[String], purge: bool) -> Result<()> {
        let mut cmd = Command::new("dnf");
        if purge {
            // dnf remove cleans up most things, autoremove can be used later
            cmd.arg("remove");
        } else {
            cmd.arg("remove");
        }
        cmd.arg("-y").args(packages);
        let status = cmd.status().context("Failed to execute dnf remove")?;

        if !status.success() {
            anyhow::bail!("dnf remove failed with status {}", status);
        }
        Ok(())
    }

    fn search(&self, query: &str) -> Result<()> {
        let status = Command::new("dnf")
            .arg("search")
            .arg(query)
            .status()
            .context("Failed to execute dnf search")?;

        if !status.success() {
            anyhow::bail!("dnf search failed with status {}", status);
        }
        Ok(())
    }

    fn list(&self) -> Result<()> {
        let status = Command::new("dnf")
            .arg("list")
            .arg("installed")
            .status()
            .context("Failed to execute dnf list installed")?;

        if !status.success() {
            anyhow::bail!("dnf list failed with status {}", status);
        }
        Ok(())
    }
}
