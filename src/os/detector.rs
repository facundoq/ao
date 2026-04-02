use super::debian::{Apt, Systemd};
use super::{PackageManager, ServiceManager};
use anyhow::{Result, bail};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DetectedSystem {
    pub pkg: Box<dyn PackageManager>,
    pub svc: Box<dyn ServiceManager>,
}

pub fn detect_system() -> Result<DetectedSystem> {
    // Read /etc/os-release to determine the distribution
    if let Ok(file) = File::open("/etc/os-release") {
        let reader = BufReader::new(file);

        // Check if it's Debian/Ubuntu based line by line
        for line in reader.lines() {
            if let Ok(line) = line {
                if line.contains("ID=ubuntu") || line.contains("ID=debian") {
                    return Ok(DetectedSystem {
                        pkg: Box::new(Apt),
                        svc: Box::new(Systemd),
                    });
                }
            }
        }
    }

    // Default or panic if unsupported
    bail!("Unsupported operating system. This version of ao only supports Debian/Ubuntu.")
}
