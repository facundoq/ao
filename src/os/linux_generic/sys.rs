use super::common::{SystemCommand, format_duration, is_completing_arg};
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
            Some(SysAction::Info { format }) => self.info(*format),
            Some(SysAction::Power { state, now, force }) => self.power(state, *now, *force),
            Some(SysAction::Time {
                action,
                value,
                format,
            }) => self.time(action, value.as_deref(), *format),
            None => self.info(OutputFormat::Table),
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

        // Memory in GB
        let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;
        let used_memory_gb = sys.used_memory() as f64 / 1_073_741_824.0;
        let total_memory_readable = format!("{:.2} GB", total_memory_gb);
        let used_memory_readable = format!("{:.2} GB", used_memory_gb);

        // CPU Model
        let cpu_model = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Physical Drives
        let physical_drives = Command::new("lsblk")
            .arg("-d")
            .arg("-n")
            .arg("-o")
            .arg("TYPE")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| l.trim() == "disk")
                    .count()
            })
            .unwrap_or(0);

        // Network Adapters
        let mut lan_adapters = Vec::new();
        let mut wifi_adapters = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "lo" {
                    continue;
                }
                let path = entry.path();
                // Check if it's a physical device by looking for the 'device' symlink
                if !path.join("device").exists() {
                    continue;
                }

                let is_wifi = path.join("wireless").exists() || path.join("phy80211").exists();
                if is_wifi {
                    wifi_adapters.push(name);
                } else {
                    lan_adapters.push(name);
                }
            }
        }

        // BT Adapters
        let mut bt_adapters = Vec::new();
        if let Ok(output) = Command::new("hciconfig").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("hci")
                    && line.contains(':')
                    && let Some(name) = line.split(':').next()
                {
                    bt_adapters.push(name.trim().to_string());
                }
            }
        }
        if bt_adapters.is_empty()
            && let Ok(output) = Command::new("bluetoothctl").arg("list").output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.split_whitespace().nth(1) {
                    bt_adapters.push(name.to_string());
                }
            }
        }

        // Monitors
        let mut monitors = Vec::new();
        let output = Command::new("xrandr")
            .arg("--query")
            .output()
            .or_else(|_| Command::new("wlr-randr").output());

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(" connected") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let name = parts[0].to_string();
                        // Try to get model from the next lines if possible
                        monitors.push(name);
                    }
                }
            }
        }

        // Users
        let mut system_users_count = 0;
        let mut common_users_count = 0;
        if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3
                    && let Ok(uid) = parts[2].parse::<u32>()
                {
                    if uid < 1000 {
                        system_users_count += 1;
                    } else {
                        common_users_count += 1;
                    }
                }
            }
        }

        // RAM Type/Model (hard to get without root, try dmidecode if available)
        let ram_type = "Unknown".to_string();
        let ram_model = "Unknown".to_string();

        let data = SysInfoData {
            hostname: System::host_name().unwrap_or_default(),
            os: System::long_os_version().unwrap_or_default(),
            kernel: System::kernel_version().unwrap_or_default(),
            architecture: System::cpu_arch(),
            uptime: format_duration(System::uptime()),
            cpu_count: sys.cpus().len(),
            cpu_model,
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            total_memory_readable,
            used_memory_readable,
            ram_type,
            ram_model,
            physical_drives,
            lan_adapters,
            wifi_adapters,
            bt_adapters,
            monitors,
            system_users_count,
            common_users_count,
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
                table.add_row(vec!["CPU Model", &data.cpu_model]);
                table.add_row(vec!["CPU Count", &data.cpu_count.to_string()]);
                table.add_row(vec!["Total Memory", &data.total_memory_readable]);
                table.add_row(vec!["Used Memory", &data.used_memory_readable]);
                table.add_row(vec!["RAM Type", &data.ram_type]);
                table.add_row(vec!["RAM Model", &data.ram_model]);
                table.add_row(vec!["Physical Drives", &data.physical_drives.to_string()]);
                table.add_row(vec!["LAN Adapters", &data.lan_adapters.join(", ")]);
                table.add_row(vec!["WiFi Adapters", &data.wifi_adapters.join(", ")]);
                table.add_row(vec!["BT Adapters", &data.bt_adapters.join(", ")]);
                table.add_row(vec!["Monitors", &data.monitors.join(", ")]);
                table.add_row(vec!["System Users", &data.system_users_count.to_string()]);
                table.add_row(vec!["Common Users", &data.common_users_count.to_string()]);
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
        "sysinfo (Rust library)".to_string()
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
        "timedatectl status".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}
