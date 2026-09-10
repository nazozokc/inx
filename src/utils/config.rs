use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils::constants::DEFAULT_REGISTRY;

/// Configuration for inx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub registry: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry: DEFAULT_REGISTRY.to_string(),
        }
    }
}

/// Get the inx directory (~/.inx).
pub fn get_inx_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".inx"))
}

/// Get the config file path (~/.inx/config.json).
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_inx_dir()?.join("config.json"))
}

/// Get the registry directory (~/.inx/registry).
pub fn get_registry_dir() -> Result<PathBuf> {
    Ok(get_inx_dir()?.join("registry"))
}

/// Get the packages directory (~/.inx/packages).
pub fn get_packages_dir() -> Result<PathBuf> {
    Ok(get_inx_dir()?.join("packages"))
}

/// Validate a registry URL. Only https: and ssh: protocols are allowed.
pub fn validate_registry_url(url: &str) -> bool {
    if let Ok(parsed) = url::Url::parse(url) {
        matches!(parsed.scheme(), "https" | "ssh")
    } else {
        false
    }
}

/// Load the configuration, falling back to defaults on any error.
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    match fs::read_to_string(&config_path) {
        Ok(data) => {
            let config: Config = serde_json::from_str(&data)
                .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;
            if !validate_registry_url(&config.registry) {
                anyhow::bail!(
                    "Invalid registry URL: {}. Only https: and ssh: protocols are allowed.",
                    config.registry
                );
            }
            Ok(config)
        }
        Err(_) => Ok(Config::default()),
    }
}

/// Ensure the inx directories exist.
pub fn ensure_inx_dirs() -> Result<()> {
    fs::create_dir_all(get_inx_dir()?)?;
    fs::create_dir_all(get_packages_dir()?)?;
    Ok(())
}