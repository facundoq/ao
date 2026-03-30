# Admin Operations (`ao`)

## 1. Philosophy
The modern Linux ecosystem is powerful but highly fragmented. Administrators must context-switch between disparate syntaxes (e.g., `ip addr`, `systemctl restart`, `usermod -aG`, `ufw allow`). 

`ao` is designed to be the **Grand Unified Wrapper** for Linux system administration. 

**Core Tenets:**
* **Ergonomic Speed:** The base command (`ao`) is designed for optimal left-right-left keystroke flow (`A` -> `O` -> `TAB`). Subcommands should prioritize short, memorable, noun-verb structures.
* **Discoverability over Memorization:** Users should not need to read `man` pages to remember how to add a user to a group. The command tree must be intuitively navigable via terminal auto-completion (`ao <TAB><TAB>`).
* **Zero Overhead:** Written in compiled Rust, `ao` must execute instantaneously. There should be no noticeable interpreter startup lag.
* **Safe Execution:** System administration is destructive by nature. `ao` should support dry-runs (`--dry-run`) where applicable, showing the exact underlying shell commands it intends to execute before pulling the trigger.
* **Abstraction, Not Replacement:** `ao` does not reinvent `systemd` or `iproute2`. It orchestrates them. It abstracts the underlying OS differences where possible (e.g., `ao pkg install` should map to `apt` on Debian and `dnf` on Fedora).

## 2. Scope
**In-Scope:**
* Local system administration and configuration.
* Wrapping standard Linux utilities (networking, services, users, storage, packages).
* Providing normalized, human-readable tabular output for system state.

**Out-of-Scope:**
* Remote infrastructure provisioning (leave this to Terraform/Ansible).
* Replacing daemon-level services (it is a client, not a server).
* Complete cross-platform support (Windows/macOS). `ao` is a Linux-first tool.

## 3. Technologies
To achieve C-level execution speed with memory safety and a maintainable codebase, the following Rust stack is recommended:

* **Language:** Rust (Latest Stable Edition).
* **CLI Parser:** `clap` (with the `derive` feature). It is the undisputed standard for Rust CLIs, automatically generating help menus, shell completions, and handling deeply nested command trees.
* **Error Handling:** `anyhow` (for bubbling up top-level application errors) and `thiserror` (for defining specific, typed library-level errors, like `UserAlreadyExists` or `NetworkInterfaceNotFound`).
* **Command Execution:** `std::process::Command` for standard execution. For complex piping between commands, the `duct` crate provides excellent ergonomics.
* **Output Formatting:** `comfy-table` for rendering dynamic, responsive terminal tables (e.g., listing users or network interfaces), and `colored` or `crossterm` for semantic color-coding (red for stopped services, green for active).
* **Serialization/Config:** `serde` and `serde_derive` alongside `toml` to parse an optional user configuration file (e.g., `~/.config/ao/ao.toml` to define default package managers or alias preferences).

## 4. Command Tree Draft
The architecture follows a strict `<base> <domain> <action> [target]` syntax. 

### Core System (`ao sys`)
* `ao sys info` (Retrieves OS, kernel, uptime, load averages)
* `ao sys power [reboot|shutdown|suspend]`
* `ao sys update` (Updates the system package tree)

### Services / Daemons (`ao service`)
* `ao service list` (Lists active services/daemons)
* `ao service up <name>` (Maps to `systemctl start` + `systemctl enable`)
* `ao service down <name>` (Maps to `systemctl stop` + `systemctl disable`)
* `ao service restart <name>`
* `ao service status <name>`

### Users & Groups (`ao user`)
* `ao user list` (Displays human users, filtering out system accounts by default)
* `ao user add <username>`
* `ao user del <username> [--purge]` (Removes user and optionally home directory)
* `ao user mod <username> --group <group>`
* `ao user passwd <username>`

### Networking (`ao net`)
* `ao net list` (Shows interfaces, IP addresses, and link states)
* `ao net up <interface>` (Maps to `ip link set dev <interface> up`)
* `ao net down <interface>`
* `ao net route` (Displays routing table)
* `ao net ping <target>`
* **Firewall Sub-domain (`ao net fw`)**
    * `ao net fw status`
    * `ao net fw allow <port>/<protocol>`
    * `ao net fw block <ip>`

### Storage & Filesystems (`ao disk`)
* `ao disk list` (Clean, tabular output of `lsblk` + `df -h`)
* `ao disk mount <device> <path>`
* `ao disk unmount <path>`
* `ao disk usage <path>` (Fast directory size calculation, similar to `du` or `ncdu`)

### Packages (`ao pkg`)
* *Note: This domain dynamically detects the host OS package manager.*
* `ao pkg install <name>`
* `ao pkg remove <name>`
* `ao pkg search <name>`
* `ao pkg list` (Lists explicitly installed user packages)

### Logs (`ao log`)
* `ao log tail <service>` (Maps to `journalctl -u <service> -f`)
* `ao log sys [--errors-only]` (Tails system-wide syslog/journalctl)

