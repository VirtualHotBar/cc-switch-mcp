#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

const platform = os.platform();
const arch = os.arch();

let binaryName;
if (platform === 'win32') {
  binaryName = 'cc-switch-mcp.exe';
} else {
  binaryName = 'cc-switch-mcp';
}

const binaryPath = path.join(__dirname, '..', 'native', `${platform}-${arch}`, binaryName);

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (err) => {
  console.error('Failed to start cc-switch-mcp:', err);
  process.exit(1);
});