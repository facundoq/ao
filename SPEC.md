# Admin Operations (`ao`)

## 1. Philosophy
The modern Linux ecosystem is powerful but highly fragmented. Administrators must context-switch between disparate syntaxes (e.g., `ip addr`, `systemctl restart`, `usermod -aG`, `ufw allow`). 

`ao` is designed to be the **Grand Unified Wrapper** for Linux system administration. 

**Core Tenets:**

* **Ergonomic Speed:** The base command (`ao`) is designed for optimal left-right-left keystroke flow (`A` -> `O` -> `TAB`). Subcommands should prioritize short, memorable, noun-verb structures. The goal is that typing `ao` becomes muscle memory. For a system administrator, speed and accuracy are paramount. By standardizing the prefix and verb structures, we reduce cognitive load significantly. Every command is predictable.

* **Discoverability over Memorization:** Users should not need to read `man` pages to remember how to add a user to a group, or what flags are needed to configure a static IP. The command tree must be intuitively navigable via terminal auto-completion (`ao <TAB><TAB>`). A hierarchical domain-action-target structure means users can intuitively guess the right command: `ao net list`, `ao user add`, `ao service restart`. It makes administration accessible to newcomers and frictionless for veterans.

* **Zero Overhead:** Written in compiled Rust, `ao` must execute instantaneously. There should be no noticeable interpreter startup lag (unlike Python-based wrappers or heavy Node.js tools). System administration often involves scripts looping over many entities; `ao` must be lean enough to be used inside shell loops without causing performance degradation. It must be as fast as calling the native underlying binaries.

* **Safe Execution:** System administration is destructive by nature. A typo can take down a network interface, delete a critical user, or format the wrong partition. `ao` should support dry-runs (`--dry-run`) where applicable, showing the exact underlying shell commands it intends to execute before pulling the trigger. Furthermore, destructive commands (like purging a user or formatting a disk) should have interactive confirmations by default unless explicitly overridden (e.g., `--force` or `-y`).

* **Abstraction, Not Replacement:** `ao` does not reinvent `systemd`, `iproute2`, or `apt`. It orchestrates them. It abstracts the underlying OS differences where possible. For instance, `ao pkg install` should seamlessly map to `apt` on Debian/Ubuntu, `dnf` on Fedora, `pacman` on Arch, and `zypper` on openSUSE. The goal is not to write a new init system, but to provide a consistent interface over whatever init system is present.

* **Standardized Output:** Across all commands, `ao` should provide standardized tabular output for listing items (users, interfaces, disks) and consistent key-value output for details. This consistency makes it easier for users to scan information visually and parse it programmatically (e.g., with `--json` or `--yaml` output modes).

## 2. Scope

**In-Scope:**
* Local system administration and configuration. This covers the everyday tasks of a sysadmin managing a single node, from setting up networks to managing storage arrays.
* Wrapping standard Linux utilities (networking, services, users, storage, packages). The wrapper must handle the complexities of these underlying tools, providing a simplified, consistent interface.
* Providing normalized, human-readable tabular output for system state. When querying state (e.g., `ao net list`), the output should be structured, optionally color-coded, and easy to parse both by humans and machines.
* Advanced system aspects previously untouched by traditional wrappers, including bootloaders, display servers, security contexts (SELinux/AppArmor), and virtualization/containerization layers.
* Integration with standard system logs, providing a unified way to tail logs for services, containers, or the entire system.

**Out-of-Scope:**
* Remote infrastructure provisioning. `ao` is not a replacement for Ansible, Chef, Puppet, or Terraform. It is the tool you might invoke *within* those provisioning scripts, but it does not handle SSH multiplexing or fleet-wide orchestration itself.
* Replacing daemon-level services. It is a client, a transient command-line utility, not a background server or daemon. It has no long-running process of its own.
* Complete cross-platform support (Windows/macOS). `ao` is a Linux-first tool, designed to integrate deeply with the Linux kernel and userland. It relies heavily on Linux-specific paradigms like `/sys`, `/proc`, `systemd`, and `iproute2`.

