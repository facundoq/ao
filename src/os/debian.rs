use super::{PackageManager, ServiceManager, UserManager, GroupManager, DiskManager, MonitorManager};
use anyhow::{Context, Result};
use std::process::Command;
use sysinfo::{Components, Disks, Networks, System};

pub struct Apt;

impl PackageManager for Apt {
    fn update(&self) -> Result<()> {
        println!("Updating package lists...");
        let status = Command::new("apt")
            .arg("update")
            .status()
            .context("Failed to execute apt update")?;

        if !status.success() {
            anyhow::bail!("apt update failed with status {}", status);
        }

        println!("Applying upgrades...");
        let status = Command::new("apt")
            .args(["upgrade", "-y"])
            .status()
            .context("Failed to execute apt upgrade")?;

        if !status.success() {
            anyhow::bail!("apt upgrade failed with status {}", status);
        }

        Ok(())
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        let mut cmd = Command::new("apt");
        cmd.arg("install").arg("-y").arg("--").args(packages);
        let status = cmd.status().context("Failed to execute apt install")?;

        if !status.success() {
            anyhow::bail!("apt install failed with status {}", status);
        }
        Ok(())
    }

    fn remove(&self, packages: &[String], purge: bool) -> Result<()> {
        let mut cmd = Command::new("apt");
        if purge {
            cmd.arg("purge");
        } else {
            cmd.arg("remove");
        }
        cmd.arg("-y").arg("--").args(packages);
        let status = cmd.status().context("Failed to execute apt remove/purge")?;

        if !status.success() {
            anyhow::bail!("apt remove failed with status {}", status);
        }
        Ok(())
    }

    fn search(&self, query: &str) -> Result<()> {
        let status = Command::new("apt")
            .arg("search")
            .arg("--")
            .arg(query)
            .status()
            .context("Failed to execute apt search")?;

        if !status.success() {
            anyhow::bail!("apt search failed with status {}", status);
        }
        Ok(())
    }

