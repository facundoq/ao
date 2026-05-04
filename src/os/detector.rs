use super::alpine::Apk;
use super::arch::Pacman;
use super::debian::Apt;
use super::fedora::Dnf;
use super::linux_generic::{
    StandardBoot, StandardDev, StandardDisk, StandardDistro, StandardGroup, StandardGui,
    StandardLog, StandardMonitor, StandardNet, StandardPartition, StandardSec, StandardSelf,
    StandardSys, StandardUser, StandardVirt, Systemd,
};
use super::{
    BootManager, DevManager, DiskManager, DistroManager, Domain, GroupManager, GuiManager,
    LogManager, MonitorManager, NetManager, PackageManager, PartitionManager, SecManager,
    SelfManager, ServiceManager, SysManager, UserManager, VirtManager,
};
use anyhow::{Result, bail};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DetectedSystem {
    pub pkg: Box<dyn PackageManager>,
    pub svc: Box<dyn ServiceManager>,
    pub net: Box<dyn NetManager>,
    pub dev: Box<dyn DevManager>,
    pub virt: Box<dyn VirtManager>,
    pub sec: Box<dyn SecManager>,
    pub boot: Box<dyn BootManager>,
    pub gui: Box<dyn GuiManager>,
    pub user: Box<dyn UserManager>,
    pub group: Box<dyn GroupManager>,
    pub disk: Box<dyn DiskManager>,
    pub partition: Box<dyn PartitionManager>,
    pub sys: Box<dyn SysManager>,
    pub log: Box<dyn LogManager>,
    pub distro: Box<dyn DistroManager>,
    pub monitor: Box<dyn MonitorManager>,
    pub self_manager: Box<dyn SelfManager>,
}

impl DetectedSystem {
    pub fn domains(&self) -> Vec<&dyn Domain> {
        vec![
            self.pkg.as_ref(),
            self.svc.as_ref(),
            self.net.as_ref(),
            self.dev.as_ref(),
            self.virt.as_ref(),
            self.sec.as_ref(),
            self.boot.as_ref(),
            self.gui.as_ref(),
            self.user.as_ref(),
            self.group.as_ref(),
            self.disk.as_ref(),
            self.partition.as_ref(),
            self.sys.as_ref(),
            self.log.as_ref(),
            self.distro.as_ref(),
            self.monitor.as_ref(),
            self.self_manager.as_ref(),
        ]
    }
}

pub fn detect_system() -> Result<DetectedSystem> {
    // Read /etc/os-release to determine the distribution
    let mut os_release = String::new();
    if let Ok(file) = File::open("/etc/os-release") {
        let reader = BufReader::new(file);
        for l in reader.lines().map_while(Result::ok) {
            os_release.push_str(&l);
            os_release.push('\n');
        }
    }

    let is_debian_based = os_release.contains("ID=ubuntu")
        || os_release.contains("ID=debian")
        || os_release.contains("ID_LIKE=debian");
    let is_arch_based = os_release.contains("ID=arch") || os_release.contains("ID_LIKE=arch");
    let is_fedora_based = os_release.contains("ID=fedora") || os_release.contains("ID_LIKE=fedora");
    let is_alpine = os_release.contains("ID=alpine");

    if is_debian_based {
        return Ok(DetectedSystem {
            pkg: Box::new(Apt),
            svc: Box::new(Systemd),
            net: Box::new(StandardNet),
            dev: Box::new(StandardDev),
            virt: Box::new(StandardVirt),
            sec: Box::new(StandardSec),
            boot: Box::new(StandardBoot),
            gui: Box::new(StandardGui),
            user: Box::new(StandardUser),
            group: Box::new(StandardGroup),
            disk: Box::new(StandardDisk),
            partition: Box::new(StandardPartition),
            sys: Box::new(StandardSys),
            log: Box::new(StandardLog),
            distro: Box::new(StandardDistro),
            monitor: Box::new(StandardMonitor),
            self_manager: Box::new(StandardSelf),
        });
    }

    if is_arch_based {
        return Ok(DetectedSystem {
            pkg: Box::new(Pacman),
            svc: Box::new(Systemd),
            net: Box::new(StandardNet),
            dev: Box::new(StandardDev),
            virt: Box::new(StandardVirt),
            sec: Box::new(StandardSec),
            boot: Box::new(StandardBoot),
            gui: Box::new(StandardGui),
            user: Box::new(StandardUser),
            group: Box::new(StandardGroup),
            disk: Box::new(StandardDisk),
            partition: Box::new(StandardPartition),
            sys: Box::new(StandardSys),
            log: Box::new(StandardLog),
            distro: Box::new(StandardDistro),
            monitor: Box::new(StandardMonitor),
            self_manager: Box::new(StandardSelf),
        });
    }

    if is_fedora_based {
        return Ok(DetectedSystem {
            pkg: Box::new(Dnf),
            svc: Box::new(Systemd),
            net: Box::new(StandardNet),
            dev: Box::new(StandardDev),
            virt: Box::new(StandardVirt),
            sec: Box::new(StandardSec),
            boot: Box::new(StandardBoot),
            gui: Box::new(StandardGui),
            user: Box::new(StandardUser),
            group: Box::new(StandardGroup),
            disk: Box::new(StandardDisk),
            partition: Box::new(StandardPartition),
            sys: Box::new(StandardSys),
            log: Box::new(StandardLog),
            distro: Box::new(StandardDistro),
            monitor: Box::new(StandardMonitor),
            self_manager: Box::new(StandardSelf),
        });
    }

    if is_alpine {
        return Ok(DetectedSystem {
            pkg: Box::new(Apk),
            svc: Box::new(Systemd), // Placeholder
            net: Box::new(StandardNet),
            dev: Box::new(StandardDev),
            virt: Box::new(StandardVirt),
            sec: Box::new(StandardSec),
            boot: Box::new(StandardBoot),
            gui: Box::new(StandardGui),
            user: Box::new(StandardUser),
            group: Box::new(StandardGroup),
            disk: Box::new(StandardDisk),
            partition: Box::new(StandardPartition),
            sys: Box::new(StandardSys),
            log: Box::new(StandardLog),
            distro: Box::new(StandardDistro),
            monitor: Box::new(StandardMonitor),
            self_manager: Box::new(StandardSelf),
        });
    }

    // Default or panic if unsupported
    bail!("Unsupported operating system. Detected: \n{}", os_release)
}
