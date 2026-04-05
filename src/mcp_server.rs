use crate::config_sync;
use crate::core::{AppType, Provider};
use crate::database::Database;
use crate::error::Result;
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
    db: Arc<Database>,
}

impl McpServer {
    pub fn new() -> Result<Self> {
        let db = Database::open()?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn run(&self) -> Result<()> {
        tracing::info!("Starting CC Switch MCP Server (standalone mode)");

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            tracing::debug!("Received: {}", line);

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
                    stdout.write_all(response_str.as_bytes())?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request)?;

            let response_str = serde_json::to_string(&response)?;
            stdout.write_all(response_str.as_bytes())?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
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

    fn handle_initialize(&self, params: Option<Value>) -> Result<Value> {
        let client_info = params.and_then(|p| p.get("clientInfo").cloned());
        tracing::info!("Client connected: {:?}", client_info);

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

    fn handle_tools_list(&self) -> Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "list_providers",
                    "description": "List all providers for a CLI tool (reads from CC Switch database)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            }
                        },
                        "required": ["app"]
                    }
                },
                {
                    "name": "switch_provider",
                    "description": "Switch to a specific provider (updates database and syncs config files)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"]
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
                    "description": "Get currently active provider",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"]
                            }
                        },
                        "required": ["app"]
                    }
                }
            ]
        }))
    }

    fn handle_tools_call(&self, params: Option<Value>) -> Result<Value> {
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

    fn parse_app_type(&self, app: &str) -> Result<AppType> {
        AppType::from_str(app).ok_or_else(|| crate::Error::InvalidApp(app.to_string()))
    }

    fn tool_list_providers(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing app parameter".into()))?;

        let app_type = self.parse_app_type(app)?;
        let providers = self.db.get_providers(app_type)?;
        let current_id = self.db.get_current_provider(app_type)?;

        let providers_json: Vec<Value> = providers
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "name": p.name,
                    "isCurrent": p.is_current,
                    "category": p.category,
                    "websiteUrl": p.website_url
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&json!({
            "providers": providers_json,
            "currentProviderId": current_id,
            "source": "cc-switch-db-standalone",
            "note": "Reading directly from CC Switch database"
        }))?)
    }

    fn tool_switch_provider(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing app parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing providerId parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        let provider = self
            .db
            .get_provider(app_type, provider_id)?
            .ok_or_else(|| crate::Error::ProviderNotFound(provider_id.to_string()))?;

        self.db.set_current_provider(app_type, provider_id)?;
        config_sync::sync_provider_to_config(app_type, &provider)?;

        tracing::info!("Switched to provider {} for {}", provider_id, app);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "message": format!("Switched to provider {} for {}", provider_id, app),
            "config_synced": true,
            "source": "cc-switch-mcp-standalone",
            "warnings": []
        }))?)
    }

    fn tool_get_current_provider(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing app parameter".into()))?;

        let app_type = self.parse_app_type(app)?;
        let current_id = self.db.get_current_provider(app_type)?;

        if current_id.is_none() {
            return Ok(serde_json::to_string_pretty(&json!({
                "current": Value::Null,
                "message": "No provider currently active",
                "source": "cc-switch-mcp-standalone"
            }))?);
        }

        let provider = self
            .db
            .get_provider(app_type, current_id.as_ref().unwrap())?
            .ok_or_else(|| crate::Error::ProviderNotFound(current_id.unwrap()))?;

        Ok(serde_json::to_string_pretty(&json!({
            "current": {
                "id": provider.id,
                "name": provider.name,
                "isCurrent": provider.is_current,
                "category": provider.category,
                "websiteUrl": provider.website_url
            },
            "source": "cc-switch-mcp-standalone"
        }))?)
    }

    fn handle_resources_list(&self) -> Result<Value> {
        Ok(json!({
            "resources": [
                {
                    "uri": "ccswitch://providers/claude",
                    "name": "Claude Providers",
                    "description": "Provider list from CC Switch database",
                    "mimeType": "application/json"
                }
            ]
        }))
    }
}
