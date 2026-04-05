mod core;
mod database;
mod config_sync;
mod error;
mod mcp_server;

pub use error::{Error, Result};
pub use mcp_server::McpServer;

pub const VERSION: &str = "0.2.0";
pub const SERVER_NAME: &str = "cc-switch-mcp";