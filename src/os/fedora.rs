use super::linux_generic::SystemCommand;
use super::{ExecutableCommand, OutputFormat, PackageManager};
use anyhow::Result;
use std::process::Command;

pub struct Dnf;

impl PackageManager for Dnf {
    fn name(&self) -> &'static str {
        "DNF"
    }

    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("check-update")))
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf")
                .arg("install")
                .arg("-y")
                .arg("--")
                .args(packages),
        ))
    }

    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf")
                .arg("remove")
                .arg("-y")
                .arg("--")
                .args(packages),
        ))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf").arg("search").arg("--").arg(query),
        ))
    }

    fn ls(&self, _format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf").arg("list").arg("--installed"),
        ))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("rpm")
            .arg("-qa")
            .arg("--queryformat")
            .arg("%{NAME}\n")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("dnf")
            .arg("repoquery")
            .arg("--qf")
            .arg("%{name}")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}
