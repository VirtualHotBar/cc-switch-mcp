use cc_switch_lib::{AppState, AppType, Database, Provider, ProviderService};
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
            .map_err(|e: cc_switch_lib::AppError| crate::Error::Database(e.to_string()))?;
        let state = Arc::new(AppState::new(Arc::new(db)));
        Ok(Self { state })
    }

    pub fn run(&self) -> crate::Result<()> {
        tracing::info!("Starting CC Switch MCP Server (v{})", SERVER_VERSION);
        tracing::info!("Using CC Switch core library for provider management");

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
                    self.write_error(&mut stdout, None, -32700, "Parse error", &e.to_string())?;
                    continue;
                }
            };

            let response = self.handle_request(request)?;

            let response_str = serde_json::to_string(&response)?;
            tracing::debug!("Sending: {}", response_str);
            stdout.write_all(response_str.as_bytes())?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn write_error(
        &self,
        stdout: &mut impl Write,
        id: Option<Value>,
        code: i32,
        message: &str,
        details: &str,
    ) -> crate::Result<()> {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: Some(json!({ "details": details })),
            }),
        };
        let response_str = serde_json::to_string(&response)?;
        stdout.write_all(response_str.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
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

    fn handle_tools_list(&self) -> crate::Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "list_providers",
                    "description": "List all providers for a CLI tool with their configurations",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"],
                                "description": "The CLI tool (claude/codex/gemini/opencode)"
                            }
                        },
                        "required": ["app"]
                    }
                },
                {
                    "name": "get_current_provider",
                    "description": "Get the currently active provider for a CLI tool",
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
                    "description": "Switch to a specific provider. Automatically syncs config files (claude.json, codex config, etc.)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"],
                                "description": "The CLI tool"
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
                    "name": "add_provider",
                    "description": "Add a new provider with specified configuration",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"],
                                "description": "The CLI tool"
                            },
                            "name": {
                                "type": "string",
                                "description": "Provider display name"
                            },
                            "baseUrl": {
                                "type": "string",
                                "description": "API base URL (e.g., https://api.anthropic.com)"
                            },
                            "apiKey": {
                                "type": "string",
                                "description": "API key/token"
                            },
                            "model": {
                                "type": "string",
                                "description": "Default model name (optional)"
                            }
                        },
                        "required": ["app", "name", "baseUrl", "apiKey"]
                    }
                },
                {
                    "name": "delete_provider",
                    "description": "Delete a provider by ID",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"],
                                "description": "The CLI tool"
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID to delete"
                            }
                        },
                        "required": ["app", "providerId"]
                    }
                },
                {
                    "name": "sync_current_to_live",
                    "description": "Sync current provider settings to live config files for all apps",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "get_custom_endpoints",
                    "description": "Get list of custom endpoints for a provider",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode"],
                                "description": "The CLI tool"
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID"
                            }
                        },
                        "required": ["app", "providerId"]
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

        tracing::info!("Tool call: {} with args: {}", tool_name, tool_args);

        let result = match tool_name {
            "list_providers" => self.tool_list_providers(tool_args)?,
            "get_current_provider" => self.tool_get_current_provider(tool_args)?,
            "switch_provider" => self.tool_switch_provider(tool_args)?,
            "add_provider" => self.tool_add_provider(tool_args)?,
            "delete_provider" => self.tool_delete_provider(tool_args)?,
            "sync_current_to_live" => self.tool_sync_current_to_live()?,
            "get_custom_endpoints" => self.tool_get_custom_endpoints(tool_args)?,
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
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        let providers = ProviderService::list(&self.state, app_type.clone())
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let current = ProviderService::current(&self.state, app_type)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let provider_list: Vec<Value> = providers
            .iter()
            .map(|(id, p)| {
                json!({
                    "id": id,
                    "name": p.name,
                    "isCurrent": id == &current,
                    "settingsConfig": p.settings_config,
                    "category": p.category,
                    "meta": p.meta,
                    "createdAt": p.created_at,
                    "icon": p.icon,
                    "inFailoverQueue": p.in_failover_queue
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&json!({
            "app": app,
            "providers": provider_list,
            "currentProviderId": current,
            "total": provider_list.len()
        }))?)
    }

    fn tool_get_current_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        let current = ProviderService::current(&self.state, app_type.clone())
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        if current.is_empty() {
            return Ok(serde_json::to_string_pretty(&json!({
                "app": app,
                "currentProvider": null,
                "message": "No provider currently active"
            }))?);
        }

        let providers = ProviderService::list(&self.state, app_type)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let provider = providers
            .get(&current)
            .ok_or_else(|| crate::Error::ProviderNotFound(current.clone()))?;

        Ok(serde_json::to_string_pretty(&json!({
            "app": app,
            "currentProvider": {
                "id": current,
                "name": provider.name,
                "settingsConfig": provider.settings_config,
                "category": provider.category,
                "meta": provider.meta,
                "createdAt": provider.created_at,
                "icon": provider.icon,
                "inFailoverQueue": provider.in_failover_queue
            }
        }))?)
    }

    fn tool_switch_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        let result = ProviderService::switch(&self.state, app_type, provider_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Switched to provider {} for {}", provider_id, app);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "providerId": provider_id,
            "configSynced": true,
            "warnings": result.warnings
        }))?)
    }

    fn tool_add_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'name' parameter".into()))?;
        let base_url = args
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'baseUrl' parameter".into()))?;
        let api_key = args
            .get("apiKey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'apiKey' parameter".into()))?;
        let model = args.get("model").and_then(|v| v.as_str());

        let app_type = self.parse_app_type(app)?;

        let provider_id = uuid::Uuid::new_v4().to_string();

        let mut settings_config = json!({
            "env": {
                "ANTHROPIC_BASE_URL": base_url,
                "ANTHROPIC_AUTH_TOKEN": api_key
            }
        });

        if let Some(m) = model {
            settings_config["env"]["ANTHROPIC_MODEL"] = json!(m);
        }

        let provider = Provider {
            id: provider_id.clone(),
            name: name.to_string(),
            settings_config,
            website_url: None,
            category: Some("custom".to_string()),
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        };

        ProviderService::add(&self.state, app_type, provider, true)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Added provider {} for {}", provider_id, app);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "providerId": provider_id,
            "name": name,
            "message": "Provider added successfully"
        }))?)
    }

    fn tool_delete_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        ProviderService::delete(&self.state, app_type.clone(), provider_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        ProviderService::remove_from_live_config(&self.state, app_type, provider_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Deleted provider {} for {}", provider_id, app);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "providerId": provider_id,
            "message": "Provider deleted and removed from live config"
        }))?)
    }

    fn tool_sync_current_to_live(&self) -> crate::Result<String> {
        ProviderService::sync_current_to_live(&self.state)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Synced current providers to live configs");

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "message": "Synced current provider settings to live config files",
            "syncedApps": ["claude", "codex", "gemini", "opencode"]
        }))?)
    }

    fn tool_get_custom_endpoints(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        let endpoints = ProviderService::get_custom_endpoints(&self.state, app_type, provider_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        Ok(serde_json::to_string_pretty(&json!({
            "app": app,
            "providerId": provider_id,
            "endpoints": endpoints,
            "total": endpoints.len()
        }))?)
    }

    fn handle_resources_list(&self) -> crate::Result<Value> {
        Ok(json!({
            "resources": [
                {
                    "uri": "ccswitch://providers/claude",
                    "name": "Claude Providers",
                    "description": "All Claude providers from CC Switch database",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/codex",
                    "name": "Codex Providers",
                    "description": "All Codex providers from CC Switch database",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/gemini",
                    "name": "Gemini Providers",
                    "description": "All Gemini providers from CC Switch database",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/opencode",
                    "name": "OpenCode Providers",
                    "description": "All OpenCode providers from CC Switch database",
                    "mimeType": "application/json"
                }
            ]
        }))
    }
}
