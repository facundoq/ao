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
