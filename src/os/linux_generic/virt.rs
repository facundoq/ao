use super::common::SystemCommand;
use crate::cli::{VirtAction, VirtArgs};
use crate::os::{ContainerInfo, Domain, ExecutableCommand, OutputFormat, VirtManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardVirt;

impl Domain for StandardVirt {
    fn name(&self) -> &'static str {
        "virt"
    }
    fn command(&self) -> ClapCommand {
        VirtArgs::augment_args(ClapCommand::new("virt").about("Manage containers and VMs"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = VirtArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(VirtAction::Ls { format }) => self.ls(*format),
            Some(VirtAction::Start { name }) => self.start(name),
            Some(VirtAction::Stop { name }) => self.stop(name),
            Some(VirtAction::Rm { name }) => self.del(name),
            Some(VirtAction::Logs { name }) => self.logs(name),
            None => self.ls(OutputFormat::Table),
        }
    }
}

impl VirtManager for StandardVirt {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            return Ok(Box::new(SystemCommand::new("docker").arg("ps")));
        }
        Ok(Box::new(VirtPsCommand { format }))
    }

    fn start(&self, name: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("docker")
                .arg("start")
                .arg("--")
                .arg(name),
        ))
    }

    fn stop(&self, name: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("docker").arg("stop").arg("--").arg(name),
        ))
    }

    fn del(&self, name: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("docker")
                .arg("rm")
                .arg("-f")
                .arg("--")
                .arg(name),
        ))
    }

    fn logs(&self, name: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("docker")
                .arg("logs")
                .arg("-f")
                .arg("--")
                .arg(name),
        ))
    }
}

pub struct VirtPsCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for VirtPsCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.ID}}\t{{.Image}}\t{{.Command}}\t{{.CreatedAt}}\t{{.Status}}\t{{.Names}}")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 6 {
                containers.push(ContainerInfo {
                    id: parts[0].to_string(),
                    image: parts[1].to_string(),
                    command: parts[2].to_string(),
                    created: parts[3].to_string(),
                    status: parts[4].to_string(),
                    names: parts[5].to_string(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["ID", "Image", "Status", "Names"]);
                for c in containers {
                    table.add_row(vec![c.id, c.image, c.status, c.names]);
                }
                println!("{}", table);
            }
            _ => self.format.print_structured(&containers)?,
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] docker ps (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("docker ps (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "docker ps".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
