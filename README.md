# ao

Admin Operations (`ao`) is a centralized, unified command line wrapper written in Rust designed to perform sysop operations across fragmented Linux environments. Instead of context-switching between `apt`, `dnf`, `systemctl`, `usermod`, and `ip`, you just use `ao`.

**ao** is distributed as a **statically compiled binary using musl**, providing a single standalone executable that works on most x86_64 Linux distributions without any external dependencies, with a relatively small footprint (there is work in progress to further reduce size; currently at ~1.5MB).

> [!CAUTION]
> **ao is currently Alpha software.** While designed for efficiency, it interacts with critical system components. Exercise caution and always verify commands in sensitive environments.
> 
> **Pro Tip:** If in doubt, run any command with the `--print` flag (e.g., `ao --print user add jane`) to see the exact bash commands that would be executed without actually running them.

## Philosophy

* **Ergonomic Speed:** Fast, predictable muscle memory (`ao <domain> <action>`).
* **Zero Overhead:** Instantaneous startup time via Rust.
* **Abstraction:** Hides the differences between Debian, Fedora, Arch, etc.

## Architecture

`ao` uses a multi-layered abstraction model to provide a consistent interface across different Linux distributions:

1. **CLI Layer (`src/cli.rs`)**: Defines the command-line interface and argument parsing using `clap`. It includes a global **Interactive Mode** for guided command execution.
2. **Domain Abstraction (`src/os/mod.rs`)**: Defines traits (e.g., `PackageManager`, `ServiceManager`, `LogManager`) and unified data structures that all backends must implement.
3. **OS Detection (`src/os/detector.rs`)**: Identifies the host distribution at runtime (via `/etc/os-release`).
4. **Backend Implementations**:
   - **Generic Linux (`src/os/linux_generic/`)**: Provides standard implementations for core domains using common tools like `ip`, `systemctl`, `journalctl`, and `lsblk`.
   - **Distro-Specific (`src/os/debian.rs`, `src/os/arch.rs`, etc.)**: Overrides generic behavior where necessary (e.g., package management via `apt` vs `pacman`).

## Command Tree

```text
ao
├── interactive
├── dashboard
├── boot
│   ├── list
│   └── module
│       ├── list
│       ├── load
│       └── unload
├── device
│   ├── list
│   ├── pci
│   ├── usb
│   ├── bluetooth
│   │   ├── status
│   │   ├── scan
│   │   ├── pair
│   │   └── connect
│   └── print
│       └── list
├── disk
│   └── list
├── distribution
│   ├── info
│   └── upgrade
├── group
│   ├── add
│   ├── delete
│   ├── list
│   └── modify
├── gui
│   ├── info
│   └── display
│       └── list
├── log
│   ├── auth
│   ├── boot
│   ├── crash
│   ├── dev
│   ├── error
│   ├── file
│   ├── package
│   ├── service
│   └── system
├── monitor
├── network
│   ├── interfaces
│   ├── ips
│   ├── routes
│   ├── firewall
│   │   ├── status
│   │   ├── allow
│   │   └── deny
│   └── wifi
│       ├── scan
│       └── connect
├── package
│   ├── add
│   ├── list
│   ├── delete
│   ├── search
│   └── update
├── partition
│   ├── list
│   ├── mount
│   ├── unmount
│   └── usage
├── security
│   ├── audit
│   └── context
├── self
│   ├── completions
│   │   ├── generate
│   │   ├── install
│   │   └── setup
│   ├── info
│   └── update
├── service
│   ├── down
│   ├── list
│   ├── reload
│   ├── restart
│   ├── status
│   └── up
├── system
│   ├── info
│   ├── power
│   └── time
├── user
│   ├── add
│   ├── delete
│   ├── list
│   ├── modify
│   ├── passwd
│   └── session
└── virtualization
    ├── list
    ├── start
    ├── stop
    ├── remove
    └── logs
```

## Installation

### Via Cargo

You can install `ao` directly from [crates.io](https://crates.io/crates/ao-cli):

```bash
cargo install ao-cli
```

**Note:** The crate is named `ao-cli` due to naming availability, but the installed binary will be named `ao`.

### Binary Downloads

Statically compiled binaries are available for every release in the [GitHub Releases](https://github.com/facundoq/ao/releases) section.

