# CC Switch MCP Server - Architecture Design

## 问题分析

当前实现直接操作数据库，存在以下问题：
1. 绕过了 CC Switch 的业务逻辑
2. 可能导致数据不一致
3. 无法复用 CC Switch 的验证和副作用（如配置文件写入）

## 解决方案

### 方案对比

| 方案 | 优点 | 缺点 | 可行性 |
|------|------|------|--------|
| 直接操作数据库 | 简单快速 | 绕过业务逻辑，不一致风险 | ⚠️ 不推荐 |
| Git submodule 引用 CC Switch | 完全复用 | 编译复杂，依赖冲突 | ❌ 困难 |
| 复制核心服务代码 | 独立控制 | 需要维护两份代码 | ⚠️ 可行 |
| 通过 IPC/API 调用 CC Switch | 完全复用 | 需要 CC Switch 运行 | ✅ 最佳 |
| 作为 CC Switch 插件 | 完美集成 | 需要 CC Switch 支持 | ✅ 推荐 |

### 推荐方案：混合架构

#### 短期方案（当前实现）
- MCP Server 独立实现
- 复用 CC Switch 的数据库 schema
- 调用相同的配置文件写入逻辑

#### 长期方案（建议 CC Switch 支持）
- CC Switch 暴露 gRPC/HTTP API
- MCP Server 作为 API 客户端
- 或者 MCP Server 作为 CC Switch 的插件

## 实施计划

### Phase 1: 当前实现优化（独立）

```rust
// 保持当前架构，但增强配置写入功能
mcp_server/
├── database.rs        // 数据库操作（与 CC Switch 兼容）
├── services/
│   ├── provider.rs    // Provider 服务（模仿 CC Switch）
│   ├── config.rs      // 配置写入服务
│   └── sync.rs        // 配置同步服务
└── mcp_server.rs      // MCP 协议实现
```

**改进点：**
1. 添加配置文件写入功能
2. 实现与 CC Switch 相同的验证逻辑
3. 添加事件通知机制

### Phase 2: API 集成（需要 CC Switch 支持）

```rust
// 当 CC Switch 提供官方 API
mcp_server/
├── api_client.rs      // CC Switch API 客户端
├── cache.rs           // 本地缓存
└── mcp_server.rs      // MCP 协议实现
```

### Phase 3: 深度集成（推荐）

```rust
// 作为 CC Switch 的内置 MCP Server
cc-switch/
├── src-tauri/
│   └── src/
│       └── mcp_server.rs  // MCP Server 作为模块
└── ...
```

## 配置写入实现

### 当前实现的问题

```rust
// 当前：只更新数据库
db.set_current_provider("claude", "provider-123")?;
// 问题：Claude 的 settings.json 没有更新！
```

### 应该实现

```rust
// 正确：更新数据库 + 写入配置文件
db.set_current_provider("claude", "provider-123")?;
provider_service.sync_to_live_config("claude", "provider-123")?;
// 这会更新 ~/.claude/settings.json
```

## 核心服务接口

```rust
pub trait ProviderService {
    // 数据库操作
    fn get_providers(&self, app: &str) -> Result<Vec<Provider>>;
    fn add_provider(&self, app: &str, provider: Provider) -> Result<()>;
    fn switch_provider(&self, app: &str, id: &str) -> Result<()>;
    
    // 配置文件同步（关键！）
    fn sync_to_live_config(&self, app: &str, provider_id: &str) -> Result<()>;
    fn import_from_live_config(&self, app: &str) -> Result<Vec<Provider>>;
}

pub trait McpServerService {
    fn get_servers(&self) -> Result<Vec<McpServerConfig>>;
    fn add_server(&self, server: McpServerConfig) -> Result<()>;
    
    // 同步到配置文件
    fn sync_to_claude(&self) -> Result<()>;
    fn sync_to_codex(&self) -> Result<()>;
    fn sync_to_gemini(&self) -> Result<()>;
}
```

## 下一步行动

1. ✅ 创建基础 MCP Server 框架
2. ⚠️ 实现配置文件写入功能
3. ⚠️ 添加配置同步服务
4. 📋 测试与 CC Switch 的兼容性
5. 📋 提交 PR 到 CC Switch（建议）

## 建议

### 给 CC Switch 的建议

建议 CC Switch 添加以下功能：

1. **命令行接口**
```bash
cc-switch provider list --app claude
cc-switch provider switch --app claude --id provider-123
cc-switch mcp list
```

2. **HTTP API Server**
```bash
cc-switch --api-server --port 9527
# 提供 RESTful API
GET    /api/providers?app=claude
POST   /api/providers
PUT    /api/providers/{id}/activate
```

3. **MCP Server 内置支持**
```json
{
  "mcpServers": {
    "cc-switch": {
      "command": "cc-switch",
      "args": ["--mcp-server"]
    }
  }
}
```

这样 MCP Server 就不需要单独存在了！