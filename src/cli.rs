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

#[derive(Parser)]
#[command(
    name = "ao",
    version = env!("CARGO_PKG_VERSION"),
    about = "Admin Operation",
    long_about = "A unified administration tool for Linux systems, providing a consistent interface for managing packages, services, users, networking, and more across various distributions."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Print the underlying command without running it
    #[arg(global = true, long, hide = true)]
    pub print: bool,

    /// Print the command and simulate execution (no system changes)
    #[arg(global = true, long, hide = true)]
    pub dry_run: bool,

    /// Dump the entire command tree as ASCII
    #[arg(global = true, long, hide = true)]
    pub dump_tree: bool,
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
pub struct PackageArgs {
    #[command(subcommand)]
    pub action: Option<PackageAction>,
}

#[derive(Subcommand)]
pub enum PackageAction {
    /// Adds one or more packages.
    Add {
        /// Packages to add
        #[arg(required = true, value_hint = ValueHint::Other)]
        packages: Vec<String>,
    },
    /// Lists all explicitly installed user packages.
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Deletes packages.
    Delete {
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
pub struct ServiceArgs {
    #[command(subcommand)]
    pub action: Option<ServiceAction>,
}

#[derive(Subcommand)]
pub enum ServiceAction {
    /// Stops and disables a service from starting on boot.
    Down {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
    },
    /// Lists all active and failed services on the system.
    List {
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
    Delete {
        #[arg(required = true, value_hint = ValueHint::Other)]
        username: String,
        #[arg(long, short)]
        purge: bool,
    },
    /// Lists users
    List {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        groups: bool,
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Modifies a user
    Modify {
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
    /// Shows login/logout sessions
    Session {
        /// Target username (optional, defaults to current user)
        #[arg(value_hint = ValueHint::Other)]
        username: Option<String>,
        /// Show sessions for ALL users
        #[arg(long, short)]
        all: bool,
        /// Limit the number of entries to show
        #[arg(long, short)]
        n: Option<u32>,
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
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
    Delete {
        #[arg(required = true, value_hint = ValueHint::Other)]
        groupname: String,
    },
    /// Lists all groups
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Modifies a group
    Modify {
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
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
        /// Show loop devices
        #[arg(long = "loop", short = 'l')]
        loop_devices: bool,
    },
}

#[derive(Args)]
pub struct PartitionArgs {
    #[command(subcommand)]
    pub action: Option<PartitionAction>,
}

#[derive(Subcommand)]
pub enum PartitionAction {
    /// Lists all partitions
    List {
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
pub struct SystemArgs {
    #[command(subcommand)]
    pub action: Option<SystemAction>,
}

#[derive(Subcommand)]
pub enum SystemAction {
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
    /// Shows authentication and security logs (logins, sudo, etc).
    Auth {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
    },
    /// Shows logs from the current or a specific boot.
    Boot {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
        /// Specific boot ID or relative offset (e.g. 0 for current, -1 for previous)
        #[arg(long)]
        id: Option<String>,
    },
    /// Shows critical system crashes, kernel panics and core dumps.
    Crash {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
    },
    /// Shows hardware and device driver (kernel) logs.
    Dev {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
    },
    /// Filters for high-priority errors and system failures.
    Error {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
    },
    /// Tails a specific log file from disk.
    File {
        /// Path to the log file
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        path: String,
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
    },
    /// Shows package manager history and update logs.
    Package {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
    },
    /// Shows logs for a specific system service.
    Service {
        /// The service name
        #[arg(required = true, value_hint = ValueHint::Other)]
        name: String,
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
    },
    /// Shows general system-wide logs.
    System {
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: u32,
        /// Follow the live log stream
        #[arg(long, short)]
        follow: bool,
    },
}

#[derive(Args)]
pub struct DistributionArgs {
    #[command(subcommand)]
    pub action: Option<DistributionAction>,
}

#[derive(Subcommand)]
pub enum DistributionAction {
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
pub struct NetworkArgs {
    #[command(subcommand)]
    pub action: Option<NetworkAction>,
}

#[derive(Subcommand)]
pub enum NetworkAction {
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
    Firewall {
        #[command(subcommand)]
        action: FirewallAction,
    },
    /// Wi-Fi management
    Wifi {
        #[command(subcommand)]
        action: WifiAction,
    },
}

#[derive(Subcommand)]
pub enum FirewallAction {
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
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Kernel module management
    Module {
        #[command(subcommand)]
        action: BootModuleAction,
    },
}

#[derive(Subcommand)]
pub enum BootModuleAction {
    /// Lists loaded kernel modules
    List {
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
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Args)]
pub struct DeviceArgs {
    #[command(subcommand)]
    pub action: Option<DeviceAction>,
}

#[derive(Subcommand)]
pub enum DeviceAction {
    /// Summarizes connected PCI and USB devices
    List {
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
    Bluetooth {
        #[command(subcommand)]
        action: BluetoothAction,
    },
    /// Printer management
    Print {
        #[command(subcommand)]
        action: PrintAction,
    },
}

#[derive(Subcommand)]
pub enum BluetoothAction {
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
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Args)]
pub struct VirtualizationArgs {
    #[command(subcommand)]
    pub action: Option<VirtualizationAction>,
}

#[derive(Subcommand)]
pub enum VirtualizationAction {
    /// Lists all running containers and active VMs
    List {
        /// The output format
        #[arg(long, short, default_value = "table")]
        format: OutputFormat,
    },
    /// Starts a stopped container or VM
    Start { name: String },
    /// Stops a running container or VM
    Stop { name: String },
    /// Removes a container or VM
    Remove { name: String },
    /// Tails the logs of a running container
    Logs { name: String },
}

#[derive(Args)]
pub struct SecurityArgs {
    #[command(subcommand)]
    pub action: Option<SecurityAction>,
}

#[derive(Subcommand)]
pub enum SecurityAction {
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
