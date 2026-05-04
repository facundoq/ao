use super::common::{Emoji, SystemCommand};
use crate::cli::{DiskAction, DiskArgs};
use crate::os::{DiskInfo, DiskManager, Domain, ExecutableCommand, OutputFormat};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardDisk;

impl Domain for StandardDisk {
    fn name(&self) -> &'static str {
        "disk"
    }
    fn command(&self) -> ClapCommand {
        DiskArgs::augment_args(ClapCommand::new("disk").about("Manage actual block devices"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = DiskArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(DiskAction::List {
                format,
                loop_devices,
            }) => self.ls(*format, *loop_devices),
            None => self.ls(OutputFormat::Table, false),
        }
    }
    fn complete(
        &self,
        _line: &str,
        _words: &[&str],
        _last_word_complete: bool,
    ) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

impl DiskManager for StandardDisk {
    fn ls(&self, format: OutputFormat, show_loop: bool) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            return Ok(Box::new(SystemCommand::new("lsblk").arg("-d")));
        }
        Ok(Box::new(DiskListCommand { format, show_loop }))
    }

    fn get_devices(&self) -> Result<Vec<String>> {
        let output = Command::new("lsblk")
            .arg("-d")
            .arg("-n")
            .arg("-o")
            .arg("NAME,PATH")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter_map(|l| l.split_whitespace().last())
            .map(|s| s.trim().to_string())
            .collect())
    }
}

pub struct DiskListCommand {
    pub format: OutputFormat,
    pub show_loop: bool,
}

impl ExecutableCommand for DiskListCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("lsblk")
            .arg("-d") // only devices, no partitions
            .arg("--json")
            .arg("-o")
            .arg("NAME,PATH,SIZE,FSTYPE,TYPE,ROTA,TRAN")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        #[derive(serde::Deserialize)]
        struct LsblkOutput {
            blockdevices: Vec<DiskInfo>,
        }
        let raw: LsblkOutput = serde_json::from_str(&stdout)?;

        let mut disks = Vec::new();
        for d in raw.blockdevices {
            if d.device_type == "disk" || (self.show_loop && d.device_type == "loop") {
                disks.push(d);
            }
        }

        disks.sort_by(|a, b| a.name.cmp(&b.name));

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "",
                    "Name",
                    "Path",
                    "Size",
                    "Medium",
                    "Transport",
                    "SMART Status",
                ]);
                for d in disks {
                    let type_emoji = match d.device_type.as_str() {
                        "loop" => Emoji::Loop.get(),
                        _ => {
                            if let Some(ref tran) = d.tran {
                                match tran.as_str() {
                                    "nvme" => Emoji::Nvme.get(),
                                    "sata" | "usb" => {
                                        if d.rota {
                                            Emoji::Hdd.get()
                                        } else {
                                            Emoji::Ssd.get()
                                        }
                                    }
                                    _ => Emoji::Disk.get(),
                                }
                            } else if d.rota {
                                Emoji::Hdd.get()
                            } else {
                                Emoji::Ssd.get()
                            }
                        }
                    };

                    let medium = if d.device_type == "loop" {
                        "Loop"
                    } else if d.rota {
                        "HDD"
                    } else {
                        "SSD"
                    };

                    let transport = d
                        .tran
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string())
                        .to_uppercase();

                    let smart_status = get_smart_status(&d.path);

                    table.add_row(vec![
                        format!("{} {}", Emoji::Physical.get(), type_emoji),
                        d.name,
                        d.path.clone(),
                        d.size,
                        medium.to_string(),
                        transport,
                        smart_status,
                    ]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&disks)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&disks)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn as_string(&self) -> String {
        "lsblk -d --json".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

fn get_smart_status(path: &str) -> String {
    // Try smartctl JSON output
    let output = Command::new("smartctl")
        .arg("-j")
        .arg("-H")
        .arg(path)
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if let Some(passed) = serde_json::from_str::<serde_json::Value>(&stdout)
            .ok()
            .and_then(|json| json.get("smart_status")?.get("passed")?.as_bool())
        {
            return if passed {
                "PASSED".to_string()
            } else {
                "FAILED".to_string()
            };
        }
    }

    // Fallback if smartctl fails or format is unrecognized
    "Unknown".to_string()
}
