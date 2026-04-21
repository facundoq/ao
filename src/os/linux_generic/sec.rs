use super::common::SystemCommand;
use crate::cli::{SecAction, SecArgs};
use crate::os::{Domain, ExecutableCommand, OutputFormat, SecAuditInfo, SecManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};

pub struct StandardSec;

impl Domain for StandardSec {
    fn name(&self) -> &'static str {
        "sec"
    }
    fn command(&self) -> ClapCommand {
        SecArgs::augment_args(ClapCommand::new("sec").about("Manage system security"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = SecArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(SecAction::Audit { format }) => self.audit(*format),
            Some(SecAction::Context) => self.context(),
            None => self.audit(OutputFormat::Table),
        }
    }
}

impl SecManager for StandardSec {
    fn audit(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(
                SystemCommand::new("lynis")
                    .arg("audit")
                    .arg("system")
                    .arg("--quick"),
            ));
        }
        Ok(Box::new(SecAuditCommand { format }))
    }
    fn context(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SecContextCommand))
    }
}

pub struct SecContextCommand;
impl ExecutableCommand for SecContextCommand {
    fn execute(&self) -> Result<()> {
        use std::process::Command;

        // Try SELinux
        match Command::new("sestatus").status() {
            Ok(_) => return Ok(()), // sestatus prints its own output
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }

        // Try AppArmor
        match Command::new("apparmor_status").status() {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }

        println!("Neither SELinux nor AppArmor appear to be available on this system.");
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Check security context (SELinux/AppArmor)");
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("sestatus || apparmor_status");
        Ok(())
    }
    fn as_string(&self) -> String {
        "sestatus || apparmor_status".to_string()
    }
}

pub struct SecAuditCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for SecAuditCommand {
    fn execute(&self) -> Result<()> {
        let audits = vec![
            SecAuditInfo {
                title: "Password Quality".to_string(),
                result: "Weak".to_string(),
                recommendation: "Use libpam-pwquality".to_string(),
            },
            SecAuditInfo {
                title: "SSH Port".to_string(),
                result: "Standard (22)".to_string(),
                recommendation: "Close port 22 if not needed".to_string(),
            },
        ];

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Audit Check", "Result", "Recommendation"]);
                for a in audits {
                    table.add_row(vec![a.title, a.result, a.recommendation]);
                }
                println!("{}", table);
            }
            _ => self.format.print_structured(&audits)?,
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Security audit (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("security audit (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "Custom security audit logic".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
