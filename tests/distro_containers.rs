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

async fn build_musl_binary() {
    println!("Building musl binary...");
    let build = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "x86_64-unknown-linux-musl",
        ])
        .status()
        .expect("Failed to build musl binary");
    assert!(build.success());
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

    // 1. Start container
    let output = Command::new(engine)
        .args(["run", "-d", "--rm", image_name, "sleep", "300"])
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

    // 2. Copy binary and script
    let binary_path = "target/x86_64-unknown-linux-musl/release/ao";
    let test_script = "tests/read_only_tests.sh";

    Command::new(engine)
        .args([
            "cp",
            binary_path,
            &format!("{}:/usr/local/bin/ao", container_id),
        ])
        .status()
        .expect("cp binary failed");
    Command::new(engine)
        .args(["cp", test_script, &format!("{}:/tmp/test.sh", container_id)])
        .status()
        .expect("cp script failed");

    // 3. Run tests
    let output = Command::new(engine)
        .args(["exec", &container_id, "bash", "/tmp/test.sh"])
        .output()
        .expect("Failed to execute tests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        println!("Tests FAILED on {}", image_name);
        println!("STDOUT:\n{}", stdout);
        println!("STDERR:\n{}", stderr);
        panic!(
            "Integration tests failed on {} (Exit code: {:?})",
            image_name,
            output.status.code()
        );
    } else {
        println!("Tests PASSED on {}", image_name);
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
    build_musl_binary().await;
    let engine = get_engine();

    let distros = ["debian", "ubuntu", "fedora", "archlinux", "alpine"];

    for distro in distros {
        let image = build_test_image(&engine, distro).await;
        run_test_in_image(&engine, &image).await;
    }
}
