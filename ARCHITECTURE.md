# CC Switch MCP Server - 架构设计

## 整体架构

```mermaid
graph TB
    subgraph "用户层"
        A[Claude Desktop]
        B[OpenCode]
        C[Gemini CLI]
        D[Codex]
    end

    subgraph "MCP 协议层"
        E[CC Switch MCP Server]
    end

    subgraph "服务层"
        F[ProviderService]
        G[ConfigService]
    end

    subgraph "数据层"
        H[(SQLite Database)]
        I[Claude Config]
        J[Codex Config]
        K[Gemini Config]
        L[OpenCode Config]
    end

    A --> E
    B --> E
    C --> E
    D --> E
    
    E --> F
    F --> G
    F --> H
    G --> I
    G --> J
    G --> K
    G --> L
```

## 组件架构

```mermaid
graph LR
    subgraph "MCP Server"
        A[main.rs<br/>入口]
        B[mcp_server.rs<br/>协议处理]
        C[provider_service.rs<br/>业务逻辑]
        D[config_service.rs<br/>配置同步]
        E[database.rs<br/>数据访问]
        F[provider.rs<br/>数据模型]
    end

    A --> B
    B --> C
    C --> D
    C --> E
    E --> F
```

## 数据流

```mermaid
sequenceDiagram
    participant User as 用户
    participant MCP as MCP Client
    participant Server as MCP Server
    participant Service as ProviderService
    participant DB as Database
    participant Config as Config Files

    User->>MCP: 切换提供商
    MCP->>Server: tools/call switch_provider
    Server->>Service: switch_provider(app, id)
    Service->>DB: set_current_provider(app, id)
    DB-->>Service: ✅ 成功
    Service->>Config: sync_to_live(app, provider)
    Config-->>Service: ✅ 已同步
    Service-->>Server: ✅ 成功
    Server-->>MCP: 返回结果
    MCP-->>User: 提供商已切换
```

## 工具分类

```mermaid
mindmap
  root((MCP Tools))
    Provider Management
      list_providers
      add_provider
      switch_provider
      delete_provider
      get_current_provider
    Universal Providers
      list_universal_providers
      add_universal_provider
      delete_universal_provider
    MCP Servers
      list_mcp_servers
      add_mcp_server
      delete_mcp_server
    Skills
      list_skills
      add_skill
      delete_skill
    Prompts
      list_prompts
      add_prompt
      delete_prompt
    Utility
      get_db_path
```

## 数据库 Schema

```mermaid
erDiagram
    PROVIDERS {
        string id PK
        string app_type PK
        string name
        json settings_config
        boolean is_current
        string notes
    }
    
    UNIVERSAL_PROVIDERS {
        string id PK
        string name
        string provider_type
        string base_url
        string api_key
        json apps
    }
    
    MCP_SERVERS {
        string id PK
        string name
        json server_config
        boolean enabled_claude
        boolean enabled_codex
        boolean enabled_gemini
        boolean enabled_opencode
    }
    
    SKILLS {
        string id PK
        string name
        string directory
        string repo_owner
        string repo_name
        boolean enabled_claude
        boolean enabled_codex
        boolean enabled_gemini
        boolean enabled_opencode
    }
    
    PROMPTS {
        string id PK
        string app_type PK
        string name
        text content
        boolean enabled
    }
```

## 配置同步流程

```mermaid
graph TB
    A[switch_provider] --> B{判断应用类型}
    
    B -->|Claude| C[写入 ~/.claude/settings.json]
    B -->|Codex| D[写入 ~/.codex/config.toml]
    B -->|Gemini| E[写入 ~/.gemini/settings.json]
    B -->|OpenCode| F[写入 ~/.opencode/opencode.json]
    B -->|OpenClaw| G[写入 ~/.openclaw/openclaw.json]
    
    C --> H[设置环境变量<br/>ANTHROPIC_BASE_URL<br/>ANTHROPIC_AUTH_TOKEN]
    D --> I[设置 TOML 配置<br/>model_provider<br/>base_url]
    E --> J[设置环境变量<br/>GOOGLE_GEMINI_BASE_URL<br/>GEMINI_API_KEY]
    F --> K[设置 JSON 配置<br/>provider settings]
    G --> L[设置 JSON 配置<br/>provider settings]
```

## NPM 包架构

```mermaid
graph TB
    A["@imvhb/cc-switch-mcp-server<br/>主包"] --> B["@imvhb/cc-switch-mcp-server-win32-x64"]
    A --> C["@imvhb/cc-switch-mcp-server-darwin-x64"]
    A --> D["@imvhb/cc-switch-mcp-server-darwin-arm64"]
    A --> E["@imvhb/cc-switch-mcp-server-linux-x64"]
    
    B --> F[Windows 二进制]
    C --> G[macOS Intel 二进制]
    D --> H[macOS ARM 二进制]
    E --> I[Linux 二进制]
    
    style A fill:#4A90E2
    style B fill:#7ED321
    style C fill:#7ED321
    style D fill:#7ED321
    style E fill:#7ED321
```

