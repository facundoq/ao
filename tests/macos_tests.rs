#[cfg(test)]
mod tests {
    #[test]
    fn test_macos_detection_interface() {
        #[cfg(target_os = "macos")]
        {
            use ao_cli::os::detector::detect_system;
            let system = detect_system().expect("MacOS system detection failed");
            assert_eq!(system.pkg.name(), "package");
        }
    }
}
