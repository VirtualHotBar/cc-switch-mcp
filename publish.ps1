#!/usr/bin/env pwsh

Write-Host "=== Publishing CC Switch MCP Server to NPM ===" -ForegroundColor Cyan
Write-Host ""

# Check if already logged in
try {
    $whoami = npm whoami 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Already logged in as: $whoami" -ForegroundColor Green
    } else {
        throw "Not logged in"
    }
} catch {
    Write-Host "⚠️  Not logged in to npm" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please login to npm:" -ForegroundColor White
    Write-Host "  npm login" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "After login, run this script again to publish." -ForegroundColor White
    exit 1
}

Write-Host ""
$version = node -p "require('./package.json').version"
Write-Host "Package: @cc-switch/mcp-server" -ForegroundColor White
Write-Host "Version: $version" -ForegroundColor White
Write-Host ""

# Check package name availability
Write-Host "Checking package name..." -ForegroundColor White
try {
    $exists = npm view @cc-switch/mcp-server version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "⚠️  Package @cc-switch/mcp-server already exists (v$exists)" -ForegroundColor Yellow
        Write-Host "This will be an update, not a new package" -ForegroundColor White
    }
} catch {
    Write-Host "✅ Package name is available" -ForegroundColor Green
}

Write-Host ""
Write-Host "Files to publish:" -ForegroundColor White
npm pack --dry-run 2>&1 | Select-String "^[a-zA-Z]" | Select-Object -First 20

Write-Host ""
$response = Read-Host "Ready to publish? (y/N)"

if ($response -eq 'y' -or $response -eq 'Y') {
    Write-Host ""
    Write-Host "Publishing..." -ForegroundColor Cyan
    npm publish --access public
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "✅ Successfully published!" -ForegroundColor Green
        Write-Host ""
        Write-Host "View on npm:" -ForegroundColor White
        Write-Host "  https://www.npmjs.com/package/@cc-switch/mcp-server" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Install with:" -ForegroundColor White
        Write-Host "  npm install -g @cc-switch/mcp-server" -ForegroundColor Cyan
    } else {
        Write-Host ""
        Write-Host "❌ Publishing failed" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Cancelled" -ForegroundColor Yellow
}