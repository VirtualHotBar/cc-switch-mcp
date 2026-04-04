# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-05

### Added
- Initial release of CC Switch MCP Server
- Provider management tools (list, add, switch, delete, get current)
- Universal providers for cross-app configuration
- MCP servers management tools
- Full MCP protocol support (2024-11-05)
- Database integration with CC Switch desktop app
- Unit tests (8 tests)
- Integration tests
- Support for Claude, Codex, Gemini CLI, OpenCode, and OpenClaw

### Features
- `list_providers` - List all providers for a CLI tool
- `add_provider` - Add a new provider configuration
- `switch_provider` - Switch to a specific provider
- `delete_provider` - Delete a provider
- `get_current_provider` - Get the currently active provider
- `list_universal_providers` - List all universal providers
- `add_universal_provider` - Add a cross-app provider
- `delete_universal_provider` - Delete a universal provider
- `list_mcp_servers` - List all MCP server configurations
- `add_mcp_server` - Add a new MCP server
- `delete_mcp_server` - Delete an MCP server
- `get_db_path` - Get the path to CC Switch database

### Technical
- Built with Rust 1.85+
- Uses Tokio async runtime
- SQLite database with rusqlite
- MCP protocol implementation
- Cross-platform support (Windows, macOS, Linux)