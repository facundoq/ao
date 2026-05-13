use super::linux_generic::{CompoundCommand, SystemCommand};
use super::{
    BootManager, ContainerInfo, DiskManager, DistroManager, Domain, ExecutableCommand, FwRuleInfo,
    GroupManager, GuiManager, LogManager, NetInterfaceInfo, NetManager, OverviewManager,
    PackageManager, PartitionManager, SecManager, SelfManager, SensorInfo, ServiceInfo,
    ServiceManager, SysInfoData, SysManager, UserManager, UserSessionInfo, VirtManager,
};
use crate::cli::OutputFormat;
use anyhow::{Result, bail};
use clap::{ArgMatches, Command};

#[derive(Default)]
pub struct MacOS;

impl MacOS {
    pub fn new() -> Self {
        Self
    }
}

pub struct MacOSPackage;
impl Domain for MacOSPackage {
    fn name(&self) -> &'static str {
        "package"
    }
    fn command(&self) -> Command {
        use crate::cli::PackageArgs;
        use clap::Args;
        PackageArgs::augment_args(
            Command::new(Domain::name(self)).about("Manage MacOS packages (homebrew)"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &Command,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        use crate::cli::{PackageAction, PackageArgs};
        use clap::FromArgMatches;
        let args = PackageArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(PackageAction::Update) => self.update(),
            Some(PackageAction::Add { packages }) => self.add(packages),
            Some(PackageAction::Delete { packages, purge: _ }) => self.del(packages, false),
            Some(PackageAction::Search { query }) => self.search(query),
            Some(PackageAction::List { format }) => self.ls(*format),
            None => self.ls(OutputFormat::Table),
        }
    }
}
impl PackageManager for MacOSPackage {
    fn name(&self) -> &'static str {
        "homebrew"
    }
    fn cmd(&self) -> SystemCommand {
        SystemCommand::new("brew")
    }
    fn add(&self, packages: &[String]) -> Result<Box<dyn super::ExecutableCommand>> {
        let mut cmd = self.cmd().arg("install");
        for p in packages {
            cmd = cmd.arg(p);
        }
        Ok(Box::new(cmd))
    }
    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        let mut cmd = self.cmd().arg("uninstall");
        for p in packages {
            cmd = cmd.arg(p);
        }
        Ok(Box::new(cmd))
    }
    fn search(&self, query: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(self.cmd().arg("search").arg(query)))
    }
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        match format {
            OutputFormat::Json | OutputFormat::Yaml => Ok(Box::new(
                self.cmd().arg("info").arg("--json=v2").arg("--installed"),
            )),
            _ => Ok(Box::new(self.cmd().arg("list").arg("--versions"))),
        }
    }
    fn get_installed_packages(&self) -> Result<Vec<String>> {
        use super::linux_generic::common::command_exists;
        if !command_exists("brew") {
            return Ok(vec![]);
        }
        let output = std::process::Command::new("brew").arg("list").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
    }
    fn get_available_packages(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct MacOSService;
impl Domain for MacOSService {
    fn name(&self) -> &'static str {
        "service"
    }
    fn command(&self) -> Command {
        use crate::cli::ServiceArgs;
        use clap::Args;
        ServiceArgs::augment_args(
            Command::new(self.name()).about("Manage MacOS services (launchctl)"),
        )
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{ServiceAction, ServiceArgs};
        use clap::FromArgMatches;
        let args = ServiceArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(ServiceAction::List { format }) => self.ls(*format),
            Some(ServiceAction::Up { name }) => self.up(name),
            Some(ServiceAction::Down { name }) => self.down(name),
            Some(ServiceAction::Restart { name }) => self.restart(name),
            Some(ServiceAction::Reload { name }) => self.reload(name),
            Some(ServiceAction::Status { name }) => self.status(name),
            None => self.ls(OutputFormat::Table),
        }
    }
}
impl ServiceManager for MacOSService {
    fn ls(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("launchctl").arg("list")))
    }
    fn up(&self, service: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("launchctl")
                .arg("load")
                .arg("-w")
                .arg(service),
        ))
    }
    fn down(&self, service: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("launchctl")
                .arg("unload")
                .arg("-w")
                .arg(service),
        ))
    }
    fn restart(&self, service: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        let down = self.down(service)?;
        let up = self.up(service)?;
        Ok(Box::new(CompoundCommand::new(vec![down, up])))
    }
    fn reload(&self, service: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        self.restart(service)
    }
    fn status(&self, service: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("launchctl").arg("list").arg(service),
        ))
    }
    fn get_services(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
    fn get_all_services_info(&self) -> Result<Vec<ServiceInfo>> {
        Ok(vec![])
    }
}

