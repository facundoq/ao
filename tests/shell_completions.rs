use std::path::Path;
use std::process::Command;

#[test]
fn test_shell_completions() {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let binary_path = format!("{}/debug/ao", target_dir);

    // Ensure the binary exists
    if !Path::new(&binary_path).exists() {
        // Skip if not built yet (though cargo test usually builds it)
        let build_output = Command::new("cargo")
            .arg("build")
            .output()
            .expect("Failed to run cargo build");
        if !build_output.status.success() {
            eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&build_output.stdout));
            eprintln!("STDERR:\n{}", String::from_utf8_lossy(&build_output.stderr));
            panic!("Failed to build binary for completion tests");
        }
    }

    let bash_check = Command::new("bash").arg("--version").output();

    if bash_check.is_err() {
        eprintln!("Warning: bash not found. Skipping shell completion tests.");
        return;
    }

    let output = Command::new("bash")
        .env("AO_EXE", binary_path)
        .arg("tests/completions_test.sh")
        .output()
        .expect("Failed to execute completion test script");

    if !output.status.success() {
        eprintln!("Shell completion tests failed!");
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("Shell completion tests failed!");
    }
}
