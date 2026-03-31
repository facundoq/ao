use super::PackageManager;
use anyhow::{Context, Result};
use std::process::Command;

pub struct Pacman;

impl PackageManager for Pacman {
    fn update(&self) -> Result<()> {
        println!("Synchronizing package databases and updating system...");
        let status = Command::new("pacman")
            .args(["-Syu", "--noconfirm"])
            .status()
            .context("Failed to execute pacman -Syu")?;

        if !status.success() {
            anyhow::bail!("pacman update failed with status {}", status);
        }

        Ok(())
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        let mut cmd = Command::new("pacman");
        cmd.arg("-S").arg("--noconfirm").args(packages);
        let status = cmd.status().context("Failed to execute pacman -S")?;

        if !status.success() {
            anyhow::bail!("pacman install failed with status {}", status);
        }
        Ok(())
    }

    fn remove(&self, packages: &[String], purge: bool) -> Result<()> {
        let mut cmd = Command::new("pacman");
        if purge {
            // -Rns removes package, its configuration files, and unneeded dependencies
            cmd.arg("-Rns");
        } else {
            // -R just removes the package
            cmd.arg("-R");
        }
        cmd.arg("--noconfirm").args(packages);
        let status = cmd.status().context("Failed to execute pacman remove")?;

        if !status.success() {
            anyhow::bail!("pacman remove failed with status {}", status);
        }
        Ok(())
    }

    fn search(&self, query: &str) -> Result<()> {
        let status = Command::new("pacman")
            .arg("-Ss")
            .arg(query)
            .status()
            .context("Failed to execute pacman search")?;

        if !status.success() {
            anyhow::bail!("pacman search failed with status {}", status);
        }
        Ok(())
    }

    fn list(&self) -> Result<()> {
        let status = Command::new("pacman")
            .arg("-Q")
            .status()
            .context("Failed to execute pacman list")?;

        if !status.success() {
            anyhow::bail!("pacman list failed with status {}", status);
        }
        Ok(())
    }
}