    fn list(&self) -> Result<()> {
        let status = Command::new("apt")
            .arg("list")
            .arg("--installed")
            .status()
            .context("Failed to execute apt list")?;

        if !status.success() {
            anyhow::bail!("apt list failed with status {}", status);
        }
        Ok(())
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
            if comp.label().to_lowercase().contains("cpu") || comp.label().to_lowercase().contains("core") {
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
            println!("{}: RX {} bytes, TX {} bytes", interface_name, data.total_received(), data.total_transmitted());
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
    fn list(&self) -> Result<()> {
        // lsblk provides block devices, df provides usage. For simple testing we wrap lsblk.
        let status = Command::new("lsblk")
            .status()
            .context("Failed to execute lsblk")?;
        if !status.success() {
            anyhow::bail!("lsblk failed with status {}", status);
        }
        Ok(())
    }

    fn mount(&self, device: &str, path: &str, fstype: Option<&str>, options: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("mount");

        if let Some(fs) = fstype {
            cmd.arg("-t").arg(fs);
        }
        if let Some(opts) = options {
            cmd.arg("-o").arg(opts);
        }

        // Use double dash to separate options from paths
        cmd.arg("--").arg(device).arg(path);

        let status = cmd.status().context("Failed to execute mount")?;
        if !status.success() {
            anyhow::bail!("mount failed with status {}", status);
        }
        Ok(())
    }

    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<()> {
        let mut cmd = Command::new("umount");

        if lazy {
            cmd.arg("-l");
        }
        if force {
            cmd.arg("-f");
        }

        cmd.arg("--").arg(target);

        let status = cmd.status().context("Failed to execute umount")?;
        if !status.success() {
            anyhow::bail!("umount failed with status {}", status);
        }
        Ok(())
    }

    fn usage(&self, path: &str, depth: Option<u32>) -> Result<()> {
        let mut cmd = Command::new("du");
        cmd.arg("-sh"); // human readable

        if let Some(d) = depth {
            cmd.arg(format!("--max-depth={}", d));
        }

        cmd.arg("--").arg(path);

        let status = cmd.status().context("Failed to execute du")?;
        if !status.success() {
            anyhow::bail!("du failed with status {}", status);
        }
        Ok(())
    }
}

pub struct Group;

impl GroupManager for Group {
    fn list(&self) -> Result<()> {
        println!("Listing groups...");
        let mut cmd = Command::new("cat");
        cmd.arg("/etc/group");
        let status = cmd.status().context("Failed to list groups")?;
        if !status.success() {
            anyhow::bail!("Listing groups failed");
        }
        Ok(())
    }

    fn add(&self, groupname: &str) -> Result<()> {
        let status = Command::new("groupadd")
            .arg("--")
            .arg(groupname)
            .status()
            .context("Failed to execute groupadd")?;

        if !status.success() {
            anyhow::bail!("groupadd failed with status {}", status);
        }
        Ok(())
    }

    fn del(&self, groupname: &str) -> Result<()> {
        let status = Command::new("groupdel")
            .arg("--")
            .arg(groupname)
            .status()
            .context("Failed to execute groupdel")?;

        if !status.success() {
            anyhow::bail!("groupdel failed with status {}", status);
        }
        Ok(())
    }

    fn mod_group(&self, groupname: &str, gid: u32) -> Result<()> {
        let status = Command::new("groupmod")
            .arg("--gid")
            .arg(gid.to_string())
            .arg("--")
            .arg(groupname)
            .status()
            .context("Failed to execute groupmod")?;

        if !status.success() {
            anyhow::bail!("groupmod failed with status {}", status);
        }
        Ok(())
    }
}

pub struct User;

impl UserManager for User {
    fn list(&self, all: bool, groups: bool) -> Result<()> {
        println!("Listing users...");
        let mut cmd = Command::new("cat");
        cmd.arg("/etc/passwd");
        // A full robust implementation would parse /etc/passwd and filter by ID >= 1000
        // unless `all` is true, and optionally query secondary groups if `groups` is true.
        // For now, we wrap a basic output to show execution.
        let status = cmd.status().context("Failed to list users")?;
        if !status.success() {
            anyhow::bail!("Listing users failed");
        }
        Ok(())
    }

    fn add(&self, username: &str, groups: Option<&str>, shell: Option<&str>, system: bool) -> Result<()> {
        let mut cmd = Command::new("useradd");
        cmd.arg("-m"); // Create home directory

        if system {
            cmd.arg("--system");
        }

        if let Some(s) = shell {
            cmd.arg("--shell").arg(s);
        }

        if let Some(g) = groups {
            cmd.arg("--groups").arg(g);
        }

        cmd.arg("--").arg(username);

        let status = cmd.status().context("Failed to add user")?;
        if !status.success() {
            anyhow::bail!("useradd failed with status {}", status);
        }
        Ok(())
    }

    fn del(&self, username: &str, purge: bool) -> Result<()> {
        let mut cmd = Command::new("userdel");
        if purge {
            cmd.arg("-r");
        }
        cmd.arg("--").arg(username);

        let status = cmd.status().context("Failed to delete user")?;
        if !status.success() {
            anyhow::bail!("userdel failed with status {}", status);
        }
        Ok(())
    }

    fn mod_user(&self, username: &str, action: &str, value: &str) -> Result<()> {
        let mut cmd = Command::new("usermod");
        match action {
            "add-group" => { cmd.arg("-aG").arg(value); },
            "del-group" => {
                // `usermod` doesn't have a simple flag to remove from a single group.
                // It requires gpasswd or deluser, but wrapping gpasswd here.
                let mut dcmd = Command::new("gpasswd");
                dcmd.arg("-d").arg(username).arg(value);
                let s = dcmd.status().context("Failed to remove group")?;
                if !s.success() { anyhow::bail!("group removal failed"); }
                return Ok(());
            },
            "shell" => { cmd.arg("-s").arg(value); },
            "home" => { cmd.arg("-d").arg(value).arg("-m"); },
            _ => anyhow::bail!("Unsupported user modification action: {}", action),
        }
        cmd.arg("--").arg(username);

        let status = cmd.status().context("Failed to modify user")?;
        if !status.success() {
            anyhow::bail!("usermod failed with status {}", status);
        }
        Ok(())
    }

    fn passwd(&self, username: &str) -> Result<()> {
        use std::io::Write;

        let password = rpassword::prompt_password("New password: ")
            .context("Failed to read password from stdin")?;
        let confirm_password = rpassword::prompt_password("Retype new password: ")
            .context("Failed to read password from stdin")?;

        if password != confirm_password {
            anyhow::bail!("Passwords do not match");
        }

        let mut child = Command::new("chpasswd")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn chpasswd")?;

        if let Some(mut stdin) = child.stdin.take() {
            let creds = format!("{}:{}", username, password);
            stdin.write_all(creds.as_bytes()).context("Failed to write to chpasswd stdin")?;
        }

        let status = child.wait().context("Failed to wait on chpasswd")?;

        if !status.success() {
            anyhow::bail!("chpasswd failed with status {}", status);
        }

        println!("passwd: password updated successfully");
        Ok(())
    }
}

pub struct Systemd;

impl ServiceManager for Systemd {
    fn list(&self) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .status()
            .context("Failed to execute systemctl list-units")?;

        if !status.success() {
            anyhow::bail!("systemctl failed with status {}", status);
        }
        Ok(())
    }

    fn up(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("enable")
            .arg("--now")
            .arg("--")
            .arg(service)
            .status()
            .context("Failed to start/enable service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to start service {}", service);
        }
        Ok(())
    }

    fn down(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("disable")
            .arg("--now")
            .arg("--")
            .arg(service)
            .status()
            .context("Failed to stop/disable service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to stop service {}", service);
        }
        Ok(())
    }

    fn restart(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("restart")
            .arg("--")
            .arg(service)
            .status()
            .context("Failed to restart service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to restart service {}", service);
        }
        Ok(())
    }

    fn reload(&self, service: &str) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("reload")
            .arg("--")
            .arg(service)
            .status()
            .context("Failed to reload service")?;

        if !status.success() {
            anyhow::bail!("systemctl failed to reload service {}", service);
        }
        Ok(())
    }

    fn status(&self, service: &str) -> Result<()> {
        // We use status to let it stream directly to the terminal stdout
        let _ = Command::new("systemctl")
            .arg("status")
            .arg("--")
            .arg(service)
            .status()
            .context("Failed to get status")?;
        Ok(())
    }
}
