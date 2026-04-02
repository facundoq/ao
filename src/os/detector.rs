use super::debian::{Apt, Disk, Group, Monitor, Systemd, User};
use super::{
    DiskManager, GroupManager, MonitorManager, PackageManager, ServiceManager, UserManager,
};
use anyhow::{Result, bail};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DetectedSystem {
    pub pkg: Box<dyn PackageManager>,
    pub svc: Box<dyn ServiceManager>,
    pub user: Box<dyn UserManager>,
    pub group: Box<dyn GroupManager>,
    pub disk: Box<dyn DiskManager>,
    pub monitor: Box<dyn MonitorManager>,
}

pub fn detect_system() -> Result<DetectedSystem> {
    // Read /etc/os-release to determine the distribution
    if let Ok(file) = File::open("/etc/os-release") {
        let reader = BufReader::new(file);

    // Check if it's Debian/Ubuntu based
    if os_release.contains("ID=ubuntu") || os_release.contains("ID=debian") {
        return Ok(DetectedSystem {
            pkg: Box::new(Apt),
            svc: Box::new(Systemd),
            user: Box::new(User),
            group: Box::new(Group),
            disk: Box::new(Disk),
            monitor: Box::new(Monitor),
        });
    }

    // Default or panic if unsupported
    bail!("Unsupported operating system. This version of ao only supports Debian/Ubuntu.")
}
