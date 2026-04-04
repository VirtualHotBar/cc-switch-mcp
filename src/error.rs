use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Invalid provider configuration: {0}")]
    InvalidConfig(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown app type: {0}")]
    UnknownAppType(String),
}

pub type Result<T> = std::result::Result<T, Error>;
