use std::process::Command;

/*
 * This test uses 'podman' (or 'docker') directly to build local test images
 * and run integration tests on them.
 */

fn get_engine() -> String {
    if Command::new("podman").arg("--version").status().is_ok() {
        "podman".to_string()
    } else {
        "docker".to_string()
    }
}

async fn build_test_image(engine: &str, distro: &str) -> String {
    let image_name = format!("ao-test-{}", distro);
    let dockerfile = format!("tests/containers/{}.Dockerfile", distro);

    println!("Building local test image: {}...", image_name);
    let status = Command::new(engine)
        .args(["build", "-t", &image_name, "-f", &dockerfile, "."])
        .status()
        .expect("Failed to build test image");

    assert!(status.success(), "Failed to build image for {}", distro);
    image_name
}

async fn run_test_in_image(engine: &str, image_name: &str) {
    println!("Running tests on {}...", image_name);

    let current_dir = std::env::current_dir().expect("Failed to get current dir");
    let current_dir_str = current_dir.to_string_lossy();
    let log_file_name = format!("test_{}.log", image_name);
    let log_file_path = current_dir.join(&log_file_name);

    // 1. Start container with bind mount
    let output = Command::new(engine)
        .args([
            "run",
            "-d",
            "--rm",
            "-v",
            &format!("{}:/ao", current_dir_str),
            image_name,
            "sleep",
            "600",
        ])
        .output()
        .expect("Failed to start container");

    if !output.status.success() {
        panic!(
            "Failed to start container {}: {}",
            image_name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let _guard = ContainerGuard {
        engine: engine.to_string(),
        id: container_id.clone(),
    };

    // 2. Run tests inside the container (compiling if needed)
    println!("Building and testing inside {}...", image_name);
    println!("Diagnostics will be saved to {}", log_file_name);

    let test_script = format!(
        "export RUST_BACKTRACE=1 && cargo test -- --nocapture > /ao/{} 2>&1 && bash tests/read_only_tests.sh >> /ao/{} 2>&1",
        log_file_name, log_file_name
    );

    let output = Command::new(engine)
        .args(["exec", &container_id, "bash", "-c", &test_script])
        .output()
        .expect("Failed to execute tests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        println!("Tests FAILED on {}", image_name);

        // Try to read the log file from the host since it's a bind mount
        if let Ok(log_content) = std::fs::read_to_string(&log_file_path) {
            println!("--- LOG FILE CONTENT ({}) ---", log_file_name);
            println!("{}", log_content);
            println!("--- END OF LOG FILE ---");
        } else {
            println!("(Could not read log file {})", log_file_name);
            println!("STDOUT:\n{}", stdout);
            println!("STDERR:\n{}", stderr);
        }

        panic!(
            "Integration tests failed on {} (Exit code: {:?})",
            image_name,
            output.status.code()
        );
    } else {
        println!("Tests PASSED on {}", image_name);
        // Optionally clean up the log file on success
        let _ = std::fs::remove_file(&log_file_path);
    }
}

struct ContainerGuard {
    engine: String,
    id: String,
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        let _ = Command::new(&self.engine)
            .args(["stop", "-t", "0", &self.id])
            .status();
    }
}

#[tokio::test]
#[ignore]
async fn test_distros() {
    let engine = get_engine();

    let distros = ["debian", "ubuntu", "fedora", "archlinux", "alpine", "macos"];

    for distro in distros {
        let image = build_test_image(&engine, distro).await;
        run_test_in_image(&engine, &image).await;
    }
}
