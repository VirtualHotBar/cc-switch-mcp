use cc_switch_lib::{AppState, AppType, Database, ProviderService};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;

const MCP_VERSION: &str = "2024-11-05";
const SERVER_INFO: &str = "cc-switch-mcp";
const SERVER_VERSION: &str = "0.2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub struct McpServer {
    state: Arc<AppState>,
}

impl McpServer {
    pub fn new() -> crate::Result<Self> {
        let db = Database::init()
            .map_err(|e| crate::Error::Database(e.to_string()))?;
        let state = Arc::new(AppState::new(Arc::new(db)));
        Ok(Self { state })
    }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub struct McpServer {
    state: Arc<AppState>,
}

impl McpServer {
    pub fn new() -> crate::Result<Self> {
        let db = Database::init()
            .map_err(|e: cc_switch_lib::error::AppError| crate::Error::Database(e.to_string()))?;
        let state = Arc::new(AppState::new(Arc::new(db)));
        Ok(Self { state })
    }

    pub fn run(&self) -> crate::Result<()> {
        tracing::info!("🚀 MCP Server running with CC Switch core library");
        tracing::info!("📍 Using CC Switch's ProviderService for all operations");

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            tracing::debug!("📥 Received: {}", line);

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: "Parse error".to_string(),
                            data: Some(json!({ "details": e.to_string() })),
                        }),
                    };
                    let response_str = serde_json::to_string(&error_response)?;
                    tracing::debug!("📤 Sending: {}", response_str);
                    stdout.write_all(response_str.as_bytes())?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request)?;

            let response_str = serde_json::to_string(&response)?;
            tracing::debug!("📤 Sending: {}", response_str);
            stdout.write_all(response_str.as_bytes())?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: JsonRpcRequest) -> crate::Result<JsonRpcResponse> {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params)?,
            "initialized" => Value::Null,
            "tools/list" => self.handle_tools_list()?,
            "tools/call" => self.handle_tools_call(request.params)?,
            "resources/list" => self.handle_resources_list()?,
            "ping" => json!({}),
            _ => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: Some(json!({ "method": request.method })),
                    }),
                });
            }
        };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        })
    }

    fn handle_initialize(&self, params: Option<Value>) -> crate::Result<Value> {
        let client_info = params.and_then(|p| p.get("clientInfo").cloned());
        tracing::info!("✅ Client connected: {:?}", client_info);

        Ok(json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": SERVER_INFO,
                "version": SERVER_VERSION
            }
        }))
    }

    fn handle_tools_list(&self) -> crate::Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "list_providers",
                    "description": "List all providers for a CLI tool (uses CC Switch core)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"],
                                "description": "The CLI tool"
                            }
                        },
                        "required": ["app"]
                    }
                },
                {
                    "name": "switch_provider",
                    "description": "Switch to a specific provider (uses CC Switch core - auto syncs config)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"]
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID to switch to"
                            }
                        },
                        "required": ["app", "providerId"]
                    }
                },
                {
                    "name": "get_current_provider",
                    "description": "Get currently active provider (uses CC Switch core)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"]
                            }
                        },
                        "required": ["app"]
                    }
                }
            ]
        }))
    }

    fn handle_tools_call(&self, params: Option<Value>) -> crate::Result<Value> {
        let params = params.ok_or_else(|| crate::Error::McpProtocol("Missing params".into()))?;
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing tool name".into()))?;
        let tool_args = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match tool_name {
            "list_providers" => self.tool_list_providers(tool_args)?,
            "switch_provider" => self.tool_switch_provider(tool_args)?,
            "get_current_provider" => self.tool_get_current_provider(tool_args)?,
            _ => {
                return Err(crate::Error::McpProtocol(format!(
                    "Unknown tool: {}",
                    tool_name
                )));
            }
        };

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": result
                }
            ]
        }))
    }

    fn parse_app_type(&self, app: &str) -> crate::Result<AppType> {
        match app {
            "claude" => Ok(AppType::Claude),
            "codex" => Ok(AppType::Codex),
            "gemini" => Ok(AppType::Gemini),
            "opencode" => Ok(AppType::OpenCode),
            _ => Err(crate::Error::InvalidApp(app.to_string())),
        }
    }

    fn tool_list_providers(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing app parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        // Use CC Switch's ProviderService
        let providers = ProviderService::list(&self.state, app_type.clone())
            .map_err(|e| crate::Error::ProviderNotFound(e.to_string()))?;

        let current = ProviderService::current(&self.state, app_type)
            .map_err(|e| crate::Error::ProviderNotFound(e.to_string()))?;

        Ok(serde_json::to_string_pretty(&json!({
            "providers": providers,
            "current": current,
            "source": "cc-switch-core-library-v3.12.3",
            "note": "Using CC Switch's ProviderService for identical behavior"
        }))?)
    }

    fn tool_switch_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing app parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing providerId parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        // Use CC Switch's ProviderService::switch - automatically syncs config files!
        let result = ProviderService::switch(&self.state, app_type, provider_id)
            .map_err(|e| crate::Error::ProviderNotFound(e.to_string()))?;

        tracing::info!(
            "✅ Switched to provider {} for {} using CC Switch core",
            provider_id,
            app
        );
        if !result.warnings.is_empty() {
            tracing::warn!("⚠️ Warnings: {:?}", result.warnings);
        }

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "message": format!("Switched to provider {} for {}", provider_id, app),
            "config_synced": true,
            "source": "cc-switch-core-library",
            "warnings": result.warnings
        }))?)
    }

    fn tool_get_current_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing app parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        // Use CC Switch's ProviderService
        let current = ProviderService::current(&self.state, app_type.clone())
            .map_err(|e| crate::Error::ProviderNotFound(e.to_string()))?;

        if current.is_empty() {
            return Ok(serde_json::to_string_pretty(&json!({
                "current": Value::Null,
                "message": "No provider currently active",
                "source": "cc-switch-core-library"
            }))?);
        }

        let providers = ProviderService::list(&self.state, app_type)
            .map_err(|e| crate::Error::ProviderNotFound(e.to_string()))?;

        let provider = providers
            .get(&current)
            .ok_or_else(|| crate::Error::ProviderNotFound(current.clone()))?;

        Ok(serde_json::to_string_pretty(&json!({
            "current": provider,
            "source": "cc-switch-core-library"
        }))?)
    }

    fn handle_resources_list(&self) -> crate::Result<Value> {
        Ok(json!({
            "resources": [
                {
                    "uri": "ccswitch://providers/claude",
                    "name": "Claude Providers (CC Switch Core)",
                    "description": "Provider list from CC Switch's ProviderService",
                    "mimeType": "application/json"
                }
            ]
        }))
    }
}
