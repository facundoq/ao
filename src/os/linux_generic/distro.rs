use super::common::SystemCommand;
use crate::cli::{DistroAction, DistroArgs};
use crate::os::{DistroInfo, DistroManager, Domain, ExecutableCommand, OutputFormat};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};

pub struct StandardDistro;

impl Domain for StandardDistro {
    fn name(&self) -> &'static str {
        "distro"
    }
    fn command(&self) -> ClapCommand {
        DistroArgs::augment_args(ClapCommand::new("distro").about("Manage distributions"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = DistroArgs::from_arg_matches(matches)?;
        match &args.action {
            DistroAction::Info { format } => self.info(*format),
            DistroAction::Upgrade => self.upgrade(),
        }
    }
}

impl DistroManager for StandardDistro {
    fn info(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(DistroInfoCommand { format }))
    }

    fn upgrade(&self) -> Result<Box<dyn ExecutableCommand>> {
        if std::path::Path::new("/usr/bin/do-release-upgrade").exists() {
            Ok(Box::new(SystemCommand::new("do-release-upgrade")))
        } else if std::path::Path::new("/usr/bin/dnf").exists() {
            Ok(Box::new(
                SystemCommand::new("dnf")
                    .arg("system-upgrade")
                    .arg("reboot"),
            ))
        } else {
            anyhow::bail!("Distribution upgrade tool not found")
        }
    }
}

pub struct DistroInfoCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for DistroInfoCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("cat").arg("/etc/os-release").execute();
        }

        let mut info = DistroInfo {
            name: String::new(),
            version: String::new(),
            id: String::new(),
            id_like: String::new(),
            pretty_name: String::new(),
        };
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    let val = parts[1].trim_matches('"').to_string();
                    match parts[0] {
                        "NAME" => info.name = val,
                        "VERSION" => info.version = val,
                        "ID" => info.id = val,
                        "ID_LIKE" => info.id_like = val,
                        "PRETTY_NAME" => info.pretty_name = val,
                        _ => {}
                    }
                }
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Property", "Value"]);
                table.add_row(vec!["Name", &info.name]);
                table.add_row(vec!["Version", &info.version]);
                table.add_row(vec!["ID", &info.id]);
                table.add_row(vec!["ID_LIKE", &info.id_like]);
                table.add_row(vec!["Pretty Name", &info.pretty_name]);
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&info)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&info)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Distro info (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("distro info (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("distro info --format {:?}", self.format)
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
