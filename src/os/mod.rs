pub use crate::cli::OutputFormat;
use anyhow::Result;
use clap::{ArgMatches, Command};
use serde::{Deserialize, Serialize};

pub mod arch;
pub mod debian;
pub mod detector;
pub mod fedora;
pub mod linux_generic;

/// Unified trait for a system domain (e.g., packages, services).
/// It defines both the CLI interface and the execution logic.
pub trait Domain {
    /// The name of the subcommand (e.g., "pkg", "user")
    fn name(&self) -> &'static str;

    /// Build the clap Command for this domain
    fn command(&self) -> Command;

    /// Execute the command based on the parsed matches.
    /// Takes the full app Command tree in case it's needed (e.g. for completions)
    fn execute(&self, matches: &ArgMatches, app: &Command) -> Result<Box<dyn ExecutableCommand>>;

    /// Provide dynamic completion suggestions
    fn complete(
        &self,
        line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        let _ = (line, words, last_word_complete);
        Ok(vec![])
    }
}

/// Represents a command that can be executed, printed, or dry-run.
pub trait ExecutableCommand {
    fn execute(&self) -> Result<()>;
    fn dry_run(&self) -> Result<()>;
    fn print(&self) -> Result<()>;
    fn as_string(&self) -> String;
    /// Returns true if this command outputs structured or raw original data (should suppress ao decorations)
    fn is_structured(&self) -> bool {
        false
    }
}

/// Abstracts shell completion management.
pub trait CompletionsManager: Domain {
    fn install(&self, shell: clap_complete::Shell, exe_path: &str) -> Result<()>;
}

#[derive(Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub status: String,
}

/// Abstracts system package management operations.
pub trait PackageManager: Domain {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>>;
    fn del(&self, packages: &[String], purge: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn get_installed_packages(&self) -> Result<Vec<String>>;
    fn get_available_packages(&self) -> Result<Vec<String>>;
}

#[derive(Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub loaded: String,
    pub active: String,
    pub status: String,
    pub description: String,
}