## 发布流程

```mermaid
graph LR
    A[git tag v0.1.1] --> B[GitHub Actions]
    B --> C[构建 4 个平台]
    C --> D[创建平台包]
    D --> E[发布到 NPM]
    E --> F[创建 GitHub Release]
    
    style A fill:#F5A623
    style B fill:#BD10E0
    style E fill:#7ED321
    style F fill:#4A90E2
```

## 技术栈

```mermaid
graph TB
    subgraph "开发语言"
        A[Rust]
        B[Node.js]
    end
    
    subgraph "核心依赖"
        C[rusqlite - SQLite]
        D[serde - 序列化]
        E[serde_json - JSON]
        F[dirs - 目录路径]
        G[uuid - ID 生成]
        H[chrono - 时间处理]
    end
    
    subgraph "协议"
        I[MCP Protocol<br/>2024-11-05]
        J[JSON-RPC 2.0]
    end
    
    A --> C
    A --> D
    A --> E
    A --> F
    A --> G
    A --> H
    A --> I
    I --> J
```

## 文件结构

```
cc-switch-mcp/
├── src/                      # Rust 源代码
│   ├── main.rs              # 程序入口
│   ├── lib.rs               # 库导出
│   ├── error.rs             # 错误处理
│   ├── provider.rs          # 数据模型
│   ├── database.rs          # 数据库层
│   ├── config_service.rs    # 配置同步服务
│   ├── provider_service.rs  # 业务逻辑层
│   └── mcp_server.rs        # MCP 协议实现
│
├── bin/                      # NPM 启动脚本
│   └── cc-switch-mcp.js     # 二进制启动器
│
├── scripts/                  # NPM 脚本
│   └── install.js           # 平台包安装脚本
│
├── .github/workflows/        # GitHub Actions
│   └── release.yml          # 自动发布流程
│
├── Cargo.toml               # Rust 配置
├── package.json             # NPM 配置
├── README.md                # 英文文档
├── README_CN.md             # 中文文档
└── ARCHITECTURE.md          # 架构文档
```

## 关键设计决策

### 1. 为什么选择 Rust？

- ✅ **性能优异** - 原生性能，启动快
- ✅ **内存安全** - 无 GC，零成本抽象
- ✅ **跨平台** - 一次编写，处处编译
- ✅ **类型安全** - 编译期类型检查

### 2. 为什么用 ConfigService？

```mermaid
graph LR
    A[直接操作数据库] -->|问题| B[配置文件不同步]
    C[ConfigService] -->|解决| D[数据库 + 配置文件同步]
    D -->|效果| E[行为与 GUI 一致]
    
    style A fill:#D0021B
    style C fill:#7ED321
```

### 3. 为什么用多平台包？

```mermaid
graph TB
    A["单一包 (~15MB)"] -->|问题| B[用户下载所有平台]
    C["多平台包"] -->|解决| D[用户只下载自己平台<br/>~3.5MB]
    
    style A fill:#D0021B
    style C fill:#7ED321
```

### 4. 数据一致性保证

```mermaid
sequenceDiagram
    participant CLI as CC Switch GUI
    participant DB as Database
    participant MCP as MCP Server
    
    CLI->>DB: 更新提供商
    Note over DB: 数据已更新
    MCP->>DB: 读取提供商
    Note over MCP: 读取到最新数据
    
    Note over CLI,MCP: 两者共享同一数据库，保证一致性
```

## 扩展性

### 支持新应用

只需在 `ConfigService` 中添加新的同步方法：

```rust
pub fn sync_provider_to_new_app(&self, provider: &Provider) -> Result<()> {
    // 1. 确定配置文件路径
    let config_path = Self::get_new_app_config_path()?;
    
    // 2. 写入配置
    // ...
    
    Ok(())
}
```

### 支持新工具

在 `mcp_server.rs` 中注册新工具：

```rust
fn handle_tools_list(&self) -> Result<Value> {
    Ok(json!({
        "tools": [
            // ... 现有工具
            {
                "name": "new_tool",
                "description": "新工具描述",
                "inputSchema": { /* ... */ }
            }
        ]
    }))
}
```

## 性能特性

- 🚀 **启动时间**: < 10ms
- 💾 **内存占用**: ~5MB
- 📦 **二进制大小**: ~3.5MB (压缩后)
- ⚡ **响应延迟**: < 1ms (本地调用)
- 💿 **数据库大小**: 共享 CC Switch 数据库

## 许可证

MIT License