pub struct MacOSUser;
impl Domain for MacOSUser {
    fn name(&self) -> &'static str {
        "user"
    }
    fn command(&self) -> Command {
        use crate::cli::UserArgs;
        use clap::Args;
        UserArgs::augment_args(Command::new(self.name()).about("Manage MacOS users (dscl)"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &Command,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        use crate::cli::{UserAction, UserArgs};
        use clap::FromArgMatches;
        let args = UserArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(UserAction::List {
                all,
                groups,
                format,
            }) => self.list(*all, *groups, *format),
            Some(UserAction::Add {
                username,
                name,
                email,
                groups,
                shell,
                system,
                no_create_home,
            }) => self.add(
                username,
                name.as_deref(),
                email.as_deref(),
                groups.as_deref(),
                shell.as_deref(),
                *system,
                *no_create_home,
            ),
            Some(UserAction::Delete { username, purge }) => self.delete(username, *purge),
            Some(UserAction::Modify {
                username,
                action,
                value,
            }) => self.modify_user(username, action, value),
            Some(UserAction::Passwd { username }) => self.passwd(username),
            Some(UserAction::Session {
                username,
                all,
                n,
                format,
            }) => self.session(username.as_deref(), *all, *n, *format),
            None => self.list(false, false, OutputFormat::Table),
        }
    }
}
impl UserManager for MacOSUser {
    fn list(
        &self,
        _all: bool,
        _groups: bool,
        _format: OutputFormat,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-list")
                .arg("/Users"),
        ))
    }
    fn add(
        &self,
        username: &str,
        _name: Option<&str>,
        _email: Option<&str>,
        _groups: Option<&str>,
        _shell: Option<&str>,
        _system: bool,
        _no_create_home: bool,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-create")
                .arg(&format!("/Users/{}", username)),
        ))
    }
    fn delete(&self, username: &str, _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-delete")
                .arg(&format!("/Users/{}", username)),
        ))
    }
    fn modify_user(
        &self,
        username: &str,
        action: &str,
        value: &str,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-create")
                .arg(&format!("/Users/{}", username))
                .arg(action)
                .arg(value),
        ))
    }
    fn passwd(&self, username: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("passwd").arg(username)))
    }
    fn session(
        &self,
        _username: Option<&str>,
        _all: bool,
        _n: Option<u32>,
        _format: OutputFormat,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("last")))
    }
    fn get_sessions(
        &self,
        username: Option<&str>,
        _all: bool,
        n: Option<u32>,
    ) -> Result<Vec<UserSessionInfo>> {
        let mut cmd = std::process::Command::new("last");
        if let Some(n) = n {
            cmd.arg("-n").arg(n.to_string());
        }
        if let Some(u) = username {
            cmd.arg(u);
        }
        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut sessions = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }
            sessions.push(UserSessionInfo {
                username: parts[0].to_string(),
                line: parts[1].to_string(),
                host: parts[2].to_string(),
                start: parts[3..].join(" "),
                end: String::new(),
                duration: String::new(),
            });
        }
        Ok(sessions)
    }
    fn get_users(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new("dscl")
            .arg(".")
            .arg("-list")
            .arg("/Users")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.starts_with('_'))
            .collect())
    }
    fn get_shells(&self) -> Result<Vec<String>> {
        let content = std::fs::read_to_string("/etc/shells")?;
        Ok(content
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|s| s.trim().to_string())
            .collect())
    }
}

