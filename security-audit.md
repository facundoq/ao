# Security Audit Report: `ao` CLI

This report outlines 20 potential security issues and areas of high risk identified during a deep security audit of the `ao` project. The issues are categorized by domain and severity.

## High Severity: Command & Argument Injection

1.  **Missing `--` Delimiter in `usermod` (Argument Injection):** In `src/os/linux_generic/user.rs` (`UserModCommand::execute`), user-supplied values for modifications (e.g., changing shell or home directory) might be passed directly to `usermod` without a `--` separator. An attacker could supply a value starting with `-` to inject arbitrary flags.
2.  **Unsanitized WiFi SSID (`ao net wifi connect`):** In `src/os/linux_generic/net.rs`, the SSID provided to `nmcli dev wifi connect -- <ssid>` is separated by `--`, which prevents flag injection, but if the SSID is used elsewhere in scripts or lacks quotes, it could lead to shell injection if interpreted by a shell down the line. (Currently mitigated by `SystemCommand` not using a shell, but risky).
3.  **Firewall Rule Injection Risk:** While `is_safe_fw_rule` exists in `src/os/linux_generic/net.rs`, complex rule inputs to `ufw allow -- <rule>` or `firewall-cmd --add-rich-rule <rule>` might still harbor logic flaws or bypasses if the regex/character filter isn't exhaustive against all `firewalld` rich rule vulnerabilities.
4.  **Mount Options Injection (`ao partition mount`):** In `src/os/linux_generic/partition.rs`, if custom mount options are accepted from the user and passed directly to `mount -o <options>`, an attacker could inject dangerous options (e.g., `suid`, `dev`, `exec` on untrusted filesystems) leading to privilege escalation.
5.  **Service Name Injection:** In `src/os/linux_generic/svc.rs` and `log.rs`, passing user-controlled service names to `systemctl` or `journalctl -u` could lead to unintended service interactions or information disclosure if a malicious service name contains wildcard characters or path traversals (e.g., `journalctl -u ../../../etc/shadow`).
6.  **Package Name Injection (Distro Modules):** In modules like `src/os/alpine.rs` or `debian.rs`, package names passed to `apk add` or `apt-get install` must be strictly validated to prevent flag injection or installing malicious local packages if the path is manipulated.

## High Severity: Arbitrary File Access & Path Traversal

7.  **Arbitrary File Tailing (`ao log file`):** In `src/os/linux_generic/log.rs`, the command allows tailing arbitrary files using `tail -n <lines> -f <path>`. A user running `ao` with elevated privileges could read sensitive files like `/etc/shadow` or private SSH keys.
8.  **Arbitrary Path Mounting (`ao partition mount/unmount`):** In `src/os/linux_generic/partition.rs`, allowing the user to mount or unmount arbitrary source and destination paths without validating if they are restricted system directories (e.g., `/sys`, `/proc`, `/etc`) can allow an attacker to overlay sensitive configuration files.
9.  **Configuration Path Hijacking:** In `src/config.rs`, the application loads configuration from a generic path (e.g., `~/.config/main`). If the directory permissions are too loose, another local user could modify this configuration to alter `ao`'s behavior when run by a victim.
10. **Path Traversal in Workspace/Disk Operations:** Commands operating on disk paths (e.g., `ao partition usage <path>`) must sanitize `<path>` to prevent traversing outside intended boundaries, especially if `ao` is ever exposed via an API or restricted shell.

## Medium Severity: Privilege & State Escalation

11. **Insecure Password Handling (`ao user passwd`):** In `src/os/linux_generic/user.rs`, passwords read from `rpassword` may reside in memory in `String` types. `String` does not zeroize its memory upon dropping, leaving plaintext passwords vulnerable to memory dumps or core dumps.
12. **Missing Privilege Escalation Checks:** The tool executes system-level commands (e.g., `systemctl`, `mount`, `useradd`) but relies on the underlying command to fail if permissions are insufficient. Providing a pre-flight privilege check (e.g., checking `UID == 0` or Polkit capabilities) would prevent partial executions or confusing error states that might leave the system in a degraded state.
13. **Insecure Modifying of Shell Completion Scripts:** In `src/os/linux_generic/self_domain.rs`, generating and installing shell completions might overwrite existing user profiles (`.bashrc`, `.zshrc`) insecurely. If an attacker controls the environment variables used to resolve home directories, they could write to arbitrary locations.
14. **Process Signal Spoofing:** If `ao` implements process killing (e.g., in a future process manager module based on the dashboard), relying solely on PID can lead to race conditions where a PID is reused by a highly privileged process just as the kill signal is sent.

## Medium/Low Severity: Denial of Service & Panics

15. **Unsafe Unwraps (`unwrap()`, `expect()`):** Throughout the codebase (e.g., parsing command outputs in `net.rs`, `sys.rs`), there are potential `unwrap()` calls on options or results that are not guaranteed to be `Some` or `Ok`. If a system tool (like `xrandr` or `dmidecode`) returns unexpected output formats, `ao` will panic, causing a Denial of Service for the CLI.
16. **Brittle Output Parsing:** Parsing the output of `ip addr`, `dmidecode`, or `/etc/passwd` manually using `split()` and specific indices is highly brittle. Changes in the output format of these underlying tools or edge cases (like users with colons in their names) will cause parsing failures or misinterpretation of security-relevant data.
17. **Unbounded Output Consumption:** When executing commands that return large amounts of data (e.g., `journalctl` without line limits in some edge cases, or reading huge files in `ao log file`), reading the entire output into memory via `String::from_utf8_lossy(&output.stdout)` can lead to Out-Of-Memory (OOM) crashes.
18. **Blocking Interactive Commands:** If an underlying command (like a package manager waiting for confirmation without the `-y` flag) hangs, `ao` will block indefinitely. `ao` must ensure all spawned commands use non-interactive flags or wrap them with timeouts.

## Low Severity: Information Disclosure & Logging

19. **Excessive Error Verbosity:** When commands fail (e.g., `systemctl` fails to start a service), passing the raw, unfiltered stderr directly back to the user might reveal internal system structures, paths, or sensitive environmental variables that were present during execution.
20. **Lack of Audit Logging for `ao` Actions:** `ao` performs critical system modifications (adding users, altering firewalls) but does not seem to log its own actions to syslog or an audit trail. If a malicious user utilizes `ao` to modify the system, it will be difficult for an administrator to trace the actions back to the specific `ao` invocation versus a direct tool invocation.

---
*Audit conducted via static code analysis of the `src/` and `src/os/` directory structures, focusing on command execution patterns, data parsing, and domain logic.*
