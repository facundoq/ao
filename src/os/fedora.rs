use super::{ExecutableCommand, PackageManager, Domain, OutputFormat};
use super::linux_generic::{SystemCommand, is_completing_arg};
use anyhow::Result;
use std::process::Command;
use clap::{Command as ClapCommand, ArgMatches, FromArgMatches, Args};
use crate::cli::{PkgArgs, PkgAction};

pub struct Dnf;

impl Domain for Dnf {
    fn name(&self) -> &'static str { "pkg" }
    fn command(&self) -> ClapCommand {
        PkgArgs::augment_args(ClapCommand::new("pkg").about("Manage packages (DNF)"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = PkgArgs::from_arg_matches(matches)?;
        match &args.action {
            PkgAction::Update => self.update(),
            PkgAction::Install { packages } => self.install(packages),
            PkgAction::Remove { packages, purge } => self.remove(packages, *purge),
            PkgAction::Search { query } => self.search(query),
            PkgAction::List { format } => self.list(*format),
        }
    }
    fn complete(&self, _line: &str, words: &[&str], last_word_complete: bool) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "pkg", "install"], 1, last_word_complete) {
            return self.get_available_packages();
        }
        if is_completing_arg(words, &["ao", "pkg", "remove"], 1, last_word_complete) {
            return self.get_installed_packages();
        }
        Ok(vec![])
    }
}

impl PackageManager for Dnf {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("check-update")))
    }

    fn install(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("install").arg("-y").arg("--").args(packages)))
    }

    fn remove(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("remove").arg("-y").arg("--").args(packages)))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("search").arg("--").arg(query)))
    }

    fn list(&self, _format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("list").arg("--installed")))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("rpm").arg("-qa").arg("--queryformat").arg("%{NAME}\n").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("dnf").arg("repoquery").arg("--qf").arg("%{name}").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}
