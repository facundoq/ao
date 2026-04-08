use anyhow::Result;
use std::process::Command;
use clap::{ArgMatches, Command as ClapCommand, FromArgMatches, Args};
use crate::os::{GuiManager, ExecutableCommand, Domain, DisplayInfo, OutputFormat};
use crate::cli::{GuiArgs, GuiAction, GuiDisplayAction};
use super::common::SystemCommand;

pub struct StandardGui;

impl Domain for StandardGui {
    fn name(&self) -> &'static str { "gui" }
    fn command(&self) -> ClapCommand {
        GuiArgs::augment_args(ClapCommand::new("gui").about("Manage displays and GUI sessions"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = GuiArgs::from_arg_matches(matches)?;
        match &args.action {
            GuiAction::Info => self.info(),
            GuiAction::Display { action } => match action {
                GuiDisplayAction::List { format } => self.list_displays(*format),
            },
        }
    }
}

impl GuiManager for StandardGui {
    fn info(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("loginctl").arg("show-session").arg("self").arg("-p").arg("Type")))
    }
    fn list_displays(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(SystemCommand::new("xrandr").arg("--query").ignore_exit_code()));
        }
        Ok(Box::new(GuiListDisplaysCommand { format }))
    }
}

pub struct GuiListDisplaysCommand { pub format: OutputFormat }
impl ExecutableCommand for GuiListDisplaysCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("xrandr").arg("--query").execute().or_else(|_| SystemCommand::new("wlr-randr").execute());
        }
        let output = Command::new("xrandr").arg("--query").output().or_else(|_| Command::new("wlr-randr").output())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut displays = Vec::new();
        for line in stdout.lines() {
            if line.contains(" connected") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                displays.push(DisplayInfo {
                    name: parts[0].to_string(),
                    connected: true,
                    resolution: parts.iter().find(|p| p.contains('x')).cloned().unwrap_or("unknown").to_string(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Display", "Connected", "Resolution"]);
                for d in displays {
                    table.add_row(vec![d.name, d.connected.to_string(), d.resolution]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&displays)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&displays)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] List displays (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("list displays (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("list displays --format {:?}", self.format) }
    fn is_structured(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original)
    }
}
