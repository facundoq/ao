use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ao", version = "0.1.0", about = "Admin Operations - Grand Unified Wrapper", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage packages (install, remove, update)
    Pkg {
        #[command(subcommand)]
        action: PkgAction,
    },
    /// Manage services (start, stop, restart)
    Svc {
        #[command(subcommand)]
        action: SvcAction,
    },
    /// Manage users
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    /// Manage groups
    Group {
        #[command(subcommand)]
        action: GroupAction,
    },
    /// Manage disks and storage
    Disk {
        #[command(subcommand)]
        action: DiskAction,
    },
}

#[derive(Subcommand)]
pub enum DiskAction {
    /// Lists all block devices and usage
    List,
    /// Mounts a block device to a directory
    Mount {
        #[arg(required = true)]
        device: String,
        #[arg(required = true)]
        path: String,
        #[arg(long, short)]
        fstype: Option<String>,
        #[arg(long, short)]
        options: Option<String>,
    },
    /// Safely unmounts a device
    Unmount {
        #[arg(required = true)]
        target: String,
        #[arg(long, short)]
        lazy: bool,
        #[arg(long, short)]
        force: bool,
    },
    /// Calculates directory size
    Usage {
        #[arg(required = true)]
        path: String,
        #[arg(long)]
        depth: Option<u32>,
    },
}

#[derive(Subcommand)]
pub enum GroupAction {
    /// Lists all groups
    List,
    /// Creates a new group
    Add {
        #[arg(required = true)]
        groupname: String,
    },
    /// Deletes a group
    Del {
        #[arg(required = true)]
        groupname: String,
    },
    /// Modifies a group
    Mod {
        #[arg(required = true)]
        groupname: String,
        #[arg(long)]
        gid: u32,
    },
}

#[derive(Subcommand)]
pub enum UserAction {
    /// Lists users
    List {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        groups: bool,
    },
    /// Creates a new user
    Add {
        #[arg(required = true)]
        username: String,
        #[arg(long)]
        groups: Option<String>,
        #[arg(long)]
        shell: Option<String>,
        #[arg(long)]
        system: bool,
    },
    /// Deletes a user
    Del {
        #[arg(required = true)]
        username: String,
        #[arg(long, short)]
        purge: bool,
    },
    /// Modifies a user
    Mod {
        #[arg(required = true)]
        username: String,
        #[arg(required = true)]
        action: String,
        #[arg(required = true)]
        value: String,
    },
    /// Changes a user's password interactively
    Passwd {
        #[arg(required = true)]
        username: String,
    },
}

#[derive(Subcommand)]
pub enum PkgAction {
    /// Update the system package tree and applies available upgrades.
    Update,
    /// Installs one or more packages.
    Install {
        /// Packages to install
        #[arg(required = true)]
        packages: Vec<String>,
    },
    /// Uninstalls packages.
    Remove {
        /// Packages to remove
        #[arg(required = true)]
        packages: Vec<String>,
        /// Completely remove configuration files alongside the binary.
        #[arg(long, short)]
        purge: bool,
    },
    /// Searches the upstream package repositories.
    Search {
        /// The query to search for
        #[arg(required = true)]
        query: String,
    },
    /// Lists all explicitly installed user packages.
    List,
}

#[derive(Subcommand)]
pub enum SvcAction {
    /// Lists all active and failed services on the system.
    List,
    /// Starts and enables a service to start on boot.
    Up {
        /// The service name
        #[arg(required = true)]
        name: String,
    },
    /// Stops and disables a service from starting on boot.
    Down {
        /// The service name
        #[arg(required = true)]
        name: String,
    },
    /// Restarts the specified service.
    Restart {
        /// The service name
        #[arg(required = true)]
        name: String,
    },
    /// Reloads the service configuration without fully stopping it.
    Reload {
        /// The service name
        #[arg(required = true)]
        name: String,
    },
    /// Displays detailed status for the service.
    Status {
        /// The service name
        #[arg(required = true)]
        name: String,
    },
}
