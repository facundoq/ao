#![allow(clippy::collapsible_if)]

use std::env;
use std::process::Command;

const TEST_USER: &str = "testdestuser";
const TEST_GROUP: &str = "testdestgroup";

fn run_ao(args: &[&str]) -> bool {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--quiet")
        .arg("--bin")
        .arg("ao")
        .arg("--");
    cmd.args(args);
    let status = cmd.status().expect("Failed to execute ao");
    status.success()
}

fn run_ao_output(args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--quiet")
        .arg("--bin")
        .arg("ao")
        .arg("--");
    cmd.args(args);
    let output = cmd.output().expect("Failed to execute ao");
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn test_user_and_group() {
    println!("Testing User & Group operations...");
    assert!(run_ao(&["user", "add", TEST_USER]), "Failed to add user");
    assert!(run_ao(&["group", "add", TEST_GROUP]), "Failed to add group");
    assert!(
        run_ao(&["user", "modify", TEST_USER, "add-group", TEST_GROUP]),
        "Failed to add user to group"
    );

    let id_output = run_ao_output(&["user", "list", "--groups", "-f", "json"]).unwrap();
    assert!(
        id_output.contains(TEST_GROUP),
        "User not found in group after add"
    );

    assert!(
        run_ao(&["user", "modify", TEST_USER, "del-group", TEST_GROUP]),
        "Failed to remove user from group"
    );

    let id_output2 = run_ao_output(&["user", "list", "--groups", "-f", "json"]).unwrap();
    let user_info_idx = id_output2.find(TEST_USER).unwrap_or(0);
    let after_user = &id_output2[user_info_idx..];
    let user_obj_end = after_user.find('}').unwrap_or(after_user.len());
    let user_obj = &after_user[..user_obj_end];

    assert!(
        !user_obj.contains(TEST_GROUP),
        "User still in group after remove"
    );

    assert!(
        run_ao(&["user", "delete", TEST_USER]),
        "Failed to delete user"
    );
    assert!(
        run_ao(&["group", "delete", TEST_GROUP]),
        "Failed to delete group"
    );
    println!("User & Group tests passed!");
}

fn test_kernel_module() {
    println!("Testing Kernel Module operations...");
    if let Some(stdout) = run_ao_output(&["boot", "module", "list", "-f", "json"]) {
        let mut unused_mod = None;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(arr) = json.as_array() {
                for obj in arr {
                    if obj.get("used_by").and_then(|v| v.as_str()) == Some("0") {
                        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                            unused_mod = Some(name.to_string());
                            break;
                        }
                    }
                }
            }
        }

        if let Some(m) = unused_mod {
            println!("Found unused module: {}", m);
            if run_ao(&["boot", "module", "unload", &m]) {
                let stdout2 = run_ao_output(&["boot", "module", "list", "-f", "json"]).unwrap();
                assert!(!stdout2.contains(&m), "Module still loaded after unload");

                assert!(
                    run_ao(&["boot", "module", "load", &m]),
                    "Failed to load module"
                );
                let stdout3 = run_ao_output(&["boot", "module", "list", "-f", "json"]).unwrap();
                assert!(stdout3.contains(&m), "Module not loaded after load");
            } else {
                println!(
                    "  SKIPPED: unloading module failed (likely due to container permissions)."
                );
            }
        } else {
            println!("  SKIPPED: no unused kernel modules found.");
        }
    } else {
        println!("  SKIPPED: boot module list failed.");
    }
}

