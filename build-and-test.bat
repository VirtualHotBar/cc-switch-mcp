@echo off
REM Build and Test Script for CC Switch MCP Server

echo === CC Switch MCP Server Build and Test ===
echo.

echo [1/4] Running unit tests...
cargo test --lib
if %ERRORLEVEL% NEQ 0 (
    echo Unit tests FAILED!
    exit /b 1
)
echo.

echo [2/4] Building release binary...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo Build FAILED!
    exit /b 1
)
echo.

echo [3/4] Running integration tests...
powershell -ExecutionPolicy Bypass -File test-integration.ps1
if %ERRORLEVEL% NEQ 0 (
    echo Integration tests FAILED!
    exit /b 1
)
echo.

echo [4/4] Binary size:
for %%I in (target\release\cc-switch-mcp.exe) do echo   %%~zI bytes (%%~nI)
echo.

echo === All tests passed! ===
echo Binary location: target\release\cc-switch-mcp.exe
echo.