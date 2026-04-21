use super::common::{Emoji, SystemCommand, is_completing_arg};
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
        DiskArgs::augment_args(ClapCommand::new("disk").about("Manage disks and storage"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = DiskArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(DiskAction::Ls { format }) => self.ls(*format),
            Some(DiskAction::Mount {
                device,
                path,
                fstype,
                options,
            }) => self.mount(device, path, fstype.as_deref(), options.as_deref()),
            Some(DiskAction::Unmount {
                target,
                lazy,
                force,
            }) => self.unmount(target, *lazy, *force),
            Some(DiskAction::Usage { path, depth }) => self.usage(path, *depth),
            None => self.ls(OutputFormat::Table),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "disk", "mount"], 1, last_word_complete) {
            return self.get_devices();
        }
        if is_completing_arg(words, &["ao", "disk", "unmount"], 1, last_word_complete) {
            return self.get_mount_points();
        }
        Ok(vec![])
    }
}

impl DiskManager for StandardDisk {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            return Ok(Box::new(SystemCommand::new("lsblk")));
        }
        Ok(Box::new(DiskListCommand { format }))
    }
    fn mount(
        &self,
        device: &str,
        path: &str,
        fstype: Option<&str>,
        options: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(DiskMountCommand {
            device: device.to_string(),
            path: path.to_string(),
            fstype: fstype.map(|s| s.to_string()),
            options: options.map(|s| s.to_string()),
        }))
    }
    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(DiskUnmountCommand {
            target: target.to_string(),
            lazy,
            force,
        }))
    }
    fn usage(&self, path: &str, depth: Option<u32>) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(DiskUsageCommand {
            path: path.to_string(),
            depth,
        }))
    }

    fn get_devices(&self) -> Result<Vec<String>> {
        let output = Command::new("lsblk")
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

    fn get_mount_points(&self) -> Result<Vec<String>> {
        let output = Command::new("lsblk")
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

pub struct DiskListCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for DiskListCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("lsblk")
            .arg("--json")
            .arg("-o")
            .arg("NAME,PATH,SIZE,MOUNTPOINT,FSTYPE,TYPE,ROTA,TRAN")
            .output()?;
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

        let mut disks = flatten_disks(raw.blockdevices, None);

        // Sort: physical first, virtual later.
        let is_physical = |t: &str| matches!(t, "disk" | "part" | "rom" | "crypt");

        disks.sort_by(|a, b| {
            let a_phys = is_physical(&a.device_type);
            let b_phys = is_physical(&b.device_type);
            if a_phys == b_phys {
                a.name.cmp(&b.name)
            } else if a_phys {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "",
                    "Name",
                    "Path",
                    "Size",
                    "Type",
                    "Class",
                    "Mountpoint",
                    "FSType",
                ]);
                for d in disks {
                    let class_is_phys = is_physical(&d.device_type);
                    let class_emoji = if class_is_phys {
                        Emoji::Physical.get()
                    } else {
                        Emoji::Virtual.get()
                    };
                    let class_text = if class_is_phys { "Physical" } else { "Virtual" };

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
                            } else if d.device_type == "disk" || d.device_type == "part" {
                                if d.rota {
                                    Emoji::Hdd.get()
                                } else {
                                    Emoji::Ssd.get()
                                }
                            } else {
                                Emoji::Disk.get()
                            }
                        }
                    };

                    table.add_row(vec![
                        format!("{} {}", class_emoji, type_emoji),
                        d.name,
                        d.path,
                        d.size,
                        d.device_type,
                        class_text.to_string(),
                        d.mountpoint.unwrap_or_default(),
                        d.fstype.unwrap_or_default(),
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
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] lsblk --json (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("lsblk --json (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "lsblk --json".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

pub struct DiskMountCommand {
    pub device: String,
    pub path: String,
    pub fstype: Option<String>,
    pub options: Option<String>,
}
impl ExecutableCommand for DiskMountCommand {
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
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] {}", self.as_string());
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
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

pub struct DiskUnmountCommand {
    pub target: String,
    pub lazy: bool,
    pub force: bool,
}
impl ExecutableCommand for DiskUnmountCommand {
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
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] {}", self.as_string());
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
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

pub struct DiskUsageCommand {
    pub path: String,
    pub depth: Option<u32>,
}
impl ExecutableCommand for DiskUsageCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = SystemCommand::new("du");
        if let Some(d) = self.depth {
            cmd = cmd.arg("-h").arg(&format!("--max-depth={}", d));
        } else {
            cmd = cmd.arg("-sh");
        }
        cmd.arg("--").arg(&self.path).execute()
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] {}", self.as_string());
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
    }
    fn as_string(&self) -> String {
        if let Some(d) = self.depth {
            format!("du -h --max-depth={} -- {}", d, self.path)
        } else {
            format!("du -sh -- {}", self.path)
        }
    }
}
