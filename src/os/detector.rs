use super::{PackageManager, ServiceManager};
use super::debian::{Apt, Systemd};
use anyhow::{Result, bail};
use std::fs;

pub struct DetectedSystem {
    pub pkg: Box<dyn PackageManager>,
    pub svc: Box<dyn ServiceManager>,
}

pub fn detect_system() -> Result<DetectedSystem> {
    // Read /etc/os-release to determine the distribution
    let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();

    // Check if it's Debian/Ubuntu based
    if os_release.contains("ID=ubuntu") || os_release.contains("ID=debian") {
        return Ok(DetectedSystem {
            pkg: Box::new(Apt),
            svc: Box::new(Systemd),
        });
    }

    // Default or panic if unsupported
    bail!("Unsupported operating system. This version of ao only supports Debian/Ubuntu.")
}