## 3. Technologies

To achieve C-level execution speed with memory safety and a maintainable codebase, the following Rust stack is mandated:

* **Language:** Rust (Latest Stable Edition). Rust provides the necessary performance characteristics while eliminating entire classes of memory safety bugs common in C/C++ systems programming. It also offers a fantastic ecosystem for CLI development.
* **CLI Parser:** `clap` (with the `derive` feature). It is the undisputed standard for Rust CLIs, automatically generating help menus, shell completions for bash/zsh/fish, and handling deeply nested command trees efficiently. We will rely heavily on `clap`'s subcommand routing.
* **Error Handling:** `anyhow` (for bubbling up top-level application errors) and `thiserror` (for defining specific, typed library-level errors, like `UserAlreadyExists`, `NetworkInterfaceNotFound`, or `PermissionDenied`). Good error messages are critical for a sysadmin tool; they must be actionable and clear, indicating exactly which underlying command failed and why.
* **Command Execution:** `std::process::Command` for standard execution. For complex piping between commands or capturing output asynchronously, the `duct` crate provides excellent ergonomics. Where we need to run commands with elevated privileges gracefully, we will handle `sudo` wrapping or check for root permissions directly at startup.
* **Output Formatting:** `comfy-table` for rendering dynamic, responsive terminal tables (e.g., listing users or network interfaces). `colored` or `crossterm` for semantic color-coding (e.g., red for stopped/failed services, green for active, yellow for warnings). We will also implement a global `--output` flag (e.g., `--output json`) utilizing `serde_json` for machine-readability.
* **Serialization/Config:** `serde` and `serde_derive` alongside `toml` to parse an optional user configuration file (e.g., `/etc/ao/ao.toml` or `~/.config/ao/ao.toml`). This allows for system-wide or user-specific defaults, such as preferred package managers, default output formats, aliases, or color preferences.
* **System Interfacing:** Crates like `nix` and `libc` for direct system calls when wrapping shell binaries is too slow or error-prone (e.g., getting the current user ID, checking file permissions, or querying basic system stats).

## 4. Comprehensive Command Syntax Tree

The architecture follows a strict `<base> <domain> <action> [target] [flags]` syntax. Every domain is isolated and has a distinct set of responsibilities.

### 4.1 Core System (`ao sys`)
This domain handles kernel, operating system updates, power management, and high-level system metrics.

* `ao sys info`
  * **Behavior:** Retrieves comprehensive OS info, kernel version, uptime, load averages, memory usage, and swap availability. Maps to a human-readable combination of `uname -a`, `/proc/loadavg`, `/proc/meminfo`, and `/etc/os-release`.
  * **Flags:** `--json` for programmatic consumption.
* `ao sys power <state>`
  * **Arguments:** `reboot`, `shutdown`, `suspend`, `hibernate`.
  * **Behavior:** Initiates system power state transitions, safely wrapping `systemctl reboot/poweroff/suspend/hibernate`.
  * **Flags:** `--force` to bypass normal init procedures (wraps `reboot -f`), `--now` to execute immediately.
* `ao sys time <action>`
  * **Actions:** `status`, `set`, `sync`.
  * **Behavior:** Modifies or views the system time and timezone. Maps to `timedatectl`. E.g., `ao sys time set "America/New_York"`.

### 4.2 Services / Daemons (`ao service` or `ao svc`)
This domain replaces `systemctl` for common service management, providing semantic color output (green/red) based on service status.

* `ao svc list`
  * **Behavior:** Lists all active and failed services on the system in a table, showing Name, State (running/stopped/failed), and Start Time.
  * **Flags:** `--all` to include inactive services, `--user` to list user-level systemd services.
* `ao svc up <name>`
  * **Behavior:** Starts and enables a service to start on boot. Maps to `systemctl enable --now <name>`.
* `ao svc down <name>`
  * **Behavior:** Stops and disables a service from starting on boot. Maps to `systemctl disable --now <name>`.
