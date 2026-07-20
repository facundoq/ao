use super::common::{BIN_LSBLK, Emoji, SystemCommand, is_completing_action};

use crate::cli::{PartitionAction, PartitionArgs};
use crate::os::{DiskInfo, Domain, ExecutableCommand, OutputFormat, PartitionManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardPartition;

impl Domain for StandardPartition {
    fn name(&self) -> &'static str {
        "partition"
    }
    fn command(&self) -> ClapCommand {
        PartitionArgs::augment_args(
            ClapCommand::new("partition").about("Manage partitions and mounts"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = PartitionArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(PartitionAction::List { format, show_loop }) => self.ls(*format, *show_loop),
            Some(PartitionAction::Mount {
                device,
                path,
                fstype,
                options,
            }) => self.mount(device, path, fstype.as_deref(), options.as_deref()),
            Some(PartitionAction::Unmount {
                target,
                lazy,
                force,
            }) => self.unmount(target, *lazy, *force),
            Some(PartitionAction::Usage { path, depth }) => self.usage(path, *depth),
            None => self.ls(OutputFormat::Table, false),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        _last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_action(words, self.name(), "mount", 1) {
            let mut devices = Vec::new();
            if let Ok(output) = Command::new(BIN_LSBLK)
                .arg("-n")
                .arg("-o")
                .arg("NAME,PATH")
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                devices.extend(
                    stdout
                        .lines()
                        .filter_map(|l| l.split_whitespace().last())
                        .map(|s| s.trim().to_string()),
                );
            }
            return Ok(devices);
        }
        if is_completing_action(words, self.name(), "unmount", 1) {
            return self.get_mount_points();
        }
        Ok(vec![])
    }
}

impl PartitionManager for StandardPartition {
    fn ls(&self, format: OutputFormat, show_loop: bool) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            let mut cmd = SystemCommand::new(BIN_LSBLK);
            if !show_loop {
                cmd = cmd.arg("-e").arg("7");
            }
            return Ok(Box::new(cmd));
        }
        Ok(Box::new(PartitionListCommand { format, show_loop }))
    }
    fn mount(
        &self,
        device: &str,
        path: &str,
        fstype: Option<&str>,
        options: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(PartitionMountCommand {
            device: device.to_string(),
            path: path.to_string(),
            fstype: fstype.map(|s| s.to_string()),
            options: options.map(|s| s.to_string()),
        }))
    }
    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(PartitionUnmountCommand {
            target: target.to_string(),
            lazy,
            force,
        }))
    }
    fn usage(&self, path: &str, depth: Option<u32>) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            PartitionUsageCommand {
                path: path.to_string(),
                depth,
                ignore_exit_code: false,
            }
            .ignore_exit_code(),
        ))
    }

    fn get_mount_points(&self) -> Result<Vec<String>> {
        let output = Command::new(BIN_LSBLK)
            .arg("-n")
            .arg("-o")
            .arg("MOUNTPOINT")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.trim().to_string())
            .collect())
    }
}

