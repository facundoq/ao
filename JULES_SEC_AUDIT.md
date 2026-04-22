# Security Audit Report for Admin Operations (`ao`)

## Executive Summary

`ao` is a centralized command-line wrapper designed for Linux system administration. During a security audit of the codebase, multiple issues were identified, particularly revolving around argument injection, incorrect parameter order with security delimiters, and missing input validation.

When `ao` is used by a regular user on a desktop/server (perhaps via `sudo` or as root, given it manages system services, users, and disks), these vulnerabilities could be exploited to perform unauthorized actions.

## Findings

### 1. Argument Injection via Incorrect Use of `--` Delimiter

While `ao` commendably attempts to use the double-dash `--` delimiter to prevent argument injection, it fails to do so correctly for several commands, especially those wrapping `usermod` and `gpasswd`. The tools do not handle `--` in the way the developers intended when followed by options or arguments out of standard order, or the tools do not support `--` properly when placed before their own sub-arguments.

**Vulnerability 1.1: `usermod -aG -- <value> <username>`**
In `src/os/linux_generic/user.rs`, the `add-group` action is implemented as:
```rust
SystemCommand::new("usermod")
    .arg("-aG")
    .arg("--")
    .arg(&self.value)
    .arg(&self.username)
    .execute()
```
The `usermod` command considers `--` as the end of options, and the first non-option argument is strictly interpreted as the `LOGIN` (username). Therefore, `usermod` treats `--` as an invalid group name or similar error depending on parsing. This completely breaks the command functionality or could allow `-aG` argument injection if `--` was meant to stop option parsing for `self.value`.

**Vulnerability 1.2: `gpasswd -d <username> -- <value>`**
In `src/os/linux_generic/user.rs`, the `del-group` action is implemented as:
```rust
SystemCommand::new("gpasswd")
    .arg("-d")
    .arg(&self.username)
    .arg("--")
    .arg(&self.value)
    .execute()
```
`gpasswd` uses `USER GROUP` as positional arguments. However, placing `--` before the group name treats `--` as the group name.

**Vulnerability 1.3: `usermod -s -- <value> <username>` and `usermod -d <value> -m -- <username>`**
Similarly, for `shell` and `home` modifications, placing `--` before the shell value or home directory makes `usermod` interpret `--` as the shell/home value rather than an option terminator.

**Vulnerability 1.4: `ufw allow -- <rule>`**
In `src/os/linux_generic/net.rs`, the `fw_allow` and `fw_deny` commands try to use `--` with `ufw`:
```rust
SystemCommand::new("ufw").arg("allow").arg("--").arg(rule)
```
`ufw` does not support the `--` delimiter in this way. Passing `--` to `ufw` causes an error or ignores the rule.

### 2. Missing Input Validation for System Users and Groups

The generic Linux domain implementation (`src/os/linux_generic/user.rs` and `src/os/linux_generic/group.rs`) does not validate user-provided strings for `username`, `groupname`, or shell/home paths before passing them to the underlying binaries (`useradd`, `userdel`, `groupadd`, etc.).

Even though `SystemCommand` (which wraps `std::process::Command`) protects against bash shell injection (like `; rm -rf /`), it does NOT protect against Argument Injection if the input starts with a `-`.

For example, `ao group add -mygroup` would execute `groupadd -- -mygroup`, which returns an error because `-mygroup` is an invalid group name. However, for commands where `--` is incorrectly placed or not present, starting an input with `-` could lead to arbitrary flag injection (e.g., passing `-o` to `useradd` to allow duplicate UIDs).

### 3. Lack of Output Sanitization (Terminal Emulator Injection)

In many `ls` or `info` commands (e.g., `ao user ls`, `ao group ls`, `ao disk ls`), the tool reads raw system files (`/etc/passwd`, `/etc/group`, `lsblk` output) and prints the contents directly to the terminal using `comfy-table` or raw strings.

If a malicious user or service modifies the GECOS field in `/etc/passwd` to contain terminal escape sequences (e.g., ANSI escape codes for cursor movement or clearing the screen), `ao user ls` will print these sequences unaltered. This could lead to log spoofing or terminal UI manipulation, potentially tricking an administrator into running commands they did not intend to.

### 4. Insufficient Firewall Rule Sanitization

In `src/os/linux_generic/net.rs`, `is_safe_fw_rule` attempts to sanitize firewall rules by rejecting shell metacharacters and checking if the rule starts with `-`.
```rust
if rule.trim_start().starts_with('-') {
    return false;
}
```
While this is a good start, `firewall-cmd --add-rich-rule <rule> --permanent` could still be abused. If a user provides a rule that is syntactically valid for `firewall-cmd` but allows malicious access (e.g., opening port 22 globally when they shouldn't be able to), `ao` blindly passes it through. The validation only checks for bash injection, not semantic validity.

## Recommendations

1. **Fix Option Terminators**: Ensure that `--` is used correctly according to each tool's argument parsing logic. For `usermod` and `gpasswd`, `--` must be placed immediately before the positional arguments (e.g., `usermod -aG <group> -- <user>`).
2. **Remove `--` for Incompatible Tools**: Remove `--` from `ufw` commands, as it breaks the command syntax.
3. **Implement Robust Validation**: Add strict regex-based validation for user-provided names (usernames, groupnames) to ensure they conform to POSIX standards (e.g., `^[a-z_][a-z0-9_-]*$`).
4. **Sanitize Terminal Output**: Strip ANSI escape sequences and non-printable characters before rendering strings in `comfy-table` or standard output.
