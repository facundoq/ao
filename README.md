# ao

Admin Operations (`ao`) is a centralized, unified command line wrapper written in Rust designed to perform sysop operations across fragmented Linux environments. Instead of context-switching between `apt`, `dnf`, `systemctl`, `usermod`, and `ip`, you just use `ao`.

## Philosophy

* **Ergonomic Speed:** Fast, predictable muscle memory (`ao <domain> <action>`).
* **Zero Overhead:** Instantaneous startup time via Rust.
* **Abstraction:** Hides the differences between Debian, Fedora, Arch, etc.

## Feature Support Matrix

This matrix tracks the current implementation status of the syntax tree defined in `SPEC.md`. Because `ao` abstracts the underlying Operating System, some features may only be implemented for specific backend distributions (e.g., Debian/Ubuntu vs Fedora/RHEL).

| Domain | Sub-domain | Description | Status | Level of Support |
| :--- | :--- | :--- | :--- | :--- |
| `ao sys` | Core System | Updates, power, time | ✅ Implemented | Generic Linux |
| `ao svc` | Services | Start, stop, list services (`systemctl`) | ✅ Implemented | Debian/Ubuntu (Systemd) |
| `ao user` | Users | Add, remove, modify users | ✅ Implemented | Generic Linux |
| `ao group` | Groups | Add, remove, modify groups | ✅ Implemented | Generic Linux |
| `ao net` | Networking | Interfaces, IPs, routes | ✅ Implemented | Generic Linux |
| `ao net fw` | Firewall | Allow, block, status | ✅ Implemented | Generic Linux |
| `ao net wifi` | Wi-Fi | Scan, connect, forget | ✅ Implemented | Generic Linux |
| `ao disk` | Storage | Mount, unmount, usage | ✅ Implemented | Generic Linux |
| `ao pkg` | Packages | Install, remove, update | ✅ Implemented | Debian/Ubuntu (`apt`) |
| `ao log` | Logs | Tail service and system logs | ✅ Implemented | Generic Linux |
| `ao boot` | Boot & Kernel | Bootloader defaults, kernel modules | ✅ Implemented | Generic Linux |
| `ao gui` | Displays | Wayland/X11 detect, resolution | ✅ Implemented | Generic Linux |
| `ao dev` | Devices | PCI/USB lists, Bluetooth, Printers | ✅ Implemented | Generic Linux |
| `ao virt` | Virtualization | Docker, Podman, libvirt abstraction | ✅ Implemented | Generic Linux |
| `ao sec` | Security | SELinux/AppArmor contexts, audits | ✅ Implemented | Generic Linux |
| `ao distro` | Distributions | OS info, major release upgrades | ✅ Implemented | Generic Linux |

## Architecture

`ao` relies on an abstraction layer (`src/os/mod.rs`). The CLI (`src/cli.rs`) routes commands to the `Detector` (`src/os/detector.rs`), which reads `/etc/os-release` and returns a Boxed Trait Object implementing the necessary command logic (e.g., `Apt` or `Systemd` in `src/os/debian.rs`).
