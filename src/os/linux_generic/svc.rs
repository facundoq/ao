use anyhow::Result;
use std::process::Command;
use clap::{ArgMatches, Command as ClapCommand, FromArgMatches, Args};
use crate::os::{ServiceManager, ExecutableCommand, Domain, ServiceInfo, OutputFormat};
use crate::cli::{SvcArgs, SvcAction};
use super::common::{SystemCommand, is_completing_arg};

pub struct Systemd;

impl Domain for Systemd {
    fn name(&self) -> &'static str { "svc" }
    fn command(&self) -> ClapCommand {
        SvcArgs::augment_args(ClapCommand::new("svc").about("Manage services"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = SvcArgs::from_arg_matches(matches)?;
        match &args.action {
            SvcAction::List { format } => self.list(*format),
            SvcAction::Up { name } => self.up(name),
            SvcAction::Down { name } => self.down(name),
            SvcAction::Restart { name } => self.restart(name),
            SvcAction::Reload { name } => self.reload(name),
            SvcAction::Status { name } => self.status(name),
        }
    }
    fn complete(&self, _line: &str, words: &[&str], last_word_complete: bool) -> Result<Vec<String>> {
        let svc_actions = ["up", "down", "restart", "reload", "status"];
        for action in svc_actions {
            if is_completing_arg(words, &["ao", "svc", action], 1, last_word_complete) {
                return self.get_services();
            }
        }
        Ok(vec![])
    }
}

impl ServiceManager for Systemd {
    fn list(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ServiceListCommand { format }))
    }

    fn up(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ServiceUpCommand { service: service.to_string() }))
    }

    fn down(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ServiceDownCommand { service: service.to_string() }))
    }

    fn restart(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ServiceRestartCommand { service: service.to_string() }))
    }

    fn reload(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ServiceReloadCommand { service: service.to_string() }))
    }

    fn status(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ServiceStatusCommand { service: service.to_string() }))
    }

    fn get_services(&self) -> Result<Vec<String>> {
        let output = Command::new("systemctl")
            .arg("list-unit-files")
            .arg("--type=service")
            .arg("--no-legend")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter_map(|l| l.split_whitespace().next())
            .map(|s| s.to_string())
            .collect())
    }
}

pub struct ServiceListCommand { pub format: OutputFormat }
impl ExecutableCommand for ServiceListCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("systemctl").arg("list-units").arg("--type=service").execute();
        }
        let output = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .arg("--no-legend")
            .arg("--no-pager")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                services.push(ServiceInfo {
                    name: parts[0].to_string(),
                    loaded: parts[1].to_string(),
                    active: parts[2].to_string(),
                    status: parts[3].to_string(),
                    description: parts[4..].join(" "),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                
                // Detect terminal width for better wrapping
                if let Ok((width, _)) = crossterm::terminal::size() {
                    table.set_width(width);
                }

                table.set_header(vec!["Service", "Loaded", "Active", "Status", "Description"]);
                
                // Constraint service name and description to wrap if they are too long
                table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
                
                use colored::Colorize;
                for s in services {
                    let loaded_colored = if s.loaded == "loaded" { s.loaded.green() } else { s.loaded.normal() };
                    let active_colored = if s.active == "active" { s.active.green() } else { s.active.normal() };
                    
                    let status_colored = match s.status.as_str() {
                        "running" => s.status.green(),
                        "exited" => s.status.yellow(),
                        "failed" => s.status.red(),
                        _ => s.status.normal(),
                    };

                    table.add_row(vec![
                        comfy_table::Cell::new(s.name),
                        comfy_table::Cell::new(loaded_colored),
                        comfy_table::Cell::new(active_colored),
                        comfy_table::Cell::new(status_colored),
                        comfy_table::Cell::new(s.description),
                    ]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&services)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&services)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] systemctl list-units (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("systemctl list-units (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("systemctl list-units --format {:?}", self.format) }
    fn is_structured(&self) -> bool { matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original) }
}

pub struct ServiceUpCommand { pub service: String }
impl ExecutableCommand for ServiceUpCommand {
    fn execute(&self) -> Result<()> { SystemCommand::new("systemctl").arg("enable").arg("--now").arg("--").arg(&self.service).execute() }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] {}", self.as_string()); Ok(()) }
    fn print(&self) -> Result<()> { println!("{}", self.as_string()); Ok(()) }
    fn as_string(&self) -> String { format!("systemctl enable --now -- {}", self.service) }
}

pub struct ServiceDownCommand { pub service: String }
impl ExecutableCommand for ServiceDownCommand {
    fn execute(&self) -> Result<()> { SystemCommand::new("systemctl").arg("disable").arg("--now").arg("--").arg(&self.service).execute() }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] {}", self.as_string()); Ok(()) }
    fn print(&self) -> Result<()> { println!("{}", self.as_string()); Ok(()) }
    fn as_string(&self) -> String { format!("systemctl disable --now -- {}", self.service) }
}

pub struct ServiceRestartCommand { pub service: String }
impl ExecutableCommand for ServiceRestartCommand {
    fn execute(&self) -> Result<()> { SystemCommand::new("systemctl").arg("restart").arg("--").arg(&self.service).execute() }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] {}", self.as_string()); Ok(()) }
    fn print(&self) -> Result<()> { println!("{}", self.as_string()); Ok(()) }
    fn as_string(&self) -> String { format!("systemctl restart -- {}", self.service) }
}

pub struct ServiceReloadCommand { pub service: String }
impl ExecutableCommand for ServiceReloadCommand {
    fn execute(&self) -> Result<()> { SystemCommand::new("systemctl").arg("reload").arg("--").arg(&self.service).execute() }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] {}", self.as_string()); Ok(()) }
    fn print(&self) -> Result<()> { println!("{}", self.as_string()); Ok(()) }
    fn as_string(&self) -> String { format!("systemctl reload -- {}", self.service) }
}

pub struct ServiceStatusCommand { pub service: String }
impl ExecutableCommand for ServiceStatusCommand {
    fn execute(&self) -> Result<()> { SystemCommand::new("systemctl").arg("status").arg("--").arg(&self.service).ignore_exit_code().execute() }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] {}", self.as_string()); Ok(()) }
    fn print(&self) -> Result<()> { println!("{}", self.as_string()); Ok(()) }
    fn as_string(&self) -> String { format!("systemctl status -- {}", self.service) }
}
