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
    LogManager, NetManager, OverviewManager, PackageDomain, PackageManager, PartitionManager,
    SecManager, SelfManager, ServiceManager, SysManager, UserManager, VirtManager,
};
use anyhow::{Result, bail};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DetectedSystem {
    pub pkg: Box<dyn Domain>,
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
    pub overview: Box<dyn OverviewManager>,
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
            self.overview.as_ref(),
            self.self_manager.as_ref(),
        ]
    }
}

const OS_RELEASE_PATH: &str = "/etc/os-release";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distro {
    Debian,
    Arch,
    Fedora,
    Alpine,
}

impl Distro {
    pub fn detect() -> Result<Self> {
        let mut os_release = String::new();
        if let Ok(file) = File::open(OS_RELEASE_PATH) {
            let reader = BufReader::new(file);
            for l in reader.lines().map_while(Result::ok) {
                os_release.push_str(&l);
                os_release.push('\n');
            }
        }

        if os_release.contains("ID=ubuntu")
            || os_release.contains("ID=debian")
            || os_release.contains("ID_LIKE=debian")
        {
            Ok(Distro::Debian)
        } else if os_release.contains("ID=arch") || os_release.contains("ID_LIKE=arch") {
            Ok(Distro::Arch)
        } else if os_release.contains("ID=fedora") || os_release.contains("ID_LIKE=fedora") {
            Ok(Distro::Fedora)
        } else if os_release.contains("ID=alpine") {
            Ok(Distro::Alpine)
        } else {
            bail!("Unsupported operating system. Detected: \n{}", os_release)
        }
    }
}

pub fn detect_system() -> Result<DetectedSystem> {
    let distro = Distro::detect()?;

    let pkg_manager: Box<dyn PackageManager> = match distro {
        Distro::Debian => Box::new(Apt),
        Distro::Arch => Box::new(Pacman),
        Distro::Fedora => Box::new(Dnf),
        Distro::Alpine => Box::new(Apk),
    };

    Ok(DetectedSystem {
        pkg: Box::new(PackageDomain {
            manager: pkg_manager,
        }),
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
        overview: Box::new(StandardMonitor),
        self_manager: Box::new(StandardSelf),
    })
}
