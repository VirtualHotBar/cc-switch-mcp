# NPM 发布指南

## 发布前准备

### 1. 登录 NPM

```bash
npm login
```

输入你的 npm 用户名、密码和邮箱。

### 2. 验证登录

```bash
npm whoami
```

### 3. 检查包名可用性

包名 `@cc-switch/mcp-server` 需要检查是否已被占用：

```bash
npm search @cc-switch/mcp-server
```

如果包名已被占用，可以修改 `package.json` 中的 `name` 字段。

## 发布步骤

### 方式一：直接发布（推荐）

```bash
cd cc-switch-mcp
npm publish
```

### 方式二：先测试再发布

```bash
# 1. 测试安装脚本
npm run build

# 2. 查看将要发布的文件
npm pack --dry-run

# 3. 如果一切正常，发布
npm publish
```

## 发布后验证

```bash
# 检查包是否发布成功
npm info @cc-switch/mcp-server

# 测试安装
npm install -g @cc-switch/mcp-server
```

## 自动化发布（可选）

可以设置 GitHub Actions 自动发布到 npm：

1. 在 GitHub 仓库设置中添加 `NPM_TOKEN` secret
2. 创建 `.github/workflows/publish.yml`

```yaml
name: Publish to npm

on:
  release:
    types: [created]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'
      - run: npm ci
      - run: npm publish
        env:
          NODE_AUTH_TOKEN: ${{secrets.NPM_TOKEN}}
```

## 注意事项

1. **包名**: `@cc-switch/mcp-server` 是一个 scoped package
   - 需要 `@cc-switch` 组织或使用个人 scope
   - 如果没有组织，改为 `@your-username/mcp-server` 或 `cc-switch-mcp`

2. **二进制文件**: 当前配置会尝试从 GitHub Releases 下载二进制文件
   - 需要先创建 GitHub Release
   - 或者让用户自己编译

3. **版本管理**: 每次发布前更新版本号
   ```bash
   npm version patch  # 0.1.0 -> 0.1.1
   npm version minor  # 0.1.0 -> 0.2.0
   npm version major  # 0.1.0 -> 1.0.0
   ```

4. **发布权限**: 
   - 如果是 scoped package (`@xxx/yyy`)，需要设置 `publishConfig.access: "public"`
   - 已在 package.json 中配置

## 故障排除

### 错误：包名已存在

修改 `package.json` 中的 `name` 字段，使用不同的 scope 或名称。

### 错误：需要 OTP

如果启用了两步验证，发布时需要提供 OTP：

```bash
npm publish --otp=123456
```

### 错误：权限不足

确保你有权限发布到该 scope 或包名。

## 手动构建和发布

如果自动下载失败，用户可以手动构建：

```bash
# 克隆仓库
git clone https://github.com/l1i1/cc-switch-mcp.git
cd cc-switch-mcp

# 安装 Rust (如果需要)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建
cargo build --release

# 复制二进制文件到 native 目录
mkdir -p native/$(uname -s)-$(uname -m)
cp target/release/cc-switch-mcp native/$(uname -s)-$(uname -m)/
```

## 更新 README

发布后，更新 README 添加安装说明：

```markdown
## Installation

```bash
npm install -g @cc-switch/mcp-server
```

Or with your favorite package manager:

```bash
yarn global add @cc-switch/mcp-server
pnpm add -g @cc-switch/mcp-server
```
```