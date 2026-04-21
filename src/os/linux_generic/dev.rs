use super::common::SystemCommand;
use crate::cli::{BtAction, DevAction, DevArgs, PrintAction};
use crate::os::{DevManager, DeviceInfo, Domain, ExecutableCommand, OutputFormat, PrinterInfo};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardDev;

impl Domain for StandardDev {
    fn name(&self) -> &'static str {
        "dev"
    }
    fn command(&self) -> ClapCommand {
        DevArgs::augment_args(ClapCommand::new("dev").about("Manage connected devices"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = DevArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(DevAction::Ls { format }) => self.ls(*format),
            Some(DevAction::Pci { format }) => self.pci(*format),
            Some(DevAction::Usb { format }) => self.usb(*format),
            Some(DevAction::Bt { action }) => match action {
                BtAction::Status => self.bt_status(),
                BtAction::Scan => self.bt_scan(),
                BtAction::Pair { address } => self.bt_pair(address),
                BtAction::Connect { address } => self.bt_connect(address),
            },
            Some(DevAction::Print { action }) => match action {
                PrintAction::Ls { format } => self.ls_printers(*format),
            },
            None => self.ls(OutputFormat::Table),
        }
    }
}

impl DevManager for StandardDev {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(DevListAllCommand { format }))
    }
    fn pci(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(SystemCommand::new("lspci")));
        }
        Ok(Box::new(DevPciCommand { format }))
    }
    fn usb(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(SystemCommand::new("lsusb")));
        }
        Ok(Box::new(DevUsbCommand { format }))
    }
    fn bt_status(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("bluetoothctl").arg("show")))
    }
    fn bt_scan(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("bluetoothctl").arg("scan").arg("on"),
        ))
    }
    fn bt_pair(&self, address: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("bluetoothctl")
                .arg("pair")
                .arg("--")
                .arg(address),
        ))
    }
    fn bt_connect(&self, address: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("bluetoothctl")
                .arg("connect")
                .arg("--")
                .arg(address),
        ))
    }
    fn ls_printers(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if format == OutputFormat::Original {
            return Ok(Box::new(SystemCommand::new("lpstat").arg("-p")));
        }
        Ok(Box::new(DevPrintersCommand { format }))
    }
}

pub struct DevListAllCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for DevListAllCommand {
    fn execute(&self) -> Result<()> {
        let mut devices = Vec::new();

        // PCI Devices
        if let Ok(output) = Command::new("lspci").arg("-mm").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split("\" \"").collect();
                if parts.len() >= 4 {
                    devices.push(DeviceInfo {
                        bus: "PCI".to_string(),
                        device: parts[1].trim_matches('"').to_string(),
                        id: parts[2].trim_matches('"').to_string(),
                        description: parts[3..].join(" ").trim_matches('"').to_string(),
                    });
                }
            }
        }

        // USB Devices
        if let Ok(output) = Command::new("lsusb").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    devices.push(DeviceInfo {
                        bus: "USB".to_string(),
                        device: "Device".to_string(),
                        id: parts[5].to_string(),
                        description: parts[6..].join(" "),
                    });
                }
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                if let Ok((width, _)) = crossterm::terminal::size() {
                    table.set_width(width);
                }
                table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
                table.set_header(vec!["", "Type", "Class", "ID", "Description"]);
                for d in devices {
                    let emoji = match d.bus.as_str() {
                        "PCI" => "🏗️",
                        "USB" => "🔌",
                        _ => "📟",
                    };
                    table.add_row(vec![emoji, &d.bus, &d.device, &d.id, &d.description]);
                }
                println!("=== Connected Devices (🏗️ PCI / 🔌 USB) ===");
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&devices)?);
            }
            OutputFormat::Original => {
                for d in devices {
                    println!("[{}] {} {} {}", d.bus, d.id, d.device, d.description);
                }
            }
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("ao dev ls");
        Ok(())
    }
    fn as_string(&self) -> String {
        "lspci -mm && lsusb".to_string()
    }
}

pub struct DevPciCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for DevPciCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("lspci").arg("-mm").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split("\" \"").collect();
            if parts.len() >= 4 {
                devices.push(DeviceInfo {
                    bus: parts[0].trim_matches('"').to_string(),
                    device: parts[1].trim_matches('"').to_string(),
                    id: parts[2].trim_matches('"').to_string(),
                    description: parts[3..].join(" ").trim_matches('"').to_string(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                if let Ok((width, _)) = crossterm::terminal::size() {
                    table.set_width(width);
                }
                table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
                table.set_header(vec!["Bus", "Class", "Vendor", "Device"]);
                for d in devices {
                    table.add_row(vec![d.bus, d.device, d.id, d.description]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&devices)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] lspci (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("lspci (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "lspci -mm".to_string()
    }
}

pub struct DevUsbCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for DevUsbCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("lsusb").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                devices.push(DeviceInfo {
                    bus: parts[1].to_string(),
                    device: parts[3].trim_matches(':').to_string(),
                    id: parts[5].to_string(),
                    description: parts[6..].join(" "),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                if let Ok((width, _)) = crossterm::terminal::size() {
                    table.set_width(width);
                }
                table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
                table.set_header(vec!["Bus", "Device", "ID", "Description"]);
                for d in devices {
                    table.add_row(vec![d.bus, d.device, d.id, d.description]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&devices)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] lsusb (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("lsusb (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "lsusb".to_string()
    }
}

pub struct DevPrintersCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for DevPrintersCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("lpstat").arg("-p").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut printers = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                printers.push(PrinterInfo {
                    name: parts[1].to_string(),
                    status: parts[2..].join(" "),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                if let Ok((width, _)) = crossterm::terminal::size() {
                    table.set_width(width);
                }
                table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
                table.set_header(vec!["Printer", "Status"]);
                for p in printers {
                    table.add_row(vec![p.name, p.status]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&printers)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&printers)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] lpstat -p (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("lpstat -p (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "lpstat -p".to_string()
    }
}