* `ao svc restart <name>`
  * **Behavior:** Restarts the specified service. Maps to `systemctl restart <name>`.
* `ao svc reload <name>`
  * **Behavior:** Reloads the service configuration without fully stopping it. Maps to `systemctl reload <name>`.
* `ao svc status <name>`
  * **Behavior:** Displays detailed status, recent log entries, and current process IDs for the service. Maps to `systemctl status <name>`.

### 4.3 Users & Groups (`ao user`)
Simplifies `useradd`, `usermod`, `passwd`, and `groupmod` into an intuitive interface.

* `ao user list`
  * **Behavior:** Displays a table of all users with IDs >= 1000 (filtering out system accounts by default). Shows Username, UID, GID, Home Directory, and Default Shell.
  * **Flags:** `--all` to show system users, `--groups` to append secondary group memberships.
* `ao user add <username>`
  * **Behavior:** Creates a new user. Automatically creates a home directory and sets up skeleton files. Maps to `useradd -m <username>`.
  * **Flags:** `--groups <g1,g2>` to append to secondary groups, `--shell <path>` to specify a default shell (e.g., `/bin/zsh`), `--system` to create a system account.
* `ao user del <username>`
  * **Behavior:** Deletes the specified user account. Maps to `userdel <username>`.
  * **Flags:** `--purge` or `-p` to remove the user's home directory and mail spool (`userdel -r`).
* `ao user mod <username> <action> <value>`
  * **Actions:** `add-group`, `del-group`, `shell`, `home`.
  * **Behavior:** Modifies user properties. E.g., `ao user mod john add-group docker` wraps `usermod -aG docker john`.
* `ao user passwd <username>`
  * **Behavior:** Initiates an interactive password reset prompt for the user. Maps to `passwd <username>`.
* **Groups Sub-domain (`ao group`)**
  * `ao group list` (Lists all groups on the system via `/etc/group`).
  * `ao group add <groupname>` (Creates a new group via `groupadd <groupname>`).
  * `ao group del <groupname>` (Deletes a group via `groupdel <groupname>`).
  * `ao group mod <groupname> --gid <id>` (Modifies an existing group).

### 4.4 Networking (`ao net`)
Replaces the disparate suite of `ip`, `ping`, `ss`, and firewall tools.

* `ao net list`
  * **Behavior:** Shows an organized table of all network interfaces, their assigned IPv4/IPv6 addresses, MAC addresses, and UP/DOWN link states. Maps to parsing `ip -j a`.
* `ao net link <interface> <state>`
  * **Arguments:** `up`, `down`.
  * **Behavior:** Brings a network interface online or offline. Maps to `ip link set dev <interface> up/down`.
* `ao net ip <action> <interface> <address>`
  * **Actions:** `add`, `del`.
  * **Behavior:** Adds or removes a static IP address to an interface. Maps to `ip addr add <addr> dev <interface>`.
* `ao net route`
  * **Behavior:** Displays the system routing table cleanly. Maps to `ip route show`.
* `ao net ping <target>`
  * **Behavior:** Pings a target IP or domain, returning latency statistics. Wraps standard `ping`.
* **Firewall Sub-domain (`ao net fw`)**
  * `ao net fw status` (Shows active firewall rules. Automatically detects `ufw`, `firewalld`, or `iptables`).
  * `ao net fw allow <port>/<protocol>` (e.g., `ao net fw allow 80/tcp`).
  * `ao net fw block <ip>` (Drops incoming traffic from an IP).
* **Wi-Fi Sub-domain (`ao net wifi`)**
  * `ao net wifi scan` (Scans and lists available SSIDs, signal strengths, and security types via `nmcli` or `iw`).
  * `ao net wifi connect <ssid>` (Connects to a network. Will interactively prompt for a password if required. Maps to `nmcli dev wifi connect <ssid>`).
  * `ao net wifi list` (Shows currently known/saved networks).
  * `ao net wifi forget <ssid>` (Deletes a saved network profile).

