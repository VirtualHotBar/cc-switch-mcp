use crate::config_service::ConfigService;
use crate::database::Database;
use crate::error::{Error, Result};
use crate::provider::{Provider, UniversalProvider};
use crate::provider_service::ProviderService;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const MCP_VERSION: &str = "2024-11-05";
const SERVER_INFO: &str = "cc-switch-mcp";
const SERVER_VERSION: &str = "0.1.0";

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
    service: ProviderService,
}

impl McpServer {
    pub fn new() -> Result<Self> {
        let service = ProviderService::new()?;
        Ok(Self { service })
    }

    pub fn new_in_memory() -> Result<Self> {
        let db = Database::new_in_memory()?;
        let config = ConfigService::new();
        let service = ProviderService { db, config };
        Ok(Self { service })
    }

    pub fn run(&self) -> Result<()> {
        tracing::info!("MCP Server running, waiting for requests...");

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
                    tracing::debug!("Sending: {}", response_str);
                    stdout.write_all(response_str.as_bytes())?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
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

    fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params)?,
            "initialized" => Value::Null,
            "tools/list" => self.handle_tools_list()?,
            "tools/call" => self.handle_tools_call(request.params)?,
            "resources/list" => self.handle_resources_list()?,
            "resources/read" => self.handle_resources_read(request.params)?,
            "prompts/list" => self.handle_prompts_list()?,
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
                "resources": {},
                "prompts": {}
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
                    "description": "List all providers for a specific CLI tool (claude, codex, gemini, opencode, openclaw)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool to list providers for"
                            }
                        },
                        "required": ["app"]
                    }
                },
                {
                    "name": "add_provider",
                    "description": "Add a new provider configuration for a CLI tool",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool to add provider to"
                            },
                            "name": {
                                "type": "string",
                                "description": "Provider name"
                            },
                            "apiKey": {
                                "type": "string",
                                "description": "API key for the provider"
                            },
                            "baseUrl": {
                                "type": "string",
                                "description": "API base URL"
                            },
                            "model": {
                                "type": "string",
                                "description": "Default model name"
                            },
                            "notes": {
                                "type": "string",
                                "description": "Optional notes"
                            }
                        },
                        "required": ["app", "name", "apiKey", "baseUrl"]
                    }
                },
                {
                    "name": "switch_provider",
                    "description": "Switch to a specific provider for a CLI tool",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool to switch provider for"
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
                    "name": "delete_provider",
                    "description": "Delete a provider configuration",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool the provider belongs to"
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
                    "name": "get_current_provider",
                    "description": "Get the currently active provider for a CLI tool",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool to check current provider for"
                            }
                        },
                        "required": ["app"]
                    }
                },
                {
                    "name": "list_universal_providers",
                    "description": "List all universal providers (cross-app shared configuration)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "add_universal_provider",
                    "description": "Add a universal provider that can be used across multiple CLI tools",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Provider name"
                            },
                            "providerType": {
                                "type": "string",
                                "enum": ["newapi", "custom", "openai", "anthropic", "gemini"],
                                "description": "Provider type"
                            },
                            "apiKey": {
                                "type": "string",
                                "description": "API key"
                            },
                            "baseUrl": {
                                "type": "string",
                                "description": "API base URL"
                            },
                            "apps": {
                                "type": "object",
                                "properties": {
                                    "claude": { "type": "boolean" },
                                    "codex": { "type": "boolean" },
                                    "gemini": { "type": "boolean" },
                                    "opencode": { "type": "boolean" },
                                    "openclaw": { "type": "boolean" }
                                },
                                "description": "Which apps to enable"
                            }
                        },
                        "required": ["name", "providerType", "apiKey", "baseUrl", "apps"]
                    }
                },
                {
                    "name": "delete_universal_provider",
                    "description": "Delete a universal provider",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID to delete"
                            }
                        },
                        "required": ["providerId"]
                    }
                },
                {
                    "name": "get_db_path",
                    "description": "Get the path to the CC Switch database file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "list_mcp_servers",
                    "description": "List all MCP server configurations",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "add_mcp_server",
                    "description": "Add a new MCP server configuration",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Server name"
                            },
                            "serverConfig": {
                                "type": "string",
                                "description": "JSON string of server configuration"
                            },
                            "description": {
                                "type": "string",
                                "description": "Server description"
                            },
                            "enabledApps": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "enum": ["claude", "codex", "gemini", "opencode"]
                                },
                                "description": "Apps to enable this server for"
                            }
                        },
                        "required": ["name", "serverConfig"]
                    }
                },
                {
                    "name": "delete_mcp_server",
                    "description": "Delete an MCP server configuration",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "serverId": {
                                "type": "string",
                                "description": "Server ID to delete"
                            }
                        },
                        "required": ["serverId"]
                    }
                }
            ]
        }))
    }

    fn handle_tools_call(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| Error::McpProtocol("Missing params".into()))?;
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing tool name".into()))?;
        let tool_args = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match tool_name {
            "list_providers" => self.tool_list_providers(tool_args)?,
            "add_provider" => self.tool_add_provider(tool_args)?,
            "switch_provider" => self.tool_switch_provider(tool_args)?,
            "delete_provider" => self.tool_delete_provider(tool_args)?,
            "get_current_provider" => self.tool_get_current_provider(tool_args)?,
            "list_universal_providers" => self.tool_list_universal_providers(tool_args)?,
            "add_universal_provider" => self.tool_add_universal_provider(tool_args)?,
            "delete_universal_provider" => self.tool_delete_universal_provider(tool_args)?,
            "get_db_path" => self.tool_get_db_path(tool_args)?,
            "list_mcp_servers" => self.tool_list_mcp_servers(tool_args)?,
            "add_mcp_server" => self.tool_add_mcp_server(tool_args)?,
            "delete_mcp_server" => self.tool_delete_mcp_server(tool_args)?,
            _ => {
                return Err(Error::McpProtocol(format!("Unknown tool: {}", tool_name)));
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

    fn tool_list_providers(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing app parameter".into()))?;

        let manager = self.service.list_providers(app)?;
        let providers = manager.list_providers();

        let result = serde_json::to_string_pretty(&json!({
            "providers": providers,
            "current": manager.current
        }))?;

        Ok(result)
    }

    fn tool_add_provider(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing app parameter".into()))?;
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing name parameter".into()))?;
        let api_key = args
            .get("apiKey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing apiKey parameter".into()))?;
        let base_url = args
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing baseUrl parameter".into()))?;
        let model = args
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        let notes = args.get("notes").and_then(|v| v.as_str());

        let id = format!("{}-{}", app, uuid::Uuid::new_v4());

        let settings_config = match app {
            "claude" => json!({
                "env": {
                    "ANTHROPIC_BASE_URL": base_url,
                    "ANTHROPIC_AUTH_TOKEN": api_key,
                    "ANTHROPIC_MODEL": model
                }
            }),
            "codex" => {
                let codex_base_url = if base_url.ends_with("/v1") {
                    base_url.to_string()
                } else {
                    format!("{}/v1", base_url.trim_end_matches('/'))
                };
                let config_toml = format!(
                    r#"model_provider = "newapi"
model = "{}"

[model_providers.newapi]
name = "NewAPI"
base_url = "{}"
wire_api = "responses"
requires_openai_auth = true"#,
                    model, codex_base_url
                );
                json!({
                    "auth": { "OPENAI_API_KEY": api_key },
                    "config": config_toml
                })
            }
            "gemini" => json!({
                "env": {
                    "GOOGLE_GEMINI_BASE_URL": base_url,
                    "GEMINI_API_KEY": api_key,
                    "GEMINI_MODEL": model
                }
            }),
            "opencode" | "openclaw" => json!({
                "npm": "@ai-sdk/openai-compatible",
                "options": {
                    "baseURL": base_url,
                    "apiKey": api_key
                }
            }),
            _ => return Err(Error::UnknownAppType(app.into())),
        };

        let mut provider = Provider::new(id.clone(), name.to_string(), settings_config);
        provider.notes = notes.map(|n| n.to_string());

        self.service.add_provider(app, &provider, true)?;

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "providerId": id,
            "message": "Provider added and activated (config synced)"
        }))?)
    }

    fn tool_switch_provider(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing app parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing providerId parameter".into()))?;

        let success = self.service.switch_provider(app, provider_id)?;

        if success {
            Ok(serde_json::to_string_pretty(&json!({
                "success": true,
                "message": format!("Switched to provider {} for {} (config synced)", provider_id, app)
            }))?)
        } else {
            Err(Error::ProviderNotFound(provider_id.into()))
        }
    }

    fn tool_delete_provider(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing app parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing providerId parameter".into()))?;

        let success = self.service.delete_provider(app, provider_id)?;

        Ok(serde_json::to_string_pretty(&json!({
            "success": success,
            "message": if success { "Provider deleted" } else { "Provider not found" }
        }))?)
    }

    fn tool_get_current_provider(&self, args: Value) -> Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing app parameter".into()))?;

        let provider = self.service.get_current_provider(app)?;

        if provider.is_none() {
            return Ok(serde_json::to_string_pretty(&json!({
                "current": serde_json::Value::Null,
                "message": "No provider currently active"
            }))?);
        }

        Ok(serde_json::to_string_pretty(&json!({
            "current": provider
        }))?)
    }

    fn tool_list_universal_providers(&self, _args: Value) -> Result<String> {
        let manager = self.service.get_db().get_universal_provider_manager()?;
        let providers = manager.list_providers();

        Ok(serde_json::to_string_pretty(&json!({
            "providers": providers
        }))?)
    }

    fn tool_add_universal_provider(&self, args: Value) -> Result<String> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing name parameter".into()))?;
        let provider_type = args
            .get("providerType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing providerType parameter".into()))?;
        let api_key = args
            .get("apiKey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing apiKey parameter".into()))?;
        let base_url = args
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing baseUrl parameter".into()))?;
        let apps = args
            .get("apps")
            .ok_or_else(|| Error::McpProtocol("Missing apps parameter".into()))?;

        let id = format!("universal-{}", uuid::Uuid::new_v4());

        let apps_config: crate::provider::UniversalProviderApps =
            serde_json::from_value(apps.clone())
                .map_err(|e| Error::InvalidConfig(format!("Invalid apps config: {}", e)))?;

        let provider = UniversalProvider::new(
            id.clone(),
            name.to_string(),
            provider_type.to_string(),
            base_url.to_string(),
            api_key.to_string(),
        );

        self.service.get_db().save_universal_provider(&provider)?;

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "providerId": id,
            "apps": apps_config,
            "message": "Universal provider added. Use switch_provider to activate it for specific apps."
        }))?)
    }

    fn tool_delete_universal_provider(&self, args: Value) -> Result<String> {
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing providerId parameter".into()))?;

        let success = self
            .service
            .get_db()
            .delete_universal_provider(provider_id)?;

        Ok(serde_json::to_string_pretty(&json!({
            "success": success,
            "message": if success { "Universal provider deleted" } else { "Provider not found" }
        }))?)
    }

    fn tool_get_db_path(&self, _args: Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(&json!({
            "dbPath": self.service.get_db().get_db_path().to_string_lossy()
        }))?)
    }

    fn tool_list_mcp_servers(&self, _args: Value) -> Result<String> {
        let servers = self.service.get_db().get_mcp_servers()?;

        Ok(serde_json::to_string_pretty(&json!({
            "servers": servers
        }))?)
    }

    fn tool_add_mcp_server(&self, args: Value) -> Result<String> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing name parameter".into()))?;
        let server_config = args
            .get("serverConfig")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing serverConfig parameter".into()))?;
        let description = args.get("description").and_then(|v| v.as_str());
        let enabled_apps = args
            .get("enabledApps")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let id = format!("mcp-{}", uuid::Uuid::new_v4());

        let server = crate::database::McpServerConfig {
            id: id.clone(),
            name: name.to_string(),
            server_config: server_config.to_string(),
            description: description.map(|s| s.to_string()),
            homepage: None,
            docs: None,
            tags: "[]".to_string(),
            enabled_claude: enabled_apps.contains(&"claude".to_string()),
            enabled_codex: enabled_apps.contains(&"codex".to_string()),
            enabled_gemini: enabled_apps.contains(&"gemini".to_string()),
            enabled_opencode: enabled_apps.contains(&"opencode".to_string()),
        };

        self.service.get_db().save_mcp_server(&server)?;

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "serverId": id,
            "message": "MCP server added successfully"
        }))?)
    }

    fn tool_delete_mcp_server(&self, args: Value) -> Result<String> {
        let server_id = args
            .get("serverId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing serverId parameter".into()))?;

        let success = self.service.get_db().delete_mcp_server(server_id)?;

        Ok(serde_json::to_string_pretty(&json!({
            "success": success,
            "message": if success { "MCP server deleted" } else { "Server not found" }
        }))?)
    }

    fn handle_resources_list(&self) -> Result<Value> {
        Ok(json!({
            "resources": [
                {
                    "uri": "ccswitch://providers/claude",
                    "name": "Claude Providers",
                    "description": "List of Claude Code providers",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/codex",
                    "name": "Codex Providers",
                    "description": "List of Codex providers",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/gemini",
                    "name": "Gemini Providers",
                    "description": "List of Gemini CLI providers",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/opencode",
                    "name": "OpenCode Providers",
                    "description": "List of OpenCode providers",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://providers/openclaw",
                    "name": "OpenClaw Providers",
                    "description": "List of OpenClaw providers",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://universal-providers",
                    "name": "Universal Providers",
                    "description": "List of cross-app universal providers",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://config/path",
                    "name": "Config Path",
                    "description": "CC Switch configuration path",
                    "mimeType": "text/plain"
                }
            ]
        }))
    }

    fn handle_resources_read(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| Error::McpProtocol("Missing params".into()))?;
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::McpProtocol("Missing uri parameter".into()))?;

        let content = if uri.starts_with("ccswitch://providers/") {
            let app = uri.replace("ccswitch://providers/", "");
            let manager = self.service.list_providers(&app)?;
            serde_json::to_string_pretty(&json!({
                "providers": manager.list_providers(),
                "current": manager.current
            }))?
        } else if uri == "ccswitch://universal-providers" {
            let manager = self.service.get_db().get_universal_provider_manager()?;
            serde_json::to_string_pretty(&json!({
                "providers": manager.list_providers()
            }))?
        } else if uri == "ccswitch://config/path" {
            self.service
                .get_db()
                .get_db_path()
                .to_string_lossy()
                .to_string()
        } else {
            return Err(Error::McpProtocol(format!("Unknown resource: {}", uri)));
        };

        Ok(json!({
            "contents": [
                {
                    "uri": uri,
                    "mimeType": if uri.ends_with("/path") { "text/plain" } else { "application/json" },
                    "text": content
                }
            ]
        }))
    }

    fn handle_prompts_list(&self) -> Result<Value> {
        Ok(json!({
            "prompts": []
        }))
    }
}
