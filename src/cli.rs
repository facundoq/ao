use crate::os::ExecutableCommand;
use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueHint};
use clap_complete::Shell;

#[derive(clap::ValueEnum, Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Original,
}

impl OutputFormat {
    pub fn print_structured<T: serde::Serialize>(&self, data: &T) -> anyhow::Result<()> {
        match self {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(data)?);
            }
            _ => unreachable!("structured data shouldn't be printed with this format"),
        }
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "ao", version = "0.1.1", about = "Admin Operation", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Print the underlying command without running it
    #[arg(global = true, long, hide = true)]
    pub print: bool,

    /// Print the command and simulate execution (no system changes)
    #[arg(global = true, long, hide = true)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Starts an interactive session to browse and execute commands
    Interactive,
}

#[derive(Args)]
pub struct MonitorArgs {
    /// The output format
    #[arg(long, short, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Args)]
pub struct PkgArgs {
    #[command(subcommand)]
    pub action: Option<PkgAction>,
}

#[derive(Subcommand)]
pub enum PkgAction {
    /// Adds one or more packages.
    Add {
        /// Packages to add
        #[arg(required = true, value_hint = ValueHint::Other)]
        packages: Vec<String>,
    },
    /// Lists all explicitly installed user packages.
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Deletes packages.
    Del {
        /// Packages to delete
        #[arg(required = true, value_hint = ValueHint::Other)]
        packages: Vec<String>,
        /// Completely remove configuration files alongside the binary.
        #[arg(long, short)]
        purge: bool,
    },
    /// Searches the upstream package repositories.
    Search {
        /// The query to search for
        #[arg(required = true, value_hint = ValueHint::Other)]
        query: String,
    },
    /// Update the system package tree and applies available upgrades.
    Update,
}

#[derive(Args)]
pub struct SvcArgs {
    #[command(subcommand)]
    pub action: Option<SvcAction>,
}

#[derive(Subcommand)]
pub enum SvcAction {
    /// Stops and disables a service from starting on boot.
    Down {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
    },
    /// Lists all active and failed services on the system.
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Reloads the service configuration without fully stopping it.
    Reload {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
    },
    /// Restarts the specified service.
    Restart {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
    },
    /// Displays detailed status for the service.
    Status {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
    },
    /// Starts and enables a service to start on boot.
    Up {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
    },
}

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub action: Option<UserAction>,
}

