use anyhow::Result;
use std::process::Command;
use clap::{ArgMatches, Command as ClapCommand, FromArgMatches, Args};
use crate::os::{LogManager, ExecutableCommand, Domain};
use crate::cli::{LogArgs, LogAction};
use super::common::{SystemCommand, is_completing_arg};

pub struct StandardLog;

impl Domain for StandardLog {
    fn name(&self) -> &'static str { "log" }
    fn command(&self) -> ClapCommand {
        LogArgs::augment_args(ClapCommand::new("log").about("Tail service and system logs"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = LogArgs::from_arg_matches(matches)?;
        match &args.action {
            LogAction::Tail { name, lines } => self.tail(name, *lines),
            LogAction::Sys { lines } => self.sys_logs(*lines),
            LogAction::File { path, lines } => self.file_logs(path, *lines),
        }
    }
    fn complete(&self, _line: &str, words: &[&str], last_word_complete: bool) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "log", "tail"], 1, last_word_complete) {
            // Suggest services for log tail
            let output = Command::new("systemctl")
                .arg("list-units")
                .arg("--type=service")
                .arg("--no-legend")
                .arg("--no-pager")
                .output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(stdout.lines()
                .filter_map(|l| l.split_whitespace().next())
                .map(|s| s.to_string())
                .collect());
        }
        Ok(vec![])
    }
}

impl LogManager for StandardLog {
    fn tail(&self, service: &str, lines: u32) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("journalctl").arg("-u").arg(service).arg("-f").arg("-n").arg(&lines.to_string())))
    }
    fn sys_logs(&self, lines: u32) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("journalctl").arg("-f").arg("-n").arg(&lines.to_string())))
    }
    fn file_logs(&self, path: &str, lines: u32) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("tail").arg("-f").arg("-n").arg(&lines.to_string()).arg("--").arg(path)))
    }
}
