use super::linux_generic::SystemCommand;
use super::{ExecutableCommand, OutputFormat, PackageInfo, PackageManager};
use anyhow::Result;
use std::process::Command;

pub struct Apk;

impl PackageManager for Apk {
    fn name(&self) -> &'static str {
        "APK"
    }

    fn cmd(&self) -> SystemCommand {
        SystemCommand::new("apk")
    }

    fn add(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(self.cmd().arg("add").arg("--").args(packages)))
    }

    fn del(&self, packages: &[String], _purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(self.cmd().arg("delete").arg("--").args(packages)))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            self.cmd().arg("search").arg("-v").arg("--").arg(query),
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
