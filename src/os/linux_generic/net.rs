use anyhow::Result;
use clap::{ArgMatches, Command as ClapCommand, FromArgMatches, Args};
use crate::os::{NetManager, ExecutableCommand, Domain, NetInterfaceInfo, NetIpInfo, NetRouteInfo, FwRuleInfo, OutputFormat};
use crate::cli::{NetArgs, NetAction, FwAction, WifiAction};
use super::common::SystemCommand;
use std::process::Command;

pub struct StandardNet;

impl Domain for StandardNet {
    fn name(&self) -> &'static str { "net" }
    fn command(&self) -> ClapCommand {
        NetArgs::augment_args(ClapCommand::new("net").about("Manage networking (interfaces, IPs, routes, firewall, Wi-Fi)"))
    }
    fn execute(&self, matches: &ArgMatches, _app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = NetArgs::from_arg_matches(matches)?;
        match &args.action {
            NetAction::Interfaces { format } => self.interfaces(*format),
            NetAction::Ips { format } => self.ips(*format),
            NetAction::Routes { format } => self.routes(*format),
            NetAction::Fw { action } => match action {
                FwAction::Status { format } => self.fw_status(*format),
                FwAction::Allow { rule } => self.fw_allow(rule),
                FwAction::Deny { rule } => self.fw_deny(rule),
            },
            NetAction::Wifi { action } => match action {
                WifiAction::Scan => self.wifi_scan(),
                WifiAction::Connect { ssid } => self.wifi_connect(ssid),
            },
        }
    }
}

impl NetManager for StandardNet {
    fn interfaces(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(NetInterfacesCommand { format }))
    }
    fn ips(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(NetIpsCommand { format }))
    }
    fn routes(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(NetRoutesCommand { format }))
    }
    fn fw_status(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(FwStatusCommand { format }))
    }
    fn fw_allow(&self, rule: &str) -> Result<Box<dyn ExecutableCommand>> {
        if std::path::Path::new("/usr/sbin/ufw").exists() {
            Ok(Box::new(SystemCommand::new("ufw").arg("allow").arg(rule)))
        } else {
            Ok(Box::new(SystemCommand::new("firewall-cmd").arg("--add-rich-rule").arg(rule).arg("--permanent")))
        }
    }
    fn fw_deny(&self, rule: &str) -> Result<Box<dyn ExecutableCommand>> {
        if std::path::Path::new("/usr/sbin/ufw").exists() {
            Ok(Box::new(SystemCommand::new("ufw").arg("deny").arg(rule)))
        } else {
            Ok(Box::new(SystemCommand::new("firewall-cmd").arg("--add-rich-rule").arg(rule).arg("--permanent")))
        }
    }
    fn wifi_scan(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("nmcli").arg("dev").arg("wifi").arg("list")))
    }
    fn wifi_connect(&self, ssid: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("nmcli").arg("dev").arg("wifi").arg("connect").arg(ssid)))
    }
}

pub struct NetInterfacesCommand { pub format: OutputFormat }
impl ExecutableCommand for NetInterfacesCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("ip").arg("addr").execute();
        }

        let output = Command::new("ip").arg("--json").arg("addr").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        #[derive(serde::Deserialize)]
        struct RawInterface { ifname: String, operstate: String, mtu: u32, address: Option<String> }
        let raw: Vec<RawInterface> = serde_json::from_str(&stdout)?;
        let interfaces: Vec<NetInterfaceInfo> = raw.into_iter().map(|r| NetInterfaceInfo {
            name: r.ifname,
            state: r.operstate,
            mtu: r.mtu,
            mac: r.address.unwrap_or_default(),
        }).collect();

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Interface", "State", "MTU", "MAC"]);
                for iface in interfaces {
                    table.add_row(vec![iface.name, iface.state, iface.mtu.to_string(), iface.mac]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&interfaces)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&interfaces)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] ip addr (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("ip addr (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("ip addr --format {:?}", self.format) }
    fn is_structured(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original)
    }
}

