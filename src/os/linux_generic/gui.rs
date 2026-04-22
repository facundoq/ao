use super::common::SystemCommand;
use crate::cli::{GuiAction, GuiArgs, GuiDisplayAction};
use crate::os::{DisplayInfo, Domain, ExecutableCommand, GuiManager, OutputFormat};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardGui;

impl Domain for StandardGui {
    fn name(&self) -> &'static str {
        "gui"
    }
    fn command(&self) -> ClapCommand {
        GuiArgs::augment_args(ClapCommand::new("gui").about("Manage displays and GUI sessions"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = GuiArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(GuiAction::Info) => self.info(),
            Some(GuiAction::Display { action }) => match action {
                GuiDisplayAction::Ls { format } => self.ls_displays(*format),
            },
            None => self.ls_displays(OutputFormat::Table),
        }
    }
}

impl GuiManager for StandardGui {
    fn info(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("loginctl")
                .arg("show-session")
                .arg("self")
                .arg("-p")
                .arg("Type")
                .ignore_exit_code(),
        ))
    }
    fn ls_displays(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(
                SystemCommand::new("xrandr")
                    .arg("--query")
                    .ignore_exit_code(),
            ));
        }
        Ok(Box::new(GuiListDisplaysCommand { format }))
    }
}

pub struct GuiListDisplaysCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for GuiListDisplaysCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("xrandr")
                .arg("--query")
                .ignore_exit_code()
                .execute()
                .or_else(|_| SystemCommand::new("wlr-randr").ignore_exit_code().execute());
        }
        let output = match Command::new("xrandr")
            .arg("--query")
            .output()
            .or_else(|_| Command::new("wlr-randr").output())
        {
            Ok(o) => o,
            Err(_) => {
                println!("xrandr or wlr-randr not found or failed to execute.");
                return Ok(());
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut displays = Vec::new();
        for line in stdout.lines() {
            if line.contains(" connected") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                displays.push(DisplayInfo {
                    name: parts[0].to_string(),
                    connected: true,
                    resolution: parts
                        .iter()
                        .find(|p| p.contains('x'))
                        .cloned()
                        .unwrap_or("unknown")
                        .to_string(),
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
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&displays)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&displays)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn as_string(&self) -> String {
        "xrandr --query || wlr-randr".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
