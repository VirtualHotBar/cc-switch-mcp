#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');

const platform = os.platform();
const arch = os.arch();

// Map to platform package names
const platformPackages = {
  'win32-x64': '@imvhb/mcp-server-win32-x64',
  'darwin-x64': '@imvhb/mcp-server-darwin-x64',
  'darwin-arm64': '@imvhb/mcp-server-darwin-arm64',
  'linux-x64': '@imvhb/mcp-server-linux-x64'
};

const platformKey = `${platform}-${arch}`;
const packageName = platformPackages[platformKey];

if (!packageName) {
  console.error(`Error: Unsupported platform: ${platform}-${arch}`);
  console.error('');
  console.error('Supported platforms:');
  console.error('  - Windows x64');
  console.error('  - macOS x64 (Intel)');
  console.error('  - macOS ARM64 (Apple Silicon)');
  console.error('  - Linux x64');
  process.exit(1);
}

// Try to find the binary in the platform package
const possiblePaths = [
  // From installed platform package
  path.join(__dirname, '..', 'node_modules', packageName, 'binary'),
  
  // From bundled binaries (development)
  path.join(__dirname, '..', 'binaries'),
  
  // From local build
  path.join(__dirname, '..', 'target', 'release')
];

let binaryDir = null;
for (const testPath of possiblePaths) {
  if (fs.existsSync(testPath)) {
    binaryDir = testPath;
    break;
  }
}

if (!binaryDir) {
  console.log('');
  console.log('⚠ Platform binary not found');
  console.log('');
  console.log('The platform-specific binary should have been installed automatically.');
  console.log('');
  console.log('Platform package:', packageName);
  console.log('');
  console.log('Try running:');
  console.log('  npm install');
  console.log('');
  console.log('Or rebuild from source:');
  console.log('  cargo build --release');
  process.exit(0);
}

// Find the binary
const binaryName = platform === 'win32' ? 'cc-switch-mcp.exe' : 'cc-switch-mcp';
const binaryPath = path.join(binaryDir, binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`Error: Binary not found at ${binaryPath}`);
  process.exit(1);
}

// Create bin directory if needed
const binDir = path.join(__dirname, '..', 'bin');
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

const targetPath = path.join(binDir, binaryName);

// Copy binary
fs.copyFileSync(binaryPath, targetPath);

// Make executable on Unix
if (platform !== 'win32') {
  fs.chmodSync(targetPath, 0o755);
}

console.log('✓ Binary installed successfully');