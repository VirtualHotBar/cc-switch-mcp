#!/bin/bash
# Test script for CC Switch MCP Server

# Test initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | ./target/release/cc-switch-mcp.exe

# Test tools/list
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./target/release/cc-switch-mcp.exe