#[derive(Subcommand)]
pub enum UserAction {
    /// Creates a new user
    Add {
        /// The unique system username
        #[arg(required = true, value_hint = ValueHint::Other)]
        username: String,
        /// The user's full name
        #[arg(long, value_hint = ValueHint::Other)]
        name: Option<String>,
        /// The user's email address
        #[arg(long, value_hint = ValueHint::Other)]
        email: Option<String>,
        /// Comma-separated list of additional groups
        #[arg(long, value_hint = ValueHint::Other)]
        groups: Option<String>,
        /// The login shell for the new user
        #[arg(long, value_hint = ValueHint::Other)]
        shell: Option<String>,
        /// Create a system account
        #[arg(long)]
        system: bool,
        /// Do not create the home directory
        #[arg(long)]
        no_create_home: bool,
    },
    /// Deletes a user
    Del {
        #[arg(required = true, value_hint = ValueHint::Other)]
        username: String,
        #[arg(long, short)]
        purge: bool,
    },
    /// Lists users
    Ls {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        groups: bool,
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Modifies a user
    Mod {
        #[arg(required = true, value_hint = ValueHint::Other)]
        username: String,
        #[arg(required = true)]
        action: String,
        #[arg(required = true)]
        value: String,
    },
    /// Changes a user's password interactively
    Passwd {
        #[arg(required = true, value_hint = ValueHint::Other)]
        username: String,
    },
}

#[derive(Args)]
pub struct GroupArgs {
    #[command(subcommand)]
    pub action: Option<GroupAction>,
}

#[derive(Subcommand)]
pub enum GroupAction {
    /// Creates a new group
    Add {
        #[arg(required = true, value_hint = ValueHint::Other)]
        groupname: String,
    },
    /// Deletes a group
    Del {
        #[arg(required = true, value_hint = ValueHint::Other)]
        groupname: String,
    },
    /// Lists all groups
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Modifies a group
    Mod {
        #[arg(required = true, value_hint = ValueHint::Other)]
        groupname: String,
        #[arg(long)]
        gid: u32,
    },
}

#[derive(Args)]
pub struct DiskArgs {
    #[command(subcommand)]
    pub action: Option<DiskAction>,
}

impl DiskArgs {
    pub fn run(
        &self,
        _system: &crate::os::detector::DetectedSystem,
    ) -> Result<Box<dyn ExecutableCommand>> {
        anyhow::bail!("DiskArgs::run is no longer used in the unified Domain architecture")
    }
}

#[derive(Subcommand)]
pub enum DiskAction {
    /// Lists all block devices and usage
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Mounts a block device to a directory
    Mount {
        #[arg(required = true, value_hint = ValueHint::Other)]
        device: String,
        #[arg(required = true, value_hint = ValueHint::DirPath)]
        path: String,
        #[arg(long, short)]
        fstype: Option<String>,
        #[arg(long, short)]
        options: Option<String>,
    },
    /// Safely unmounts a device
    Unmount {
        #[arg(required = true, value_hint = ValueHint::Other)]
        target: String,
        #[arg(long, short)]
        lazy: bool,
        #[arg(long, short)]
        force: bool,
    },
    /// Calculates directory size
    Usage {
        #[arg(required = true, value_hint = ValueHint::DirPath)]
        path: String,
        #[arg(long)]
        depth: Option<u32>,
    },
}

#[derive(Args)]
pub struct SysArgs {
    #[command(subcommand)]
    pub action: Option<SysAction>,
}

#[derive(Subcommand)]
pub enum SysAction {
    /// Retrieves OS info, kernel version, uptime, etc.
    Info {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Initiates system power state transitions.
    Power {
        /// The power state transition (reboot, shutdown, suspend, hibernate)
        #[arg(required = true)]
        state: String,
        /// Execute immediately
        #[arg(long)]
        now: bool,
        /// Bypass normal init procedures
        #[arg(long)]
        force: bool,
    },
    /// Modifies or views the system time and timezone.
    Time {
        /// The time action (status, set, sync)
        #[arg(required = true)]
        action: String,
        /// The value to set (e.g. timezone)
        value: Option<String>,
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Args)]
pub struct LogArgs {
    #[command(subcommand)]
    pub action: Option<LogAction>,
}

#[derive(Subcommand)]
pub enum LogAction {
    /// Tails a specific log file from disk.
    File {
        /// Path to the log file
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        path: String,
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
    },
    /// Tails the system-wide kernel and boot logs.
    Sys {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
    },
    /// Tails the live logs of a specific service.
    Tail {
        /// The service name
        #[arg(required = true)]
        name: String,
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
    },
}

#[derive(Args)]
pub struct DistroArgs {
    #[command(subcommand)]
    pub action: Option<DistroAction>,
}

#[derive(Subcommand)]
pub enum DistroAction {
    /// Shows detailed distribution metadata.
    Info {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Upgrades the entire distribution to the next major release.
    Upgrade,
}

#[derive(Args)]
pub struct NetArgs {
    #[command(subcommand)]
    pub action: Option<NetAction>,
}

#[derive(Subcommand)]
pub enum NetAction {
    /// Lists all network interfaces
    Interfaces {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Shows assigned IP addresses
    Ips {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Shows routing table
    Routes {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Firewall management
    Fw {
        #[command(subcommand)]
        action: FwAction,
    },
    /// Wi-Fi management
    Wifi {
        #[command(subcommand)]
        action: WifiAction,
    },
}

#[derive(Subcommand)]
pub enum FwAction {
    /// Shows firewall status
    Status {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Allows traffic on a port/service
    Allow { rule: String },
    /// Denies traffic on a port/service
    Deny { rule: String },
}

#[derive(Subcommand)]
pub enum WifiAction {
    /// Scans for available Wi-Fi networks
    Scan,
    /// Connects to a Wi-Fi network
    Connect { ssid: String },
}

#[derive(Args)]
pub struct BootArgs {
    #[command(subcommand)]
    pub action: Option<BootAction>,
}

#[derive(Subcommand)]
pub enum BootAction {
    /// Lists boot entries
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Kernel module management
    Mod {
        #[command(subcommand)]
        action: BootModAction,
    },
}

#[derive(Subcommand)]
pub enum BootModAction {
    /// Lists loaded kernel modules
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Loads a kernel module
    Load { name: String },
    /// Unloads a kernel module
    Unload { name: String },
}

#[derive(Args)]
pub struct GuiArgs {
    #[command(subcommand)]
    pub action: Option<GuiAction>,
}

#[derive(Subcommand)]
pub enum GuiAction {
    /// Displays GUI session info (Wayland/X11)
    Info,
    /// Display and monitor management
    Display {
        #[command(subcommand)]
        action: GuiDisplayAction,
    },
}

#[derive(Subcommand)]
pub enum GuiDisplayAction {
    /// Lists connected displays and resolutions
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Args)]
pub struct DevArgs {
    #[command(subcommand)]
    pub action: Option<DevAction>,
}

#[derive(Subcommand)]
pub enum DevAction {
    /// Summarizes connected PCI and USB devices
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Lists detailed PCI devices
    Pci {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Lists detailed USB devices
    Usb {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Bluetooth management
    Bt {
        #[command(subcommand)]
        action: BtAction,
    },
    /// Printer management
    Print {
        #[command(subcommand)]
        action: PrintAction,
    },
}

#[derive(Subcommand)]
pub enum BtAction {
    /// Checks bluetooth status
    Status,
    /// Scans for nearby devices
    Scan,
    /// Pairs with a device
    Pair { address: String },
    /// Connects to a paired device
    Connect { address: String },
}

#[derive(Subcommand)]
pub enum PrintAction {
    /// Lists configured printers
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Args)]
pub struct VirtArgs {
    #[command(subcommand)]
    pub action: Option<VirtAction>,
}

#[derive(Subcommand)]
pub enum VirtAction {
    /// Lists all running containers and active VMs
    Ls {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Starts a stopped container or VM
    Start { name: String },
    /// Stops a running container or VM
    Stop { name: String },
    /// Removes a container or VM
    Rm { name: String },
    /// Tails the logs of a running container
    Logs { name: String },
}

#[derive(Args)]
pub struct SecArgs {
    #[command(subcommand)]
    pub action: Option<SecAction>,
}

#[derive(Subcommand)]
pub enum SecAction {
    /// Runs a basic security audit
    Audit {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Outputs the current state of SELinux or AppArmor
    Context,
}

#[derive(Args)]
pub struct SelfArgs {
    #[command(subcommand)]
    pub action: Option<SelfAction>,
}

#[derive(Subcommand)]
pub enum SelfAction {
    /// Shell completion management
    Completions {
        #[command(subcommand)]
        action: CompletionsAction,
    },
    /// Displays information about ao itself
    Info {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Updates ao to the latest version
    Update,
}

#[derive(Subcommand)]
pub enum CompletionsAction {
    /// Generate shell completions to stdout
    Generate {
        /// The shell to generate completions for
        shell: Shell,
    },
    /// Install shell completions into your shell's configuration file
    Install {
        /// The shell to install completions for
        shell: Shell,
    },
    /// Print the command to source completions in the current session
    Setup {
        /// The shell to setup completions for
        shell: Shell,
    },
}