### 4.5 Storage & Filesystems (`ao disk`)
Replaces `lsblk`, `df`, `du`, `mount`, and `umount`.

* `ao disk list`
  * **Behavior:** Produces a clean, combined tabular output of block devices (`lsblk`) and filesystem usage (`df -h`). Shows Device, Type, Size, Used%, and Mountpoint.
* `ao disk mount <device> <path>`
  * **Behavior:** Mounts a block device to a directory. Maps to `mount <device> <path>`.
  * **Flags:** `--type` to specify the filesystem (`-t`), `--options` or `-o` for mount flags (e.g., `ro,noexec`).
* `ao disk unmount <path_or_device>`
  * **Behavior:** Safely unmounts a device. Maps to `umount <target>`.
  * **Flags:** `--lazy` (`-l`), `--force` (`-f`).
* `ao disk usage <path>`
  * **Behavior:** Calculates directory size incredibly fast (similar to `ncdu` or `du -sh`).
  * **Flags:** `--depth <N>` to control directory traversal depth.

### 4.6 Packages (`ao pkg`)
Dynamically abstracts over `apt`, `dnf`, `pacman`, `zypper`, and `apk`.

* `ao pkg update`
  * **Behavior:** Updates the system package tree and applies available upgrades. Wraps the underlying package manager (`apt update && apt upgrade -y`, `dnf update`, `pacman -Syu`).
  * **Flags:** `--dry-run` to list what would be updated without executing.
* `ao pkg install <name...>`
  * **Behavior:** Installs one or more packages.
* `ao pkg remove <name...>`
  * **Behavior:** Uninstalls packages.
  * **Flags:** `--purge` to completely remove configuration files alongside the binary.
* `ao pkg search <query>`
  * **Behavior:** Searches the upstream package repositories.
* `ao pkg list`
  * **Behavior:** Lists all explicitly installed user packages (filtering out dependencies if supported by the underlying PM).

### 4.7 Logs (`ao log`)
Provides a unified view into system and service logs.

* `ao log tail <service>`
  * **Behavior:** Tails the live logs of a specific service. Maps to `journalctl -u <service> -f`.
  * **Flags:** `--lines <N>` to show the last N lines (`-n N`).
* `ao log sys`
  * **Behavior:** Tails system-wide syslog or journald.
  * **Flags:** `--errors` (`-p err`) to filter output to errors and critical failures only.
* `ao log file <path>`
  * **Behavior:** Fast trailing of generic text log files. A structured wrapper around `tail -f`.

### 4.8 Boot & Kernel (`ao boot`)
Manages the bootloader (GRUB/systemd-boot) and Linux kernel modules.

* `ao boot list`
  * **Behavior:** Lists available kernel entries in the bootloader.
* `ao boot default <entry>`
  * **Behavior:** Sets the default boot entry for the next boot. Maps to `grub-set-default` or `bootctl set-default`.
* `ao boot update`
  * **Behavior:** Regenerates the bootloader configuration. Maps to `update-grub`, `grub-mkconfig`, or `bootctl update`.
* **Kernel Modules Sub-domain (`ao boot mod`)**
  * `ao boot mod list` (Lists currently loaded kernel modules via `lsmod`).
  * `ao boot mod load <module>` (Loads a module via `modprobe <module>`).
  * `ao boot mod unload <module>` (Unloads a module via `modprobe -r <module>`).

### 4.9 Desktop Environments & GUI (`ao gui`)
A specialized domain for managing display servers, Wayland/X11, and monitors.

* `ao gui info`
  * **Behavior:** Detects and outputs the active Display Server (Wayland/X11), Desktop Environment (GNOME, KDE, Sway), and Window Manager.
* `ao gui restart`
  * **Behavior:** Attempts to gracefully restart the display manager (e.g., `systemctl restart gdm` or `sddm`).
