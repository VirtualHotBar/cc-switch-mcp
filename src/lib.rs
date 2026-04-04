mod error;
mod mcp_server;
mod provider;
mod database;

pub use error::{Error, Result};
pub use mcp_server::McpServer;
pub use provider::{Provider, ProviderManager, UniversalProvider};

pub const VERSION: &str = "0.1.0";
pub const SERVER_NAME: &str = "cc-switch-mcp";