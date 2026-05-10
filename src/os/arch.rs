use super::linux_generic::SystemCommand;
use super::{ExecutableCommand, OutputFormat, PackageManager};
use anyhow::Result;
use std::process::Command;

pub struct Pacman;

impl PackageManager for Pacman {
    fn name(&self) -> &'static str {
        "Pacman"
    }

    fn cmd(&self) -> SystemCommand {
        SystemCommand::new("pacman")
    }

    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(self.cmd().arg("-Syu")))
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            self.cmd()
                .arg("-S")
                .arg("--noconfirm")
                .arg("--")
                .args(packages),
        ))
    }

    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            self.cmd()
                .arg("-Rs")
                .arg("--noconfirm")
                .arg("--")
                .args(packages),
        ))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(self.cmd().arg("-Ss").arg("--").arg(query)))
    }

    fn ls(&self, _format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(self.cmd().arg("-Q")))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("pacman").arg("-Qq").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("pacman").arg("-Slq").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}
