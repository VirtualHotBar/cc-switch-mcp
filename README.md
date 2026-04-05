# CC Switch MCP Server

[![npm version](https://badge.fury.io/js/@imvhb%2Fcc-switch-mcp-server.svg)](https://badge.fury.io/js/@imvhb/cc-switch-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-VirtualHotBar%2Fcc--switch--mcp-blue)](https://github.com/VirtualHotBar/cc-switch-mcp)

English | [简体中文](./README_CN.md)

Standalone MCP Server for [CC Switch](https://github.com/farion1231/cc-switch) - providing provider management capabilities for Claude Code, Codex, Gemini CLI, OpenCode, and OpenClaw.

**Key Features:**
- 🚀 Built with CC Switch's core library for identical behavior
- 🔄 Automatic config file synchronization
- 💾 Direct database access to CC Switch's SQLite database
- 🪶 No Tauri dependencies - lightweight binary
- 🤝 Works alongside CC Switch desktop app

## Installation

### From NPM

```bash
npm install -g @imvhb/cc-switch-mcp-server
```

### From Source

```bash
git clone https://github.com/VirtualHotBar/cc-switch-mcp.git
cd cc-switch-mcp
cargo build --release --no-default-features
```

The binary will be at `target/release/cc-switch-mcp` (Linux/macOS) or `target/release/cc-switch-mcp.exe` (Windows).

## Usage

### Configure with Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "cc-switch-mcp"
    }
  }
}
```

### Configure with OpenCode

Add to your OpenCode configuration (`~/.config/opencode/config.json`):

```json
{
  "mcp": {
    "servers": {
      "cc-switch": {
        "command": "cc-switch-mcp"
      }
    }
  }
}
```

### Configure with Gemini CLI

Add to your Gemini configuration (`~/.gemini/settings.json`):

```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "cc-switch-mcp"
    }
  }
}
```

## Available Tools

| Tool | Description |
|------|-------------|
| `list_providers` | List all providers for a CLI tool with their configurations |
| `get_current_provider` | Get the currently active provider for a CLI tool |
| `switch_provider` | Switch to a specific provider (auto-syncs config files) |
| `add_provider` | Add a new provider with specified configuration |
| `delete_provider` | Delete a provider by ID |
| `sync_current_to_live` | Sync current provider settings to live config files |
| `get_custom_endpoints` | Get list of custom endpoints for a provider |

## Example Usage

### List Providers

```json
{
  "tool": "list_providers",
  "arguments": {
    "app": "claude"
  }
}
```

Response:
```json
{
  "app": "claude",
  "providers": [
    {
      "id": "default",
      "name": "default",
      "isCurrent": false,
      "settingsConfig": { ... }
    },
    {
      "id": "my-provider",
      "name": "My Provider",
      "isCurrent": true,
      "settingsConfig": { ... }
    }
  ],
  "currentProviderId": "my-provider",
  "total": 2
}
```

### Switch Provider

```json
{
  "tool": "switch_provider",
  "arguments": {
    "app": "claude",
    "providerId": "my-provider"
  }
}
```

Response:
```json
{
  "success": true,
  "app": "claude",
  "providerId": "my-provider",
  "configSynced": true,
  "warnings": []
}
```

### Add Provider

```json
{
  "tool": "add_provider",
  "arguments": {
    "app": "claude",
    "name": "My API",
    "baseUrl": "https://api.example.com",
    "apiKey": "sk-xxx",
    "model": "claude-3-sonnet"
  }
}
```

## Supported Apps

| App | Config File | Description |
|-----|-------------|-------------|
| Claude Code | `~/.claude.json` | Anthropic's CLI tool |
| Codex | `~/.codex/config.toml` | OpenAI's Codex CLI |
| Gemini CLI | `~/.gemini/settings.json` | Google's Gemini CLI |
| OpenCode | `~/.config/opencode/config.json` | OpenCode CLI |
| OpenClaw | `~/.openclaw/config.json` | OpenClaw CLI |

## Architecture

This MCP server uses CC Switch's actual core library (`cc_switch_lib`) compiled without Tauri GUI support:

```
┌─────────────────────┐
│   MCP Protocol      │
│   (JSON-RPC 2.0)    │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│   cc-switch-mcp     │
│   (This Server)     │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│   cc_switch_lib     │
│   (Core Library)    │
│  - ProviderService  │
│  - Database         │
│  - Config Sync      │
└─────────────────────┘
```

This ensures identical behavior with the CC Switch desktop app.

## Database

The server reads directly from the CC Switch SQLite database:

- **Location**: `~/.cc-switch/cc-switch.db`
- **Compatible**: Fully compatible with the desktop app
- **Safe**: Can be used alongside CC Switch desktop

## Development

```bash
# Run in development mode with logging
RUST_LOG=debug cargo run --no-default-features

# Build release
cargo build --release --no-default-features

# Run tests
cargo test --no-default-features
```

## License

MIT

## Credits

- [CC Switch](https://github.com/farion1231/cc-switch) - The original desktop application
- [Model Context Protocol](https://modelcontextprotocol.io/) - The protocol specification