pub struct PartitionListCommand {
    pub format: OutputFormat,
    pub show_loop: bool,
}
impl ExecutableCommand for PartitionListCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = Command::new(BIN_LSBLK);
        cmd.arg("--json")
            .arg("-o")
            .arg("NAME,PATH,SIZE,MOUNTPOINT,FSTYPE,TYPE,ROTA,TRAN");
        if !self.show_loop {
            cmd.arg("-e").arg("7");
        }
        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        #[derive(serde::Deserialize)]
        struct LsblkOutput {
            blockdevices: Vec<DiskInfo>,
        }
        let raw: LsblkOutput = serde_json::from_str(&stdout)?;

        fn flatten_disks(disks: Vec<DiskInfo>, parent_tran: Option<String>) -> Vec<DiskInfo> {
            let mut flat = Vec::new();
            for mut d in disks {
                if d.tran.is_none() {
                    d.tran = parent_tran.clone();
                }
                let children = d.children.take();
                let current_tran = d.tran.clone();
                flat.push(d);
                if let Some(c) = children {
                    flat.extend(flatten_disks(c, current_tran));
                }
            }
            flat
        }

        let disks = flatten_disks(raw.blockdevices, None);
        let mut partitions: Vec<_> = disks
            .into_iter()
            .filter(|d| {
                if !self.show_loop && d.device_type == "loop" {
                    return false;
                }
                d.device_type == "part"
                    || d.device_type == "crypt"
                    || d.device_type == "lvm"
                    || d.device_type == "rom"
                    || d.mountpoint.is_some()
            })
            .collect();

        partitions.sort_by(|a, b| a.name.cmp(&b.name));

        let sys_disks = sysinfo::Disks::new_with_refreshed_list();
        for d in &mut partitions {
            if let Some(disk) = d.mountpoint.as_ref().and_then(|mp| {
                sys_disks
                    .iter()
                    .find(|disk| disk.mount_point() == std::path::Path::new(mp))
            }) {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                d.used_space = Some(super::common::format_bytes(used));
                let pct = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                d.usage_pct = Some(format!("{:.1}%", pct));
            }
        }

        let config = crate::config::Config::load().unwrap_or_default();

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "",
                    "Name",
                    "Path",
                    "Used",
                    "Size",
                    "Usage %",
                    "Type",
                    "Mountpoint",
                    "FSType",
                ]);
                for d in partitions {
                    let usage_str = d.usage_pct.clone().unwrap_or_default();
                    let displayed_usage = if config.ui.use_colors && !usage_str.is_empty() {
                        use colored::Colorize;
                        if let Some(pct) = usage_str
                            .strip_suffix('%')
                            .and_then(|s| s.parse::<f64>().ok())
                        {
                            let theme = &config.ui.theme;
                            let color = if pct >= 90.0 {
                                theme.storage_color_high
                            } else if pct >= 50.0 {
                                theme.storage_color_med
                            } else {
                                theme.storage_color_low
                            };
                            usage_str
                                .truecolor(color[0], color[1], color[2])
                                .to_string()
                        } else {
                            usage_str
                        }
                    } else {
                        usage_str
                    };

                    table.add_row(vec![
                        Emoji::Disk.get().to_string(),
                        d.name,
                        d.path,
                        d.used_space.clone().unwrap_or_default(),
                        d.size,
                        displayed_usage,
                        d.device_type,
                        d.mountpoint.unwrap_or_default(),
                        d.fstype.unwrap_or_default(),
                    ]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&partitions)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&partitions)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn as_string(&self) -> String {
        if self.show_loop {
            "lsblk --json".to_string()
        } else {
            "lsblk --json -e 7".to_string()
        }
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

pub struct PartitionMountCommand {
    pub device: String,
    pub path: String,
    pub fstype: Option<String>,
    pub options: Option<String>,
}
impl ExecutableCommand for PartitionMountCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = SystemCommand::new("mount");
        if let Some(ref fs) = self.fstype {
            cmd = cmd.arg("-t").arg(fs);
        }
        if let Some(ref opts) = self.options {
            cmd = cmd.arg("-o").arg(opts);
        }
        cmd.arg("--").arg(&self.device).arg(&self.path).execute()
    }
    fn as_string(&self) -> String {
        let mut s = "mount".to_string();
        if let Some(ref fs) = self.fstype {
            s.push_str(&format!(" -t {}", fs));
        }
        if let Some(ref opts) = self.options {
            s.push_str(&format!(" -o {}", opts));
        }
        s.push_str(&format!(" -- {} {}", self.device, self.path));
        s
    }
}

pub struct PartitionUnmountCommand {
    pub target: String,
    pub lazy: bool,
    pub force: bool,
}
impl ExecutableCommand for PartitionUnmountCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = SystemCommand::new("umount");
        if self.lazy {
            cmd = cmd.arg("-l");
        }
        if self.force {
            cmd = cmd.arg("-f");
        }
        cmd.arg("--").arg(&self.target).execute()
    }
    fn as_string(&self) -> String {
        let mut s = "umount".to_string();
        if self.lazy {
            s.push_str(" -l");
        }
        if self.force {
            s.push_str(" -f");
        }
        s.push_str(&format!(" -- {}", self.target));
        s
    }
}

pub struct PartitionUsageCommand {
    pub path: String,
    pub depth: Option<u32>,
    pub ignore_exit_code: bool,
}

impl PartitionUsageCommand {
    pub fn ignore_exit_code(mut self) -> Self {
        self.ignore_exit_code = true;
        self
    }
}

impl ExecutableCommand for PartitionUsageCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = SystemCommand::new("du");
        if self.ignore_exit_code {
            cmd = cmd.ignore_exit_code();
        }
        if let Some(d) = self.depth {
            cmd = cmd.arg("-h").arg(&format!("--max-depth={}", d));
        } else {
            cmd = cmd.arg("-sh");
        }
        cmd.arg("--").arg(&self.path).execute()
    }
    fn as_string(&self) -> String {
        if let Some(d) = self.depth {
            format!("du -h --max-depth={} -- {}", d, self.path)
        } else {
            format!("du -sh -- {}", self.path)
        }
    }
}