pub struct MacOSGroup;
impl Domain for MacOSGroup {
    fn name(&self) -> &'static str {
        "group"
    }
    fn command(&self) -> Command {
        use crate::cli::GroupArgs;
        use clap::Args;
        GroupArgs::augment_args(Command::new(self.name()).about("Manage MacOS groups (dscl)"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &Command,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        use crate::cli::{GroupAction, GroupArgs};
        use clap::FromArgMatches;
        let args = GroupArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(GroupAction::List { format }) => self.ls(*format),
            Some(GroupAction::Add { groupname }) => self.add(groupname),
            Some(GroupAction::Delete { groupname }) => self.del(groupname),
            Some(GroupAction::Modify { groupname, gid }) => self.mod_group(groupname, *gid),
            None => self.ls(OutputFormat::Table),
        }
    }
}
impl GroupManager for MacOSGroup {
    fn ls(&self, _format: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-list")
                .arg("/Groups"),
        ))
    }
    fn add(&self, groupname: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-create")
                .arg(&format!("/Groups/{}", groupname)),
        ))
    }
    fn del(&self, groupname: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-delete")
                .arg(&format!("/Groups/{}", groupname)),
        ))
    }
    fn mod_group(&self, groupname: &str, gid: u32) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dscl")
                .arg(".")
                .arg("-create")
                .arg(&format!("/Groups/{}", groupname))
                .arg("PrimaryGroupID")
                .arg(&gid.to_string()),
        ))
    }
    fn get_groups(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new("dscl")
            .arg(".")
            .arg("-list")
            .arg("/Groups")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.starts_with('_'))
            .collect())
    }
}

pub struct MacOSNet;
impl Domain for MacOSNet {
    fn name(&self) -> &'static str {
        "network"
    }
    fn command(&self) -> Command {
        use crate::cli::NetworkArgs;
        use clap::Args;
        NetworkArgs::augment_args(
            Command::new(self.name()).about("Manage MacOS networking (networksetup)"),
        )
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{FirewallAction, NetworkAction, NetworkArgs, WifiAction};
        use clap::FromArgMatches;
        let args = NetworkArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(NetworkAction::Interfaces { format }) => self.interfaces(*format),
            Some(NetworkAction::Ips { format }) => self.ips(*format),
            Some(NetworkAction::Routes { format }) => self.routes(*format),
            Some(NetworkAction::Firewall { action }) => match action {
                FirewallAction::Status { format } => self.fw_status(*format),
                FirewallAction::Allow { rule } => self.fw_allow(rule),
                FirewallAction::Deny { rule } => self.fw_deny(rule),
            },
            Some(NetworkAction::Wifi { action }) => match action {
                WifiAction::Scan => self.wifi_scan(),
                WifiAction::Connect { ssid } => self.wifi_connect(ssid),
            },
            None => self.interfaces(OutputFormat::Table),
        }
    }
}
impl NetManager for MacOSNet {
    fn interfaces(&self, _format: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("networksetup").arg("-listallnetworkservices"),
        ))
    }
    fn ips(&self, _format: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("ifconfig")))
    }
    fn routes(&self, _format: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("netstat").arg("-nr")))
    }
    fn fw_status(&self, _format: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
                .arg("--getglobalstate"),
        ))
    }
    fn fw_allow(&self, _rule: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Use socketfilterfw manually")
    }
    fn fw_deny(&self, _rule: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Use socketfilterfw manually")
    }
    fn wifi_scan(&self) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport").arg("-s")))
    }
    fn wifi_connect(&self, ssid: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("networksetup")
                .arg("-setairportnetwork")
                .arg("en0")
                .arg(ssid),
        ))
    }
    fn get_interfaces(&self) -> Result<Vec<NetInterfaceInfo>> {
        let output = std::process::Command::new("ifconfig").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();
        let mut current_iface: Option<NetInterfaceInfo> = None;

        for line in stdout.lines() {
            if line.contains(':') && !line.starts_with('\t') {
                if let Some(iface) = current_iface.take() {
                    interfaces.push(iface);
                }
                let name = line.split(':').next().unwrap_or("").to_string();
                current_iface = Some(NetInterfaceInfo {
                    name,
                    state: "UP".to_string(), // Simplified
                    mtu: 1500,
                    mac: String::new(),
                    interface_type: "Physical".to_string(),
                    ips: Vec::new(),
                });
            } else if let Some(ref mut iface) = current_iface {
                let line = line.trim();
                if line.starts_with("inet ") {
                    let ip = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    iface.ips.push(ip);
                } else if line.starts_with("ether ") {
                    iface.mac = line.split_whitespace().nth(1).unwrap_or("").to_string();
                }
            }
        }
        if let Some(iface) = current_iface {
            interfaces.push(iface);
        }
        Ok(interfaces)
    }
    fn get_fw_rules(&self) -> Result<Vec<FwRuleInfo>> {
        Ok(vec![])
    }
}

