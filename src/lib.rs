mod config_service;
mod database;
mod error;
mod mcp_server;
mod provider;
mod provider_service;

pub use config_service::ConfigService;
pub use database::Database;
pub use error::{Error, Result};
pub use mcp_server::McpServer;
pub use provider::{Provider, ProviderManager, UniversalProvider};
pub use provider_service::ProviderService;

pub const VERSION: &str = "0.1.1";
pub const SERVER_NAME: &str = "cc-switch-mcp";
