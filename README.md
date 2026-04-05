# CC Switch MCP Server

[![npm version](https://badge.fury.io/js/@imvhb%2Fcc-switch-mcp-server.svg)](https://badge.fury.io/js/@imvhb/cc-switch-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-VirtualHotBar%2Fcc--switch--mcp-blue)](https://github.com/VirtualHotBar/cc-switch-mcp)

English | [简体中文](./README_CN.md)

MCP Server implementation for [CC Switch](https://github.com/farion1231/cc-switch) - providing provider management capabilities for Claude Code, Codex, Gemini CLI, OpenCode, and OpenClaw.

## Installation

### From NPM

```bash
npm install -g @imvhb/cc-switch-mcp-server
```

Or with your favorite package manager:

```bash
yarn global add @imvhb/cc-switch-mcp-server
pnpm add -g @imvhb/cc-switch-mcp-server
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

### Provider Management

| Tool | Description |
|------|-------------|
| `list_providers` | List all providers for a CLI tool |
| `add_provider` | Add a new provider configuration |
| `switch_provider` | Switch to a specific provider |
| `delete_provider` | Delete a provider |
| `get_current_provider` | Get the currently active provider |

### Universal Providers

| Tool | Description |
|------|-------------|
| `list_universal_providers` | List all universal providers |
| `add_universal_provider` | Add a cross-app provider |
| `delete_universal_provider` | Delete a universal provider |

### MCP Servers Management

| Tool | Description |
|------|-------------|
| `list_mcp_servers` | List all MCP server configurations |
| `add_mcp_server` | Add a new MCP server |
| `delete_mcp_server` | Delete an MCP server |

### Skills Management

| Tool | Description |
|------|-------------|
| `list_skills` | List all installed skills |
| `add_skill` | Install a new skill |
| `delete_skill` | Delete an installed skill |

### Prompts Management

| Tool | Description |
|------|-------------|
| `list_prompts` | List all prompts for a CLI tool |
| `add_prompt` | Add a new prompt |
| `delete_prompt` | Delete a prompt |

### Utility

| Tool | Description |
|------|-------------|
| `get_db_path` | Get the path to CC Switch database |

## Example Usage

### Add a Provider

```
Tool: add_provider
Arguments: {
  "app": "claude",
  "name": "My API Provider",
  "apiKey": "sk-xxx",
  "baseUrl": "https://api.example.com",
  "model": "claude-sonnet-4-20250514"
}
```

### Switch Provider

```
Tool: switch_provider
Arguments: {
  "app": "claude",
  "providerId": "claude-123e4567-e89b-12d3-a456-426614174000"
}
```

### List Providers

```
Tool: list_providers
Arguments: {
  "app": "claude"
}
```

### Add MCP Server

```
Tool: add_mcp_server
Arguments: {
  "name": "My MCP Server",
  "serverConfig": "{\"command\": \"npx\", \"args\": [\"-y\", \"my-mcp-server\"], \"type\": \"stdio\"}",
  "description": "My custom MCP server",
  "enabledApps": ["claude", "opencode"]
}
```

### List MCP Servers

```
Tool: list_mcp_servers
Arguments: {}
```

## Resources

The MCP server exposes the following resources:

- `ccswitch://providers/claude` - Claude Code providers
- `ccswitch://providers/codex` - Codex providers
- `ccswitch://providers/gemini` - Gemini CLI providers
- `ccswitch://providers/opencode` - OpenCode providers
- `ccswitch://providers/openclaw` - OpenClaw providers
- `ccswitch://universal-providers` - Universal providers
- `ccswitch://config/path` - Configuration path

## Database

The server uses the same SQLite database as the CC Switch desktop application:

- Location: `~/.cc-switch/cc-switch.db`
- Fully compatible with the desktop app
- Can be used alongside CC Switch desktop

## Supported Apps

| App | Config Format |
|-----|---------------|
| Claude Code | JSON (settings.json) |
| Codex | TOML (config.toml) |
| Gemini CLI | JSON (.gemini/settings.json) |
| OpenCode | JSON (opencode.json) |
| OpenClaw | JSON (openclaw.json) |

## Development

```bash
# Run in development mode with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## License

MIT