* **Display Sub-domain (`ao gui display`)**
  * `ao gui display list` (Lists connected monitors, current resolutions, and refresh rates using `xrandr` or `wlr-randr`).
  * `ao gui display set <output> <resolution>` (Sets a specific output to a resolution. E.g., `ao gui display set DP-1 1920x1080@144`).

### 4.10 Devices (`ao dev`)
Manages physical peripheral devices, including USBs, Bluetooth, and Printers.

* `ao dev list`
  * **Behavior:** Summarizes connected PCI and USB devices. Wraps `lspci` and `lsusb` into a cleaner table.
* **Bluetooth Sub-domain (`ao dev bt`)**
  * `ao dev bt status` (Checks if bluetooth daemon is running and radio is powered).
  * `ao dev bt scan` (Scans for nearby devices).
  * `ao dev bt pair <mac>` (Pairs with a device via `bluetoothctl`).
  * `ao dev bt connect <mac>` (Connects to a paired device).
* **Printer Sub-domain (`ao dev print`)**
  * `ao dev print list` (Lists active CUPS printers and their queues).
  * `ao dev print cancel <job_id>` (Cancels a print job).

### 4.11 Virtualization & Containers (`ao virt`)
Abstracts management of Docker, Podman, and KVM/libvirt.

* `ao virt ps`
  * **Behavior:** Lists all running containers and active VMs. Dynamically detects if Docker, Podman, or Libvirt are installed and merges their status outputs.
* `ao virt start <id_or_name>`
  * **Behavior:** Starts a stopped container or VM.
* `ao virt stop <id_or_name>`
  * **Behavior:** Stops a running container or VM.
* `ao virt rm <id_or_name>`
  * **Behavior:** Removes a container or VM.
* `ao virt logs <id_or_name>`
  * **Behavior:** Tails the logs of a running container.

### 4.12 Security & Permissions (`ao sec`)
Manages system-level security contexts, capabilities,

* `ao sec audit`
  * **Behavior:** Runs a basic security audit. Checks for open privileged ports, files with SUID bits set, and active firewall status.
* `ao sec context`
  * **Behavior:** Outputs the current state of SELinux or AppArmor (e.g., Enforcing, Permissive, Disabled). Maps to `sestatus` or `aa-status`.

### 4.13 Distributions (`ao distro`)
Manages the lifecycle and major version upgrades of the host operating system.

* `ao distro info`
  * **Behavior:** Outputs detailed release information, architecture, and end-of-life status for the current distribution.
* `ao distro upgrade`
  * **Behavior:** Initiates a major version upgrade of the host OS. This wraps complex, distribution-specific tooling gracefully (e.g., `do-release-upgrade` on Ubuntu, `dnf system-upgrade` on Fedora).
  * **Flags:** `--check-only` to verify if a new release is available, `--devel` to upgrade to a pre-release version.


## 5. Parameter Handling Proposal

A robust CLI must handle parameters—both optional and required—gracefully. Administrators hate tools that fail abruptly with vague "Missing Argument X" errors after they've already typed a long command. The `ao` tool proposes a hybrid model for parameter parsing using `clap`'s advanced features, structured arguments, and intelligent interactive prompts.

### 5.1 The `clap` Parse-and-Validate Pipeline

When a user types a command (e.g., `ao user add --groups wheel --shell /bin/zsh john`), the following pipeline executes:

1.  **Lexical Analysis:** The shell tokenizer splits the command into an array of strings.
2.  **`clap` Parsing:** The `clap` parser maps strings to specific fields in an internal Rust struct (e.g., `AddUserCommand`). `clap` handles the ordering, allowing flags to appear before or after the positional arguments (e.g., `ao user add john --shell /bin/bash` is identical to `ao user add --shell /bin/bash john`).
3.  **Type Validation:** `clap` validates types immediately. If `--uid` expects an integer, providing `--uid root` fails before any business logic executes.
4.  **Domain Validation:** Custom Rust code validates the domain context. Does the user `john` already exist? Does the group `wheel` exist? If validation fails, `ao` returns a typed error mapped to an exit code.

