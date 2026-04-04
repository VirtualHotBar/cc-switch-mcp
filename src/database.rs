use crate::error::{Error, Result};
use crate::provider::{Provider, ProviderManager, UniversalProvider, UniversalProviderManager};
use dirs::home_dir;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const DB_FILENAME: &str = "cc-switch.db";

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl Database {
    pub fn new() -> Result<Self> {
        let home = home_dir().ok_or_else(|| Error::Config("Cannot find home directory".into()))?;
        let cc_switch_dir = home.join(".cc-switch");
        let db_path = cc_switch_dir.join(DB_FILENAME);

        if !cc_switch_dir.exists() {
            std::fs::create_dir_all(&cc_switch_dir)?;
        }

        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };
        db.init_tables()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS providers (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                name TEXT NOT NULL,
                settings_config TEXT NOT NULL,
                website_url TEXT,
                category TEXT,
                created_at INTEGER,
                sort_index INTEGER,
                notes TEXT,
                icon TEXT,
                icon_color TEXT,
                meta TEXT NOT NULL DEFAULT '{}',
                is_current BOOLEAN NOT NULL DEFAULT 0,
                in_failover_queue BOOLEAN NOT NULL DEFAULT 0,
                PRIMARY KEY (id, app_type)
            );

            CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                server_config TEXT NOT NULL,
                description TEXT,
                homepage TEXT,
                docs TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                enabled_claude BOOLEAN NOT NULL DEFAULT 0,
                enabled_codex BOOLEAN NOT NULL DEFAULT 0,
                enabled_gemini BOOLEAN NOT NULL DEFAULT 0,
                enabled_opencode BOOLEAN NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                directory TEXT NOT NULL,
                repo_owner TEXT,
                repo_name TEXT,
                repo_branch TEXT DEFAULT 'main',
                readme_url TEXT,
                enabled_claude BOOLEAN NOT NULL DEFAULT 0,
                enabled_codex BOOLEAN NOT NULL DEFAULT 0,
                enabled_gemini BOOLEAN NOT NULL DEFAULT 0,
                enabled_opencode BOOLEAN NOT NULL DEFAULT 0,
                installed_at INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                description TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER,
                updated_at INTEGER,
                PRIMARY KEY (id, app_type)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_providers_app_type ON providers(app_type);
            "#,
        )?;
        Ok(())
    }

    fn normalize_app_type(app: &str) -> String {
        match app {
            "claude" => "claude".to_string(),
            "codex" => "codex".to_string(),
            "gemini" => "gemini".to_string(),
            "opencode" => "opencode".to_string(),
            "openclaw" => "opencode".to_string(),
            _ => app.to_string(),
        }
    }

    pub fn get_provider_manager(&self, app: &str) -> Result<ProviderManager> {
        let app_type = Self::normalize_app_type(app);
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        let mut stmt = conn.prepare(
            "SELECT id, name, settings_config, website_url, category, created_at, sort_index, notes, meta, icon, icon_color, in_failover_queue, is_current 
             FROM providers WHERE app_type = ?1 ORDER BY sort_index"
        )?;

        let providers = stmt.query_map(params![app_type], |row| {
            let settings_config_str: String = row.get(2)?;
            let settings_config: serde_json::Value = serde_json::from_str(&settings_config_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            let meta_str: String = row.get(8)?;
            let meta: Option<crate::provider::ProviderMeta> = serde_json::from_str(&meta_str).ok();

            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                settings_config,
                website_url: row.get(3)?,
                category: row.get(4)?,
                created_at: row.get(5)?,
                sort_index: row.get(6)?,
                notes: row.get(7)?,
                meta,
                icon: row.get(9)?,
                icon_color: row.get(10)?,
                in_failover_queue: row.get::<_, bool>(11)?,
            })
        })?;

        let mut manager = ProviderManager::new();
        for provider in providers {
            let provider = provider?;
            manager.providers.insert(provider.id.clone(), provider);
        }

        let current: Option<String> = conn
            .query_row(
                "SELECT id FROM providers WHERE app_type = ?1 AND is_current = 1",
                params![app_type],
                |row| row.get(0),
            )
            .ok();

        manager.current = current.unwrap_or_default();

        Ok(manager)
    }

    pub fn save_provider(&self, app: &str, provider: &Provider, is_current: bool) -> Result<()> {
        let app_type = Self::normalize_app_type(app);
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        let settings_config_str = serde_json::to_string(&provider.settings_config)?;
        let meta_str = provider
            .meta
            .as_ref()
            .map(|m| serde_json::to_string(m))
            .transpose()?
            .unwrap_or_else(|| "{}".to_string());

        conn.execute(
            "INSERT OR REPLACE INTO providers (
                id, app_type, name, settings_config, website_url, category, created_at, sort_index, notes, meta, icon, icon_color, in_failover_queue, is_current
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                provider.id,
                app_type,
                provider.name,
                settings_config_str,
                provider.website_url,
                provider.category,
                provider.created_at,
                provider.sort_index,
                provider.notes,
                meta_str,
                provider.icon,
                provider.icon_color,
                provider.in_failover_queue,
                is_current,
            ],
        )?;

        if is_current {
            conn.execute(
                "UPDATE providers SET is_current = 0 WHERE app_type = ?1 AND id != ?2",
                params![app_type, provider.id],
            )?;
        }

        Ok(())
    }

    pub fn delete_provider(&self, app: &str, id: &str) -> Result<bool> {
        let app_type = Self::normalize_app_type(app);
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        let rows = conn.execute(
            "DELETE FROM providers WHERE id = ?1 AND app_type = ?2",
            params![id, app_type],
        )?;

        Ok(rows > 0)
    }

    pub fn set_current_provider(&self, app: &str, id: &str) -> Result<bool> {
        let app_type = Self::normalize_app_type(app);
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM providers WHERE id = ?1 AND app_type = ?2",
            params![id, app_type],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(false);
        }

        conn.execute(
            "UPDATE providers SET is_current = 0 WHERE app_type = ?1",
            params![app_type],
        )?;

        conn.execute(
            "UPDATE providers SET is_current = 1 WHERE id = ?1 AND app_type = ?2",
            params![id, app_type],
        )?;

        Ok(true)
    }

    pub fn get_universal_provider_manager(&self) -> Result<UniversalProviderManager> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS universal_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                apps TEXT NOT NULL DEFAULT '{}',
                base_url TEXT NOT NULL,
                api_key TEXT NOT NULL,
                models TEXT DEFAULT '{}',
                website_url TEXT,
                notes TEXT,
                icon TEXT,
                icon_color TEXT,
                meta TEXT,
                created_at INTEGER,
                sort_index INTEGER
            )",
            [],
        )
        .ok();

        let mut stmt = conn.prepare(
            "SELECT id, name, provider_type, apps, base_url, api_key, models, website_url, notes, icon, icon_color, meta, created_at, sort_index
             FROM universal_providers ORDER BY sort_index"
        )?;

        let providers = stmt.query_map([], |row| {
            let apps_str: String = row.get(3)?;
            let apps: crate::provider::UniversalProviderApps = serde_json::from_str(&apps_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            let models_str: String = row.get(6)?;
            let models: crate::provider::UniversalProviderModels =
                serde_json::from_str(&models_str)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            let meta_str: Option<String> = row.get(11)?;
            let meta: Option<crate::provider::ProviderMeta> =
                meta_str.and_then(|s| serde_json::from_str(&s).ok());

            Ok(UniversalProvider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                apps,
                base_url: row.get(4)?,
                api_key: row.get(5)?,
                models,
                website_url: row.get(7)?,
                notes: row.get(8)?,
                icon: row.get(9)?,
                icon_color: row.get(10)?,
                meta,
                created_at: row.get(12)?,
                sort_index: row.get(13)?,
            })
        });

        let mut manager = UniversalProviderManager::new();
        match providers {
            Ok(providers_iter) => {
                for provider in providers_iter {
                    match provider {
                        Ok(p) => manager.providers.insert(p.id.clone(), p),
                        Err(e) => {
                            eprintln!("Error parsing provider: {:?}", e);
                            continue;
                        }
                    };
                }
            }
            Err(e) => {
                eprintln!("Error querying providers: {:?}", e);
            }
        }

        Ok(manager)
    }

    pub fn save_universal_provider(&self, provider: &UniversalProvider) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS universal_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                apps TEXT NOT NULL DEFAULT '{}',
                base_url TEXT NOT NULL,
                api_key TEXT NOT NULL,
                models TEXT DEFAULT '{}',
                website_url TEXT,
                notes TEXT,
                icon TEXT,
                icon_color TEXT,
                meta TEXT,
                created_at INTEGER,
                sort_index INTEGER
            )",
            [],
        )?;

        let apps_str = serde_json::to_string(&provider.apps)?;
        let models_str = serde_json::to_string(&provider.models)?;
        let meta_str = provider
            .meta
            .as_ref()
            .map(|m| serde_json::to_string(m))
            .transpose()?;

        conn.execute(
            "INSERT OR REPLACE INTO universal_providers (
                id, name, provider_type, apps, base_url, api_key, models, website_url, notes, icon, icon_color, meta, created_at, sort_index
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                provider.id,
                provider.name,
                provider.provider_type,
                apps_str,
                provider.base_url,
                provider.api_key,
                models_str,
                provider.website_url,
                provider.notes,
                provider.icon,
                provider.icon_color,
                meta_str,
                provider.created_at,
                provider.sort_index,
            ],
        )?;

        Ok(())
    }

    pub fn delete_universal_provider(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        let rows = conn.execute("DELETE FROM universal_providers WHERE id = ?1", params![id])?;

        Ok(rows > 0)
    }

    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    pub fn get_mcp_servers(&self) -> Result<Vec<McpServerConfig>> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        let mut stmt = conn.prepare(
            "SELECT id, name, server_config, description, homepage, docs, tags, enabled_claude, enabled_codex, enabled_gemini, enabled_opencode FROM mcp_servers"
        )?;

        let servers = stmt.query_map([], |row| {
            Ok(McpServerConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                server_config: row.get(2)?,
                description: row.get(3)?,
                homepage: row.get(4)?,
                docs: row.get(5)?,
                tags: row.get(6)?,
                enabled_claude: row.get(7)?,
                enabled_codex: row.get(8)?,
                enabled_gemini: row.get(9)?,
                enabled_opencode: row.get(10)?,
            })
        })?;

        let mut result = Vec::new();
        for server in servers {
            result.push(server?);
        }

        Ok(result)
    }

    pub fn save_mcp_server(&self, server: &McpServerConfig) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO mcp_servers (
                id, name, server_config, description, homepage, docs, tags, enabled_claude, enabled_codex, enabled_gemini, enabled_opencode
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                server.id,
                server.name,
                server.server_config,
                server.description,
                server.homepage,
                server.docs,
                server.tags,
                server.enabled_claude,
                server.enabled_codex,
                server.enabled_gemini,
                server.enabled_opencode,
            ],
        )?;

        Ok(())
    }

    pub fn delete_mcp_server(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidPath(
                "Cannot lock connection".into(),
            ))
        })?;
        Ok(conn.execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])? > 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub server_config: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub enabled_claude: bool,
    #[serde(default)]
    pub enabled_codex: bool,
    #[serde(default)]
    pub enabled_gemini: bool,
    #[serde(default)]
    pub enabled_opencode: bool,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Provider, UniversalProvider};
    use serde_json::json;

    #[test]
    fn test_database_new_in_memory() {
        let db = Database::new_in_memory().expect("Failed to create in-memory database");
        assert_eq!(db.get_db_path().to_str(), Some(":memory:"));
    }

    #[test]
    fn test_save_and_list_providers() {
        let db = Database::new_in_memory().expect("Failed to create database");

        let provider = Provider::new(
            "test-id".to_string(),
            "Test Provider".to_string(),
            json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.test.com",
                    "ANTHROPIC_AUTH_TOKEN": "test-key"
                }
            }),
        );

        db.save_provider("claude", &provider, false)
            .expect("Failed to save provider");

        let manager = db
            .get_provider_manager("claude")
            .expect("Failed to get providers");
        assert_eq!(manager.providers.len(), 1);
        assert!(manager.providers.contains_key("test-id"));
    }

    #[test]
    fn test_set_current_provider() {
        let db = Database::new_in_memory().expect("Failed to create database");

        let provider1 = Provider::new(
            "provider-1".to_string(),
            "Provider 1".to_string(),
            json!({"env": {"API_KEY": "key1"}}),
        );
        let provider2 = Provider::new(
            "provider-2".to_string(),
            "Provider 2".to_string(),
            json!({"env": {"API_KEY": "key2"}}),
        );

        db.save_provider("claude", &provider1, false)
            .expect("Failed to save provider1");
        db.save_provider("claude", &provider2, false)
            .expect("Failed to save provider2");

        let success = db
            .set_current_provider("claude", "provider-2")
            .expect("Failed to set current");
        assert!(success);

        let manager = db
            .get_provider_manager("claude")
            .expect("Failed to get providers");
        assert_eq!(manager.current, "provider-2");
    }

    #[test]
    fn test_delete_provider() {
        let db = Database::new_in_memory().expect("Failed to create database");

        let provider = Provider::new("test-delete".to_string(), "Test".to_string(), json!({}));

        db.save_provider("claude", &provider, false)
            .expect("Failed to save");

        let success = db
            .delete_provider("claude", "test-delete")
            .expect("Failed to delete");
        assert!(success);

        let manager = db
            .get_provider_manager("claude")
            .expect("Failed to get providers");
        assert_eq!(manager.providers.len(), 0);
    }

    #[test]
    fn test_universal_provider() {
        let db = Database::new_in_memory().expect("Failed to create database");

        let provider = UniversalProvider::new(
            "universal-test".to_string(),
            "Test Universal".to_string(),
            "newapi".to_string(),
            "https://api.test.com".to_string(),
            "test-key".to_string(),
        );

        db.save_universal_provider(&provider)
            .expect("Failed to save universal provider");

        let manager = db
            .get_universal_provider_manager()
            .expect("Failed to get universal providers");
        assert_eq!(manager.providers.len(), 1);
        assert!(manager.providers.contains_key("universal-test"));
    }

    #[test]
    fn test_provider_to_claude_config() {
        let mut provider = UniversalProvider::new(
            "test".to_string(),
            "Test".to_string(),
            "newapi".to_string(),
            "https://api.test.com".to_string(),
            "test-key".to_string(),
        );
        provider.apps.claude = true;

        let claude_provider = provider
            .to_claude_provider()
            .expect("Should return claude provider");
        assert!(claude_provider.id.starts_with("universal-claude-"));
        assert!(claude_provider.settings_config.get("env").is_some());
    }

    #[test]
    fn test_normalize_app_type() {
        assert_eq!(Database::normalize_app_type("claude"), "claude");
        assert_eq!(Database::normalize_app_type("codex"), "codex");
        assert_eq!(Database::normalize_app_type("openclaw"), "opencode");
        assert_eq!(Database::normalize_app_type("unknown"), "unknown");
    }

    #[test]
    fn test_mcp_servers() {
        let db = Database::new_in_memory().expect("Failed to create database");

        let server = McpServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            server_config: r#"{"command": "test"}"#.to_string(),
            description: Some("A test server".to_string()),
            homepage: None,
            docs: None,
            tags: "[]".to_string(),
            enabled_claude: true,
            enabled_codex: false,
            enabled_gemini: false,
            enabled_opencode: false,
        };

        db.save_mcp_server(&server)
            .expect("Failed to save MCP server");

        let servers = db.get_mcp_servers().expect("Failed to get MCP servers");
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "Test Server");
        assert!(servers[0].enabled_claude);
    }
}
