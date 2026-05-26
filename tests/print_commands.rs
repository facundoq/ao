use std::process::Command;
use std::str;

fn run_print(args: &[&str]) -> String {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let binary_path = format!("{}/debug/ao", target_dir);
    let output = Command::new(binary_path)
        .arg("--print")
        .args(args)
        .output()
        .expect("Failed to execute ao");

    assert!(
        output.status.success(),
        "Command failed: ao --print {}",
        args.join(" ")
    );
    str::from_utf8(&output.stdout).unwrap().trim().to_string()
}

#[test]
fn test_command_printing() {
    // Ensure binary is built
    let build_output = Command::new("cargo")
        .arg("build")
        .output()
        .expect("Failed to run cargo build");
    if !build_output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&build_output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&build_output.stderr));
        panic!("Failed to build binary for command printing tests");
    }

    // 1. User List
    let user_list = run_print(&["user", "list"]);
    assert!(user_list.contains("cat /etc/passwd") || user_list.contains("dscl . -list /Users"));

    // 2. Network Interfaces
    let net_ifaces = run_print(&["network", "interfaces"]);
    assert!(
        net_ifaces.contains("ip addr")
            || net_ifaces.contains("networksetup -listallnetworkservices")
    );

    // 3. Service Status
    let svc_status = run_print(&["service", "status", "cron"]);
    assert!(
        svc_status.contains("systemctl status")
            || svc_status.contains("launchctl list")
            || svc_status.contains("rc-service")
    );

    // 4. Disk List
    let disk_list = run_print(&["disk", "list"]);
    assert!(disk_list.contains("lsblk") || disk_list.contains("diskutil list"));

    // 5. System Info (Library usage)
    let sys_info = run_print(&["system", "info"]);
    assert!(sys_info.contains("sysinfo (Rust library)") || sys_info.contains("system_profiler"));

    // 6. Auth Logs
    let auth_logs = run_print(&["log", "auth", "--lines", "10"]);
    assert!(auth_logs.contains("journalctl") || auth_logs.contains("log show --predicate"));

    // 7. Package Install
    let pkg_print = run_print(&["package", "add", "vim"]);
    assert!(
        pkg_print.contains("apt install")
            || pkg_print.contains("apt-get install")
            || pkg_print.contains("dnf install")
            || pkg_print.contains("pacman -S")
            || pkg_print.contains("apk add")
            || pkg_print.contains("brew install")
    );

    // 8. User Modification
    let user_mod = run_print(&["user", "modify", "john", "shell", "/bin/zsh"]);
    assert!(
        user_mod.contains("usermod -s")
            || user_mod.contains("dscl . -create /Users/john")
            || user_mod.contains("adduser")
    );
}
