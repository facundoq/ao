use super::{
    DiskManager, ExecutableCommand, GroupManager, MonitorManager, PackageManager, ServiceManager,
    UserManager,
};
use anyhow::{Context, Result};
use std::process::Command;
use sysinfo::{Components, Disks, Networks, System};

pub struct SystemCommand {
    binary: String,
    args: Vec<String>,
    stdin_data: Option<String>,
    ignore_exit_code: bool,
}

impl SystemCommand {
    pub fn new(binary: &str) -> Self {
        Self {
            binary: binary.to_string(),
            args: Vec::new(),
            stdin_data: None,
            ignore_exit_code: false,
        }
    }

    pub fn ignore_exit_code(mut self) -> Self {
        self.ignore_exit_code = true;
        self
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn args(mut self, args: &[String]) -> Self {
        for arg in args {
            self.args.push(arg.clone());
        }
        self
    }

    pub fn stdin(mut self, data: &str) -> Self {
        self.stdin_data = Some(data.to_string());
        self
    }
}

impl ExecutableCommand for SystemCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(&self.args);

        if let Some(data) = &self.stdin_data {
            cmd.stdin(std::process::Stdio::piped());
            use std::io::Write;
            let mut child = cmd
                .spawn()
                .with_context(|| format!("Failed to spawn {}", self.binary))?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(data.as_bytes())
                    .with_context(|| format!("Failed to write to {} stdin", self.binary))?;
            }
            let status = child
                .wait()
                .with_context(|| format!("Failed to wait on {}", self.binary))?;
            if !self.ignore_exit_code && !status.success() {
                anyhow::bail!("{} failed with status {}", self.binary, status);
            }
        } else {
            let status = cmd
                .status()
                .with_context(|| format!("Failed to execute {}", self.binary))?;
            if !self.ignore_exit_code && !status.success() {
                anyhow::bail!("{} failed with status {}", self.binary, status);
            }
        }
        Ok(())
    }

    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Executing: {}", self.as_string());
        if self.stdin_data.is_some() {
            println!("[DRY RUN] (With secure stdin payload)");
        }
        Ok(())
    }

    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
    }

    fn as_string(&self) -> String {
        format!("{} {}", self.binary, self.args.join(" "))
    }
}

pub struct CompoundCommand {
    commands: Vec<Box<dyn ExecutableCommand>>,
}

impl CompoundCommand {
    pub fn new(commands: Vec<Box<dyn ExecutableCommand>>) -> Self {
        Self { commands }
    }
}

impl ExecutableCommand for CompoundCommand {
    fn execute(&self) -> Result<()> {
        for cmd in &self.commands {
            cmd.execute()?;
        }
        Ok(())
    }

    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Executing: {}", self.as_string());
        Ok(())
    }

    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
    }

    fn as_string(&self) -> String {
        self.commands
            .iter()
            .map(|cmd| cmd.as_string())
            .collect::<Vec<String>>()
            .join(" && ")
    }
}

pub struct Apt;

impl PackageManager for Apt {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(CompoundCommand::new(vec![
            Box::new(SystemCommand::new("apt").arg("update")),
            Box::new(SystemCommand::new("apt").arg("upgrade").arg("-y")),
        ])))
    }

    fn install(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apt")
                .arg("install")
                .arg("-y")
                .arg("--")
                .args(packages),
        ))
    }

    fn remove(&self, packages: &[String], purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("apt");
        if purge {
            cmd = cmd.arg("purge");
        } else {
            cmd = cmd.arg("remove");
        }
        Ok(Box::new(cmd.arg("-y").arg("--").args(packages)))
    }

    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apt").arg("search").arg("--").arg(query),
        ))
    }

    fn list(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("apt").arg("list").arg("--installed"),
        ))
    }
}

pub struct Monitor;

impl MonitorManager for Monitor {
    fn live_stats(&self) -> Result<()> {
        let mut sys = System::new_all();
        // Wait a bit to get accurate CPU usage
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_all();

        println!("=== System Monitor ===");
        println!("CPU usage: {:.1}%", sys.global_cpu_usage());

        let components = Components::new_with_refreshed_list();
        for comp in &components {
            if comp.label().to_lowercase().contains("cpu")
                || comp.label().to_lowercase().contains("core")
            {
                if let Some(temp) = comp.temperature() {
                    println!("Temperature ({}): {:.1}°C", comp.label(), temp);
                }
            }
        }

        println!("RAM: {} / {} bytes", sys.used_memory(), sys.total_memory());
        println!("Swap: {} / {} bytes", sys.used_swap(), sys.total_swap());

        let networks = Networks::new_with_refreshed_list();
        println!("\n=== Networks ===");
        for (interface_name, data) in &networks {
            println!(
                "{}: RX {} bytes, TX {} bytes",
                interface_name,
                data.total_received(),
                data.total_transmitted()
            );
        }

        let disks = Disks::new_with_refreshed_list();
        println!("\n=== Disks ===");
        for disk in &disks {
            println!(
                "{:?}: {} / {} bytes",
                disk.name(),
                disk.total_space() - disk.available_space(),
                disk.total_space()
            );
        }

        Ok(())
    }
}

