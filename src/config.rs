use std::fs;
use std::path::PathBuf;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
impl Default for Config {
    fn default() -> Self {
        Self {
            api_base_url: "https://modparks.pitan76.net/api/v1".to_string(),
            api_key: String::new(),
        }
    }
}
    pub fn get_path() -> Result<PathBuf> {
        let mut path = config_dir().context("Unable to locate config directory")?;
        path.push("modparks-cli");
        fs::create_dir_all(&path).context("Failed to create config directory")?;
        path.push("config.json");
        Ok(path)
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        if !path.exists() {
            // Return default config
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let mut cfg: Config = serde_json::from_str(&content).context("Failed to parse config JSON")?;
        if cfg.api_base_url.is_empty() {
            cfg.api_base_url = Self::default().api_base_url;
        }
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let json = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, json).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }
}
