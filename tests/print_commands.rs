use std::process::Command;
use std::str;

fn run_print(args: &[&str]) -> String {
    let output = Command::new("./target/debug/ao")
        .arg("--print")
        .args(args)
        .output()
        .expect("Failed to execute ao");
    
    assert!(output.status.success(), "Command failed: ao --print {}", args.join(" "));
    str::from_utf8(&output.stdout).unwrap().trim().to_string()
}

#[test]
fn test_command_printing() {
    // Ensure binary is built
    let build = Command::new("cargo").arg("build").status().unwrap();
    assert!(build.success());

    // Pkg
    assert!(run_print(&["pkg", "install", "vim"]).contains("install"));
    assert!(run_print(&["pkg", "remove", "vim"]).contains("remove"));
    assert!(run_print(&["pkg", "update"]).contains("update") || run_print(&["pkg", "update"]).contains("upgrade"));
    assert!(run_print(&["pkg", "list"]).contains("list"));

    // Svc
    assert_eq!(run_print(&["svc", "up", "nginx"]), "systemctl enable --now -- nginx");
    assert_eq!(run_print(&["svc", "down", "nginx"]), "systemctl disable --now -- nginx");
    assert_eq!(run_print(&["svc", "status", "nginx"]), "systemctl status -- nginx");
    assert!(run_print(&["svc", "list"]).contains("systemctl list-units (format: Table)"));

    // User
    assert!(run_print(&["user", "add", "bob"]).contains("useradd"));
    assert!(run_print(&["user", "del", "bob"]).contains("userdel"));
    assert!(run_print(&["user", "mod", "bob", "add-group", "sudo"]).contains("usermod"));
    assert!(run_print(&["user", "list"]).contains("list users (format: Table)"));

    // Group
    assert!(run_print(&["group", "add", "devs"]).contains("groupadd"));
    assert!(run_print(&["group", "del", "devs"]).contains("groupdel"));
    assert!(run_print(&["group", "list"]).contains("list groups (format: Table)"));

    // Disk
    assert!(run_print(&["disk", "list"]).contains("lsblk --json (format: Table)"));
    assert!(run_print(&["disk", "mount", "/dev/sdb1", "/mnt"]).contains("mount"));
    assert!(run_print(&["disk", "usage", "/var"]).contains("du"));

    // Sys
    assert!(run_print(&["sys", "info"]).contains("sys info --format Table"));
    assert!(run_print(&["sys", "info", "--format", "json"]).contains("sys info --format Json"));
    assert!(run_print(&["sys", "info", "--format", "yaml"]).contains("sys info --format Yaml"));
    assert!(run_print(&["sys", "power", "reboot"]).contains("reboot"));
    assert!(run_print(&["sys", "time", "status"]).contains("timedatectl status (format: Table)"));

    // Net
    assert!(run_print(&["net", "interfaces"]).contains("ip addr (format: Table)"));
    assert!(run_print(&["net", "ips"]).contains("ip addr (for IPs) (format: Table)"));
    assert!(run_print(&["net", "routes"]).contains("ip route (format: Table)"));
    assert!(run_print(&["net", "fw", "status"]).contains("Firewall status (format: Table)"));
    assert!(run_print(&["net", "wifi", "scan"]).contains("nmcli"));

    // Log
    assert!(run_print(&["log", "tail", "nginx"]).contains("journalctl"));
    assert!(run_print(&["log", "file", "/var/log/syslog"]).contains("tail"));

    // Boot
    assert!(run_print(&["boot", "list"]).contains("bootctl list (format: Table)"));
    assert!(run_print(&["boot", "mod", "list"]).contains("lsmod (format: Table)"));

    // Gui
    assert!(run_print(&["gui", "info"]).contains("loginctl"));
    assert!(run_print(&["gui", "display", "list"]).contains("list displays (format: Table)"));

    // Dev
    assert!(run_print(&["dev", "pci"]).contains("lspci (format: Table)"));
    assert!(run_print(&["dev", "usb"]).contains("lsusb (format: Table)"));
    assert!(run_print(&["dev", "bt", "status"]).contains("bluetoothctl"));

    // Virt
    assert!(run_print(&["virt", "ps"]).contains("docker ps (format: Table)"));

    // Sec
    assert!(run_print(&["sec", "audit"]).contains("security audit (format: Table)"));
    assert!(run_print(&["sec", "context"]).contains("sestatus"));

    // Distro
    assert!(run_print(&["distro", "info"]).contains("distro info (format: Table)"));
}