pub struct Disk;

impl DiskManager for Disk {
    fn list(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("lsblk")))
    }

    fn mount(
        &self,
        device: &str,
        path: &str,
        fstype: Option<&str>,
        options: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("mount");

        if let Some(fs) = fstype {
            cmd = cmd.arg("-t").arg(fs);
        }
        if let Some(opts) = options {
            cmd = cmd.arg("-o").arg(opts);
        }

        Ok(Box::new(cmd.arg("--").arg(device).arg(path)))
    }

    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("umount");

        if lazy {
            cmd = cmd.arg("-l");
        }
        if force {
            cmd = cmd.arg("-f");
        }

        Ok(Box::new(cmd.arg("--").arg(target)))
    }

    fn usage(&self, path: &str, depth: Option<u32>) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("du");

        if let Some(d) = depth {
            cmd = cmd.arg("-h").arg(&format!("--max-depth={}", d));
        } else {
            cmd = cmd.arg("-sh");
        }

        Ok(Box::new(cmd.arg("--").arg(path)))
    }
}

pub struct Group;

impl GroupManager for Group {
    fn list(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("cat").arg("/etc/group")))
    }

    fn add(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("groupadd").arg("--").arg(groupname),
        ))
    }

    fn del(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("groupdel").arg("--").arg(groupname),
        ))
    }

    fn mod_group(&self, groupname: &str, gid: u32) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("groupmod")
                .arg("--gid")
                .arg(&gid.to_string())
                .arg("--")
                .arg(groupname),
        ))
    }
}

pub struct User;

impl UserManager for User {
    fn list(&self, _all: bool, _groups: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SystemCommand::new("cat").arg("/etc/passwd")))
    }

    fn add(
        &self,
        username: &str,
        groups: Option<&str>,
        shell: Option<&str>,
        system: bool,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("useradd").arg("-m");

        if system {
            cmd = cmd.arg("--system");
        }
        if let Some(s) = shell {
            cmd = cmd.arg("--shell").arg(s);
        }
        if let Some(g) = groups {
            cmd = cmd.arg("--groups").arg(g);
        }

        Ok(Box::new(cmd.arg("--").arg(username)))
    }

    fn del(&self, username: &str, purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("userdel");
        if purge {
            cmd = cmd.arg("-r");
        }
        Ok(Box::new(cmd.arg("--").arg(username)))
    }

    fn mod_user(
        &self,
        username: &str,
        action: &str,
        value: &str,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let mut cmd = SystemCommand::new("usermod");
        match action {
            "add-group" => {
                cmd = cmd.arg("-aG").arg(value);
            }
            "del-group" => {
                return Ok(Box::new(
                    SystemCommand::new("gpasswd")
                        .arg("-d")
                        .arg(username)
                        .arg("--")
                        .arg(value),
                ));
            }
            "shell" => {
                cmd = cmd.arg("-s").arg(value);
            }
            "home" => {
                cmd = cmd.arg("-d").arg(value).arg("-m");
            }
            _ => anyhow::bail!("Unsupported user modification action: {}", action),
        }
        Ok(Box::new(cmd.arg("--").arg(username)))
    }

    fn passwd(&self, username: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(PasswdCommand {
            username: username.to_string(),
        }))
    }
}

pub struct PasswdCommand {
    username: String,
}

impl ExecutableCommand for PasswdCommand {
    fn execute(&self) -> Result<()> {
        let password = rpassword::prompt_password("New password: ")
            .context("Failed to read password from stdin")?;
        let confirm_password = rpassword::prompt_password("Retype new password: ")
            .context("Failed to read password from stdin")?;

        if password != confirm_password {
            anyhow::bail!("Passwords do not match");
        }

        let creds = format!("{}:{}", self.username, password);
        let cmd = SystemCommand::new("chpasswd").stdin(&creds);
        cmd.execute()
    }

    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Executing: chpasswd (for user {})", self.username);
        println!("[DRY RUN] (With secure stdin payload)");
        Ok(())
    }

    fn print(&self) -> Result<()> {
        println!("chpasswd (for user {})", self.username);
        Ok(())
    }

    fn as_string(&self) -> String {
        format!("chpasswd (for user {})", self.username)
    }
}

pub struct Systemd;

impl ServiceManager for Systemd {
    fn list(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("systemctl")
                .arg("list-units")
                .arg("--type=service"),
        ))
    }

    fn up(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("systemctl")
                .arg("enable")
                .arg("--now")
                .arg("--")
                .arg(service),
        ))
    }

    fn down(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("systemctl")
                .arg("disable")
                .arg("--now")
                .arg("--")
                .arg(service),
        ))
    }

    fn restart(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("systemctl")
                .arg("restart")
                .arg("--")
                .arg(service),
        ))
    }

    fn reload(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("systemctl")
                .arg("reload")
                .arg("--")
                .arg(service),
        ))
    }

    fn status(&self, service: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(
            SystemCommand::new("systemctl")
                .arg("status")
                .arg("--")
                .arg(service)
                .ignore_exit_code(),
        ))
    }
}
