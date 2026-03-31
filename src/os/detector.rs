use super::{PackageManager, ServiceManager};
use super::debian::Apt;
use super::fedora::Dnf;
use super::arch::Pacman;
use super::systemd::Systemd;
use anyhow::{Result, bail};
use std::fs;

pub struct DetectedSystem {
    pub pkg: Box<dyn PackageManager>,
    pub svc: Box<dyn ServiceManager>,
}

pub fn detect_system() -> Result<DetectedSystem> {
    // Read /etc/os-release to determine the distribution
    let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();

    if os_release.contains("ID=ubuntu") || os_release.contains("ID=debian") {
        return Ok(DetectedSystem {
            pkg: Box::new(Apt),
            svc: Box::new(Systemd),
        });
    }

    if os_release.contains("ID=fedora") {
        return Ok(DetectedSystem {
            pkg: Box::new(Dnf),
            svc: Box::new(Systemd),
        });
    }

    if os_release.contains("ID=arch") {
        return Ok(DetectedSystem {
            pkg: Box::new(Pacman),
            svc: Box::new(Systemd),
        });
    }

    // Default or panic if unsupported
    bail!("Unsupported operating system. This version of ao supports Debian, Ubuntu, Fedora, and Arch Linux.")
}