### 5.2 Structured Arguments

Complex tasks require structured parameters. Rather than parsing long string representations manually, `ao` relies heavily on standard struct types.

For example, network interface configuration:

```rust
#[derive(Parser)]
pub struct AddIpCommand {
    /// The network interface (e.g., eth0, wlan0)
    #[arg(required = true)]
    pub interface: String,

    /// The IP address to assign in CIDR notation (e.g., 192.168.1.50/24)
    #[arg(required = true, value_parser = parse_cidr)]
    pub address: IpNet,
}
```

The `value_parser` attribute invokes a custom function (`parse_cidr`) that parses the string into a valid `IpNet` struct using the `ipnet` crate. If the user passes an invalid CIDR block, the error message is precise: `Error: '192.168.1.50' is not a valid CIDR block. Expected format: <ip>/<prefix>`.

### 5.3 Interactive Prompting (The "Missing Parameter" Fallback)

The most significant ergonomic improvement `ao` offers is interactive fallback.

If a command requires multiple parameters but the user forgets them, `ao` should not immediately fail. Instead, it should enter an interactive mode (unless piped or running non-interactively).

**Scenario: User Creation**

Traditional `useradd`:
```bash
$ useradd
Usage: useradd [options] LOGIN
```

`ao` execution:
```bash
$ ao user add
? Username: jsmith
? Primary Group [jsmith]: developers
? Additional Groups (comma separated) []: wheel,docker
? Shell [/bin/bash]: /bin/zsh
? Create home directory? [Y/n]: y

Executing: useradd -m -g developers -G wheel,docker -s /bin/zsh jsmith
User 'jsmith' created successfully.
```

**Implementation Strategy:**

1.  **Detect TTY:** `ao` uses `crossterm::tty::IsTty` or `atty` to determine if `stdout` and `stdin` are connected to a terminal.
2.  **Evaluate Completeness:** If the parsed `clap` struct is missing required fields, and the process is attached to a TTY, `ao` transitions into prompt mode rather than exiting.
3.  **Dialogue Crate:** The `dialoguer` crate is used to present clean, interactive prompts with sensible defaults. The prompts are mapped directly to the missing fields in the `clap` struct.
4.  **Non-Interactive Execution:** If the user specifies `--non-interactive` (or `-y`), or if `stdin` is a pipe (e.g., `echo "jsmith" | ao user add`), the command fails immediately with a standard missing argument error.

### 5.4 Global Flags and Overrides

Certain parameters must be globally available across all domains. These are handled by a top-level `GlobalOptions` struct that encapsulates all subcommands.

*   `--dry-run` (`-n`): Print the underlying shell commands without executing them. Crucial for destructive operations (disk formatting, network down).
*   `--force` (`-f`): Bypass confirmation prompts (e.g., `ao user del jsmith -f`).
*   `--output <format>`: Override standard terminal table output with `json`, `yaml`, or `csv`. Useful for programmatic ingestion.
*   `--sudo`: Explicitly elevate privileges for the entire command tree execution.
*   `--verbose` (`-v`, `-vv`, `-vvv`): Increase logging verbosity (using the `log` and `env_logger` crates).

### 5.5 Extensibility and Aliasing

To accommodate varying administrator preferences, parameters can be customized via the `~/.config/ao/ao.toml` configuration file.

Administrators can define defaults. If a user always prefers `/bin/zsh` for new users, they can set it:

```toml
[user.add]
default_shell = "/bin/zsh"
default_groups = ["users"]
```

When `ao user add` executes, the parameter resolution order is:
1.  **Explicit CLI Flag** (e.g., `--shell /bin/fish`) - Highest priority.
2.  **Configuration File** (e.g., `default_shell = "/bin/zsh"`).
3.  **Hardcoded Program Default** (e.g., `/bin/bash`).

This hierarchical parameter resolution ensures `ao` is predictable, configurable, and significantly faster to use than raw Linux commands.