/// Abstracts system service management operations.
pub trait ServiceManager: Domain {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn up(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn down(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn restart(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn reload(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn status(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn get_services(&self) -> Result<Vec<String>>;
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub uid: String,
    pub gid: String,
    pub home: String,
    pub shell: String,
    pub groups: Vec<String>,
    #[serde(rename = "type")]
    pub user_type: String,
}

/// Abstracts system user management operations.
pub trait UserManager: Domain {
    fn ls(
        &self,
        all: bool,
        groups: bool,
        format: OutputFormat,
    ) -> Result<Box<dyn ExecutableCommand>>;
    #[allow(clippy::too_many_arguments)]
    fn add(
        &self,
        username: &str,
        name: Option<&str>,
        email: Option<&str>,
        groups: Option<&str>,
        shell: Option<&str>,
        system: bool,
        no_create_home: bool,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn del(&self, username: &str, purge: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn mod_user(
        &self,
        username: &str,
        action: &str,
        value: &str,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn passwd(&self, username: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn get_users(&self) -> Result<Vec<String>>;
    fn get_shells(&self) -> Result<Vec<String>>;
}

#[derive(Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub gid: String,
    pub members: Vec<String>,
}

/// Abstracts system group management operations.
pub trait GroupManager: Domain {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn add(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn del(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn mod_group(&self, groupname: &str, gid: u32) -> Result<Box<dyn ExecutableCommand>>;
    fn get_groups(&self) -> Result<Vec<String>>;
}

#[derive(Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    pub size: String,
    pub mountpoint: Option<String>,
    pub fstype: Option<String>,
    #[serde(rename = "type")]
    pub device_type: String,
    pub rota: bool,
    pub tran: Option<String>,
    pub children: Option<Vec<DiskInfo>>,
}

/// Abstracts system disk management operations.
pub trait DiskManager: Domain {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn mount(
        &self,
        device: &str,
        path: &str,
        fstype: Option<&str>,
        options: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn usage(&self, path: &str, depth: Option<u32>) -> Result<Box<dyn ExecutableCommand>>;
    fn get_devices(&self) -> Result<Vec<String>>;
    fn get_mount_points(&self) -> Result<Vec<String>>;
}

#[derive(Serialize, Deserialize)]
pub struct SysInfoData {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub architecture: String,
    pub uptime: String,
    pub cpu_count: usize,
    pub cpu_model: String,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_memory_readable: String,
    pub used_memory_readable: String,
    pub ram_type: String,
    pub ram_model: String,
    pub physical_drives: usize,
    pub lan_adapters: Vec<String>,
    pub wifi_adapters: Vec<String>,
    pub bt_adapters: Vec<String>,
    pub monitors: Vec<String>,
    pub system_users_count: usize,
    pub common_users_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SysTimeData {
    pub local_time: String,
    pub universal_time: String,
    pub rtc_time: String,
    pub time_zone: String,
    pub system_clock_synchronized: String,
    pub ntp_service: String,
    pub rtc_in_local_tz: String,
}

/// Abstracts system management operations.
pub trait SysManager: Domain {
    fn info(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn power(&self, state: &str, now: bool, force: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn time(
        &self,
        action: &str,
        value: Option<&str>,
        format: OutputFormat,
    ) -> Result<Box<dyn ExecutableCommand>>;
}

/// Abstracts system log operations.
pub trait LogManager: Domain {
    fn auth(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn boot(
        &self,
        lines: u32,
        follow: bool,
        id: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn crash(&self, lines: u32) -> Result<Box<dyn ExecutableCommand>>;
    fn dev(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn error(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn file(&self, path: &str, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn pkg(&self, lines: u32) -> Result<Box<dyn ExecutableCommand>>;
    fn svc(&self, service: &str, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn sys_logs(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct DistroInfo {
    pub name: String,
    pub version: String,
    pub id: String,
    pub id_like: String,
    pub pretty_name: String,
}

/// Abstracts distribution management.
pub trait DistroManager: Domain {
    fn info(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn upgrade(&self) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct NetInterfaceInfo {
    pub name: String,
    pub state: String,
    pub mtu: u32,
    pub mac: String,
    #[serde(rename = "type")]
    pub interface_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetIpInfo {
    pub interface: String,
    pub family: String,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetRouteInfo {
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: String,
}

#[derive(Serialize, Deserialize)]
pub struct FwRuleInfo {
    pub to: String,
    pub action: String,
    pub from: String,
}

/// Abstracts networking.
pub trait NetManager: Domain {
    fn interfaces(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn ips(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn routes(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn fw_status(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn fw_allow(&self, rule: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn fw_deny(&self, rule: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn wifi_scan(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn wifi_connect(&self, ssid: &str) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct BootEntryInfo {
    pub title: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct KernelModInfo {
    pub name: String,
    pub size: u64,
    pub used_by: String,
}

/// Abstracts boot management.
pub trait BootManager: Domain {
    fn ls_entries(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn ls_modules(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn load_module(&self, name: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn unload_module(&self, name: &str) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct DisplayInfo {
    pub name: String,
    pub connected: bool,
    pub resolution: String,
}

/// Abstracts GUI management.
pub trait GuiManager: Domain {
    fn info(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn ls_displays(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct DeviceInfo {
    pub bus: String,
    pub device: String,
    pub id: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct PrinterInfo {
    pub name: String,
    pub status: String,
}

/// Abstracts device management.
pub trait DevManager: Domain {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn pci(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn usb(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn bt_status(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn bt_scan(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn bt_pair(&self, address: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn bt_connect(&self, address: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn ls_printers(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub image: String,
    pub command: String,
    pub created: String,
    pub status: String,
    pub names: String,
}

/// Abstracts virtualization.
pub trait VirtManager: Domain {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn start(&self, name: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn stop(&self, name: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn del(&self, name: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn logs(&self, name: &str) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct SecAuditInfo {
    pub title: String,
    pub result: String,
    pub recommendation: String,
}

/// Abstracts security.
pub trait SecManager: Domain {
    fn audit(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn context(&self) -> Result<Box<dyn ExecutableCommand>>;
}

#[derive(Serialize, Deserialize)]
pub struct SelfInfo {
    pub version: String,
    pub architecture: String,
    pub os: String,
}

/// Abstracts ao self management.
pub trait SelfManager: Domain {
    fn info(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
    fn update(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn install_completions(&self, shell: clap_complete::Shell, exe_path: &str) -> Result<()>;
}

#[derive(Serialize, Deserialize)]
pub struct MonitorEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub subtype: String,
    pub value: String,
    pub description: String,
}

/// Abstracts system monitoring operations.
pub trait MonitorManager: Domain {
    fn live_stats(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>>;
}
