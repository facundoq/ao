use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, PkgAction, SvcAction, UserAction};

pub mod cli;
pub mod os;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Abstract the OS detection away from the specific logic
    let system = os::detector::detect_system()?;

    match cli.command {
        Commands::Pkg { action } => match action {
            PkgAction::Update => {
                system.pkg.update()?;
            }
            PkgAction::Install { packages } => {
                system.pkg.install(&packages)?;
            }
            PkgAction::Remove { packages, purge } => {
                system.pkg.remove(&packages, purge)?;
            }
            PkgAction::Search { query } => {
                system.pkg.search(&query)?;
            }
            PkgAction::List => {
                system.pkg.list()?;
            }
        },
        Commands::Svc { action } => match action {
            SvcAction::List => {
                system.svc.list()?;
            }
            SvcAction::Up { name } => {
                system.svc.up(&name)?;
            }
            SvcAction::Down { name } => {
                system.svc.down(&name)?;
            }
            SvcAction::Restart { name } => {
                system.svc.restart(&name)?;
            }
            SvcAction::Reload { name } => {
                system.svc.reload(&name)?;
            }
            SvcAction::Status { name } => {
                system.svc.status(&name)?;
            }
        },
        Commands::User { action } => match action {
            UserAction::List { all, groups } => {
                system.user.list(all, groups)?;
            }
            UserAction::Add { username, groups, shell, system: sys } => {
                system.user.add(&username, groups.as_deref(), shell.as_deref(), sys)?;
            }
            UserAction::Del { username, purge } => {
                system.user.del(&username, purge)?;
            }
            UserAction::Mod { username, action, value } => {
                system.user.mod_user(&username, &action, &value)?;
            }
            UserAction::Passwd { username } => {
                system.user.passwd(&username)?;
            }
        },
    }

    Ok(())
}
