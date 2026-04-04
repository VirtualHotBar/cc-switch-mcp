use crate::config_service::ConfigService;
use crate::database::Database;
use crate::error::Result;
use crate::provider::{Provider, ProviderManager};

pub struct ProviderService {
    pub db: Database,
    pub config: ConfigService,
}

impl ProviderService {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        let config = ConfigService::new();
        Ok(Self { db, config })
    }

    pub fn list_providers(&self, app: &str) -> Result<ProviderManager> {
        self.db.get_provider_manager(app)
    }

    pub fn add_provider(&self, app: &str, provider: &Provider, set_current: bool) -> Result<()> {
        self.db.save_provider(app, provider, set_current)?;

        if set_current {
            self.config.sync_provider_to_live(app, provider)?;
        }

        Ok(())
    }

    pub fn switch_provider(&self, app: &str, id: &str) -> Result<bool> {
        let success = self.db.set_current_provider(app, id)?;

        if success {
            if let Some(provider) = self.db.get_provider_manager(app)?.get_provider(id) {
                self.config.sync_provider_to_live(app, provider)?;
            }
        }

        Ok(success)
    }

    pub fn delete_provider(&self, app: &str, id: &str) -> Result<bool> {
        let success = self.db.delete_provider(app, id)?;
        Ok(success)
    }

    pub fn get_current_provider(&self, app: &str) -> Result<Option<Provider>> {
        let manager = self.db.get_provider_manager(app)?;
        if manager.current.is_empty() {
            Ok(None)
        } else {
            Ok(manager.get_provider(&manager.current).cloned())
        }
    }

    pub fn get_db(&self) -> &Database {
        &self.db
    }
}