pub struct MacOSSys;
impl Domain for MacOSSys {
    fn name(&self) -> &'static str {
        "system"
    }
    fn command(&self) -> Command {
        use crate::cli::SystemArgs;
        use clap::Args;
        SystemArgs::augment_args(Command::new(self.name()).about("Manage MacOS core system"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &Command,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        use crate::cli::{SystemAction, SystemArgs};
        use clap::FromArgMatches;
        let args = SystemArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(SystemAction::Info { format }) => self.info(*format),
            Some(SystemAction::Power { state, now, force }) => self.power(state, *now, *force),
            Some(SystemAction::Time {
                action,
                value,
                format,
            }) => self.time(action, value.as_deref(), *format),
            None => self.info(OutputFormat::Table),
        }
    }
}
impl SysManager for MacOSSys {
    fn info(&self, _f: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("system_profiler")
                .arg("SPSoftwareDataType")
                .arg("SPHardwareDataType"),
        ))
    }
    fn get_info(&self) -> Result<SysInfoData> {
        let hostname = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("kern.hostname")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let os = std::process::Command::new("sw_vers")
            .arg("-productName")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let os_ver = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let kernel = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("kern.osrelease")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let arch = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.machine")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let cpu_count = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.ncpu")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let cpu_model = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let total_memory = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        use super::linux_generic::common::format_bytes;

        Ok(SysInfoData {
            hostname,
            os: format!("{} {}", os, os_ver),
            kernel,
            architecture: arch,
            uptime: "Unknown".to_string(), // Simplified
            cpu_count,
            cpu_model,
            total_memory,
            used_memory: 0,
            total_memory_readable: format_bytes(total_memory),
            used_memory_readable: "0 B".to_string(),
            ram_type: String::new(),
            ram_model: String::new(),
            ram_config: String::new(),
            ram_speed: String::new(),
            physical_drives: 0,
            lan_adapters: vec![],
            wifi_adapters: vec![],
            bt_adapters: vec![],
            monitors: vec![],
            system_users_count: 0,
            common_users_count: 0,
        })
    }
    fn power(
        &self,
        state: &str,
        _now: bool,
        _force: bool,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        match state {
            "reboot" => Ok(Box::new(
                SystemCommand::new("sudo")
                    .arg("shutdown")
                    .arg("-r")
                    .arg("now"),
            )),
            "shutdown" => Ok(Box::new(
                SystemCommand::new("sudo")
                    .arg("shutdown")
                    .arg("-h")
                    .arg("now"),
            )),
            _ => bail!("Unknown power state"),
        }
    }
    fn time(
        &self,
        _a: &str,
        _v: Option<&str>,
        _f: OutputFormat,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("date")))
    }
}

