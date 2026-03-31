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
| `ao sys` | Core System | Updates, power, time | ❌ Not Implemented | Planned |
| `ao svc` | Services | Start, stop, list services (`systemctl`) | ✅ Implemented | Debian/Ubuntu (Systemd) |
| `ao user` | Users | Add, remove, modify users | ❌ Not Implemented | Planned |
| `ao group` | Groups | Add, remove, modify groups | ❌ Not Implemented | Planned |
| `ao net` | Networking | Interfaces, IPs, routes | ❌ Not Implemented | Planned |
| `ao net fw` | Firewall | Allow, block, status | ❌ Not Implemented | Planned |
| `ao net wifi` | Wi-Fi | Scan, connect, forget | ❌ Not Implemented | Planned |
| `ao disk` | Storage | Mount, unmount, usage | ❌ Not Implemented | Planned |
| `ao pkg` | Packages | Install, remove, update | ✅ Implemented | Debian/Ubuntu (`apt`) |
| `ao log` | Logs | Tail service and system logs | ❌ Not Implemented | Planned |
| `ao boot` | Boot & Kernel | Bootloader defaults, kernel modules | ❌ Not Implemented | Planned |
| `ao gui` | Displays | Wayland/X11 detect, resolution | ❌ Not Implemented | Planned |
| `ao dev` | Devices | PCI/USB lists, Bluetooth, Printers | ❌ Not Implemented | Planned |
| `ao virt` | Virtualization | Docker, Podman, libvirt abstraction | ❌ Not Implemented | Planned |
| `ao sec` | Security | SELinux/AppArmor contexts, audits | ❌ Not Implemented | Planned |
| `ao distro` | Distributions | OS info, major release upgrades | ❌ Not Implemented | Planned |

## Architecture

`ao` relies on an abstraction layer (`src/os/mod.rs`). The CLI (`src/cli.rs`) routes commands to the `Detector` (`src/os/detector.rs`), which reads `/etc/os-release` and returns a Boxed Trait Object implementing the necessary command logic (e.g., `Apt` or `Systemd` in `src/os/debian.rs`).