pub struct NetIpsCommand { pub format: OutputFormat }
impl ExecutableCommand for NetIpsCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("ip").arg("addr").execute();
        }

        let output = Command::new("ip").arg("--json").arg("addr").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        #[derive(serde::Deserialize)]
        struct IpAddrEntry { ifname: String, addr_info: Vec<IpAddrInfo> }
        #[derive(serde::Deserialize)]
        struct IpAddrInfo { family: String, local: String }

        let raw: Vec<IpAddrEntry> = serde_json::from_str(&stdout)?;
        let mut ips = Vec::new();
        for entry in raw {
            for info in entry.addr_info {
                ips.push(NetIpInfo {
                    interface: entry.ifname.clone(),
                    family: info.family,
                    address: info.local,
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Interface", "Family", "Address"]);
                for ip in ips {
                    table.add_row(vec![ip.interface, ip.family, ip.address]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&ips)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&ips)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] ip addr (for IPs) (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("ip addr (for IPs) (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("ip addr --format {:?}", self.format) }
    fn is_structured(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original)
    }
}

pub struct NetRoutesCommand { pub format: OutputFormat }
impl ExecutableCommand for NetRoutesCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            return SystemCommand::new("ip").arg("route").execute();
        }

        let output = Command::new("ip").arg("--json").arg("route").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        #[derive(serde::Deserialize)]
        struct RawRoute { dst: String, gateway: Option<String>, dev: String }
        let raw: Vec<RawRoute> = serde_json::from_str(&stdout)?;
        let routes: Vec<NetRouteInfo> = raw.into_iter().map(|r| NetRouteInfo {
            destination: r.dst,
            gateway: r.gateway,
            interface: r.dev,
        }).collect();

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Destination", "Gateway", "Interface"]);
                for r in routes {
                    table.add_row(vec![r.destination, r.gateway.unwrap_or_default(), r.interface]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&routes)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&routes)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] ip route (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("ip route (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("ip route --format {:?}", self.format) }
    fn is_structured(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original)
    }
}

pub struct FwStatusCommand { pub format: OutputFormat }
impl ExecutableCommand for FwStatusCommand {
    fn execute(&self) -> Result<()> {
        if matches!(self.format, OutputFormat::Original) {
            if std::path::Path::new("/usr/sbin/ufw").exists() {
                return SystemCommand::new("ufw").arg("status").execute();
            } else {
                return SystemCommand::new("firewall-cmd").arg("--list-all").execute();
            }
        }

        let (rules, status_msg) = if std::path::Path::new("/usr/sbin/ufw").exists() {
            let output = Command::new("ufw").arg("status").output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_ufw(&stdout)
        } else {
            let output = Command::new("firewall-cmd").arg("--list-all").output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_firewalld(&stdout)
        };

        match self.format {
            OutputFormat::Table => {
                println!("Status: {}", status_msg);
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["To", "Action", "From"]);
                for r in rules {
                    table.add_row(vec![r.to, r.action, r.from]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => { println!("{}", serde_json::to_string_pretty(&rules)?); }
            OutputFormat::Yaml => { println!("{}", serde_yaml::to_string(&rules)?); }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> { println!("[DRY RUN] Firewall status (format: {:?})", self.format); Ok(()) }
    fn print(&self) -> Result<()> { println!("Firewall status (format: {:?})", self.format); Ok(()) }
    fn as_string(&self) -> String { format!("firewall status --format {:?}", self.format) }
    fn is_structured(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original)
    }
}

impl FwStatusCommand {
    fn parse_ufw(&self, stdout: &str) -> (Vec<FwRuleInfo>, String) {
        let mut rules = Vec::new();
        let mut status = "unknown".to_string();
        let mut parsing_rules = false;
        for line in stdout.lines() {
            if line.starts_with("Status: ") {
                status = line["Status: ".len()..].to_string();
            } else if line.contains("--") && line.contains("Action") {
                parsing_rules = true;
                continue;
            } else if parsing_rules && !line.trim().is_empty() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    rules.push(FwRuleInfo {
                        to: parts[0].to_string(),
                        action: parts[1].to_string(),
                        from: parts[2].to_string(),
                    });
                }
            }
        }
        (rules, status)
    }

    fn parse_firewalld(&self, stdout: &str) -> (Vec<FwRuleInfo>, String) {
        let mut rules = Vec::new();
        for line in stdout.lines() {
            if line.contains("services:") || line.contains("ports:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    for item in parts[1].split_whitespace() {
                        rules.push(FwRuleInfo {
                            to: item.to_string(),
                            action: "allow".to_string(),
                            from: "any".to_string(),
                        });
                    }
                }
            }
        }
        (rules, "active".to_string())
    }
}
