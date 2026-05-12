use super::{Domain, PackageManager, ServiceManager};
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
        Command::new(Domain::name(self))
    }
    fn execute(
        &self,
        _matches: &ArgMatches,
        _app: &Command,
    ) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("MacOS package manager not implemented")
    }
}
impl PackageManager for MacOSPackage {
    fn name(&self) -> &'static str {
        "homebrew"
    }
    fn cmd(&self) -> super::linux_generic::SystemCommand {
        super::linux_generic::SystemCommand::new("brew")
    }
    fn add(&self, _p: &[String]) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn del(&self, _p: &[String], _purge: bool) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn search(&self, _q: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn ls(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn get_installed_packages(&self) -> Result<Vec<String>> {
        Ok(vec![])
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
        Command::new(self.name())
    }
    fn execute(&self, _m: &ArgMatches, _a: &Command) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
}
impl ServiceManager for MacOSService {
    fn ls(&self, _f: OutputFormat) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn up(&self, _s: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn down(&self, _s: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn restart(&self, _s: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn reload(&self, _s: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn status(&self, _s: &str) -> Result<Box<dyn super::ExecutableCommand>> {
        bail!("Not implemented")
    }
    fn get_services(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
    fn get_all_services_info(&self) -> Result<Vec<super::ServiceInfo>> {
        Ok(vec![])
    }
}
