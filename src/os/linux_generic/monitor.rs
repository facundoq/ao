use super::common::{Emoji, format_bytes};
use crate::cli::{MonitorArgs, OutputFormat};
use crate::os::{Domain, ExecutableCommand, MonitorEntry, MonitorManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use sysinfo::{Components, Disks, Networks, System};

pub struct StandardMonitor;

impl Domain for StandardMonitor {
    fn name(&self) -> &'static str {
        "monitor"
    }
    fn command(&self) -> ClapCommand {
        MonitorArgs::augment_args(ClapCommand::new("monitor").about("Monitor live system stats"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = MonitorArgs::from_arg_matches(matches)?;
        self.live_stats(args.format)
    }
}

impl MonitorManager for StandardMonitor {
    fn live_stats(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(LiveStatsCommand { format }))
    }
}

pub struct LiveStatsCommand {
    pub format: OutputFormat,
}

impl ExecutableCommand for LiveStatsCommand {
    fn execute(&self) -> Result<()> {
        let mut sys = System::new_all();
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_all();

        let mut entries = Vec::new();

        // CPU
        let cpu_model = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        entries.push(MonitorEntry {
            entry_type: "CPU".to_string(),
            subtype: "Physical".to_string(),
            value: format!("{:.1}%", sys.global_cpu_usage()),
            description: cpu_model.clone(),
        });

        let components = Components::new_with_refreshed_list();
        for comp in &components {
            if (comp.label().to_lowercase().contains("cpu")
                || comp.label().to_lowercase().contains("core"))
                && comp.temperature().is_some()
            {
                entries.push(MonitorEntry {
                    entry_type: "CPU Temp".to_string(),
                    subtype: "Physical".to_string(),
                    value: format!("{:.1}°C", comp.temperature().unwrap()),
                    description: comp.label().to_string(),
                });
            }
        }

        // RAM
        entries.push(MonitorEntry {
            entry_type: "RAM".to_string(),
            subtype: "Physical".to_string(),
            value: format!(
                "{} / {}",
                format_bytes(sys.used_memory()),
                format_bytes(sys.total_memory())
            ),
            description: "System Memory".to_string(),
        });

        // Network
        let networks = Networks::new_with_refreshed_list();
        for (name, data) in &networks {
            let mut subtype = "Virtual".to_string();
            let sys_path = format!("/sys/class/net/{}", name);
            let path = std::path::Path::new(&sys_path);
            if path.join("wireless").exists() || path.join("phy80211").exists() {
                subtype = "Wireless".to_string();
            } else if path.join("device").exists() {
                subtype = "Physical".to_string();
            }

            entries.push(MonitorEntry {
                entry_type: "Network".to_string(),
                subtype,
                value: format!(
                    "RX {}, TX {}",
                    format_bytes(data.total_received()),
                    format_bytes(data.total_transmitted())
                ),
                description: name.to_string(),
            });
        }

        // Disk
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            let subtype = if disk.is_removable() {
                "Removable".to_string()
            } else if disk.name().to_string_lossy().contains("loop") {
                "Virtual".to_string()
            } else {
                "Physical".to_string()
            };

            entries.push(MonitorEntry {
                entry_type: "Disk".to_string(),
                subtype,
                value: format!(
                    "{} / {}",
                    format_bytes(disk.total_space() - disk.available_space()),
                    format_bytes(disk.total_space())
                ),
                description: format!(
                    "{:?} ({})",
                    disk.name(),
                    disk.file_system().to_string_lossy()
                ),
            });
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["", "Type", "Subtype", "Value", "Description"]);

                for e in entries {
                    let type_emoji = match e.entry_type.as_str() {
                        "CPU" | "CPU Temp" => Emoji::Cpu.get(),
                        "RAM" => Emoji::Ram.get(),
                        "Network" => Emoji::Network.get(),
                        "Disk" => Emoji::Disk.get(),
                        _ => "📊",
                    };
                    let subtype_emoji = match e.subtype.to_lowercase().as_str() {
                        "physical" => Emoji::Physical.get(),
                        "wireless" => Emoji::Wireless.get(),
                        "virtual" => Emoji::Virtual.get(),
                        _ => "📟",
                    };

                    let display_value = if e.entry_type == "RAM" || e.entry_type == "Disk" {
                        let parts: Vec<&str> = e.value.split(" / ").collect();
                        if parts.len() == 2 {
                            format!(
                                "{} {} / {} {}",
                                Emoji::Used.get(),
                                parts[0],
                                Emoji::Total.get(),
                                parts[1]
                            )
                        } else {
                            e.value.clone()
                        }
                    } else if e.entry_type == "Network" {
                        e.value.replace("RX", "📥 RX").replace("TX", "📤 TX")
                    } else {
                        e.value.clone()
                    };

                    table.add_row(vec![
                        format!("{} {}", type_emoji, subtype_emoji),
                        e.entry_type,
                        e.subtype,
                        display_value,
                        e.description,
                    ]);
                }
                println!(
                    "=== System Monitor ({} Used / {} Total) ===",
                    Emoji::Used.get(),
                    Emoji::Total.get()
                );
                println!("{}", table);
            }

            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&entries)?);
            }
            OutputFormat::Original => {
                // Keep it similar to table but maybe simpler?
                // The user specifically asked for emojis in table mode.
                for e in entries {
                    println!("{}: {} ({})", e.entry_type, e.value, e.description);
                }
            }
        }

        Ok(())
    }
    fn as_string(&self) -> String {
        "sysinfo (Rust library)".to_string()
    }
}