pub struct MacOSVirt;
impl Domain for MacOSVirt {
    fn name(&self) -> &'static str {
        "virtualization"
    }
    fn command(&self) -> Command {
        use crate::cli::VirtualizationArgs;
        use clap::Args;
        VirtualizationArgs::augment_args(
            Command::new(self.name()).about("Manage MacOS virtualization (docker)"),
        )
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{VirtualizationAction, VirtualizationArgs};
        use clap::FromArgMatches;
        let args = VirtualizationArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(VirtualizationAction::List { format }) => self.ls(*format),
            Some(VirtualizationAction::Start { name }) => self.start(name),
            Some(VirtualizationAction::Stop { name }) => self.stop(name),
            Some(VirtualizationAction::Remove { name }) => self.del(name),
            Some(VirtualizationAction::Logs { name }) => self.logs(name),
            None => self.ls(OutputFormat::Table),
        }
    }
}
impl VirtManager for MacOSVirt {
    fn ls(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("docker").arg("ps").arg("-a")))
    }
    fn start(&self, name: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("docker").arg("start").arg(name),
        ))
    }
    fn stop(&self, name: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("docker").arg("stop").arg(name)))
    }
    fn del(&self, name: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("docker").arg("rm").arg(name)))
    }
    fn logs(&self, name: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("docker").arg("logs").arg(name)))
    }
    fn get_containers(&self) -> Result<Vec<ContainerInfo>> {
        let output = std::process::Command::new("docker")
            .arg("ps")
            .arg("-a")
            .arg("--format")
            .arg("{{json .}}")
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();
        for line in stdout.lines() {
            if let Ok(c) = serde_json::from_str::<serde_json::Value>(line) {
                containers.push(ContainerInfo {
                    id: c["ID"].as_str().unwrap_or("").to_string(),
                    image: c["Image"].as_str().unwrap_or("").to_string(),
                    command: c["Command"].as_str().unwrap_or("").to_string(),
                    created: c["CreatedAt"].as_str().unwrap_or("").to_string(),
                    status: c["Status"].as_str().unwrap_or("").to_string(),
                    names: c["Names"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        Ok(containers)
    }
}

pub struct MacOSOverview;
impl Domain for MacOSOverview {
    fn name(&self) -> &'static str {
        "overview"
    }
    fn command(&self) -> Command {
        Command::new(self.name()).about("Show MacOS system overview")
    }
    fn execute(&self, _m: &ArgMatches, _a: &Command) -> Result<Box<dyn super::ExecutableCommand>> {
        self.live_stats(OutputFormat::Table)
    }
}
impl OverviewManager for MacOSOverview {
    fn live_stats(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("top")
                .arg("-l")
                .arg("1")
                .arg("-n")
                .arg("10"),
        ))
    }
    fn get_sensors(&self) -> Result<Vec<SensorInfo>> {
        // MacOS sensors are tricky without 3rd party tools, trying a few common sysctls
        let mut sensors = Vec::new();
        let sysctls = ["machdep.cpu.temperature", "hw.sensors.cpu0.temp"];
        for s in sysctls {
            if let Some(output) = std::process::Command::new("sysctl")
                .arg("-n")
                .arg(s)
                .output()
                .ok()
                .filter(|o| o.status.success())
            {
                let val = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<f32>()
                    .unwrap_or(0.0);
                if val > 0.0 {
                    sensors.push(SensorInfo {
                        label: s.to_string(),
                        temperature: val,
                        critical: Some(100.0),
                        max: Some(90.0),
                    });
                }
            }
        }
        Ok(sensors)
    }
}

pub struct MacOSDisk;
impl Domain for MacOSDisk {
    fn name(&self) -> &'static str {
        "disk"
    }
    fn command(&self) -> Command {
        use crate::cli::DiskArgs;
        use clap::Args;
        DiskArgs::augment_args(Command::new(self.name()).about("Manage MacOS disks (diskutil)"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{DiskAction, DiskArgs};
        use clap::FromArgMatches;
        let args = DiskArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(DiskAction::List {
                format,
                loop_devices: _,
            }) => self.ls(*format, false),
            None => self.ls(OutputFormat::Table, false),
        }
    }
}
impl DiskManager for MacOSDisk {
    fn ls(&self, _f: OutputFormat, _s: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("diskutil").arg("list")))
    }
    fn get_devices(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct MacOSPartition;
impl Domain for MacOSPartition {
    fn name(&self) -> &'static str {
        "partition"
    }
    fn command(&self) -> Command {
        use crate::cli::PartitionArgs;
        use clap::Args;
        PartitionArgs::augment_args(
            Command::new(self.name()).about("Manage MacOS partitions (diskutil)"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &Command,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        use crate::cli::{PartitionAction, PartitionArgs};
        use clap::FromArgMatches;
        let args = PartitionArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(PartitionAction::List { format }) => self.ls(*format),
            Some(PartitionAction::Mount {
                device,
                path,
                fstype,
                options,
            }) => self.mount(device, path, fstype.as_deref(), options.as_deref()),
            Some(PartitionAction::Unmount {
                target,
                lazy,
                force,
            }) => self.unmount(target, *lazy, *force),
            Some(PartitionAction::Usage { path, depth }) => self.usage(path, *depth),
            None => self.ls(OutputFormat::Table),
        }
    }
}
impl PartitionManager for MacOSPartition {
    fn ls(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("df").arg("-h")))
    }
    fn mount(
        &self,
        device: &str,
        path: &str,
        _f: Option<&str>,
        _o: Option<&str>,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("diskutil")
                .arg("mount")
                .arg("-mountPoint")
                .arg(path)
                .arg(device),
        ))
    }
    fn unmount(
        &self,
        target: &str,
        _l: bool,
        _f: bool,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("diskutil").arg("unmount").arg(target),
        ))
    }
    fn usage(&self, path: &str, _d: Option<u32>) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("du").arg("-sh").arg(path)))
    }
    fn get_mount_points(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct MacOSLog;
impl Domain for MacOSLog {
    fn name(&self) -> &'static str {
        "log"
    }
    fn command(&self) -> Command {
        use crate::cli::LogArgs;
        use clap::Args;
        LogArgs::augment_args(Command::new(self.name()).about("Manage MacOS logs (log)"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{LogAction, LogArgs};
        use clap::FromArgMatches;
        let args = LogArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(LogAction::Auth { lines, follow }) => self.auth(*lines, *follow),
            Some(LogAction::Boot { lines, follow, id }) => {
                self.boot(*lines, *follow, id.as_deref())
            }
            Some(LogAction::Crash { lines }) => self.crash(*lines),
            Some(LogAction::Dev { lines, follow }) => self.dev(*lines, *follow),
            Some(LogAction::Error { lines, follow }) => self.error(*lines, *follow),
            Some(LogAction::File {
                path,
                lines,
                follow,
            }) => self.file(path, *lines, *follow),
            Some(LogAction::Package { lines }) => self.pkg(*lines),
            Some(LogAction::Service {
                name,
                lines,
                follow,
            }) => self.svc(name, *lines, *follow),
            Some(LogAction::System { lines, follow }) => self.sys_logs(*lines, *follow),
            None => self.sys_logs(50, false),
        }
    }
}
impl LogManager for MacOSLog {
    fn auth(&self, _l: u32, _f: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("log")
                .arg("show")
                .arg("--predicate")
                .arg("eventMessage contains \"auth\""),
        ))
    }
    fn boot(
        &self,
        _l: u32,
        _f: bool,
        _id: Option<&str>,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("log")
                .arg("show")
                .arg("--predicate")
                .arg("eventMessage contains \"boot\""),
        ))
    }
    fn crash(&self, _l: u32) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("ls")
                .arg("-t")
                .arg("/Library/Logs/DiagnosticReports"),
        ))
    }
    fn dev(&self, _l: u32, _f: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("log")
                .arg("show")
                .arg("--predicate")
                .arg("subsystem contains \"com.apple.CoreBluetooth\""),
        ))
    }
    fn error(&self, _l: u32, _f: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("log")
                .arg("show")
                .arg("--predicate")
                .arg("messageType == error"),
        ))
    }
    fn file(&self, path: &str, _l: u32, _f: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("tail").arg("-n").arg("50").arg(path),
        ))
    }
    fn pkg(&self, _l: u32) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("tail")
                .arg("-n")
                .arg("50")
                .arg("/var/log/install.log"),
        ))
    }
    fn svc(&self, service: &str, _l: u32, _f: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("log")
                .arg("show")
                .arg("--predicate")
                .arg(&format!("process == \"{}\"", service)),
        ))
    }
    fn sys_logs(&self, _l: u32, _f: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("log")
                .arg("show")
                .arg("--last")
                .arg("1h"),
        ))
    }
}

