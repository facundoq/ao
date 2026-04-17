use super::common::{SystemCommand, is_completing_arg};
use crate::cli::{SysAction, SysArgs};
use crate::os::{Domain, ExecutableCommand, OutputFormat, SysInfoData, SysManager, SysTimeData};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;
use sysinfo::System;

pub struct StandardSys;

impl Domain for StandardSys {
    fn name(&self) -> &'static str {
        "sys"
    }
    fn command(&self) -> ClapCommand {
        SysArgs::augment_args(
            ClapCommand::new("sys").about("Manage core system (updates, power, time)"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = SysArgs::from_arg_matches(matches)?;
        match &args.action {
            SysAction::Info { format } => self.info(*format),
            SysAction::Power { state, now, force } => self.power(state, *now, *force),
            SysAction::Time {
                action,
                value,
                format,
            } => self.time(action, value.as_deref(), *format),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "sys", "power"], 1, last_word_complete) {
            return Ok(vec![
                "reboot".to_string(),
                "shutdown".to_string(),
                "suspend".to_string(),
                "hibernate".to_string(),
            ]);
        }
        if is_completing_arg(words, &["ao", "sys", "time"], 1, last_word_complete) {
            return Ok(vec![
                "status".to_string(),
                "set".to_string(),
                "sync".to_string(),
            ]);
        }
        Ok(vec![])
    }
}

impl SysManager for StandardSys {
    fn info(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            let cmd = crate::os::linux_generic::common::CompoundCommand::new(vec![
                Box::new(SystemCommand::new("uname").arg("-a")),
                Box::new(SystemCommand::new("uptime")),
                Box::new(SystemCommand::new("cat").arg("/etc/os-release")),
            ]);
            return Ok(Box::new(cmd));
        }
        Ok(Box::new(SysInfoCommand { format }))
    }

    fn power(&self, state: &str, now: bool, force: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("systemctl");
        match state {
            "reboot" => cmd = cmd.arg("reboot"),
            "shutdown" => cmd = cmd.arg("poweroff"),
            "suspend" => cmd = cmd.arg("suspend"),
            "hibernate" => cmd = cmd.arg("hibernate"),
            _ => anyhow::bail!("Unsupported power state: {}", state),
        }
        if now {
            cmd = cmd.arg("--now");
        }
        if force {
            cmd = cmd.arg("--force");
        }
        Ok(Box::new(cmd))
    }

    fn time(
        &self,
        action: &str,
        value: Option<&str>,
        format: OutputFormat,
    ) -> Result<Box<dyn ExecutableCommand>> {
        if action == "status" {
            if matches!(format, OutputFormat::Original) {
                return Ok(Box::new(SystemCommand::new("timedatectl").arg("status")));
            }
            return Ok(Box::new(SysTimeCommand { format }));
        }

        let mut cmd = SystemCommand::new("timedatectl");
        match action {
            "set" => {
                if let Some(v) = value {
                    cmd = cmd.arg("set-timezone").arg("--").arg(v);
                } else {
                    anyhow::bail!("Timezone value required for set action");
                }
            }
            "sync" => cmd = cmd.arg("set-ntp").arg("true"),
            _ => anyhow::bail!("Unsupported time action: {}", action),
        }
        Ok(Box::new(cmd))
    }
}

pub struct SysInfoCommand {
    pub format: OutputFormat,
}

impl ExecutableCommand for SysInfoCommand {
    fn execute(&self) -> Result<()> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let data = SysInfoData {
            hostname: System::host_name().unwrap_or_default(),
            os: System::long_os_version().unwrap_or_default(),
            kernel: System::kernel_version().unwrap_or_default(),
            architecture: System::cpu_arch(),
            uptime: format!("{} seconds", System::uptime()),
            cpu_count: sys.cpus().len(),
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
        };

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Property", "Value"]);
                table.add_row(vec!["Hostname", &data.hostname]);
                table.add_row(vec!["OS", &data.os]);
                table.add_row(vec!["Kernel", &data.kernel]);
                table.add_row(vec!["Architecture", &data.architecture]);
                table.add_row(vec!["Uptime", &data.uptime]);
                table.add_row(vec!["CPU Count", &data.cpu_count.to_string()]);
                table.add_row(vec![
                    "Total Memory",
                    &format!("{} bytes", data.total_memory),
                ]);
                table.add_row(vec!["Used Memory", &format!("{} bytes", data.used_memory)]);
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&data)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Sys info (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("sys info --format {:?}", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("sys info --format {:?}", self.format)
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

pub struct SysTimeCommand {
    pub format: OutputFormat,
}

impl ExecutableCommand for SysTimeCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("timedatectl").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut data = SysTimeData {
            local_time: String::new(),
            universal_time: String::new(),
            rtc_time: String::new(),
            time_zone: String::new(),
            system_clock_synchronized: String::new(),
            ntp_service: String::new(),
            rtc_in_local_tz: String::new(),
        };

        for line in stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Local time:") {
                data.local_time = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("Universal time:") {
                data.universal_time = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("RTC time:") {
                data.rtc_time = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("Time zone:") {
                data.time_zone = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("System clock synchronized:") {
                data.system_clock_synchronized = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("NTP service:") {
                data.ntp_service = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("RTC in local TZ:") {
                data.rtc_in_local_tz = val.trim().to_string();
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Property", "Value"]);
                table.add_row(vec!["Local Time", &data.local_time]);
                table.add_row(vec!["Universal Time", &data.universal_time]);
                table.add_row(vec!["RTC Time", &data.rtc_time]);
                table.add_row(vec!["Time Zone", &data.time_zone]);
                table.add_row(vec!["Clock Synced", &data.system_clock_synchronized]);
                table.add_row(vec!["NTP Service", &data.ntp_service]);
                table.add_row(vec!["RTC in Local TZ", &data.rtc_in_local_tz]);
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&data)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] timedatectl status (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("timedatectl status (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("timedatectl status --format {:?}", self.format)
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
