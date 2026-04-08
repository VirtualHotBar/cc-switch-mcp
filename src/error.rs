use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Invalid app type: {0}")]
    InvalidApp(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("Invalid API key format: {0}")]
    InvalidApiKey(String),
}

pub type Result<T> = std::result::Result<T, Error>;
