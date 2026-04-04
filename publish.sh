#!/bin/bash

echo "=== Publishing CC Switch MCP Server to NPM ==="
echo ""

# Check if already logged in
if npm whoami > /dev/null 2>&1; then
    echo "✅ Already logged in as: $(npm whoami)"
else
    echo "⚠️  Not logged in to npm"
    echo ""
    echo "Please login to npm:"
    echo "  npm login"
    echo ""
    echo "After login, run this script again to publish."
    exit 1
fi

echo ""
echo "Package: @cc-switch/mcp-server"
echo "Version: $(node -p "require('./package.json').version")"
echo ""

# Check package name availability
echo "Checking package name..."
if npm view @cc-switch/mcp-server > /dev/null 2>&1; then
    echo "⚠️  Package @cc-switch/mcp-server already exists"
    echo "This will be an update, not a new package"
else
    echo "✅ Package name is available"
fi

echo ""
echo "Files to publish:"
npm pack --dry-run 2>&1 | grep -E "^[a-zA-Z]" | head -20

echo ""
read -p "Ready to publish? (y/N) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Publishing..."
    npm publish --access public
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Successfully published!"
        echo ""
        echo "View on npm:"
        echo "  https://www.npmjs.com/package/@cc-switch/mcp-server"
        echo ""
        echo "Install with:"
        echo "  npm install -g @cc-switch/mcp-server"
    else
        echo ""
        echo "❌ Publishing failed"
        exit 1
    fi
else
    echo "Cancelled"
fi