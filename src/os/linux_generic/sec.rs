use anyhow::Result;
use clap::{ArgMatches, Command as ClapCommand, FromArgMatches, Args};
use crate::os::{SecManager, ExecutableCommand, Domain, SecAuditInfo, OutputFormat};
use crate::cli::{SecArgs, SecAction};
use super::common::SystemCommand;

pub struct StandardSec;

impl Domain for StandardSec {
    fn name(&self) -> &'static str { "sec" }
    fn command(&self) -> ClapCommand {
        SecArgs::augment_args(ClapCommand::new("sec").about("Manage system security"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = SecArgs::from_arg_matches(matches)?;
        match &args.action {
            SecAction::Audit { format } => self.audit(*format),
            SecAction::Context => self.context(),
        }
    }
}

impl SecManager for StandardSec {
    fn audit(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(SystemCommand::new("lynis").arg("audit").arg("system").arg("--quick")));
        }
        Ok(Box::new(SecAuditCommand { format }))
    }
    fn context(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("sestatus").ignore_exit_code()))
    }
}

pub struct SecAuditCommand { pub format: OutputFormat }
impl ExecutableCommand for SecAuditCommand {
    fn execute(&self) -> Result<()> {
        let mut audits = Vec::new();
        // Mock audit checks
        audits.push(SecAuditInfo {
            title: "Password Quality".to_string(),
            result: "OK".to_string(),
            recommendation: "None".to_string(),
        });
        audits.push(SecAuditInfo {
            title: "Open Ports".to_string(),
            result: "Warning".to_string(),
            recommendation: "Close port 22 if not needed".to_string(),
        });

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Audit Check", "Result", "Recommendation"]);
                for a in audits {
                    table.add_row(vec![a.title, a.result, a.recommendation]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&audits)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&audits)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] Security audit (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("security audit (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("security audit --format {:?}", self.format) }
    fn is_structured(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original)
    }
}
