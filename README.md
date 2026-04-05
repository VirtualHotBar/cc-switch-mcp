# CC Switch MCP Server

[![npm version](https://badge.fury.io/js/@imvhb%2Fcc-switch-mcp-server.svg)](https://badge.fury.io/js/@imvhb/cc-switch-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-VirtualHotBar%2Fcc--switch--mcp-blue)](https://github.com/VirtualHotBar/cc-switch-mcp)

English | [简体中文](./README_CN.md)

Standalone MCP Server for [CC Switch](https://github.com/farion1231/cc-switch) - providing provider management capabilities for Claude Code, Codex, Gemini CLI, OpenCode, and OpenClaw.

**Key Features:**
- Direct database access to CC Switch's SQLite database
- Automatic config file synchronization
- No Tauri dependencies - lightweight (~2.8MB)
- Works alongside CC Switch desktop app

## Installation

### From NPM

```bash
npm install -g @imvhb/cc-switch-mcp-server
```

### From Source

```bash
git clone https://github.com/VirtualHotBar/cc-switch-mcp.git
cd cc-switch-mcp
cargo build --release
```

The binary will be at `target/release/cc-switch-mcp` (Linux/macOS) or `target/release/cc-switch-mcp.exe` (Windows).

## Usage

### Configure with Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "/path/to/cc-switch-mcp"
    }
  }
}
```

### Configure with OpenCode

Add to your OpenCode configuration:

```json
{
  "mcp": {
    "servers": {
      "cc-switch": {
        "command": "/path/to/cc-switch-mcp"
      }
    }
  }
}
```

## Available Tools

| Tool | Description |
|------|-------------|
| `list_providers` | List all providers for a CLI tool |
| `switch_provider` | Switch to a specific provider (updates database + syncs config files) |
| `get_current_provider` | Get the currently active provider |

## Example Usage

### List Providers

```
Tool: list_providers
Arguments: {
  "app": "claude"
}
```

### Switch Provider

```
Tool: switch_provider
Arguments: {
  "app": "claude",
  "providerId": "my-provider-id"
}
```

### Get Current Provider

```
Tool: get_current_provider
Arguments: {
  "app": "claude"
}
```

## Database

The server reads directly from the CC Switch SQLite database:

- Location: `~/.cc-switch/cc-switch.db`
- Fully compatible with the desktop app
- Can be used alongside CC Switch desktop

## Supported Apps

| App | Config File |
|-----|-------------|
| Claude Code | `~/.claude.json` |
| Codex | `~/.codex/config.toml` |
| Gemini CLI | `~/.gemini/settings.json` |
| OpenCode | `~/.config/opencode/config.json` |
| OpenClaw | `~/.openclaw/config.json` |

## Development

```bash
# Run in development mode with logging
RUST_LOG=debug cargo run

# Build release
cargo build --release
```

## License

MIT