#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const platforms = [
  { name: 'win32-x64', os: 'win32', cpu: 'x64' },
  { name: 'darwin-x64', os: 'darwin', cpu: 'x64' },
  { name: 'darwin-arm64', os: 'darwin', cpu: 'arm64' },
  { name: 'linux-x64', os: 'linux', cpu: 'x64' }
];

const mainPackage = require('../package.json');
const version = mainPackage.version;

platforms.forEach(platform => {
  const packageName = `@imvhb/mcp-server-${platform.name}`;
  const packageDir = path.join(__dirname, '..', 'packages', platform.name);
  
  // Create directory
  if (!fs.existsSync(packageDir)) {
    fs.mkdirSync(packageDir, { recursive: true });
  }
  
  // Create package.json
  const packageJson = {
    name: packageName,
    version: version,
    description: `CC Switch MCP Server binary for ${platform.name}`,
    license: "MIT",
    repository: {
      type: "git",
      url: "git+https://github.com/VirtualHotBar/cc-switch-mcp.git",
      directory: `packages/${platform.name}`
    },
    author: "CC Switch Team",
    os: [platform.os],
    cpu: [platform.cpu],
    files: [
      "binary/",
      "README.md"
    ],
    publishConfig: {
      access: "public"
    }
  };
  
  fs.writeFileSync(
    path.join(packageDir, 'package.json'),
    JSON.stringify(packageJson, null, 2)
  );
  
  // Create README
  const readme = `# ${packageName}

Binary package for CC Switch MCP Server (${platform.name}).

## Installation

This package is automatically installed as an optional dependency of \`@imvhb/mcp-server\`.

\`\`\`bash
npm install @imvhb/mcp-server
\`\`\`

## License

MIT
`;
  
  fs.writeFileSync(
    path.join(packageDir, 'README.md'),
    readme
  );
  
  // Create binary directory
  const binaryDir = path.join(packageDir, 'binary');
  if (!fs.existsSync(binaryDir)) {
    fs.mkdirSync(binaryDir, { recursive: true });
  }
  
  console.log(`✓ Created ${packageName}`);
});

console.log('\nAll platform packages created!');