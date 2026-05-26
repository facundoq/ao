use ao_cli::os::detector::detect_system;
use std::path::Path;
use std::process::Command;

#[test]
fn test_shell_completions_dynamic() {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let binary_path = format!("{}/debug/ao", target_dir);

    // Ensure the binary exists (integration tests usually have it, but let's be safe)
    if !Path::new(&binary_path).exists() {
        let build_status = Command::new("cargo")
            .arg("build")
            .status()
            .expect("Failed to run cargo build");
        assert!(build_status.success(), "Failed to build ao binary");
    }

    let system = detect_system().expect("Failed to detect system");

    // 1. Test Users completion
    let expected_users = system
        .user
        .get_users()
        .expect("Failed to get users via UserManager");
    assert!(
        !expected_users.is_empty(),
        "System should have at least one user"
    );

    let actual_users = get_completions(&binary_path, "ao user modify ");
    for user in &expected_users {
        assert!(
            actual_users.contains(user),
            "User '{}' missing from 'ao user modify ' completions.\nExpected: {:?}\nActual: {:?}",
            user,
            expected_users,
            actual_users
        );
    }

    // 2. Test Groups completion
    let expected_groups = system
        .group
        .get_groups()
        .expect("Failed to get groups via GroupManager");
    assert!(
        !expected_groups.is_empty(),
        "System should have at least one group"
    );

    let actual_groups = get_completions(&binary_path, "ao group modify ");
    for group in &expected_groups {
        assert!(
            actual_groups.contains(group),
            "Group '{}' missing from 'ao group modify ' completions.\nExpected: {:?}\nActual: {:?}",
            group,
            expected_groups,
            actual_groups
        );
    }

    // 3. Test Shells completion (nested completion)
    // Find a real user to test modify <user> shell <TAB>
    let test_user = if expected_users.contains(&"root".to_string()) {
        "root"
    } else {
        &expected_users[0]
    };
    let expected_shells = system
        .user
        .get_shells()
        .expect("Failed to get shells via UserManager");
    let actual_shells = get_completions(
        &binary_path,
        &format!("ao user modify {} shell ", test_user),
    );

    for shell in &expected_shells {
        assert!(
            actual_shells.contains(shell),
            "Shell '{}' missing from completions for user {}.\nActual: {:?}",
            shell,
            test_user,
            actual_shells
        );
    }

    // 4. Test Service completion
    let expected_services = system
        .svc
        .get_services()
        .expect("Failed to get services via ServiceManager");
    // Only test if we actually found services (some CI environments might not have systemd/openrc)
    if !expected_services.is_empty() {
        let actual_services = get_completions(&binary_path, "ao service status ");
        // We might not want to check ALL services if there are thousands, but let's check at least 10 or all if few.
        let to_check = if expected_services.len() > 10 {
            &expected_services[0..10]
        } else {
            &expected_services[..]
        };
        for svc in to_check {
            // Some completions might append .service if missing, or handle it differently.
            // StandardService adds .service if missing in get_services().
            assert!(
                actual_services.contains(svc),
                "Service '{}' missing from completions.\nActual: {:?}",
                svc,
                actual_services
            );
        }
    }

    // 5. Test static completions (subcommands)
    let system_power = get_completions(&binary_path, "ao system power ");
    assert!(
        system_power.contains(&"reboot".to_string()),
        "system power completions missing 'reboot'. Actual: {:?}",
        system_power
    );

    let self_completions = get_completions(&binary_path, "ao self completions generate ");
    assert!(
        self_completions.contains(&"bash".to_string()),
        "self completions generate missing 'bash'. Actual: {:?}",
        self_completions
    );

    let virt_completions = get_completions(&binary_path, "ao virtualization ");
    assert!(
        virt_completions.contains(&"list".to_string()),
        "virtualization completions missing 'list'. Actual: {:?}",
        virt_completions
    );
}

fn get_completions(binary_path: &str, cmd: &str) -> Vec<String> {
    let mut words: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
    if cmd.ends_with(' ') {
        words.push("".to_string());
    }

    // Index of the word to complete
    let index = words.len() - 1;

    let output = Command::new(binary_path)
        .env("AO_COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", index.to_string())
        .arg("--")
        .args(&words)
        .output()
        .expect("Failed to run completion command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}
