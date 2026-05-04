use super::linux_generic::{SystemCommand, is_completing_arg};
use super::{Domain, ExecutableCommand, OutputFormat, PackageInfo, PackageManager};
use crate::cli::{PackageAction, PackageArgs};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct Apt;

impl Domain for Apt {
    fn name(&self) -> &'static str {
        "package"
    }
    fn command(&self) -> ClapCommand {
        PackageArgs::augment_args(ClapCommand::new("package").about("Manage packages (APT)"))
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

impl PackageManager for Apt {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("apt-get").arg("update")))
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apt-get")
                .arg("install")
                .arg("-y")
                .arg("--")
                .args(packages),
        ))
    }

    fn del(&self, packages: &[String], purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("apt-get");
        if purge {
            cmd = cmd.arg("purge");
        } else {
            cmd = cmd.arg("remove");
        }
        Ok(Box::new(cmd.arg("-y").arg("--").args(packages)))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apt-cache")
                .arg("search")
                .arg("--")
                .arg(query),
        ))
    }

    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(AptListCommand { format }))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("dpkg-query")
            .arg("-W")
            .arg("-f=${Package}\n")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("apt-cache").arg("pkgnames").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}

pub struct AptListCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for AptListCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("apt")
                .arg("list")
                .arg("--installed")
                .execute();
        }

        let output = Command::new("dpkg-query")
            .arg("-W")
            .arg("-f=${Package}\t${Version}\t${Architecture}\t${db:Status-Abbrev}\n")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut packages = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                packages.push(PackageInfo {
                    name: parts[0].to_string(),
                    version: parts[1].to_string(),
                    architecture: parts[2].to_string(),
                    status: parts[3].to_string(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                if let Ok((width, _)) = crossterm::terminal::size() {
                    table.set_width(width);
                }
                table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
                table.set_header(vec!["Package", "Version", "Architecture", "Status"]);
                for p in packages {
                    table.add_row(vec![p.name, p.version, p.architecture, p.status]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&packages)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&packages)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn as_string(&self) -> String {
        "dpkg-query -W".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
