//! HTTP代理服务器 (Stub)
//!
//! 在没有 Tauri GUI 支持时的空实现

use super::types::*;
use crate::database::Database;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 代理服务器状态（共享）
#[derive(Clone)]
pub struct ProxyState {
    pub db: Arc<Database>,
    pub config: Arc<RwLock<ProxyConfig>>,
    pub status: Arc<RwLock<ProxyStatus>>,
    pub start_time: Arc<RwLock<Option<std::time::Instant>>>,
    pub current_providers: Arc<RwLock<std::collections::HashMap<String, (String, String)>>>,
    pub provider_router: Arc<super::provider_router::ProviderRouter>,
}

/// 代理HTTP服务器 (Stub)
pub struct ProxyServer {
    _config: ProxyConfig,
    _state: ProxyState,
}

impl ProxyServer {
    pub fn new(
        config: ProxyConfig,
        db: Arc<Database>,
        _app_handle: Option<()>,
    ) -> Self {
        let status = Arc::new(RwLock::new(ProxyStatus::default()));
        let current_providers = Arc::new(RwLock::new(std::collections::HashMap::new()));
        let provider_router = Arc::new(super::provider_router::ProviderRouter::new(db.clone()));
        
        Self {
            _config: config.clone(),
            _state: ProxyState {
                db,
                config: Arc::new(RwLock::new(config)),
                status,
                start_time: Arc::new(RwLock::new(None)),
                current_providers,
                provider_router,
            },
        }
    }

    pub async fn start(&self) -> Result<ProxyServerInfo, String> {
        Err("代理服务器需要 Tauri GUI 支持".to_string())
    }

    pub async fn stop(&self) -> Result<(), String> {
        Ok(())
    }

    pub async fn get_status(&self) -> ProxyStatus {
        ProxyStatus::default()
    }

    pub async fn set_active_target(&self, _app_type: &str, _provider_id: &str, _provider_name: &str) {
        // Stub - do nothing
    }

    pub async fn apply_runtime_config(&self, _config: &ProxyConfig) {
        // Stub - do nothing
    }

    pub async fn update_circuit_breaker_configs(&self, _config: super::circuit_breaker::CircuitBreakerConfig) {
        // Stub - do nothing
    }

    pub async fn reset_provider_circuit_breaker(&self, _provider_id: &str, _app_type: &str) {
        // Stub - do nothing
    }
}