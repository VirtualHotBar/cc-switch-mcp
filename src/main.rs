use cc_switch_mcp::McpServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CC Switch MCP Server...");

    let server = McpServer::new()?;
    server.run()?;

    Ok(())
}
