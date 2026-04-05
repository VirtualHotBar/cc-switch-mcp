use crate::core::{get_home_dir, AppType, Provider};
use crate::error::Result;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;

pub fn sync_provider_to_config(app_type: AppType, provider: &Provider) -> Result<()> {
    let home = get_home_dir();
    let config_path = app_type.config_path(&home);

    match app_type {
        AppType::Claude => sync_claude_config(&config_path, provider),
        AppType::Codex => sync_codex_config(&config_path, provider),
        AppType::Gemini => sync_gemini_config(&config_path, provider),
        AppType::OpenCode => sync_opencode_config(&config_path, provider),
        AppType::OpenClaw => sync_openclaw_config(&config_path, provider),
    }
}

fn sync_claude_config(path: &PathBuf, provider: &Provider) -> Result<()> {
    let mut config: JsonValue = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        JsonValue::Object(serde_json::Map::new())
    };

    if let JsonValue::Object(ref mut obj) = config {
        if let JsonValue::Object(ref settings) = provider.settings_config {
            if let Some(env) = settings.get("env") {
                obj.insert("env".to_string(), env.clone());
            }
        }
    }

    fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn sync_codex_config(path: &PathBuf, provider: &Provider) -> Result<()> {
    let config_content = if let JsonValue::Object(ref settings) = provider.settings_config {
        settings
            .get("config")
            .and_then(|c| c.as_str())
            .unwrap_or("")
    } else {
        ""
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, config_content)?;
    Ok(())
}

fn sync_gemini_config(path: &PathBuf, provider: &Provider) -> Result<()> {
    let mut config: JsonValue = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        JsonValue::Object(serde_json::Map::new())
    };

    if let JsonValue::Object(ref mut obj) = config {
        if let JsonValue::Object(ref settings) = provider.settings_config {
            for (key, value) in settings.iter() {
                obj.insert(key.clone(), value.clone());
            }
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn sync_opencode_config(path: &PathBuf, provider: &Provider) -> Result<()> {
    let mut config: JsonValue = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        JsonValue::Object(serde_json::Map::new())
    };

    if let JsonValue::Object(ref mut obj) = config {
        if let JsonValue::Object(ref settings) = provider.settings_config {
            for (key, value) in settings.iter() {
                obj.insert(key.clone(), value.clone());
            }
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn sync_openclaw_config(path: &PathBuf, provider: &Provider) -> Result<()> {
    let mut config: JsonValue = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        JsonValue::Object(serde_json::Map::new())
    };

    if let JsonValue::Object(ref mut obj) = config {
        if let JsonValue::Object(ref settings) = provider.settings_config {
            for (key, value) in settings.iter() {
                obj.insert(key.clone(), value.clone());
            }
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}
