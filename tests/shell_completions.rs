use std::path::Path;
use std::process::Command;

#[test]
fn test_shell_completions() {
    // Ensure the binary exists
    if !Path::new("./target/debug/ao").exists() {
        // Skip if not built yet (though cargo test usually builds it)
        let build_status = Command::new("cargo")
            .arg("build")
            .status()
            .expect("Failed to run cargo build");
        assert!(build_status.success());
    }

    let status = Command::new("bash")
        .arg("tests/completions_test.sh")
        .status()
        .expect("Failed to execute completion test script");

    assert!(status.success(), "Shell completion tests failed!");
}