pub struct MacOSGui;
impl Domain for MacOSGui {
    fn name(&self) -> &'static str {
        "gui"
    }
    fn command(&self) -> Command {
        use crate::cli::GuiArgs;
        use clap::Args;
        GuiArgs::augment_args(Command::new(self.name()).about("Manage MacOS GUI"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{GuiAction, GuiArgs, GuiDisplayAction};
        use clap::FromArgMatches;
        let args = GuiArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(GuiAction::Info) => self.info(),
            Some(GuiAction::Display { action }) => match action {
                GuiDisplayAction::List { format } => self.ls_displays(*format),
            },
            None => self.info(),
        }
    }
}
impl GuiManager for MacOSGui {
    fn info(&self) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("system_profiler").arg("SPDisplaysDataType"),
        ))
    }
    fn ls_displays(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        self.info()
    }
}

pub struct MacOSBoot;
impl Domain for MacOSBoot {
    fn name(&self) -> &'static str {
        "boot"
    }
    fn command(&self) -> Command {
        use crate::cli::BootArgs;
        use clap::Args;
        BootArgs::augment_args(Command::new(self.name()).about("Manage MacOS boot"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{BootAction, BootArgs, BootModuleAction};
        use clap::FromArgMatches;
        let args = BootArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(BootAction::List { format }) => self.ls_entries(*format),
            Some(BootAction::Module { action }) => match action {
                BootModuleAction::List { format } => self.ls_modules(*format),
                BootModuleAction::Load { name } => self.load_module(name),
                BootModuleAction::Unload { name } => self.unload_module(name),
            },
            None => self.ls_entries(OutputFormat::Table),
        }
    }
}
impl BootManager for MacOSBoot {
    fn ls_entries(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("nvram").arg("-p")))
    }
    fn ls_modules(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("kextstat")))
    }
    fn load_module(&self, name: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("sudo").arg("kextload").arg(name),
        ))
    }
    fn unload_module(&self, name: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("sudo").arg("kextunload").arg(name),
        ))
    }
}

