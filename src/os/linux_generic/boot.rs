use super::common::SystemCommand;
use crate::cli::{BootAction, BootArgs, BootModAction};
use crate::os::{
    BootEntryInfo, BootManager, Domain, ExecutableCommand, KernelModInfo, OutputFormat,
};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardBoot;

impl Domain for StandardBoot {
    fn name(&self) -> &'static str {
        "boot"
    }
    fn command(&self) -> ClapCommand {
        BootArgs::augment_args(
            ClapCommand::new("boot").about("Manage bootloader and kernel modules"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = BootArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(BootAction::Ls { format }) => self.ls_entries(*format),
            Some(BootAction::Mod { action }) => match action {
                BootModAction::Ls { format } => self.ls_modules(*format),
                BootModAction::Load { name } => self.load_module(name),
                BootModAction::Unload { name } => self.unload_module(name),
            },
            None => self.ls_entries(OutputFormat::Table),
        }
    }
}

impl BootManager for StandardBoot {
    fn ls_entries(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(BootListEntriesCommand { format }))
    }
    fn ls_modules(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(BootListModulesCommand { format }))
    }
    fn load_module(&self, name: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("modprobe").arg("--").arg(name)))
    }
    fn unload_module(&self, name: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("modprobe").arg("-r").arg("--").arg(name),
        ))
    }
}

pub struct BootListEntriesCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for BootListEntriesCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("bootctl").arg("list").execute();
        }

        let output = match Command::new("bootctl")
            .arg("list")
            .arg("--no-pager")
            .output()
        {
            Ok(o) => o,
            Err(_) => {
                println!("bootctl command not found or failed to execute.");
                return Ok(());
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();
        for line in stdout.lines() {
            if line.contains("title:") || line.contains("id:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    entries.push(BootEntryInfo {
                        title: parts[1].trim().to_string(),
                        id: "unknown".to_string(),
                    });
                }
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Entry Title", "ID"]);
                for e in entries {
                    table.add_row(vec![e.title, e.id]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&entries)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] bootctl list (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("bootctl list (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "bootctl list".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

pub struct BootListModulesCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for BootListModulesCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("lsmod").execute();
        }

        let output = match Command::new("lsmod").output() {
            Ok(o) => o,
            Err(_) => {
                println!("lsmod command not found or failed to execute.");
                return Ok(());
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut modules = Vec::new();
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                modules.push(KernelModInfo {
                    name: parts[0].to_string(),
                    size: parts[1].parse().unwrap_or(0),
                    used_by: parts[2].to_string(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Module", "Size", "Used By"]);
                for m in modules {
                    table.add_row(vec![m.name, m.size.to_string(), m.used_by]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&modules)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&modules)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] lsmod (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("lsmod (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "lsmod".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
