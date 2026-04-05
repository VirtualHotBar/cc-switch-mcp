# CC Switch MCP Server

[![npm version](https://badge.fury.io/js/@imvhb%2Fcc-switch-mcp-server.svg)](https://badge.fury.io/js/@imvhb/cc-switch-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-VirtualHotBar%2Fcc--switch--mcp-blue)](https://github.com/VirtualHotBar/cc-switch-mcp)

[English](./README.md) | 简体中文

[CC Switch](https://github.com/farion1231/cc-switch) 的 MCP Server 实现 - 为 Claude Code、Codex、Gemini CLI、OpenCode 和 OpenClaw 提供提供商管理能力。

**核心特性：**
- 🚀 使用 CC Switch 核心库构建，行为完全一致
- 🔄 自动同步配置文件
- 💾 直接访问 CC Switch SQLite 数据库
- 🪶 无 Tauri 依赖 - 轻量级二进制
- 🤝 可与 CC Switch 桌面应用同时使用

## 安装

### 从 NPM 安装

```bash
npm install -g @imvhb/cc-switch-mcp-server
```

### 从源码构建

```bash
git clone https://github.com/VirtualHotBar/cc-switch-mcp.git
cd cc-switch-mcp
cargo build --release --no-default-features
```

二进制文件位于 `target/release/cc-switch-mcp`（Linux/macOS）或 `target/release/cc-switch-mcp.exe`（Windows）。

## 使用方法

### 配置 Claude Desktop

在 Claude Desktop 配置文件中添加（macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`）：

```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "cc-switch-mcp"
    }
  }
}
```

### 配置 OpenCode

在 OpenCode 配置文件中添加（`~/.config/opencode/config.json`）：

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

### 配置 Gemini CLI

在 Gemini 配置文件中添加（`~/.gemini/settings.json`）：

```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "cc-switch-mcp"
    }
  }
}
```

## 可用工具

| 工具 | 说明 |
|------|------|
| `list_providers` | 列出指定应用的所有提供商及其配置 |
| `get_current_provider` | 获取当前激活的提供商 |
| `switch_provider` | 切换到指定提供商（自动同步配置文件） |
| `add_provider` | 添加新的提供商配置 |
| `delete_provider` | 删除提供商 |
| `sync_current_to_live` | 同步当前提供商设置到配置文件 |
| `get_custom_endpoints` | 获取提供商的自定义端点列表 |

## 使用示例

### 列出提供商

```json
{
  "tool": "list_providers",
  "arguments": {
    "app": "claude"
  }
}
```

响应：
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
      "name": "我的提供商",
      "isCurrent": true,
      "settingsConfig": { ... }
    }
  ],
  "currentProviderId": "my-provider",
  "total": 2
}
```

### 切换提供商

```json
{
  "tool": "switch_provider",
  "arguments": {
    "app": "claude",
    "providerId": "my-provider"
  }
}
```

响应：
```json
{
  "success": true,
  "app": "claude",
  "providerId": "my-provider",
  "configSynced": true,
  "warnings": []
}
```

### 添加提供商

```json
{
  "tool": "add_provider",
  "arguments": {
    "app": "claude",
    "name": "我的 API",
    "baseUrl": "https://api.example.com",
    "apiKey": "sk-xxx",
    "model": "claude-3-sonnet"
  }
}
```

## 支持的应用

| 应用 | 配置文件 | 说明 |
|------|----------|------|
| Claude Code | `~/.claude.json` | Anthropic 的 CLI 工具 |
| Codex | `~/.codex/config.toml` | OpenAI 的 Codex CLI |
| Gemini CLI | `~/.gemini/settings.json` | Google 的 Gemini CLI |
| OpenCode | `~/.config/opencode/config.json` | OpenCode CLI |
| OpenClaw | `~/.openclaw/config.json` | OpenClaw CLI |

## 架构

本 MCP 服务器使用 CC Switch 的实际核心库（`cc_switch_lib`），在无 Tauri GUI 支持下编译：

```
┌─────────────────────┐
│   MCP 协议          │
│   (JSON-RPC 2.0)    │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│   cc-switch-mcp     │
│   (本服务器)        │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│   cc_switch_lib     │
│   (核心库)          │
│  - ProviderService  │
│  - Database         │
│  - Config Sync      │
└─────────────────────┘
```

这确保了与 CC Switch 桌面应用行为完全一致。

## 数据库

服务器直接读取 CC Switch SQLite 数据库：

- **位置**：`~/.cc-switch/cc-switch.db`
- **兼容**：与桌面应用完全兼容
- **安全**：可与 CC Switch 桌面版同时使用

## 开发

```bash
# 开发模式运行（带日志）
RUST_LOG=debug cargo run --no-default-features

# 构建发布版本
cargo build --release --no-default-features

# 运行测试
cargo test --no-default-features
```

## 许可证

MIT

## 致谢

- [CC Switch](https://github.com/farion1231/cc-switch) - 原始桌面应用
- [Model Context Protocol](https://modelcontextprotocol.io/) - 协议规范