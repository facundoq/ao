use crate::os::ExecutableCommand;
use anyhow::{Context, Result, anyhow};
use std::io::Write;
use std::process::Command;
use std::time::Duration;

/// Run a `Command`, killing it and returning an error if it doesn't finish within `timeout`.
/// This prevents TUI hangs when subprocesses block indefinitely (e.g. systemctl over SSH
/// where D-Bus is unavailable, or docker when the socket is slow).
pub fn run_with_timeout(mut cmd: Command, timeout: Duration) -> Result<std::process::Output> {
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let start = std::time::Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut r| {
                        let mut v = Vec::new();
                        std::io::Read::read_to_end(&mut r, &mut v).ok();
                        v
                    })
                    .unwrap_or_default();
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut r| {
                        let mut v = Vec::new();
                        std::io::Read::read_to_end(&mut r, &mut v).ok();
                        v
                    })
                    .unwrap_or_default();
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            None => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(anyhow!("command timed out after {:?}", timeout));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

pub fn escape_shell_arg(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == '-'
            || c == '_'
            || c == '/'
            || c == '.'
            || c == ','
            || c == ':'
            || c == '@'
            || c == '='
            || c == '+'
    }) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

pub struct SystemCommand {
    pub binary: String,
    pub args: Vec<String>,
    pub stdin_data: Option<String>,
    pub ignore_exit_code: bool,
}

impl SystemCommand {
    pub fn new(binary: &str) -> Self {
        Self {
            binary: binary.to_string(),
            args: Vec::new(),
            stdin_data: None,
            ignore_exit_code: false,
        }
    }

    pub fn ignore_exit_code(mut self) -> Self {
        self.ignore_exit_code = true;
        self
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn args(mut self, args: &[String]) -> Self {
        for arg in args {
            self.args.push(arg.clone());
        }
        self
    }

    pub fn stdin(mut self, data: &str) -> Self {
        self.stdin_data = Some(data.to_string());
        self
    }
}

impl ExecutableCommand for SystemCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(&self.args);

        if let Some(data) = &self.stdin_data {
            cmd.stdin(std::process::Stdio::piped());
            let mut child = cmd
                .spawn()
                .with_context(|| format!("Failed to spawn {}", self.binary))?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(data.as_bytes())
                    .with_context(|| format!("Failed to write to {} stdin", self.binary))?;
            }
            let status = child
                .wait()
                .with_context(|| format!("Failed to wait on {}", self.binary))?;
            if !self.ignore_exit_code && !status.success() {
                anyhow::bail!("{} failed with status {}", self.binary, status);
            }
        } else {
            let status = cmd
                .status()
                .with_context(|| format!("Failed to execute {}", self.binary))?;
            if !self.ignore_exit_code && !status.success() {
                anyhow::bail!("{} failed with status {}", self.binary, status);
            }
        }
        Ok(())
    }

    fn as_string(&self) -> String {
        let mut parts = vec![escape_shell_arg(&self.binary)];
        for arg in &self.args {
            parts.push(escape_shell_arg(arg));
        }
        parts.join(" ")
    }
}

pub struct CompoundCommand {
    pub commands: Vec<Box<dyn ExecutableCommand>>,
}

impl CompoundCommand {
    pub fn new(commands: Vec<Box<dyn ExecutableCommand>>) -> Self {
        Self { commands }
    }
}

impl ExecutableCommand for CompoundCommand {
    fn execute(&self) -> Result<()> {
        for cmd in &self.commands {
            cmd.execute()?;
        }
        Ok(())
    }

    fn as_string(&self) -> String {
        self.commands
            .iter()
            .map(|cmd| cmd.as_string())
            .collect::<Vec<String>>()
            .join(" && ")
    }
}

pub struct NoopCommand;
impl ExecutableCommand for NoopCommand {
    fn execute(&self) -> Result<()> {
        Ok(())
    }
    fn as_string(&self) -> String {
        "".to_string()
    }
}

pub const BIN_SYSTEMCTL: &str = "systemctl";
pub const BIN_DOCKER: &str = "docker";
pub const BIN_LSBLK: &str = "lsblk";
pub const BIN_IP: &str = "ip";
pub const BIN_UFW: &str = "ufw";
pub const BIN_NMCLI: &str = "nmcli";

pub fn is_completing_arg(
    words: &[&str],
    cmd_parts: &[&str],
    arg_pos: usize,
    _last_word_complete: bool,
) -> bool {
    if words.len() < cmd_parts.len() {
        return false;
    }
    if !words.starts_with(cmd_parts) {
        return false;
    }

    let words_after_cmd = words.len() - cmd_parts.len();
    words_after_cmd == arg_pos
}

pub fn is_completing_action(
    words: &[&str],
    domain_name: &str,
    action: &str,
    arg_pos: usize,
) -> bool {
    is_completing_arg(
        words,
        &[crate::os::EXECUTABLE_NAME, domain_name, action],
        arg_pos,
        false,
    )
}

pub fn command_exists(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{}s", secs));
    }

    parts.join(" ")
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub enum Emoji {
    Up,
    Down,
    Unknown,
    Physical,
    Wireless,
    Virtual,
    Cpu,
    Ram,
    Network,
    Disk,
    Used,
    Total,
    Pci,
    Usb,
    Loop,
    Nvme,
    Ssd,
    Hdd,
    Printer,
}

impl Emoji {
    pub fn get(&self) -> &'static str {
        match self {
            Emoji::Up => "🟢",
            Emoji::Down => "🔴",
            Emoji::Unknown => "🟡",
            Emoji::Physical => "🏗️",
            Emoji::Wireless => "📶",
            Emoji::Virtual => "☁️",
            Emoji::Cpu => "💻",
            Emoji::Ram => "🧠",
            Emoji::Network => "🌐",
            Emoji::Disk => "💾",
            Emoji::Used => "⬛",
            Emoji::Total => "⬜",
            Emoji::Pci => "🏗️",
            Emoji::Usb => "🔌",
            Emoji::Loop => "🔁",
            Emoji::Nvme => "🚀",
            Emoji::Ssd => "⚡",
            Emoji::Hdd => "💾",
            Emoji::Printer => "🖨️",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists_true() {
        assert!(command_exists("ls"));
    }

    #[test]
    fn test_command_exists_false() {
        assert!(!command_exists("this_command_definitely_does_not_exist_12345"));
    }
}
