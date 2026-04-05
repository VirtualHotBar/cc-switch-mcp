# CC Switch MCP Server - 集成方案

## 当前状态

✅ 已完成：
- MCP Server 基础功能
- 18 个工具实现
- 配置文件同步
- 多平台发布

⚠️ 架构问题：
- 直接操作数据库
- 未复用 CC Switch 代码
- 可能存在行为差异

## 集成方案选择

### 方案 A: Git Submodule（已尝试）

```bash
git submodule add https://github.com/farion1231/cc-switch.git
```

**问题：**
- ❌ submodule 初始化失败
- ❌ CC Switch 仓库较大（包含前端代码）

### 方案 B: 仅引用核心代码

```bash
# 使用 sparse checkout 只下载需要的文件
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/farion1231/cc-switch.git
  
cd cc-switch
git sparse-checkout set src-tauri/src/services src-tauri/src/database
```

### 方案 C: 复制关键服务代码

直接复制 CC Switch 的服务层代码到项目中。

**优点：**
- ✅ 完全控制
- ✅ 可以修改适配
- ✅ 无依赖问题

**缺点：**
- ❌ 需要手动同步更新
- ❌ 代码重复

### 方案 D: 等待官方支持（推荐）

**在 CC Switch 项目创建 Issue：**

```
标题：[Feature Request] Export core services as reusable library

描述：
We're building an MCP Server for CC Switch to allow AI agents to manage providers programmatically.

Currently we're duplicating database operations, which:
- May cause inconsistencies with the GUI
- Requires maintaining duplicate code
- Cannot benefit from CC Switch's business logic

We noticed CC Switch already compiles to `cc_switch_lib`, which is great!

Could you please:
1. Publish to crates.io so we can use it as dependency
2. Provide public API documentation
3. Consider built-in MCP server support: `cc-switch --mcp-server`

This would benefit the entire ecosystem and ensure behavior consistency.

Related: VirtualHotBar/cc-switch-mcp
```

## 推荐实施方案

### 短期（当前）：保持独立实现

**原因：**
1. Submodule 技术问题
2. 需要与 CC Switch 项目协调
3. 当前实现已可用

**改进：**
- ✅ 严格遵循 CC Switch 数据库 schema
- ✅ 实现配置文件同步（已完成）
- ✅ 添加测试验证行为一致性

### 中期：协作集成

1. 在 CC Switch 创建 Issue 讨论
2. 等待 CC Switch 发布库到 crates.io
3. 或者等待 CC Switch 内置 MCP Server 支持

### 长期：深度集成

**最佳方案：CC Switch 内置 MCP Server**

```rust
// cc-switch/src-tauri/src/lib.rs
#[tauri::command]
pub fn start_mcp_server() -> Result<()> {
    mcp_server::run()
}
```

这样就不需要单独的 MCP Server 项目了！

## 下一步行动

### 1. 创建 GitHub Issue

前往 https://github.com/farion1231/cc-switch/issues/new

**标题：** [Feature Request] Export core services as reusable library / MCP server support

**标签：** enhancement, discussion

### 2. 继续优化当前实现

- 添加更多测试
- 完善文档
- 确保行为一致性

### 3. 建立协作关系

- Fork CC Switch
- 研究 API
- 准备 PR（如果需要）

## 测试行为一致性

创建集成测试脚本：

```bash
#!/bin/bash
# test-consistency.sh

# 1. 使用 GUI 切换提供商
# 2. 检查数据库状态
# 3. 使用 MCP Server 切换提供商
# 4. 对比结果
```

## 架构演进路线

```
Phase 1 (当前)
└── MCP Server 独立实现
    └── 直接操作数据库

Phase 2 (计划中)
└── MCP Server 引用 cc_switch_lib
    └── 调用 CC Switch 服务层

Phase 3 (最终目标)
└── CC Switch 内置 MCP Server
    └── 无需独立项目
```

## 联系方式

CC Switch 作者：https://github.com/farion1231

建议通过 GitHub Issue 沟通，讨论最佳集成方案。