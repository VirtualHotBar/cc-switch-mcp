# CC Switch MCP Server

[![npm version](https://badge.fury.io/js/@imvhb%2Fmcp-server.svg)](https://badge.fury.io/js/@imvhb/mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-VirtualHotBar%2Fcc--switch--mcp-blue)](https://github.com/VirtualHotBar/cc-switch-mcp)

[English](./README.md) | 简体中文

[CC Switch](https://github.com/farion1231/cc-switch) 的 MCP Server 实现 - 为 Claude Code、Codex、Gemini CLI、OpenCode 和 OpenClaw 提供提供商管理能力。

## 功能特性

- ✅ **提供商管理** - 添加、删除、切换 API 提供商
- ✅ **通用提供商** - 跨应用共享提供商配置
- ✅ **MCP 服务器管理** - 管理 MCP 服务器配置
- ✅ **技能管理** - 安装和管理技能
- ✅ **提示词管理** - 管理自定义提示词
- ✅ **配置同步** - 自动同步配置到应用配置文件
- ✅ **完全兼容** - 与 CC Switch 桌面应用共享同一数据库

## 安装

### 从 NPM 安装（推荐）

```bash
npm install -g @imvhb/mcp-server
```

或使用其他包管理器：

```bash
yarn global add @imvhb/mcp-server
pnpm add -g @imvhb/mcp-server
```

### 从源码构建

```bash
git clone https://github.com/VirtualHotBar/cc-switch-mcp.git
cd cc-switch-mcp
cargo build --release
```

二进制文件位于 `target/release/cc-switch-mcp`（Linux/macOS）或 `target/release/cc-switch-mcp.exe`（Windows）。

## 使用方法

### 配置 Claude Desktop

在 Claude Desktop 配置文件中添加（macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`）：

```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "/path/to/cc-switch-mcp"
    }
  }
}
```

### 配置 OpenCode

在 OpenCode 配置文件中添加：

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

## 可用工具

### 提供商管理

| 工具 | 说明 |
|------|------|
| `list_providers` | 列出指定应用的所有提供商 |
| `add_provider` | 添加新的提供商配置 |
| `switch_provider` | 切换到指定提供商 |
| `delete_provider` | 删除提供商 |
| `get_current_provider` | 获取当前激活的提供商 |

### 通用提供商

| 工具 | 说明 |
|------|------|
| `list_universal_providers` | 列出所有通用提供商 |
| `add_universal_provider` | 添加跨应用共享的提供商 |
| `delete_universal_provider` | 删除通用提供商 |

### MCP 服务器管理

| 工具 | 说明 |
|------|------|
| `list_mcp_servers` | 列出所有 MCP 服务器配置 |
| `add_mcp_server` | 添加新的 MCP 服务器 |
| `delete_mcp_server` | 删除 MCP 服务器 |

### 技能管理

| 工具 | 说明 |
|------|------|
| `list_skills` | 列出所有已安装的技能 |
| `add_skill` | 安装新技能 |
| `delete_skill` | 删除已安装的技能 |

### 提示词管理

| 工具 | 说明 |
|------|------|
| `list_prompts` | 列出指定应用的所有提示词 |
| `add_prompt` | 添加新的提示词 |
| `delete_prompt` | 删除提示词 |

### 工具函数

| 工具 | 说明 |
|------|------|
| `get_db_path` | 获取 CC Switch 数据库路径 |

## 使用示例

### 添加提供商

```
工具: add_provider
参数: {
  "app": "claude",
  "name": "我的 API 提供商",
  "apiKey": "sk-xxx",
  "baseUrl": "https://api.example.com",
  "model": "claude-sonnet-4-20250514"
}
```

### 切换提供商

```
工具: switch_provider
参数: {
  "app": "claude",
  "providerId": "claude-123e4567-e89b-12d3-a456-426614174000"
}
```

### 列出提供商

```
工具: list_providers
参数: {
  "app": "claude"
}
```

### 添加 MCP 服务器

```
工具: add_mcp_server
参数: {
  "name": "我的 MCP 服务器",
  "serverConfig": "{\"command\": \"npx\", \"args\": [\"-y\", \"my-mcp-server\"], \"type\": \"stdio\"}",
  "description": "我的自定义 MCP 服务器",
  "enabledApps": ["claude", "opencode"]
}
```

### 安装技能

```
工具: add_skill
参数: {
  "name": "代码审查技能",
  "directory": "~/.claude/skills/code-review",
  "repoOwner": "example",
  "repoName": "code-review-skill",
  "description": "自动代码审查技能",
  "enabledApps": ["claude", "opencode"]
}
```

## 资源

MCP 服务器提供以下资源：

- `ccswitch://providers/claude` - Claude Code 提供商
- `ccswitch://providers/codex` - Codex 提供商
- `ccswitch://providers/gemini` - Gemini CLI 提供商
- `ccswitch://providers/opencode` - OpenCode 提供商
- `ccswitch://providers/openclaw` - OpenClaw 提供商
- `ccswitch://universal-providers` - 通用提供商
- `ccswitch://config/path` - 配置路径

## 数据库

服务器使用与 CC Switch 桌面应用相同的 SQLite 数据库：

- 位置：`~/.cc-switch/cc-switch.db`
- 完全兼容桌面应用
- 可与 CC Switch 桌面版同时使用

## 支持的应用

| 应用 | 配置格式 |
|------|----------|
| Claude Code | JSON (settings.json) |
| Codex | TOML (config.toml) |
| Gemini CLI | JSON (.gemini/settings.json) |
| OpenCode | JSON (opencode.json) |
| OpenClaw | JSON (openclaw.json) |

## 开发

```bash
# 开发模式运行（带日志）
RUST_LOG=debug cargo run

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

## 工作原理

当您通过 MCP 工具切换提供商时，服务器会：

1. 更新 CC Switch 数据库
2. 自动同步配置到对应应用的配置文件
3. 确保行为与 CC Switch 桌面应用完全一致

例如，切换 Claude 提供商时会同时更新：
- `~/.cc-switch/cc-switch.db`（数据库）
- `~/.claude/settings.json`（Claude 配置文件）

## 许可证

MIT

## 相关链接

- **GitHub**: https://github.com/VirtualHotBar/cc-switch-mcp
- **NPM**: https://www.npmjs.com/package/@imvhb/mcp-server
- **CC Switch**: https://github.com/farion1231/cc-switch