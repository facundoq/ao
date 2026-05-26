#[cfg(all(test, target_os = "macos"))]
mod tests {
    use ao_cli::os::detector::detect_system;

    #[test]
    fn test_macos_detection_interface() {
        let system = detect_system().expect("MacOS system detection failed");
        assert_eq!(system.pkg.name(), "package");
        assert_eq!(system.svc.name(), "service");
        assert_eq!(system.net.name(), "network");
        assert_eq!(system.sys.name(), "system");
    }

    #[test]
    fn test_macos_sys_info() {
        let system = detect_system().expect("MacOS system detection failed");
        let info = system.sys.get_info().expect("Failed to get sys info");
        assert!(!info.hostname.is_empty());
        assert!(info.os.contains("macOS") || info.os.contains("Mac OS X"));
        assert!(!info.uptime.contains("Unknown"));
        assert!(info.total_memory > 0);
    }

    #[test]
    fn test_macos_services() {
        let system = detect_system().expect("MacOS system detection failed");
        let services = system.svc.get_services().expect("Failed to get services");
        // There should be some services on any MacOS system
        assert!(!services.is_empty());
    }

    #[test]
    fn test_macos_network() {
        let system = detect_system().expect("MacOS system detection failed");
        let interfaces = system
            .net
            .get_interfaces()
            .expect("Failed to get interfaces");
        assert!(!interfaces.is_empty());

        // Check for lo0 or en0 which are standard on macos
        let has_standard = interfaces
            .iter()
            .any(|i| i.name == "lo0" || i.name == "en0");
        assert!(has_standard);
    }

    #[test]
    fn test_macos_storage() {
        let system = detect_system().expect("MacOS system detection failed");
        let devices = system.disk.get_devices().expect("Failed to get devices");
        assert!(!devices.is_empty());
        assert!(devices.iter().any(|d| d.starts_with("/dev/disk")));

        let mounts = system
            .partition
            .get_mount_points()
            .expect("Failed to get mount points");
        assert!(!mounts.is_empty());
        assert!(mounts.iter().any(|m| m == "/"));
    }
}
