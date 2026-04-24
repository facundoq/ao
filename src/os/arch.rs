use super::linux_generic::{SystemCommand, is_completing_arg};
use super::{Domain, ExecutableCommand, OutputFormat, PackageManager};
use crate::cli::{PackageAction, PackageArgs};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct Pacman;

impl Domain for Pacman {
    fn name(&self) -> &'static str {
        "package"
    }
    fn command(&self) -> ClapCommand {
        PackageArgs::augment_args(ClapCommand::new("package").about("Manage packages (Pacman)"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = PackageArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(PackageAction::Update) => self.update(),
            Some(PackageAction::Add { packages }) => self.add(packages),
            Some(PackageAction::Del { packages, purge }) => self.del(packages, *purge),
            Some(PackageAction::Search { query }) => self.search(query),
            Some(PackageAction::Ls { format }) => self.ls(*format),
            None => self.ls(OutputFormat::Table),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "package", "add"], 1, last_word_complete) {
            return self.get_available_packages();
        }
        if is_completing_arg(words, &["ao", "package", "del"], 1, last_word_complete) {
            return self.get_installed_packages();
        }
        Ok(vec![])
    }
}

impl PackageManager for Pacman {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("pacman").arg("-Syu")))
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("pacman")
                .arg("-S")
                .arg("--noconfirm")
                .arg("--")
                .args(packages),
        ))
    }

    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("pacman")
                .arg("-Rs")
                .arg("--noconfirm")
                .arg("--")
                .args(packages),
        ))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("pacman").arg("-Ss").arg("--").arg(query),
        ))
    }

    fn ls(&self, _format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("pacman").arg("-Q")))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("pacman").arg("-Qq").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("pacman").arg("-Slq").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}
