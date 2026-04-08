use cc_switch_lib::{AppState, AppType, Database, McpApps, McpServer as McpServerConfig, McpService, Provider, ProviderService, ProviderSortUpdate, UniversalProvider};
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

    /// Validate a URL format
    fn validate_url(&self, url: &str) -> crate::Result<()> {
        if url.is_empty() {
            return Err(crate::Error::InvalidUrl("URL cannot be empty".to_string()));
        }

        // Check basic URL format
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(crate::Error::InvalidUrl(
                format!("URL must start with http:// or https://, got: {}", url)
            ));
        }

        // Try to parse the URL
        match url::Url::parse(url) {
            Ok(parsed) => {
                // Ensure host is present
                if parsed.host().is_none() {
                    return Err(crate::Error::InvalidUrl(
                        format!("URL must have a valid host: {}", url)
                    ));
                }
                Ok(())
            }
            Err(e) => Err(crate::Error::InvalidUrl(
                format!("Invalid URL format '{}': {}", url, e)
            )),
        }
    }

    /// Validate API key format
    fn validate_api_key(&self, key: &str) -> crate::Result<()> {
        if key.is_empty() {
            return Err(crate::Error::InvalidApiKey("API key cannot be empty".to_string()));
        }

        if key.len() < 8 {
            return Err(crate::Error::InvalidApiKey(
                format!("API key seems too short ({} chars). Minimum recommended length is 8 characters.", key.len())
            ));
        }

        // Check for common placeholder patterns
        let lower = key.to_lowercase();
        if lower.contains("your_") || lower.contains("placeholder") || lower.contains("example") {
            return Err(crate::Error::InvalidApiKey(
                "API key appears to be a placeholder. Please provide a real API key.".to_string()
            ));
        }

        Ok(())
    }

    /// Validate provider name
    fn validate_provider_name(&self, name: &str) -> crate::Result<()> {
        if name.is_empty() {
            return Err(crate::Error::Validation("Provider name cannot be empty".to_string()));
        }

        if name.len() > 100 {
            return Err(crate::Error::Validation(
                format!("Provider name too long ({} chars, max 100)", name.len())
            ));
        }

        // Check for valid characters (alphanumeric, spaces, hyphens, underscores)
        if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_' || c == '.') {
            return Err(crate::Error::Validation(
                "Provider name contains invalid characters. Use only letters, numbers, spaces, hyphens, underscores, and dots.".to_string()
            ));
        }

        Ok(())
    }

    /// Validate model name format
    fn validate_model_name(&self, model: &str) -> crate::Result<()> {
        if model.is_empty() {
            return Err(crate::Error::Validation("Model name cannot be empty".to_string()));
        }

        if model.len() > 200 {
            return Err(crate::Error::Validation(
                format!("Model name too long ({} chars, max 200)", model.len())
            ));
        }

        Ok(())
    }

    /// Validate universal provider type
    fn validate_provider_type(&self, provider_type: &str) -> crate::Result<()> {
        let valid_types = ["newapi", "custom"];
        if !valid_types.contains(&provider_type) {
            return Err(crate::Error::Validation(
                format!("Invalid provider type '{}'. Must be one of: {:?}", provider_type, valid_types)
            ));
        }
        Ok(())
    }

    /// Validate provider IDs array
    fn validate_provider_ids(&self, ids: &[String]) -> crate::Result<()> {
        if ids.is_empty() {
            return Err(crate::Error::Validation("Provider IDs array cannot be empty".to_string()));
        }

        if ids.len() > 1000 {
            return Err(crate::Error::Validation(
                format!("Too many provider IDs ({}). Maximum is 1000.", ids.len())
            ));
        }

        // Check for duplicates
        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        if unique_count != ids.len() {
            return Err(crate::Error::Validation(
                "Provider IDs array contains duplicates".to_string()
            ));
        }

        Ok(())
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
            "initialized" => {
                // Client initialized - send capabilities notification
                self.send_notification("notifications/initialized", json!({}))?;
                Value::Null
            }
            "tools/list" => self.handle_tools_list()?,
            "tools/call" => self.handle_tools_call(request.params)?,
            "resources/list" => self.handle_resources_list()?,
            "resources/read" => self.handle_resources_read(request.params)?,
            "prompts/list" => self.handle_prompts_list()?,
            "prompts/get" => self.handle_prompts_get(request.params)?,
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

    /// Send a notification to the client (server-initiated message)
    fn send_notification(&self, method: &str, params: Value) -> crate::Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let notification_str = serde_json::to_string(&notification)?;
        tracing::info!("Sending notification: {}", method);

        // Write directly to stdout since notifications don't have an id
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        stdout.write_all(notification_str.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;

        Ok(())
    }

    /// Send a tool list changed notification
    fn notify_tools_changed(&self) -> crate::Result<()> {
        self.send_notification("notifications/tools/list_changed", json!({}))
    }

    /// Send a resource list changed notification
    fn notify_resources_changed(&self) -> crate::Result<()> {
        self.send_notification("notifications/resources/list_changed", json!({}))
    }

    /// Send a prompt list changed notification
    fn notify_prompts_changed(&self) -> crate::Result<()> {
        self.send_notification("notifications/prompts/list_changed", json!({}))
    }

    /// Send a logging message notification to the client
    fn send_log_message(&self, level: &str, message: &str, logger: Option<&str>) -> crate::Result<()> {
        let params = json!({
            "level": level,
            "message": message,
            "logger": logger.unwrap_or("cc-switch-mcp")
        });
        self.send_notification("notifications/message", params)
    }

    /// Send a progress notification for long-running operations
    fn send_progress_notification(&self, progress_token: &str, progress: f64, total: Option<f64>) -> crate::Result<()> {
        let params = json!({
            "progressToken": progress_token,
            "progress": progress,
            "total": total
        });
        self.send_notification("notifications/progress", params)
    }

    fn handle_initialize(&self, params: Option<Value>) -> crate::Result<Value> {
        let client_info = params.and_then(|p| p.get("clientInfo").cloned());
        tracing::info!("Client connected: {:?}", client_info);

        Ok(json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {},
                "notifications": {}
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
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool (claude/codex/gemini/opencode/openclaw)"
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
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
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
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
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
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
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
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
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
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID"
                            }
                        },
                        "required": ["app", "providerId"]
                    }
                },
                {
                    "name": "update_provider",
                    "description": "Update an existing provider's configuration (name, API key, base URL, model, etc.)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID to update"
                            },
                            "name": {
                                "type": "string",
                                "description": "New provider display name (optional)"
                            },
                            "baseUrl": {
                                "type": "string",
                                "description": "New API base URL (optional)"
                            },
                            "apiKey": {
                                "type": "string",
                                "description": "New API key/token (optional)"
                            },
                            "model": {
                                "type": "string",
                                "description": "New default model name (optional)"
                            }
                        },
                        "required": ["app", "providerId"]
                    }
                },
                {
                    "name": "add_custom_endpoint",
                    "description": "Add a custom endpoint URL for a provider",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID"
                            },
                            "url": {
                                "type": "string",
                                "description": "Custom endpoint URL to add"
                            }
                        },
                        "required": ["app", "providerId", "url"]
                    }
                },
                {
                    "name": "remove_custom_endpoint",
                    "description": "Remove a custom endpoint URL from a provider",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            },
                            "providerId": {
                                "type": "string",
                                "description": "Provider ID"
                            },
                            "url": {
                                "type": "string",
                                "description": "Custom endpoint URL to remove"
                            }
                        },
                        "required": ["app", "providerId", "url"]
                    }
                },
                {
                    "name": "list_universal_providers",
                    "description": "List all universal providers that can be shared across multiple CLI tools",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "add_universal_provider",
                    "description": "Add a new universal provider that can be shared across multiple CLI tools",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Provider display name"
                            },
                            "providerType": {
                                "type": "string",
                                "enum": ["newapi", "custom"],
                                "description": "Provider type (newapi or custom)"
                            },
                            "baseUrl": {
                                "type": "string",
                                "description": "API base URL"
                            },
                            "apiKey": {
                                "type": "string",
                                "description": "API key/token"
                            },
                            "apps": {
                                "type": "object",
                                "description": "Which apps to enable this provider for",
                                "properties": {
                                    "claude": { "type": "boolean" },
                                    "codex": { "type": "boolean" },
                                    "gemini": { "type": "boolean" },
                                    "opencode": { "type": "boolean" },
                                    "openclaw": { "type": "boolean" }
                                }
                            },
                            "models": {
                                "type": "object",
                                "description": "Model configurations for each app",
                                "properties": {
                                    "claude": {
                                        "type": "object",
                                        "properties": {
                                            "model": { "type": "string" },
                                            "haikuModel": { "type": "string" },
                                            "sonnetModel": { "type": "string" },
                                            "opusModel": { "type": "string" }
                                        }
                                    },
                                    "codex": {
                                        "type": "object",
                                        "properties": {
                                            "model": { "type": "string" }
                                        }
                                    },
                                    "gemini": {
                                        "type": "object",
                                        "properties": {
                                            "model": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        },
                        "required": ["name", "providerType", "baseUrl", "apiKey"]
                    }
                },
                {
                    "name": "delete_universal_provider",
                    "description": "Delete a universal provider by ID",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "providerId": {
                                "type": "string",
                                "description": "Universal provider ID to delete"
                            }
                        },
                        "required": ["providerId"]
                    }
                },
                {
                    "name": "update_universal_provider",
                    "description": "Update an existing universal provider",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "providerId": {
                                "type": "string",
                                "description": "Universal provider ID to update"
                            },
                            "name": {
                                "type": "string",
                                "description": "Provider display name"
                            },
                            "baseUrl": {
                                "type": "string",
                                "description": "API base URL"
                            },
                            "apiKey": {
                                "type": "string",
                                "description": "API key/token"
                            },
                            "apps": {
                                "type": "object",
                                "description": "Which apps to enable this provider for",
                                "properties": {
                                    "claude": { "type": "boolean" },
                                    "codex": { "type": "boolean" },
                                    "gemini": { "type": "boolean" },
                                    "opencode": { "type": "boolean" },
                                    "openclaw": { "type": "boolean" }
                                }
                            },
                            "models": {
                                "type": "object",
                                "description": "Model configurations for each app",
                                "properties": {
                                    "claude": {
                                        "type": "object",
                                        "properties": {
                                            "model": { "type": "string" },
                                            "haikuModel": { "type": "string" },
                                            "sonnetModel": { "type": "string" },
                                            "opusModel": { "type": "string" }
                                        }
                                    },
                                    "codex": {
                                        "type": "object",
                                        "properties": {
                                            "model": { "type": "string" }
                                        }
                                    },
                                    "gemini": {
                                        "type": "object",
                                        "properties": {
                                            "model": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        },
                        "required": ["providerId"]
                    }
                },
                {
                    "name": "sync_universal_provider",
                    "description": "Sync a universal provider to all enabled apps",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "providerId": {
                                "type": "string",
                                "description": "Universal provider ID to sync"
                            }
                        },
                        "required": ["providerId"]
                    }
                },
                {
                    "name": "update_provider_order",
                    "description": "Update the display order of providers for a CLI tool",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            },
                            "providerIds": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Array of provider IDs in the desired order"
                            }
                        },
                        "required": ["app", "providerIds"]
                    }
                },
                {
                    "name": "send_log",
                    "description": "Send a log message through the MCP server (for testing notifications)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "level": {
                                "type": "string",
                                "enum": ["debug", "info", "warning", "error"],
                                "description": "Log level"
                            },
                            "message": {
                                "type": "string",
                                "description": "Log message content"
                            },
                            "logger": {
                                "type": "string",
                                "description": "Optional logger name (defaults to 'cc-switch-mcp')"
                            }
                        },
                        "required": ["level", "message"]
                    }
                },
                {
                    "name": "list_mcp_servers",
                    "description": "List all MCP servers configured in CC Switch",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "get_mcp_server",
                    "description": "Get details of a specific MCP server by ID",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "serverId": {
                                "type": "string",
                                "description": "MCP server ID to retrieve"
                            }
                        },
                        "required": ["serverId"]
                    }
                },
                {
                    "name": "add_mcp_server",
                    "description": "Add a new MCP server to CC Switch",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Display name for the MCP server"
                            },
                            "serverConfig": {
                                "type": "object",
                                "description": "MCP server configuration (command, args, env)"
                            },
                            "apps": {
                                "type": "object",
                                "description": "Which CLI tools to enable this server for",
                                "properties": {
                                    "claude": { "type": "boolean" },
                                    "codex": { "type": "boolean" },
                                    "gemini": { "type": "boolean" },
                                    "opencode": { "type": "boolean" },
                                    "openclaw": { "type": "boolean" }
                                }
                            },
                            "description": {
                                "type": "string",
                                "description": "Optional description of the server"
                            },
                            "homepage": {
                                "type": "string",
                                "description": "Optional homepage URL"
                            },
                            "docs": {
                                "type": "string",
                                "description": "Optional documentation URL"
                            },
                            "tags": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional tags for the server"
                            }
                        },
                        "required": ["name", "serverConfig"]
                    }
                },
                {
                    "name": "delete_mcp_server",
                    "description": "Delete an MCP server by ID",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "serverId": {
                                "type": "string",
                                "description": "MCP server ID to delete"
                            }
                        },
                        "required": ["serverId"]
                    }
                },
                {
                    "name": "toggle_mcp_server",
                    "description": "Enable or disable an MCP server for specific CLI tools",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "serverId": {
                                "type": "string",
                                "description": "MCP server ID"
                            },
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool"
                            },
                            "enabled": {
                                "type": "boolean",
                                "description": "Whether to enable or disable the server"
                            }
                        },
                        "required": ["serverId", "app", "enabled"]
                    }
                },
                {
                    "name": "import_mcp_from_app",
                    "description": "Import MCP servers from a CLI tool's existing configuration",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "app": {
                                "type": "string",
                                "enum": ["claude", "codex", "gemini", "opencode", "openclaw"],
                                "description": "The CLI tool to import from"
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

        tracing::info!("Tool call: {} with args: {}", tool_name, tool_args);

        let result = match tool_name {
            "list_providers" => self.tool_list_providers(tool_args)?,
            "get_current_provider" => self.tool_get_current_provider(tool_args)?,
            "switch_provider" => self.tool_switch_provider(tool_args)?,
            "add_provider" => self.tool_add_provider(tool_args)?,
            "delete_provider" => self.tool_delete_provider(tool_args)?,
            "sync_current_to_live" => self.tool_sync_current_to_live()?,
            "get_custom_endpoints" => self.tool_get_custom_endpoints(tool_args)?,
            "update_provider" => self.tool_update_provider(tool_args)?,
            "add_custom_endpoint" => self.tool_add_custom_endpoint(tool_args)?,
            "remove_custom_endpoint" => self.tool_remove_custom_endpoint(tool_args)?,
            "list_universal_providers" => self.tool_list_universal_providers()?,
            "add_universal_provider" => self.tool_add_universal_provider(tool_args)?,
            "update_universal_provider" => self.tool_update_universal_provider(tool_args)?,
            "delete_universal_provider" => self.tool_delete_universal_provider(tool_args)?,
            "sync_universal_provider" => self.tool_sync_universal_provider(tool_args)?,
            "update_provider_order" => self.tool_update_provider_order(tool_args)?,
            "send_log" => self.tool_send_log(tool_args)?,
            "list_mcp_servers" => self.tool_list_mcp_servers()?,
            "get_mcp_server" => self.tool_get_mcp_server(tool_args)?,
            "add_mcp_server" => self.tool_add_mcp_server(tool_args)?,
            "delete_mcp_server" => self.tool_delete_mcp_server(tool_args)?,
            "toggle_mcp_server" => self.tool_toggle_mcp_server(tool_args)?,
            "import_mcp_from_app" => self.tool_import_mcp_from_app(tool_args)?,
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
            "openclaw" => Ok(AppType::OpenClaw),
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

        // Send log notification about the provider switch
        let _ = self.send_log_message(
            "info",
            &format!("Successfully switched to provider '{}' for {}", provider_id, app),
            Some("provider-switch")
        );

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

        // Validate inputs
        self.validate_provider_name(name)?;
        self.validate_url(base_url)?;
        self.validate_api_key(api_key)?;
        if let Some(m) = model {
            self.validate_model_name(m)?;
        }

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

        // Send log notification
        let _ = self.send_log_message(
            "info",
            &format!("Added new provider '{}' ({}) for {}", name, provider_id, app),
            Some("provider-management")
        );

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

        // Send log notification
        let _ = self.send_log_message(
            "info",
            &format!("Deleted provider '{}' from {}", provider_id, app),
            Some("provider-management")
        );

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

        // Send log notification
        let _ = self.send_log_message(
            "info",
            "Synced all current provider settings to live config files",
            Some("sync")
        );

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "message": "Synced current provider settings to live config files",
            "syncedApps": ["claude", "codex", "gemini", "opencode", "openclaw"]
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

    fn tool_update_provider(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        // Get existing provider
        let providers = ProviderService::list(&self.state, app_type.clone())
            .map_err(|e| crate::Error::Database(e.to_string()))?;
        let mut provider = providers
            .get(provider_id)
            .ok_or_else(|| crate::Error::ProviderNotFound(provider_id.to_string()))?
            .clone();

        // Update fields if provided
        if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
            provider.name = name.to_string();
        }

        // Update settings_config based on app type
        match app_type {
            AppType::Claude => {
                if let Some(obj) = provider.settings_config.as_object_mut() {
                    let env = obj.get_mut("env").and_then(|v| v.as_object_mut());
                    if let Some(env) = env {
                        if let Some(base_url) = args.get("baseUrl").and_then(|v| v.as_str()) {
                            env.insert("ANTHROPIC_BASE_URL".to_string(), json!(base_url));
                        }
                        if let Some(api_key) = args.get("apiKey").and_then(|v| v.as_str()) {
                            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), json!(api_key));
                        }
                        if let Some(model) = args.get("model").and_then(|v| v.as_str()) {
                            env.insert("ANTHROPIC_MODEL".to_string(), json!(model));
                        }
                    }
                }
            }
            AppType::Codex => {
                if let Some(obj) = provider.settings_config.as_object_mut() {
                    if let Some(api_key) = args.get("apiKey").and_then(|v| v.as_str()) {
                        if let Some(auth) = obj.get_mut("auth").and_then(|v| v.as_object_mut()) {
                            auth.insert("OPENAI_API_KEY".to_string(), json!(api_key));
                        }
                    }
                    if let Some(base_url) = args.get("baseUrl").and_then(|v| v.as_str()) {
                        // Update config.toml base_url
                        if let Some(config_val) = obj.get("config").and_then(|v| v.as_str()) {
                            let mut config_doc = config_val.parse::<toml_edit::DocumentMut>()
                                .map_err(|e| crate::Error::McpProtocol(format!("Failed to parse config TOML: {}", e)))?;
                            if let Some(table) = config_doc.get_mut("model_providers").and_then(|v| v.as_table_mut()) {
                                if let Some(first_provider) = table.iter_mut().next() {
                                    if let Some(provider_table) = first_provider.1.as_table_mut() {
                                        provider_table.insert("base_url", toml_edit::value(base_url));
                                    }
                                }
                            } else {
                                // Try to find base_url in root
                                config_doc.insert("base_url", toml_edit::value(base_url));
                            }
                            obj.insert("config".to_string(), json!(config_doc.to_string()));
                        }
                    }
                    if let Some(model) = args.get("model").and_then(|v| v.as_str()) {
                        if let Some(config_val) = obj.get("config").and_then(|v| v.as_str()) {
                            let mut config_doc = config_val.parse::<toml_edit::DocumentMut>()
                                .map_err(|e| crate::Error::McpProtocol(format!("Failed to parse config TOML: {}", e)))?;
                            config_doc.insert("model", toml_edit::value(model));
                            obj.insert("config".to_string(), json!(config_doc.to_string()));
                        }
                    }
                }
            }
            AppType::Gemini => {
                if let Some(obj) = provider.settings_config.as_object_mut() {
                    let env = obj.get_mut("env").and_then(|v| v.as_object_mut());
                    if let Some(env) = env {
                        if let Some(base_url) = args.get("baseUrl").and_then(|v| v.as_str()) {
                            env.insert("GOOGLE_GEMINI_BASE_URL".to_string(), json!(base_url));
                        }
                        if let Some(api_key) = args.get("apiKey").and_then(|v| v.as_str()) {
                            env.insert("GEMINI_API_KEY".to_string(), json!(api_key));
                        }
                    }
                }
            }
            AppType::OpenCode => {
                if let Some(obj) = provider.settings_config.as_object_mut() {
                    if let Some(options) = obj.get_mut("options").and_then(|v| v.as_object_mut()) {
                        if let Some(base_url) = args.get("baseUrl").and_then(|v| v.as_str()) {
                            options.insert("baseURL".to_string(), json!(base_url));
                        }
                        if let Some(api_key) = args.get("apiKey").and_then(|v| v.as_str()) {
                            options.insert("apiKey".to_string(), json!(api_key));
                        }
                    }
                }
            }
            AppType::OpenClaw => {
                if let Some(obj) = provider.settings_config.as_object_mut() {
                    if let Some(base_url) = args.get("baseUrl").and_then(|v| v.as_str()) {
                        obj.insert("baseUrl".to_string(), json!(base_url));
                    }
                    if let Some(api_key) = args.get("apiKey").and_then(|v| v.as_str()) {
                        obj.insert("apiKey".to_string(), json!(api_key));
                    }
                }
            }
        }

        ProviderService::update(&self.state, app_type, None, provider)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Updated provider {} for {}", provider_id, app);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "providerId": provider_id,
            "message": "Provider updated successfully"
        }))?)
    }

    fn tool_add_custom_endpoint(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'url' parameter".into()))?;

        // Validate URL
        self.validate_url(url)?;

        let app_type = self.parse_app_type(app)?;

        ProviderService::add_custom_endpoint(&self.state, app_type, provider_id, url.to_string())
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Added custom endpoint {} for provider {} {}", url, app, provider_id);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "providerId": provider_id,
            "url": url,
            "message": "Custom endpoint added successfully"
        }))?)
    }

    fn tool_remove_custom_endpoint(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'url' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        ProviderService::remove_custom_endpoint(&self.state, app_type, provider_id, url.to_string())
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Removed custom endpoint {} from provider {} {}", url, app, provider_id);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "providerId": provider_id,
            "url": url,
            "message": "Custom endpoint removed successfully"
        }))?)
    }

    fn tool_list_universal_providers(&self) -> crate::Result<String> {
        let providers = ProviderService::list_universal(&self.state)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let provider_list: Vec<Value> = providers
            .iter()
            .map(|(id, p)| {
                json!({
                    "id": id,
                    "name": p.name,
                    "providerType": p.provider_type,
                    "baseUrl": p.base_url,
                    "apps": p.apps,
                    "models": p.models,
                    "websiteUrl": p.website_url,
                    "notes": p.notes,
                    "icon": p.icon,
                    "iconColor": p.icon_color
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&json!({
            "providers": provider_list,
            "total": provider_list.len()
        }))?)
    }

    fn tool_add_universal_provider(&self, args: Value) -> crate::Result<String> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'name' parameter".into()))?;
        let provider_type = args
            .get("providerType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerType' parameter".into()))?;
        let base_url = args
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'baseUrl' parameter".into()))?;
        let api_key = args
            .get("apiKey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'apiKey' parameter".into()))?;

        // Validate inputs
        self.validate_provider_name(name)?;
        self.validate_provider_type(provider_type)?;
        self.validate_url(base_url)?;
        self.validate_api_key(api_key)?;

        let provider_id = uuid::Uuid::new_v4().to_string();

        let provider = UniversalProvider {
            id: provider_id.clone(),
            name: name.to_string(),
            provider_type: provider_type.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            apps: serde_json::from_value(args.get("apps").cloned().unwrap_or(json!({})))
                .unwrap_or_default(),
            models: serde_json::from_value(args.get("models").cloned().unwrap_or(json!({})))
                .unwrap_or_default(),
            website_url: None,
            notes: None,
            icon: None,
            icon_color: None,
            meta: None,
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            sort_index: None,
        };

        ProviderService::upsert_universal(&self.state, provider)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Added universal provider {}", provider_id);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "providerId": provider_id,
            "name": name,
            "message": "Universal provider added successfully"
        }))?)
    }

    fn tool_update_universal_provider(&self, args: Value) -> crate::Result<String> {
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        // Get existing provider
        let providers = ProviderService::list_universal(&self.state)
            .map_err(|e| crate::Error::Database(e.to_string()))?;
        let mut provider = providers
            .get(provider_id)
            .ok_or_else(|| crate::Error::ProviderNotFound(provider_id.to_string()))?
            .clone();

        // Update fields if provided
        if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
            provider.name = name.to_string();
        }
        if let Some(base_url) = args.get("baseUrl").and_then(|v| v.as_str()) {
            provider.base_url = base_url.to_string();
        }
        if let Some(api_key) = args.get("apiKey").and_then(|v| v.as_str()) {
            provider.api_key = api_key.to_string();
        }
        if let Some(apps) = args.get("apps") {
            provider.apps = serde_json::from_value(apps.clone()).unwrap_or_default();
        }
        if let Some(models) = args.get("models") {
            provider.models = serde_json::from_value(models.clone()).unwrap_or_default();
        }

        ProviderService::upsert_universal(&self.state, provider)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Updated universal provider {}", provider_id);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "providerId": provider_id,
            "message": "Universal provider updated successfully"
        }))?)
    }

    fn tool_delete_universal_provider(&self, args: Value) -> crate::Result<String> {
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        ProviderService::delete_universal(&self.state, provider_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Deleted universal provider {}", provider_id);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "providerId": provider_id,
            "message": "Universal provider deleted successfully"
        }))?)
    }

    fn tool_sync_universal_provider(&self, args: Value) -> crate::Result<String> {
        let provider_id = args
            .get("providerId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerId' parameter".into()))?;

        ProviderService::sync_universal_to_apps(&self.state, provider_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Synced universal provider {} to all enabled apps", provider_id);

        // Send log notification
        let _ = self.send_log_message(
            "info",
            &format!("Synced universal provider '{}' to all enabled apps", provider_id),
            Some("universal-provider")
        );

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "providerId": provider_id,
            "message": "Universal provider synced to all enabled apps"
        }))?)
    }

    fn tool_update_provider_order(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let provider_ids = args
            .get("providerIds")
            .and_then(|v| v.as_array())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'providerIds' parameter".into()))?;

        let app_type = self.parse_app_type(app)?;

        let ids: Vec<String> = provider_ids
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        // Validate provider IDs
        self.validate_provider_ids(&ids)?;

        let updates: Vec<ProviderSortUpdate> = ids
            .into_iter()
            .enumerate()
            .map(|(index, id)| {
                serde_json::from_value(json!({
                    "id": id,
                    "sortIndex": index
                }))
                .unwrap()
            })
            .collect();

        ProviderService::update_sort_order(&self.state, app_type, updates)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Updated provider order for {}", app);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "message": "Provider order updated successfully"
        }))?)
    }

    fn tool_send_log(&self, args: Value) -> crate::Result<String> {
        let level = args
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'level' parameter".into()))?;
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'message' parameter".into()))?;
        let logger = args.get("logger").and_then(|v| v.as_str());

        // Validate level
        let valid_levels = ["debug", "info", "warning", "error"];
        if !valid_levels.contains(&level) {
            return Err(crate::Error::McpProtocol(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                level, valid_levels
            )));
        }

        // Send the log notification
        self.send_log_message(level, message, logger)?;

        tracing::info!("Sent log message: [{}] {}", level, message);

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "level": level,
            "message": message,
            "logger": logger.unwrap_or("cc-switch-mcp"),
            "notificationSent": true
        }))?)
    }

    fn tool_list_mcp_servers(&self) -> crate::Result<String> {
        let servers = McpService::get_all_servers(&self.state)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let server_list: Vec<Value> = servers
            .iter()
            .map(|(id, server)| {
                json!({
                    "id": id,
                    "name": server.name,
                    "serverConfig": server.server,
                    "apps": server.apps,
                    "description": server.description,
                    "homepage": server.homepage,
                    "docs": server.docs,
                    "tags": server.tags
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&json!({
            "servers": server_list,
            "total": server_list.len()
        }))?)
    }

    fn tool_get_mcp_server(&self, args: Value) -> crate::Result<String> {
        let server_id = args
            .get("serverId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'serverId' parameter".into()))?;

        let servers = McpService::get_all_servers(&self.state)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let server = servers
            .get(server_id)
            .ok_or_else(|| crate::Error::ProviderNotFound(server_id.to_string()))?;

        Ok(serde_json::to_string_pretty(&json!({
            "id": server_id,
            "name": server.name,
            "serverConfig": server.server,
            "apps": server.apps,
            "description": server.description,
            "homepage": server.homepage,
            "docs": server.docs,
            "tags": server.tags
        }))?)
    }

    fn tool_add_mcp_server(&self, args: Value) -> crate::Result<String> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'name' parameter".into()))?;
        let server_config = args
            .get("serverConfig")
            .cloned()
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'serverConfig' parameter".into()))?;

        // Validate name
        if name.is_empty() {
            return Err(crate::Error::Validation("Server name cannot be empty".to_string()));
        }

        let server_id = uuid::Uuid::new_v4().to_string();

        // Parse apps configuration
        let apps: McpApps = serde_json::from_value(args.get("apps").cloned().unwrap_or(json!({})))
            .unwrap_or_default();

        let server = McpServerConfig {
            id: server_id.clone(),
            name: name.to_string(),
            server: server_config,
            apps,
            description: args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            homepage: args.get("homepage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            docs: args.get("docs").and_then(|v| v.as_str()).map(|s| s.to_string()),
            tags: args.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        };

        McpService::upsert_server(&self.state, server)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Added MCP server {}", server_id);

        // Send log notification
        let _ = self.send_log_message(
            "info",
            &format!("Added new MCP server '{}' ({})", name, server_id),
            Some("mcp-management")
        );

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "serverId": server_id,
            "name": name,
            "message": "MCP server added successfully"
        }))?)
    }

    fn tool_delete_mcp_server(&self, args: Value) -> crate::Result<String> {
        let server_id = args
            .get("serverId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'serverId' parameter".into()))?;

        let deleted = McpService::delete_server(&self.state, server_id)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        if deleted {
            tracing::info!("Deleted MCP server {}", server_id);

            // Send log notification
            let _ = self.send_log_message(
                "info",
                &format!("Deleted MCP server '{}'", server_id),
                Some("mcp-management")
            );

            Ok(serde_json::to_string_pretty(&json!({
                "success": true,
                "serverId": server_id,
                "message": "MCP server deleted successfully"
            }))?)
        } else {
            Ok(serde_json::to_string_pretty(&json!({
                "success": false,
                "serverId": server_id,
                "message": "MCP server not found"
            }))?)
        }
    }

    fn tool_toggle_mcp_server(&self, args: Value) -> crate::Result<String> {
        let server_id = args
            .get("serverId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'serverId' parameter".into()))?;
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;
        let enabled = args
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'enabled' parameter".into()))?;

        let app_type = match app {
            "claude" => AppType::Claude,
            "codex" => AppType::Codex,
            "gemini" => AppType::Gemini,
            "opencode" => AppType::OpenCode,
            "openclaw" => AppType::OpenClaw,
            _ => return Err(crate::Error::InvalidApp(app.to_string())),
        };

        McpService::toggle_app(&self.state, server_id, app_type, enabled)
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let status = if enabled { "enabled" } else { "disabled" };
        tracing::info!("{} MCP server {} for {}", status, server_id, app);

        // Send log notification
        let _ = self.send_log_message(
            "info",
            &format!("{} MCP server '{}' for {}", status, server_id, app),
            Some("mcp-management")
        );

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "serverId": server_id,
            "app": app,
            "enabled": enabled,
            "message": format!("MCP server {} for {}", status, app)
        }))?)
    }

    fn tool_import_mcp_from_app(&self, args: Value) -> crate::Result<String> {
        let app = args
            .get("app")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'app' parameter".into()))?;

        let app_type = match app {
            "claude" => AppType::Claude,
            "codex" => AppType::Codex,
            "gemini" => AppType::Gemini,
            "opencode" => AppType::OpenCode,
            "openclaw" => AppType::OpenClaw,
            _ => return Err(crate::Error::InvalidApp(app.to_string())),
        };

        let imported_count = match app_type {
            AppType::Claude => McpService::import_from_claude(&self.state),
            AppType::Codex => McpService::import_from_codex(&self.state),
            AppType::Gemini => McpService::import_from_gemini(&self.state),
            AppType::OpenCode => McpService::import_from_opencode(&self.state),
            AppType::OpenClaw => Ok(0),
        }
        .map_err(|e| crate::Error::Database(e.to_string()))?;

        tracing::info!("Imported {} MCP servers from {}", imported_count, app);

        // Send log notification
        let _ = self.send_log_message(
            "info",
            &format!("Imported {} MCP servers from {}", imported_count, app),
            Some("mcp-management")
        );

        Ok(serde_json::to_string_pretty(&json!({
            "success": true,
            "app": app,
            "importedCount": imported_count,
            "message": format!("Imported {} MCP servers from {}", imported_count, app)
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
                },
                {
                    "uri": "ccswitch://providers/openclaw",
                    "name": "OpenClaw Providers",
                    "description": "All OpenClaw providers from CC Switch database",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://universal-providers",
                    "name": "Universal Providers",
                    "description": "All universal providers shared across multiple CLI tools",
                    "mimeType": "application/json"
                },
                {
                    "uri": "ccswitch://mcp-servers",
                    "name": "MCP Servers",
                    "description": "All MCP servers configured in CC Switch",
                    "mimeType": "application/json"
                }
            ]
        }))
    }

    fn handle_resources_read(&self, params: Option<Value>) -> crate::Result<Value> {
        let params = params.ok_or_else(|| crate::Error::McpProtocol("Missing params".into()))?;
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'uri' parameter".into()))?;

        tracing::info!("Reading resource: {}", uri);

        // Handle universal providers resource
        if uri == "ccswitch://universal-providers" {
            let providers = ProviderService::list_universal(&self.state)
                .map_err(|e| crate::Error::Database(e.to_string()))?;

            let provider_list: Vec<Value> = providers
                .iter()
                .map(|(id, p)| {
                    json!({
                        "id": id,
                        "name": p.name,
                        "providerType": p.provider_type,
                        "baseUrl": p.base_url,
                        "apps": p.apps,
                        "models": p.models,
                        "websiteUrl": p.website_url,
                        "notes": p.notes,
                        "icon": p.icon,
                        "iconColor": p.icon_color
                    })
                })
                .collect();

            let result = json!({
                "providers": provider_list,
                "total": provider_list.len()
            });

            let content = serde_json::to_string_pretty(&result)?;

            return Ok(json!({
                "contents": [
                    {
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": content
                    }
                ]
            }));
        }

        // Handle MCP servers resource
        if uri == "ccswitch://mcp-servers" {
            let servers = McpService::get_all_servers(&self.state)
                .map_err(|e| crate::Error::Database(e.to_string()))?;

            let server_list: Vec<Value> = servers
                .iter()
                .map(|(id, server)| {
                    json!({
                        "id": id,
                        "name": server.name,
                        "serverConfig": server.server,
                        "apps": server.apps,
                        "description": server.description,
                        "homepage": server.homepage,
                        "docs": server.docs,
                        "tags": server.tags
                    })
                })
                .collect();

            let result = json!({
                "servers": server_list,
                "total": server_list.len()
            });

            let content = serde_json::to_string_pretty(&result)?;

            return Ok(json!({
                "contents": [
                    {
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": content
                    }
                ]
            }));
        }

        // Parse URI to extract app type
        let app_str = uri.strip_prefix("ccswitch://providers/")
            .ok_or_else(|| crate::Error::McpProtocol(format!("Invalid resource URI: {}", uri)))?;

        let app_type = self.parse_app_type(app_str)?;

        // Get providers for this app
        let providers = ProviderService::list(&self.state, app_type.clone())
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        let current = ProviderService::current(&self.state, app_type.clone())
            .map_err(|e| crate::Error::Database(e.to_string()))?;

        // Build provider list with current marker
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

        let result = json!({
            "app": app_str,
            "providers": provider_list,
            "currentProviderId": current,
            "total": provider_list.len()
        });

        let content = serde_json::to_string_pretty(&result)?;

        Ok(json!({
            "contents": [
                {
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": content
                }
            ]
        }))
    }

    fn handle_prompts_list(&self) -> crate::Result<Value> {
        Ok(json!({
            "prompts": [
                {
                    "name": "switch_provider_guide",
                    "description": "Guide for switching to a different provider",
                    "arguments": [
                        {
                            "name": "app",
                            "description": "The CLI tool (claude/codex/gemini/opencode/openclaw)",
                            "required": false
                        }
                    ]
                },
                {
                    "name": "troubleshoot_connection",
                    "description": "Troubleshoot provider connection issues",
                    "arguments": [
                        {
                            "name": "app",
                            "description": "The CLI tool with connection issues",
                            "required": false
                        }
                    ]
                },
                {
                    "name": "setup_new_provider",
                    "description": "Step-by-step guide to set up a new custom provider",
                    "arguments": [
                        {
                            "name": "providerType",
                            "description": "Type of provider (newapi/custom)",
                            "required": false
                        }
                    ]
                },
                {
                    "name": "universal_provider_guide",
                    "description": "Guide for creating and managing universal providers",
                    "arguments": []
                },
                {
                    "name": "best_practices",
                    "description": "Best practices for managing multiple providers",
                    "arguments": []
                }
            ]
        }))
    }

    fn handle_prompts_get(&self, params: Option<Value>) -> crate::Result<Value> {
        let params = params.ok_or_else(|| crate::Error::McpProtocol("Missing params".into()))?;
        let prompt_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::McpProtocol("Missing 'name' parameter".into()))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        tracing::info!("Getting prompt: {}", prompt_name);

        let (description, messages) = match prompt_name {
            "switch_provider_guide" => {
                let app_arg = args.get("app").and_then(|v| v.as_str());
                let app_specific = app_arg.map(|a| format!(" for {}", a)).unwrap_or_default();
                let description = format!("Guide for switching providers{}", app_specific);
                let messages = json!([
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("I want to switch to a different AI provider{}. Please help me:\n\n1. First, list all available providers to see my options\n2. Show me the current provider so I know what I'm switching from\n3. Help me switch to the provider I choose\n\nPlease guide me through this process step by step.", app_specific)
                        }
                    }
                ]);
                (description, messages)
            }
            "troubleshoot_connection" => {
                let app_arg = args.get("app").and_then(|v| v.as_str());
                let app_context = app_arg.map(|a| format!(" for {}", a)).unwrap_or_default();
                let description = format!("Troubleshoot connection issues{}", app_context);
                let messages = json!([
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("I'm having connection issues with my AI provider{}. Please help me troubleshoot by:\n\n1. Check the current provider configuration\n2. Verify the API endpoint settings\n3. Suggest common fixes for connection problems\n4. If needed, help me switch to a backup provider\n\nWhat information do you need from me to help diagnose the issue?", app_context)
                        }
                    }
                ]);
                (description, messages)
            }
            "setup_new_provider" => {
                let provider_type = args.get("providerType").and_then(|v| v.as_str()).unwrap_or("custom");
                let description = format!("Set up a new {} provider", provider_type);
                let type_guidance = if provider_type == "newapi" {
                    "NEWAPI is a standard format compatible with OpenAI API specification."
                } else {
                    "Custom providers allow you to configure any API endpoint with specific headers and settings."
                };
                let messages = json!([
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("I want to set up a new {} provider. {}\n\nPlease guide me through:\n\n1. What information I need to provide (API URL, key, model names)\n2. Which CLI tools I can use this provider with\n3. How to test the connection after setup\n4. How to make it the default provider\n\nLet's start!", provider_type, type_guidance)
                        }
                    }
                ]);
                (description, messages)
            }
            "universal_provider_guide" => {
                let description = "Universal Provider management guide".to_string();
                let messages = json!([
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Explain universal providers and how they work:\n\n1. What is a universal provider and why should I use it?\n2. Show me my current universal providers\n3. How do I create a new universal provider?\n4. How do I sync a universal provider to multiple CLI tools?\n5. What's the difference between universal and app-specific providers?\n\nPlease explain with examples."
                        }
                    }
                ]);
                (description, messages)
            }
            "best_practices" => {
                let description = "Best practices for provider management".to_string();
                let messages = json!([
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "What are the best practices for managing multiple AI providers with CC Switch?\n\nCover:\n1. How to organize providers for different use cases\n2. When to use universal vs app-specific providers\n3. Tips for switching providers efficiently\n4. How to backup and restore provider configurations\n5. Security recommendations for API keys\n\nGive me actionable advice."
                        }
                    }
                ]);
                (description, messages)
            }
            _ => {
                return Err(crate::Error::McpProtocol(format!("Unknown prompt: {}", prompt_name)));
            }
        };

        Ok(json!({
            "description": description,
            "messages": messages
        }))
    }
}
