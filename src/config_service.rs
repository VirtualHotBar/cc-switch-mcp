use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::provider::Provider;

pub struct ConfigService;

impl ConfigService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_claude_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".claude").join("settings.json"))
    }

    pub fn get_codex_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".codex").join("config.toml"))
    }

    pub fn get_gemini_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".gemini").join("settings.json"))
    }

    pub fn get_opencode_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".opencode").join("opencode.json"))
    }

    pub fn get_openclaw_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".openclaw").join("openclaw.json"))
    }

    pub fn sync_provider_to_claude(&self, provider: &Provider) -> Result<()> {
        let config_path = Self::get_claude_config_path()?;

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<serde_json::Value>(&content)?
        } else {
            json!({})
        };

        if let Some(env) = provider.settings_config.get("env") {
            if let Some(env_obj) = config.as_object_mut() {
                if let Some(env_map) = env.as_object() {
                    for (key, value) in env_map {
                        env_obj.insert(format!("env.{}", key), value.clone());
                    }
                }
            }
        }

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;

        tracing::info!("Synced provider {} to Claude config", provider.id);
        Ok(())
    }

    pub fn sync_provider_to_codex(&self, provider: &Provider) -> Result<()> {
        let config_path = Self::get_codex_config_path()?;

        if let Some(config_toml) = provider.settings_config.get("config") {
            if let Some(toml_str) = config_toml.as_str() {
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&config_path, toml_str)?;
                tracing::info!("Synced provider {} to Codex config", provider.id);
            }
        }

        Ok(())
    }

    pub fn sync_provider_to_gemini(&self, provider: &Provider) -> Result<()> {
        let config_path = Self::get_gemini_config_path()?;

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<serde_json::Value>(&content)?
        } else {
            json!({})
        };

        if let Some(env) = provider.settings_config.get("env") {
            if let Some(config_obj) = config.as_object_mut() {
                if let Some(env_map) = env.as_object() {
                    for (key, value) in env_map {
                        config_obj.insert(format!("env.{}", key), value.clone());
                    }
                }
            }
        }

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;

        tracing::info!("Synced provider {} to Gemini config", provider.id);
        Ok(())
    }

    pub fn sync_provider_to_opencode(&self, provider: &Provider) -> Result<()> {
        let config_path = Self::get_opencode_config_path()?;

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<serde_json::Value>(&content)?
        } else {
            json!({})
        };

        if let Some(settings) = provider.settings_config.get("settings") {
            if let Some(settings_map) = settings.as_object() {
                if let Some(config_obj) = config.as_object_mut() {
                    for (key, value) in settings_map {
                        config_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;

        tracing::info!("Synced provider {} to OpenCode config", provider.id);
        Ok(())
    }

    pub fn sync_provider_to_live(&self, app: &str, provider: &Provider) -> Result<()> {
        match app {
            "claude" => self.sync_provider_to_claude(provider),
            "codex" => self.sync_provider_to_codex(provider),
            "gemini" => self.sync_provider_to_gemini(provider),
            "opencode" | "openclaw" => self.sync_provider_to_opencode(provider),
            _ => Err(anyhow::anyhow!("Unknown app: {}", app)),
        }
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}