pub struct MacOSDistro;
impl Domain for MacOSDistro {
    fn name(&self) -> &'static str {
        "distribution"
    }
    fn command(&self) -> Command {
        use crate::cli::DistributionArgs;
        use clap::Args;
        DistributionArgs::augment_args(Command::new(self.name()).about("Manage MacOS version"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{DistributionAction, DistributionArgs};
        use clap::FromArgMatches;
        let args = DistributionArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(DistributionAction::Info { format }) => self.info(*format),
            Some(DistributionAction::Upgrade) => self.upgrade(),
            None => self.info(OutputFormat::Table),
        }
    }
}
impl DistroManager for MacOSDistro {
    fn info(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("sw_vers")))
    }
    fn upgrade(&self) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("softwareupdate").arg("-l")))
    }
}

pub struct MacOSSec;
impl Domain for MacOSSec {
    fn name(&self) -> &'static str {
        "security"
    }
    fn command(&self) -> Command {
        use crate::cli::SecurityArgs;
        use clap::Args;
        SecurityArgs::augment_args(Command::new(self.name()).about("Manage MacOS security"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{SecurityAction, SecurityArgs};
        use clap::FromArgMatches;
        let args = SecurityArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(SecurityAction::Audit { format }) => self.audit(*format),
            Some(SecurityAction::Context) => self.context(),
            None => self.audit(OutputFormat::Table),
        }
    }
}
impl SecManager for MacOSSec {
    fn audit(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("spctl").arg("--status")))
    }
    fn context(&self) -> Result<Box<dyn super::ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("id")))
    }
}

pub struct MacOSSelf;
impl Domain for MacOSSelf {
    fn name(&self) -> &'static str {
        "self"
    }
    fn command(&self) -> Command {
        use crate::cli::SelfArgs;
        use clap::Args;
        SelfArgs::augment_args(Command::new(self.name()).about("Manage ao on MacOS"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &Command) -> Result<Box<dyn ExecutableCommand>> {
        use crate::cli::{CompletionsAction, SelfAction, SelfArgs};
        use clap::FromArgMatches;
        let args = SelfArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(SelfAction::Info { format }) => self.info(*format),
            Some(SelfAction::Update) => self.update(),
            Some(SelfAction::Completions { action }) => {
                match action {
                    CompletionsAction::Setup { shell } => {
                        let exe = std::env::current_exe()?.to_string_lossy().to_string();
                        self.install_completions(*shell, &exe)?;
                    }
                    _ => bail!("Use ao self completions setup <shell>"),
                }
                Ok(Box::new(super::linux_generic::NoopCommand))
            }
            None => self.info(OutputFormat::Table),
        }
    }
}
impl SelfManager for MacOSSelf {
    fn info(&self, _f: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("echo").arg(&format!("ao v{}", env!("CARGO_PKG_VERSION"))),
        ))
    }
    fn update(&self) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Use brew upgrade ao-cli")
    }
    fn install_completions(&self, _shell: clap_complete::Shell, _exe: &str) -> Result<()> {
        Ok(())
    }
}
