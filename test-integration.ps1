# CC Switch MCP Server - Integration Tests

Write-Host "=== CC Switch MCP Server Integration Tests ===" -ForegroundColor Cyan
Write-Host ""

$exePath = ".\target\release\cc-switch-mcp.exe"

if (-not (Test-Path $exePath)) {
    Write-Host "Error: Binary not found at $exePath" -ForegroundColor Red
    Write-Host "Please run 'cargo build --release' first" -ForegroundColor Yellow
    exit 1
}

Write-Host "Using binary: $exePath" -ForegroundColor Green
Write-Host ""

function Test-McpRequest {
    param(
        [string]$TestName,
        [string]$Request
    )
    
    Write-Host "Test: $TestName" -ForegroundColor Yellow
    $response = $Request | & $exePath 2>&1 | ConvertFrom-Json
    
    if ($response.result) {
        Write-Host "  ✓ Success" -ForegroundColor Green
        return $true
    } elseif ($response.error) {
        Write-Host "  ✗ Failed: $($response.error.message)" -ForegroundColor Red
        return $false
    } else {
        Write-Host "  ✗ Failed: Unknown error" -ForegroundColor Red
        return $false
    }
}

Write-Host "--- Basic Protocol Tests ---" -ForegroundColor Cyan
Test-McpRequest "Initialize" '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"test","version":"1.0.0"}}}'
Test-McpRequest "Tools List" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
Test-McpRequest "Ping" '{"jsonrpc":"2.0","id":3,"method":"ping"}'
Write-Host ""

Write-Host "--- Provider Management Tests ---" -ForegroundColor Cyan
Test-McpRequest "List Claude Providers" '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"list_providers","arguments":{"app":"claude"}}}'
Test-McpRequest "List Codex Providers" '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"list_providers","arguments":{"app":"codex"}}}'
Test-McpRequest "List Gemini Providers" '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"list_providers","arguments":{"app":"gemini"}}}'
Test-McpRequest "Get Current Provider" '{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"get_current_provider","arguments":{"app":"claude"}}}'
Write-Host ""

Write-Host "--- MCP Servers Tests ---" -ForegroundColor Cyan
Test-McpRequest "List MCP Servers" '{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"list_mcp_servers","arguments":{}}}'
Write-Host ""

Write-Host "--- Universal Providers Tests ---" -ForegroundColor Cyan
Test-McpRequest "List Universal Providers" '{"jsonrpc":"2.0","id":30,"method":"tools/call","params":{"name":"list_universal_providers","arguments":{}}}'
Write-Host ""

Write-Host "--- Utility Tests ---" -ForegroundColor Cyan
Test-McpRequest "Get DB Path" '{"jsonrpc":"2.0","id":40,"method":"tools/call","params":{"name":"get_db_path","arguments":{}}}'
Write-Host ""

Write-Host "=== Tests Complete ===" -ForegroundColor Cyan