use super::alpine::Apk;
use super::arch::Pacman;
use super::debian::Apt;
use super::fedora::Dnf;
use super::linux_generic::{
    StandardBoot, StandardDev, StandardDisk, StandardDistro, StandardGroup, StandardGui,
    StandardLog, StandardMonitor, StandardNet, StandardPartition, StandardSec, StandardSelf,
    StandardSys, StandardUser, StandardVirt, Systemd,
};
use super::macos;
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
    MacOS,
}

impl Distro {
    pub fn detect() -> Result<Self> {
        #[cfg(target_os = "macos")]
        return Ok(Distro::MacOS);

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

    match distro {
        Distro::MacOS => Ok(DetectedSystem {
            pkg: Box::new(PackageDomain {
                manager: Box::new(macos::MacOSPackage),
            }),
            svc: Box::new(macos::MacOSService),
            net: Box::new(macos::MacOSNet),
            dev: Box::new(StandardDev),
            virt: Box::new(macos::MacOSVirt),
            sec: Box::new(macos::MacOSSec),
            boot: Box::new(macos::MacOSBoot),
            gui: Box::new(macos::MacOSGui),
            user: Box::new(macos::MacOSUser),
            group: Box::new(macos::MacOSGroup),
            disk: Box::new(macos::MacOSDisk),
            partition: Box::new(macos::MacOSPartition),
            sys: Box::new(macos::MacOSSys),
            log: Box::new(macos::MacOSLog),
            distro: Box::new(macos::MacOSDistro),
            overview: Box::new(macos::MacOSOverview),
            self_manager: Box::new(macos::MacOSSelf),
        }),
        _ => {
            let (pkg_manager, svc_manager, overview_manager): (
                Box<dyn PackageManager>,
                Box<dyn ServiceManager>,
                Box<dyn OverviewManager>,
            ) = match distro {
                Distro::Debian => (Box::new(Apt), Box::new(Systemd), Box::new(StandardMonitor)),
                Distro::Arch => (
                    Box::new(Pacman),
                    Box::new(Systemd),
                    Box::new(StandardMonitor),
                ),
                Distro::Fedora => (Box::new(Dnf), Box::new(Systemd), Box::new(StandardMonitor)),
                Distro::Alpine => (Box::new(Apk), Box::new(Systemd), Box::new(StandardMonitor)),
                Distro::MacOS => unreachable!(),
            };

            Ok(DetectedSystem {
                pkg: Box::new(PackageDomain {
                    manager: pkg_manager,
                }),
                svc: svc_manager,
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
                overview: overview_manager,
                self_manager: Box::new(StandardSelf),
            })
        }
    }
}
