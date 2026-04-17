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

#[derive(Serialize, Deserialize, Debug)]
pub struct UiConfig {
    pub preferred_format: OutputFormat,
    pub show_command: bool,
    pub use_colors: bool,
    pub table_style: String, // e.g. "modern", "blank", "rounded"
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            preferred_format: OutputFormat::Table,
            show_command: true,
            use_colors: true,
            table_style: "modern".to_string(),
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
