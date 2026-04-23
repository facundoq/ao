use std::path::Path;
use std::process::Command;

#[test]
fn test_shell_completions() {
    // Ensure the binary exists
    if !Path::new("./target/debug/ao").exists() {
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

    let output = Command::new("bash")
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
