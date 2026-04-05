use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub settings_config: JsonValue,
    pub website_url: Option<String>,
    pub category: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Claude,
    Codex,
    Gemini,
    OpenCode,
    OpenClaw,
}

impl AppType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppType::Claude => "claude",
            AppType::Codex => "codex",
            AppType::Gemini => "gemini",
            AppType::OpenCode => "opencode",
            AppType::OpenClaw => "openclaw",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(AppType::Claude),
            "codex" => Some(AppType::Codex),
            "gemini" => Some(AppType::Gemini),
            "opencode" => Some(AppType::OpenCode),
            "openclaw" => Some(AppType::OpenClaw),
            _ => None,
        }
    }

    pub fn config_path(&self, home: &PathBuf) -> PathBuf {
        match self {
            AppType::Claude => home.join(".claude.json"),
            AppType::Codex => home.join(".codex").join("config.toml"),
            AppType::Gemini => home.join(".gemini").join("settings.json"),
            AppType::OpenCode => home.join(".config").join("opencode").join("config.json"),
            AppType::OpenClaw => home.join(".openclaw").join("config.json"),
        }
    }
}

pub fn get_cc_switch_db_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cc-switch")
        .join("cc-switch.db")
}

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().expect("Could not find home directory")
}
