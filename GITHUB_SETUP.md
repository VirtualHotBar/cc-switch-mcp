# GitHub 仓库设置指南

## 当前状态

代码已提交到本地 Git 仓库，但需要创建 GitHub 远程仓库。

## 步骤

### 1. 在 GitHub 上创建仓库

访问 https://github.com/new 并填写：
- **Owner**: VirtualHotBar
- **Repository name**: cc-switch-mcp
- **Description**: MCP Server for CC Switch - Provider management for Claude Code, Codex, Gemini CLI, OpenCode & OpenClaw
- **Visibility**: Public
- **不要勾选** "Add a README file"（我们已经有了）

### 2. 创建后推送

创建仓库后，GitHub 会显示推送命令，或者直接运行：

```bash
cd cc-switch-mcp
git remote add origin https://github.com/VirtualHotBar/cc-switch-mcp.git
git branch -M main
git push -u origin main
```

### 3. 验证

推送后访问：https://github.com/VirtualHotBar/cc-switch-mcp

## 已配置的信息

✅ package.json - 仓库 URL 已更新为 VirtualHotBar/cc-switch-mcp
✅ scripts/install.js - 下载 URL 已更新
✅ 本地 git 仓库已初始化并提交

## 后续步骤

推送成功后：
1. 在 GitHub 上添加 Topics: `mcp`, `claude`, `rust`, `llm`
2. 在仓库 Settings → Pages 启用主页（可选）
3. 创建第一个 Release 并上传二进制文件
4. 发布到 NPM

## 快速命令

如果仓库已创建，直接运行：

```bash
cd cc-switch-mcp
git push -u origin main
```