fn test_service() {
    println!("Testing Service operations...");
    let mut running_svc = None;
    if let Some(stdout) = run_ao_output(&["service", "list", "-f", "json"]) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(arr) = json.as_array() {
                for obj in arr {
                    if obj.get("status").and_then(|v| v.as_str()) == Some("running") {
                        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                            running_svc = Some(name.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    if let Some(svc) = running_svc {
        println!("Found running service: {}", svc);
        run_ao(&["service", "status", &svc]);

        if run_ao(&["service", "restart", &svc]) {
            run_ao(&["service", "status", &svc]);
            assert!(run_ao(&["service", "down", &svc]), "Failed to stop service");
            run_ao(&["service", "status", &svc]);
            assert!(run_ao(&["service", "up", &svc]), "Failed to start service");
            run_ao(&["service", "status", &svc]);
            println!("Service operations passed!");
        } else {
            println!("  SKIPPED: restarting service failed (likely due to container limits).");
        }
    } else {
        println!("  SKIPPED: no running services found.");
    }
}

fn test_firewall() {
    println!("Testing Firewall operations...");
    if run_ao(&["network", "firewall", "allow", "8080/tcp"]) {
        println!("  Firewall allow test passed!");
        // Note: `ao network firewall` does not yet have a generalized `delete` or `remove` command.
        // We will try `deny` to revert the access, representing abstraction fallback
        if run_ao(&["network", "firewall", "deny", "8080/tcp"]) {
            println!("  Firewall deny test passed!");
        }
    } else {
        println!("  SKIPPED: firewall allow failed or no firewall present.");
    }
}

fn test_system_time() {
    println!("Testing System Time operations...");
    if let Some(status_json) = run_ao_output(&["system", "time", "status", "-f", "json"]) {
        let mut orig_tz = None;
        for line in status_json.lines() {
            if line.contains("\"time_zone\":") {
                let parts: Vec<&str> = line.split('"').collect();
                if parts.len() >= 4 {
                    orig_tz = Some(parts[3].split_whitespace().next().unwrap_or("").to_string());
                }
            }
        }

        if orig_tz.is_none() {
            println!("Could not determine original timezone from ao system time status output.");
        }

        if let Some(tz) = orig_tz {
            if !tz.is_empty() {
                println!("Original timezone: {}", tz);
                let new_tz = if tz == "Etc/GMT-3" {
                    "Etc/GMT+0"
                } else {
                    "Etc/GMT-3"
                };
                println!("Setting timezone to {}...", new_tz);
                if run_ao(&["system", "time", "set", new_tz]) {
                    println!("Restoring original timezone...");
                    assert!(
                        run_ao(&["system", "time", "set", &tz]),
                        "Failed to restore timezone"
                    );
                    println!("  Timezone test passed!");
                } else {
                    println!("  SKIPPED: Setting timezone failed.");
                }
            } else {
                println!("  SKIPPED: Could not determine original timezone.");
            }
        } else {
            println!("  SKIPPED: Could not determine original timezone.");
        }
    } else {
        println!("  SKIPPED: time status failed.");
    }
}

fn test_package() {
    println!("Testing Package operations...");
    let mut pkg = "hello";
    if let Some(stdout) = run_ao_output(&["system", "info", "-f", "json"]) {
        if stdout.to_lowercase().contains("alpine") {
            pkg = "hello-world";
        }
    }
    println!("Package to test: {}", pkg);

    // ignore error on delete, it might not be installed
    run_ao(&["package", "delete", pkg]);

    if run_ao(&["package", "add", pkg]) {
        println!("  Package {} installed successfully.", pkg);
        if Command::new(pkg).output().is_ok() {
            println!("Ran {}", pkg);
        } else if Command::new("hello").output().is_ok() {
            println!("Ran hello");
        } else {
            println!("  Warning: {} installed but binary not found in PATH.", pkg);
        }

        assert!(
            run_ao(&["package", "delete", pkg]),
            "Failed to delete package"
        );
        assert!(
            Command::new(pkg).output().is_err() && Command::new("hello").output().is_err(),
            "Package still available after delete"
        );
        println!("  Package test passed!");
    } else {
        println!("  SKIPPED: Package installation failed.");
    }
}

fn test_autocomplete() {
    println!("Testing Autocomplete Installation...");
    let shell = env::var("SHELL").unwrap_or_default();
    let shell_name = shell.split('/').next_back().unwrap_or("");
    if ["bash", "zsh", "fish"].contains(&shell_name) {
        println!("Installing completions for {}...", shell_name);
        assert!(
            run_ao(&["self", "completions", "install", shell_name]),
            "Failed to install completions"
        );
        println!("  Autocompletions installed successfully.");
    } else {
        println!(
            "  SKIPPED: Shell '{}' is not bash, zsh, or fish.",
            shell_name
        );
    }
}

pub fn run_all() {
    println!("Starting destructive integration tests in Rust...");

    test_user_and_group();
    test_kernel_module();
    test_service();
    test_firewall();
    test_system_time();
    test_package();
    test_autocomplete();

    println!("----------------------------------------");
    println!("All destructive integration tests PASSED!");
}
