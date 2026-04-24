use crate::os::ExecutableCommand;
use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;

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
        format!("{} {}", self.binary, self.args.join(" "))
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
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.00 TB");
        assert_eq!(
            format_bytes(1024 * 1024 * 1024 * 1024 + 256 * 1024 * 1024 * 1024),
            "1.25 TB"
        );
    }
}
