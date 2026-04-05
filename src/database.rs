use crate::core::{get_cc_switch_db_path, AppType, Provider};
use crate::error::Result;
use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open() -> Result<Self> {
        let db_path = get_cc_switch_db_path();
        if !db_path.exists() {
            return Err(crate::Error::Database(
                "CC Switch database not found. Please ensure CC Switch is installed.".into(),
            ));
        }

        let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_providers(&self, app_type: AppType) -> Result<Vec<Provider>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, settings_config, website_url, category, is_current 
             FROM providers WHERE app_type = ?1 ORDER BY sort_index, name",
        )?;

        let providers = stmt.query_map(params![app_type.as_str()], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let settings_config_str: String = row.get(2)?;
            let website_url: Option<String> = row.get(3)?;
            let category: Option<String> = row.get(4)?;
            let is_current: bool = row.get::<_, i32>(5)? != 0;

            let settings_config: JsonValue = serde_json::from_str(&settings_config_str)
                .unwrap_or(JsonValue::Object(serde_json::Map::new()));

            Ok(Provider {
                id,
                name,
                settings_config,
                website_url,
                category,
                is_current,
            })
        })?;

        providers
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| crate::Error::Database(e.to_string()))
    }

    pub fn get_current_provider(&self, app_type: AppType) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let current_id: Option<String> = conn
            .query_row(
                "SELECT id FROM providers WHERE app_type = ?1 AND is_current = 1",
                params![app_type.as_str()],
                |row| row.get(0),
            )
            .ok();

        Ok(current_id)
    }

    pub fn set_current_provider(&self, app_type: AppType, provider_id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        conn.execute(
            "UPDATE providers SET is_current = 0 WHERE app_type = ?1",
            params![app_type.as_str()],
        )?;

        conn.execute(
            "UPDATE providers SET is_current = 1 WHERE app_type = ?1 AND id = ?2",
            params![app_type.as_str(), provider_id],
        )?;

        Ok(())
    }

    pub fn get_provider(&self, app_type: AppType, provider_id: &str) -> Result<Option<Provider>> {
        let providers = self.get_providers(app_type)?;
        Ok(providers.into_iter().find(|p| p.id == provider_id))
    }
}
