use super::linux_generic::{SystemCommand, is_completing_arg};
use super::{Domain, ExecutableCommand, OutputFormat, PackageInfo, PackageManager};
use crate::cli::{PkgAction, PkgArgs};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct Apk;

impl Domain for Apk {
    fn name(&self) -> &'static str {
        "pkg"
    }
    fn command(&self) -> ClapCommand {
        PkgArgs::augment_args(ClapCommand::new("pkg").about("Manage packages (APK)"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = PkgArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(PkgAction::Update) => self.update(),
            Some(PkgAction::Add { packages }) => self.add(packages),
            Some(PkgAction::Del { packages, purge }) => self.del(packages, *purge),
            Some(PkgAction::Search { query }) => self.search(query),
            Some(PkgAction::Ls { format }) => self.ls(*format),
            None => self.ls(OutputFormat::Table),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "pkg", "add"], 1, last_word_complete) {
            return self.get_available_packages();
        }
        if is_completing_arg(words, &["ao", "pkg", "del"], 1, last_word_complete) {
            return self.get_installed_packages();
        }
        Ok(vec![])
    }
}

impl PackageManager for Apk {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("apk").arg("update")))
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apk")
                .arg("add")
                .arg("--")
                .args(packages),
        ))
    }

    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apk")
                .arg("del")
                .arg("--")
                .args(packages),
        ))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apk")
                .arg("search")
                .arg("-v")
                .arg("--")
                .arg(query),
        ))
    }

    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(ApkListCommand { format }))
    }

    fn get_installed_packages(&self) -> Result<Vec<String>> {
        let output = Command::new("apk").arg("info").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_available_packages(&self) -> Result<Vec<String>> {
        // This can be slow and might need a better way for completions
        let output = Command::new("apk").arg("search").arg("-q").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}

pub struct ApkListCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for ApkListCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("apk").arg("info").arg("-v").execute();
        }

        let output = Command::new("apk").args(["info", "-v"]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut packages = Vec::new();
        for line in stdout.lines() {
            // apk info -v output format: name-version
            // We'll try to split at the last hyphen that starts with a digit
            if let Some(pos) = line.rfind('-') {
                let (name, ver) = line.split_at(pos);
                packages.push(PackageInfo {
                    name: name.to_string(),
                    version: ver.trim_start_matches('-').to_string(),
                    architecture: "unknown".to_string(),
                    status: "installed".to_string(),
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
        "apk info -v".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
