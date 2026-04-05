#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const platform = os.platform();
const arch = os.arch();

// Map to platform package names
const platformPackages = {
  'win32-x64': '@imvhb/cc-switch-mcp-server-win32-x64',
  'darwin-x64': '@imvhb/cc-switch-mcp-server-darwin-x64',
  'darwin-arm64': '@imvhb/cc-switch-mcp-server-darwin-arm64',
  'linux-x64': '@imvhb/cc-switch-mcp-server-linux-x64'
};

const platformKey = `${platform}-${arch}`;
const binaryName = platform === 'win32' ? 'cc-switch-mcp.exe' : 'cc-switch-mcp';

// Possible binary locations (in order of priority)
const possiblePaths = [
  // 1. From installed platform package
  path.join(__dirname, '..', 'node_modules', platformPackages[platformKey], 'binary', binaryName),
  
  // 2. From bin directory (installed by postinstall)
  path.join(__dirname, binaryName),
  
  // 3. From local build (development)
  path.join(__dirname, '..', 'target', 'release', binaryName),
  
  // 4. Global fallback
  path.join(__dirname, '..', '..', '..', platformPackages[platformKey], 'binary', binaryName)
];

// Find the binary
let binaryPath = null;
for (const testPath of possiblePaths) {
  if (fs.existsSync(testPath)) {
    binaryPath = testPath;
    break;
  }
}

if (!binaryPath) {
  console.error('Error: cc-switch-mcp binary not found');
  console.error('');
  console.error(`Platform: ${platformKey}`);
  console.error('');
  console.error('Searched locations:');
  possiblePaths.forEach(p => console.error(`  - ${p}`));
  console.error('');
  console.error('Please reinstall the package:');
  console.error('  npm install @imvhb/cc-switch-mcp-server');
  process.exit(1);
}

// Spawn the binary with all arguments
const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (err) => {
  console.error('Failed to start cc-switch-mcp:', err.message);
  console.error('Binary path:', binaryPath);
  process.exit(1);
});