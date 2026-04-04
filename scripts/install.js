#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');

const platform = os.platform();
const arch = os.arch();

const version = require('../package.json').version;
const binaryName = platform === 'win32' ? 'cc-switch-mcp.exe' : 'cc-switch-mcp';
const downloadUrl = `https://github.com/l1i1/cc-switch-mcp/releases/download/v${version}/${platform}-${arch}-${binaryName}`;

const nativeDir = path.join(__dirname, '..', 'native', `${platform}-${arch}`);
const binaryPath = path.join(nativeDir, binaryName);

console.log('Downloading cc-switch-mcp binary...');
console.log(`Platform: ${platform}-${arch}`);
console.log(`Version: ${version}`);

if (!fs.existsSync(nativeDir)) {
  fs.mkdirSync(nativeDir, { recursive: true });
}

const file = fs.createWriteStream(binaryPath);

https.get(downloadUrl, (response) => {
  if (response.statusCode === 302 || response.statusCode === 301) {
    https.get(response.headers.location, (redirectResponse) => {
      redirectResponse.pipe(file);
      file.on('finish', () => {
        file.close();
        console.log('Download complete!');
      });
    }).on('error', (err) => {
      console.error('Download failed:', err.message);
      process.exit(1);
    });
  } else if (response.statusCode === 200) {
    response.pipe(file);
    file.on('finish', () => {
      file.close();
      console.log('Download complete!');
    });
  } else {
    console.error(`Download failed: HTTP ${response.statusCode}`);
    console.error('Binary not found. Please build from source.');
    process.exit(1);
  }
}).on('error', (err) => {
  console.error('Download failed:', err.message);
  console.error('Falling back to building from source...');
  
  const { spawn } = require('child_process');
  const cargo = spawn('cargo', ['build', '--release'], {
    cwd: path.join(__dirname, '..'),
    stdio: 'inherit'
  });
  
  cargo.on('exit', (code) => {
    if (code === 0) {
      console.log('Build complete!');
    } else {
      console.error('Build failed');
      process.exit(1);
    }
  });
});