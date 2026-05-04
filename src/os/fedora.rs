use super::linux_generic::{SystemCommand, is_completing_arg};
use super::{Domain, ExecutableCommand, OutputFormat, PackageManager};
use crate::cli::{PackageAction, PackageArgs};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct Dnf;

impl Domain for Dnf {
    fn name(&self) -> &'static str {
        "package"
    }
    fn command(&self) -> ClapCommand {
        PackageArgs::augment_args(ClapCommand::new("package").about("Manage packages (DNF)"))
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
            Some(PackageAction::Delete { packages, purge }) => self.del(packages, *purge),
            Some(PackageAction::Search { query }) => self.search(query),
            Some(PackageAction::List { format }) => self.ls(*format),
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
        if is_completing_arg(words, &["ao", "package", "delete"], 1, last_word_complete) {
            return self.get_installed_packages();
        }
        Ok(vec![])
    }
}

impl PackageManager for Dnf {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("dnf").arg("check-update")))
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf")
                .arg("install")
                .arg("-y")
                .arg("--")
                .args(packages),
        ))
    }

    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf")
                .arg("remove")
                .arg("-y")
                .arg("--")
                .args(packages),
        ))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf").arg("search").arg("--").arg(query),
        ))
    }

    fn ls(&self, _format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("dnf").arg("list").arg("--installed"),
        ))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("rpm")
            .arg("-qa")
            .arg("--queryformat")
            .arg("%{NAME}\n")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("dnf")
            .arg("repoquery")
            .arg("--qf")
            .arg("%{name}")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}
