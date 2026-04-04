# 发布指南

## 架构说明

本项目使用 **多平台包 + optionalDependencies** 方案：

### 包结构

```
@imvhb/mcp-server                    # 主包 (JS 代码 + 自动选择平台包)
├── @imvhb/mcp-server-win32-x64     # Windows x64 二进制
├── @imvhb/mcp-server-darwin-x64    # macOS Intel 二进制
├── @imvhb/mcp-server-darwin-arm64  # macOS Apple Silicon 二进制
└── @imvhb/mcp-server-linux-x64     # Linux x64 二进制
```

### 工作原理

1. 用户安装主包：`npm install -g @imvhb/mcp-server`
2. npm 根据 `optionalDependencies` 和当前平台，自动安装对应的平台包
3. 主包的 `bin/cc-switch-mcp.js` 会找到并执行正确的二进制文件

## 发布流程

### 自动发布 (推荐)

1. **创建 Git 标签**
   ```bash
   git tag v0.1.1
   git push --tags
   ```

2. **GitHub Actions 自动执行**
   - 为每个平台构建二进制文件
   - 创建平台包
   - 发布所有包到 npm
   - 创建 GitHub Release

### 手动发布

如果需要手动发布，按以下步骤操作：

#### 1. 构建二进制文件

```bash
# 本地构建 (当前平台)
cargo build --release

# 或使用 GitHub Actions 手动触发
# 在 GitHub 仓库页面 -> Actions -> Build and Publish -> Run workflow
```

#### 2. 创建平台包

```bash
node scripts/create-packages.js
```

这会在 `packages/` 目录下创建 4 个平台包。

#### 3. 复制二进制文件

```bash
# Windows
cp target/release/cc-switch-mcp.exe packages/win32-x64/binary/

# macOS (在 macOS 上执行)
cp target/release/cc-switch-mcp packages/darwin-x64/binary/
cp target/release/cc-switch-mcp packages/darwin-arm64/binary/

# Linux
cp target/release/cc-switch-mcp packages/linux-x64/binary/
```

#### 4. 发布到 npm

```bash
# 登录 npm
npm login --registry https://registry.npmjs.org/

# 发布平台包
cd packages/win32-x64 && npm publish --access public && cd ../..
cd packages/darwin-x64 && npm publish --access public && cd ../..
cd packages/darwin-arm64 && npm publish --access public && cd ../..
cd packages/linux-x64 && npm publish --access public && cd ../..

# 发布主包 (最后)
npm publish --access public
```

## 版本更新

### 更新版本号

```bash
# 更新主包版本
npm version patch  # 0.1.0 -> 0.1.1
npm version minor  # 0.1.0 -> 0.2.0
npm version major  # 0.1.0 -> 1.0.0

# 推送标签
git push --tags
```

### 同步平台包版本

package.json 中的 `optionalDependencies` 版本号需要与平台包版本一致：

```json
{
  "optionalDependencies": {
    "@imvhb/mcp-server-win32-x64": "0.1.1",
    "@imvhb/mcp-server-darwin-x64": "0.1.1",
    "@imvhb/mcp-server-darwin-arm64": "0.1.1",
    "@imvhb/mcp-server-linux-x64": "0.1.1"
  }
}
```

## 验证发布

```bash
# 查看包信息
npm info @imvhb/mcp-server
npm info @imvhb/mcp-server-win32-x64
npm info @imvhb/mcp-server-darwin-x64
npm info @imvhb/mcp-server-darwin-arm64
npm info @imvhb/mcp-server-linux-x64

# 测试安装
npm install -g @imvhb/mcp-server

# 验证
cc-switch-mcp --version
```

## 包大小参考

- 主包: ~10 KB
- 每个平台包: ~3.5 MB
- 用户实际下载: 主包 + 1个平台包 ≈ 3.5 MB

## 优势

✅ 用户只需安装一个包：`npm install @imvhb/mcp-server`
✅ npm 自动选择正确的平台包
✅ 每个平台包独立，用户只下载需要的
✅ 支持离线安装
✅ 跨平台兼容性好

## 注意事项

1. **NPM Token**: 需要在 GitHub Secrets 中设置 `NPM_TOKEN`
2. **OTP**: 如果 npm 账户启用了两步验证，需要在 GitHub Actions 中配置
3. **同步发布**: 所有平台包和主包必须同时发布，版本号必须一致
4. **回滚**: 如果发布失败，需要回滚所有已发布的包