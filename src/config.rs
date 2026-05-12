use crate::cli::OutputFormat;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Pid,
    Cpu,
    Mem,
    Name,
    User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UiConfig {
    pub preferred_format: OutputFormat,
    pub show_command: bool,
    pub use_colors: bool,
    pub table_style: String,
    pub show_kernel_processes: bool,
    pub show_user_threads: bool,
    pub process_sort: ProcessSort,
    pub process_sort_descending: bool,
    pub process_use_tree_view: bool,
    pub process_tree_depth: u32,
    pub process_current_user_only: bool,
    pub process_filter: String,
    pub max_rx_throughput: u64,
    pub max_tx_throughput: u64,
    pub refresh_rate_ms: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            preferred_format: OutputFormat::Table,
            show_command: true,
            use_colors: true,
            table_style: "modern".to_string(),
            show_kernel_processes: false,
            show_user_threads: false,
            process_sort: ProcessSort::Cpu,
            process_sort_descending: true,
            process_use_tree_view: true,
            process_tree_depth: 2,
            process_current_user_only: false,
            process_filter: String::new(),
            max_rx_throughput: 1024,
            max_tx_throughput: 1024,
            refresh_rate_ms: 1000,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemConfig {
    pub sudo_command: Option<String>, // e.g. "sudo", "doas"
    pub editor: Option<String>,
    pub dry_run_by_default: bool,
}

impl Config {
    pub fn save(&self) -> Result<()> {
        let home = std::env::var("HOME").context("Failed to find HOME directory")?;
        let config_path = PathBuf::from(&home).join(".config/ao/config.toml");
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let home = std::env::var("HOME").context("Failed to find HOME directory")?;
        // Prompt requested ~/.config/main (odd but following instructions)
        // I'll check both ~/.config/main and ~/.config/ao/config.toml
        let paths = vec![
            PathBuf::from(&home).join(".config/main"),
            PathBuf::from(&home).join(".config/ao/config.toml"),
        ];

        for config_path in paths {
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)
                    .with_context(|| format!("Failed to read config at {:?}", config_path))?;

                let config: Config = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse TOML config at {:?}", config_path))?;

                return Ok(config);
            }
        }

        Ok(Config::default())
    }
}
