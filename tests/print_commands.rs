use std::process::Command;
use std::str;

fn run_print(args: &[&str]) -> String {
    let output = Command::new("./target/debug/ao")
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
    assert_eq!(run_print(&["user", "ls"]), "cat /etc/passwd");

    // 2. Network Interfaces
    assert_eq!(run_print(&["net", "interfaces"]), "ip addr");

    // 3. Service Status
    assert_eq!(
        run_print(&["svc", "status", "cron"]),
        "systemctl status -- cron"
    );

    // 4. Disk List
    assert_eq!(run_print(&["disk", "ls"]), "lsblk --json");

    // 5. System Info (Library usage)
    assert_eq!(run_print(&["sys", "info"]), "sysinfo (Rust library)");

    // 6. Auth Logs
    assert_eq!(
        run_print(&["log", "auth", "--lines", "10"]),
        "journalctl -n 10 _FACILITY=4 _FACILITY=10 --"
    );

    // 7. Package Install (Distro specific, but assuming debian/apt for this environment)
    let pkg_print = run_print(&["pkg", "add", "vim"]);
    assert!(
        pkg_print.contains("apt install")
            || pkg_print.contains("apt-get install")
            || pkg_print.contains("dnf install")
            || pkg_print.contains("pacman -S")
    );
}
