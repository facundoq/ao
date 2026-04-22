use super::common::{SystemCommand, is_completing_arg};
use crate::cli::{LogAction, LogArgs};
use crate::os::{Domain, ExecutableCommand, LogManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardLog;

impl Domain for StandardLog {
    fn name(&self) -> &'static str {
        "log"
    }
    fn command(&self) -> ClapCommand {
        LogArgs::augment_args(
            ClapCommand::new("log").about("Comprehensive system and service logs"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = LogArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(LogAction::Auth { lines, follow }) => self.auth(*lines, *follow),
            Some(LogAction::Boot { lines, follow, id }) => {
                self.boot(*lines, *follow, id.as_deref())
            }
            Some(LogAction::Crash { lines }) => self.crash(*lines),
            Some(LogAction::Dev { lines, follow }) => self.dev(*lines, *follow),
            Some(LogAction::Error { lines, follow }) => self.error(*lines, *follow),
            Some(LogAction::File {
                path,
                lines,
                follow,
            }) => self.file(path, *lines, *follow),
            Some(LogAction::Pkg { lines }) => self.pkg(*lines),
            Some(LogAction::Svc {
                name,
                lines,
                follow,
            }) => self.svc(name, *lines, *follow),
            Some(LogAction::Sys { lines, follow }) => self.sys_logs(*lines, *follow),
            None => self.sys_logs(50, false),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "log", "svc"], 1, last_word_complete) {
            // Suggest services for log tail
            let output = Command::new("systemctl")
                .arg("list-units")
                .arg("--type=service")
                .arg("--no-legend")
                .arg("--no-pager")
                .output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(stdout
                .lines()
                .filter_map(|l| l.split_whitespace().next())
                .map(|s| s.to_string())
                .collect());
        }
        Ok(vec![])
    }
}

impl LogManager for StandardLog {
    fn auth(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("journalctl");
        if follow {
            cmd = cmd.arg("-f");
        }
        Ok(Box::new(
            cmd.arg("-n")
                .arg(&lines.to_string())
                .arg("_FACILITY=4")
                .arg("_FACILITY=10")
                .arg("--"),
        ))
    }

    fn boot(
        &self,
        lines: u32,
        follow: bool,
        id: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("journalctl");
        if follow {
            cmd = cmd.arg("-f");
        }
        if let Some(boot_id) = id {
            cmd = cmd.arg("-b").arg(boot_id);
        } else {
            cmd = cmd.arg("-b");
        }
        Ok(Box::new(cmd.arg("-n").arg(&lines.to_string()).arg("--")))
    }

    fn crash(&self, lines: u32) -> Result<Box<dyn ExecutableCommand>> {
        // Look for kernel panics, core dumps, and segfaults in high-priority logs
        Ok(Box::new(
            SystemCommand::new("journalctl")
                .arg("-p")
                .arg("0..3") // emerg, alert, crit, err
                .arg("-n")
                .arg(&lines.to_string())
                .arg("-k") // kernel logs often contain crashes
                .arg("--"),
        ))
    }

    fn dev(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("journalctl");
        if follow {
            cmd = cmd.arg("-f");
        }
        Ok(Box::new(
            cmd.arg("-k") // kernel logs for devices
                .arg("-n")
                .arg(&lines.to_string())
                .arg("--"),
        ))
    }

    fn error(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("journalctl");
        if follow {
            cmd = cmd.arg("-f");
        }
        Ok(Box::new(
            cmd.arg("-p")
                .arg("err..emerg")
                .arg("-n")
                .arg(&lines.to_string())
                .arg("--"),
        ))
    }

    fn file(&self, path: &str, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("tail");
        if follow {
            cmd = cmd.arg("-f");
        }
        Ok(Box::new(
            cmd.arg("-n").arg(&lines.to_string()).arg("--").arg(path),
        ))
    }

    fn pkg(&self, lines: u32) -> Result<Box<dyn ExecutableCommand>> {
        // Distro-aware package history. Try common paths or journal tags.
        let cmd = SystemCommand::new("journalctl");
        // Debian/Ubuntu uses 'apt' and 'dpkg', Fedora uses 'dnf'
        Ok(Box::new(
            cmd.arg("-t")
                .arg("apt")
                .arg("-t")
                .arg("dpkg")
                .arg("-t")
                .arg("dnf")
                .arg("-n")
                .arg(&lines.to_string())
                .arg("--"),
        ))
    }

    fn svc(&self, service: &str, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("journalctl");
        if follow {
            cmd = cmd.arg("-f");
        }
        Ok(Box::new(
            cmd.arg("-u")
                .arg(service)
                .arg("-n")
                .arg(&lines.to_string())
                .arg("--"),
        ))
    }

    fn sys_logs(&self, lines: u32, follow: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("journalctl");
        if follow {
            cmd = cmd.arg("-f");
        }
        Ok(Box::new(cmd.arg("-n").arg(&lines.to_string()).arg("--")))
    }
}
