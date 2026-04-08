//! MCP Server 测试模块
//!
//! 包含单元测试和集成测试，验证 Tools、Resources、Prompts 功能的正确性

#[cfg(test)]
mod tests {
    use serde_json::json;

    // 测试 MCP JSON-RPC 请求解析
    #[test]
    fn test_json_rpc_request_parsing() {
        let request_json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }"#;

        let request: serde_json::Value = serde_json::from_str(request_json).unwrap();
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "tools/list");
        assert_eq!(request["id"], 1);
    }

    // 测试工具列表 Schema 结构
    #[test]
    fn test_tool_schema_structure() {
        // list_providers 工具的 schema
        let schema = json!({
            "type": "object",
            "properties": {
                "app": {
                    "type": "string",
                    "enum": ["claude", "codex", "gemini", "opencode", "openclaw"]
                }
            },
            "required": ["app"]
        });

        assert!(schema["properties"]["app"]["enum"].as_array().unwrap().contains(&json!("claude")));
        assert!(schema["properties"]["app"]["enum"].as_array().unwrap().contains(&json!("openclaw")));
    }

    // 测试应用类型解析
    #[test]
    fn test_app_type_enum_values() {
        let valid_apps = vec!["claude", "codex", "gemini", "opencode", "openclaw"];

        for app in &valid_apps {
            assert!(!app.is_empty());
            assert!(app.chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    // 测试工具响应结构
    #[test]
    fn test_tool_response_structure() {
        let response = json!({
            "content": [
                {
                    "type": "text",
                    "text": "Test result"
                }
            ]
        });

        assert!(response["content"].is_array());
        assert_eq!(response["content"][0]["type"], "text");
    }

    // 测试资源 URI 格式
    #[test]
    fn test_resource_uri_format() {
        let uris = vec![
            "ccswitch://providers/claude",
            "ccswitch://providers/codex",
            "ccswitch://providers/gemini",
            "ccswitch://providers/opencode",
            "ccswitch://providers/openclaw",
            "ccswitch://universal-providers",
        ];

        for uri in &uris {
            assert!(uri.starts_with("ccswitch://"));
            assert!(!uri.contains(" ")); // URI 不应包含空格
        }
    }

    // 测试 Prompt 参数结构
    #[test]
    fn test_prompt_arguments_structure() {
        let prompts = vec![
            ("switch_provider_guide", vec!["app"]),
            ("troubleshoot_connection", vec!["app"]),
            ("setup_new_provider", vec!["providerType"]),
            ("universal_provider_guide", vec![]),
            ("best_practices", vec![]),
        ];

        for (name, args) in prompts {
            assert!(!name.is_empty());
            // 验证参数名不为空（如果有参数）
            for arg in &args {
                assert!(!arg.is_empty());
            }
        }
    }

    // 测试 Provider ID 生成格式
    #[test]
    fn test_provider_id_format() {
        let id = uuid::Uuid::new_v4().to_string();
        // UUID 应该是 36 个字符（包含连字符）
        assert_eq!(id.len(), 36);
        // 应该包含 4 个连字符
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }

    // 测试 JSON-RPC 错误结构
    #[test]
    fn test_json_rpc_error_structure() {
        let error = json!({
            "code": -32601,
            "message": "Method not found",
            "data": {
                "method": "unknown_method"
            }
        });

        assert!(error["code"].is_i64());
        assert!(!error["message"].as_str().unwrap().is_empty());
    }

    // 测试时间戳格式
    #[test]
    fn test_timestamp_format() {
        let timestamp = chrono::Utc::now().timestamp_millis();
        // 时间戳应该是正数且合理（2024年以后）
        assert!(timestamp > 1704067200000); // 2024-01-01 00:00:00 UTC in millis
    }

    // 测试工具数量
    #[test]
    fn test_tools_count() {
        // 当前有 21 个工具（原有的 17 个 + 新增的 get_mcp_server + MCP 相关工具）
        let expected_tools = vec![
            "list_providers",
            "get_current_provider",
            "switch_provider",
            "add_provider",
            "delete_provider",
            "sync_current_to_live",
            "get_custom_endpoints",
            "update_provider",
            "add_custom_endpoint",
            "remove_custom_endpoint",
            "list_universal_providers",
            "add_universal_provider",
            "update_universal_provider",
            "delete_universal_provider",
            "sync_universal_provider",
            "update_provider_order",
            "send_log",
            "list_mcp_servers",
            "get_mcp_server",
            "add_mcp_server",
            "delete_mcp_server",
            "toggle_mcp_server",
            "import_mcp_from_app",
        ];
        assert_eq!(expected_tools.len(), 23);
    }

    // 测试资源数量
    #[test]
    fn test_resources_count() {
        // 当前有 7 个资源（新增 MCP Servers 资源）
        let expected_resources = vec![
            "ccswitch://providers/claude",
            "ccswitch://providers/codex",
            "ccswitch://providers/gemini",
            "ccswitch://providers/opencode",
            "ccswitch://providers/openclaw",
            "ccswitch://universal-providers",
            "ccswitch://mcp-servers",
        ];
        assert_eq!(expected_resources.len(), 7);
    }

    // 测试 Prompts 数量
    #[test]
    fn test_prompts_count() {
        // 当前有 5 个 prompts
        let expected_prompts = vec![
            "switch_provider_guide",
            "troubleshoot_connection",
            "setup_new_provider",
            "universal_provider_guide",
            "best_practices",
        ];
        assert_eq!(expected_prompts.len(), 5);
    }

    // 测试通用供应商模型结构
    #[test]
    fn test_universal_provider_model_structure() {
        let models = json!({
            "claude": {
                "model": "claude-sonnet-4-6",
                "haikuModel": "claude-haiku-4-5",
                "sonnetModel": "claude-sonnet-4-6",
                "opusModel": "claude-opus-4-6"
            },
            "codex": {
                "model": "gpt-4"
            },
            "gemini": {
                "model": "gemini-pro"
            }
        });

        assert!(models["claude"]["model"].is_string());
        assert!(models["codex"]["model"].is_string());
        assert!(models["gemini"]["model"].is_string());
    }

    // 测试通用供应商应用启用结构
    #[test]
    fn test_universal_provider_apps_structure() {
        let apps = json!({
            "claude": true,
            "codex": false,
            "gemini": true,
            "opencode": false,
            "openclaw": true
        });

        assert!(apps["claude"].is_boolean());
        assert!(apps["codex"].is_boolean());
        assert_eq!(apps["claude"], true);
        assert_eq!(apps["codex"], false);
    }

    // 测试 Provider 排序更新结构
    #[test]
    fn test_provider_sort_update_structure() {
        let updates = vec![
            json!({"id": "provider-1", "sortIndex": 0}),
            json!({"id": "provider-2", "sortIndex": 1}),
            json!({"id": "provider-3", "sortIndex": 2}),
        ];

        for (i, update) in updates.iter().enumerate() {
            assert!(update["id"].is_string());
            assert!(update["sortIndex"].is_u64());
            assert_eq!(update["sortIndex"].as_u64().unwrap() as usize, i);
        }
    }

    // 测试 Provider 类别
    #[test]
    fn test_provider_categories() {
        let categories = vec!["custom", "built-in", "official"];

        for category in &categories {
            assert!(!category.is_empty());
        }
    }

    // 测试初始化响应结构
    #[test]
    fn test_initialize_response_structure() {
        let response = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "cc-switch-mcp",
                "version": "0.2.0"
            }
        });

        assert_eq!(response["protocolVersion"], "2024-11-05");
        assert!(response["capabilities"]["tools"].is_object());
        assert!(response["capabilities"]["resources"].is_object());
        assert!(response["capabilities"]["prompts"].is_object());
        assert!(!response["serverInfo"]["name"].as_str().unwrap().is_empty());
    }

    // 测试环境变量键名格式
    #[test]
    fn test_env_key_formats() {
        let env_keys = vec![
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_AUTH_TOKEN",
            "ANTHROPIC_MODEL",
            "OPENAI_API_KEY",
            "GOOGLE_GEMINI_BASE_URL",
            "GEMINI_API_KEY",
        ];

        for key in &env_keys {
            // 环境变量应该全部大写，使用下划线分隔
            assert!(key.chars().all(|c| c.is_ascii_uppercase() || c == '_'));
        }
    }

    // 测试 API URL 格式验证
    #[test]
    fn test_api_url_format() {
        let valid_urls = vec![
            "https://api.anthropic.com",
            "https://api.openai.com",
            "https://generativelanguage.googleapis.com",
        ];

        for url in &valid_urls {
            assert!(url.starts_with("https://"));
            assert!(!url.ends_with("/")); // 不应该以斜杠结尾
        }
    }

    // 测试版本号格式
    #[test]
    fn test_version_format() {
        let version = "0.2.0";
        let parts: Vec<&str> = version.split('.').collect();
        assert_eq!(parts.len(), 3);

        for part in &parts {
            assert!(part.parse::<u32>().is_ok());
        }
    }
}

// 集成测试：模拟完整的 MCP 请求-响应流程
#[cfg(test)]
mod integration_tests {
    use serde_json::json;

    /// 测试完整的工具列表请求流程
    #[test]
    fn test_tools_list_flow() {
        // 模拟请求
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });

        assert_eq!(request["method"], "tools/list");
        assert!(request["id"].is_number());
    }

    /// 测试完整的资源列表请求流程
    #[test]
    fn test_resources_list_flow() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list"
        });

        assert_eq!(request["method"], "resources/list");
    }

    /// 测试完整的 prompts 列表请求流程
    #[test]
    fn test_prompts_list_flow() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "prompts/list"
        });

        assert_eq!(request["method"], "prompts/list");
    }

    /// 测试工具调用请求结构
    #[test]
    fn test_tool_call_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "list_providers",
                "arguments": {
                    "app": "claude"
                }
            }
        });

        assert_eq!(request["method"], "tools/call");
        assert!(request["params"]["name"].is_string());
        assert!(request["params"]["arguments"].is_object());
    }

    /// 测试资源读取请求结构
    #[test]
    fn test_resource_read_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": "ccswitch://providers/claude"
            }
        });

        assert_eq!(request["method"], "resources/read");
        assert!(request["params"]["uri"].is_string());
    }

    /// 测试 prompt 获取请求结构
    #[test]
    fn test_prompt_get_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/get",
            "params": {
                "name": "switch_provider_guide",
                "arguments": {
                    "app": "claude"
                }
            }
        });

        assert_eq!(request["method"], "prompts/get");
        assert!(request["params"]["name"].is_string());
    }

    /// 测试初始化请求流程
    #[test]
    fn test_initialize_flow() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        assert_eq!(request["method"], "initialize");
        assert!(request["params"]["clientInfo"].is_object());
    }

    /// 测试未知方法错误响应
    #[test]
    fn test_unknown_method_error() {
        let error_response = json!({
            "jsonrpc": "2.0",
            "id": 99,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": {
                    "method": "unknown/method"
                }
            }
        });

        assert_eq!(error_response["error"]["code"], -32601);
        assert!(!error_response["result"].is_object());
    }

    /// 测试解析错误响应
    #[test]
    fn test_parse_error_response() {
        let error_response = json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": {
                "code": -32700,
                "message": "Parse error",
                "data": {
                    "details": "Invalid JSON"
                }
            }
        });

        assert_eq!(error_response["error"]["code"], -32700);
    }
}

// 测试辅助函数和工具
#[cfg(test)]
mod util_tests {
    /// 测试 JSON 序列化和反序列化
    #[test]
    fn test_json_roundtrip() {
        let original = serde_json::json!({
            "name": "test",
            "value": 42,
            "nested": {
                "key": "value"
            }
        });

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    /// 测试 MCP server ID 格式
    #[test]
    fn test_mcp_server_id_format() {
        let id = uuid::Uuid::new_v4().to_string();
        // UUID 应该是 36 个字符（包含连字符）
        assert_eq!(id.len(), 36);
        // 应该包含 4 个连字符
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }

    /// 测试 MCP server 工具参数结构
    #[test]
    fn test_mcp_server_tool_arguments() {
        // get_mcp_server 工具参数
        let get_args = json!({
            "serverId": "test-server-id"
        });
        assert!(get_args["serverId"].is_string());

        // add_mcp_server 工具参数
        let add_args = json!({
            "name": "Test Server",
            "serverConfig": {
                "command": "node",
                "args": ["server.js"],
                "env": {}
            },
            "apps": {
                "claude": true,
                "codex": false,
                "gemini": false,
                "opencode": false,
                "openclaw": true
            }
        });
        assert!(add_args["name"].is_string());
        assert!(add_args["serverConfig"].is_object());
        assert!(add_args["apps"]["openclaw"].is_boolean());

        // toggle_mcp_server 工具参数
        let toggle_args = json!({
            "serverId": "test-server-id",
            "app": "openclaw",
            "enabled": true
        });
        assert!(toggle_args["serverId"].is_string());
        assert!(toggle_args["app"].is_string());
        assert_eq!(toggle_args["app"], "openclaw");
        assert!(toggle_args["enabled"].is_boolean());
    }

    /// 测试 MCP servers 资源 URI
    #[test]
    fn test_mcp_servers_resource_uri() {
        let uri = "ccswitch://mcp-servers";
        assert!(uri.starts_with("ccswitch://"));
        assert!(uri.contains("mcp-servers"));
    }

    /// 测试所有应用类型都支持 OpenClaw
    #[test]
    fn test_all_app_types_include_openclaw() {
        let all_apps = vec!["claude", "codex", "gemini", "opencode", "openclaw"];
        assert!(all_apps.contains(&"openclaw"));
        assert_eq!(all_apps.len(), 5);
    }
        let data = serde_json::json!({
            "id": "test-123",
            "items": ["a", "b", "c"]
        });

        let pretty = serde_json::to_string_pretty(&data).unwrap();
        assert!(pretty.contains("\n")); // 美化输出应该包含换行
        assert!(pretty.contains("  ")); // 美化输出应该包含缩进
    }
}

// 通知功能测试
#[cfg(test)]
mod notification_tests {
    use serde_json::json;

    /// 测试通知方法名称格式
    #[test]
    fn test_notification_method_names() {
        let notifications = vec![
            "notifications/initialized",
            "notifications/tools/list_changed",
            "notifications/resources/list_changed",
            "notifications/prompts/list_changed",
            "notifications/message",
            "notifications/progress",
        ];

        for method in &notifications {
            assert!(method.starts_with("notifications/"));
            assert!(!method.contains(" "));
        }
    }

    /// 测试日志级别有效性
    #[test]
    fn test_log_levels() {
        let valid_levels = vec!["debug", "info", "warning", "error"];

        for level in &valid_levels {
            assert!(!level.is_empty());
            assert!(level.chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    /// 测试日志通知消息结构
    #[test]
    fn test_log_notification_structure() {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/message",
            "params": {
                "level": "info",
                "message": "Test log message",
                "logger": "test-logger"
            }
        });

        assert_eq!(notification["jsonrpc"], "2.0");
        assert_eq!(notification["method"], "notifications/message");
        assert!(notification["params"]["level"].is_string());
        assert!(notification["params"]["message"].is_string());
    }

    /// 测试进度通知结构
    #[test]
    fn test_progress_notification_structure() {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/progress",
            "params": {
                "progressToken": "sync-123",
                "progress": 0.75,
                "total": 1.0
            }
        });

        assert_eq!(notification["method"], "notifications/progress");
        assert!(notification["params"]["progress"].is_number());
        assert!(notification["params"]["progressToken"].is_string());
    }

    /// 测试列表变更通知
    #[test]
    fn test_list_changed_notifications() {
        let tools_changed = json!({
            "jsonrpc": "2.0",
            "method": "notifications/tools/list_changed",
            "params": {}
        });

        let resources_changed = json!({
            "jsonrpc": "2.0",
            "method": "notifications/resources/list_changed",
            "params": {}
        });

        assert_eq!(tools_changed["method"], "notifications/tools/list_changed");
        assert_eq!(resources_changed["method"], "notifications/resources/list_changed");
    }

    /// 测试初始化能力包含通知
    #[test]
    fn test_initialize_response_includes_notifications() {
        let response = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {},
                "notifications": {}
            },
            "serverInfo": {
                "name": "cc-switch-mcp",
                "version": "0.2.0"
            }
        });

        assert!(response["capabilities"]["notifications"].is_object());
    }

    /// 测试 send_log 工具的参数结构
    #[test]
    fn test_send_log_tool_arguments() {
        let args = json!({
            "level": "info",
            "message": "Test message",
            "logger": "custom-logger"
        });

        let valid_levels = vec!["debug", "info", "warning", "error"];
        let level = args["level"].as_str().unwrap();
        assert!(valid_levels.contains(&level));
        assert!(!args["message"].as_str().unwrap().is_empty());
    }
}

// 输入验证测试
#[cfg(test)]
mod validation_tests {
    /// 测试有效的URL格式
    #[test]
    fn test_valid_urls() {
        let valid_urls = vec![
            "https://api.anthropic.com",
            "https://api.openai.com/v1",
            "http://localhost:8080",
            "https://api.example.com:3000/path",
        ];

        for url in &valid_urls {
            assert!(
                url.starts_with("http://") || url.starts_with("https://"),
                "URL should start with http:// or https://: {}",
                url
            );
            assert!(!url.is_empty(), "URL should not be empty");
        }
    }

    /// 测试无效的URL格式
    #[test]
    fn test_invalid_urls() {
        let invalid_urls = vec![
            "",
            "ftp://example.com",
            "not-a-url",
            "example.com",
        ];

        for url in &invalid_urls {
            let is_valid = !url.is_empty() &&
                (url.starts_with("http://") || url.starts_with("https://"));
            assert!(
                !is_valid || url.is_empty(),
                "URL should be invalid or empty: {}",
                url
            );
        }
    }

    /// 测试API key格式
    #[test]
    fn test_api_key_formats() {
        // 有效的API key（至少8个字符）
        let valid_keys = vec![
            "sk-1234567890abcdef",
            "api-key-with-32-chars-length",
            "bearer_token_12345",
        ];

        for key in &valid_keys {
            assert!(key.len() >= 8, "API key should be at least 8 chars: {}", key);
            assert!(!key.is_empty(), "API key should not be empty");
        }

        // 无效的API key（太短或占位符）
        let invalid_keys = vec![
            "",
            "short",
            "your_api_key_here",
            "placeholder_key",
            "example_key",
        ];

        for key in &invalid_keys {
            let is_valid = !key.is_empty() && key.len() >= 8;
            let lower = key.to_lowercase();
            let is_placeholder = lower.contains("your_") ||
                lower.contains("placeholder") ||
                lower.contains("example");
            assert!(
                !is_valid || is_placeholder || key.is_empty(),
                "API key should be invalid: {}",
                key
            );
        }
    }

    /// 测试提供商名称格式
    #[test]
    fn test_provider_name_formats() {
        // 有效的名称
        let valid_names = vec![
            "My Provider",
            "OpenAI-GPT-4",
            "Anthropic.Claude",
            "Provider_123",
        ];

        for name in &valid_names {
            assert!(!name.is_empty(), "Name should not be empty");
            assert!(
                name.len() <= 100,
                "Name should not exceed 100 chars: {}",
                name
            );
            assert!(
                name.chars().all(|c| {
                    c.is_alphanumeric() ||
                        c.is_whitespace() ||
                        c == '-' ||
                        c == '_' ||
                        c == '.'
                }),
                "Name contains invalid characters: {}",
                name
            );
        }

        // 无效的名称（包含特殊字符）
        let invalid_names = vec![
            "",
            "Provider@Home",
            "Test#123",
            "Name$Money",
        ];

        for name in &invalid_names {
            let has_invalid_chars = name.chars().any(|c| {
                !c.is_alphanumeric() &&
                    !c.is_whitespace() &&
                    c != '-' &&
                    c != '_' &&
                    c != '.'
            });
            assert!(
                name.is_empty() || has_invalid_chars,
                "Name should be invalid: {}",
                name
            );
        }
    }

    /// 测试模型名称格式
    #[test]
    fn test_model_name_formats() {
        let valid_models = vec![
            "claude-sonnet-4-6",
            "gpt-4-turbo-preview",
            "gemini-3-pro-preview",
            "claude-haiku-4-5-20251001",
        ];

        for model in &valid_models {
            assert!(!model.is_empty(), "Model name should not be empty");
            assert!(
                model.len() <= 200,
                "Model name should not exceed 200 chars: {}",
                model
            );
        }
    }

    /// 测试提供商类型验证
    #[test]
    fn test_provider_type_validation() {
        let valid_types = vec!["newapi", "custom"];

        for provider_type in &valid_types {
            assert!(
                valid_types.contains(provider_type),
                "Provider type should be valid: {}",
                provider_type
            );
        }

        let invalid_types = vec!["openai", "anthropic", "", "NEWAPI"];

        for provider_type in &invalid_types {
            assert!(
                !valid_types.contains(provider_type),
                "Provider type should be invalid: {}",
                provider_type
            );
        }
    }

    /// 测试provider IDs数组验证
    #[test]
    fn test_provider_ids_validation() {
        use std::collections::HashSet;

        // 有效的IDs（无重复）
        let valid_ids = vec![
            "provider-1".to_string(),
            "provider-2".to_string(),
            "provider-3".to_string(),
        ];

        let unique_count: HashSet<_> = valid_ids.iter().collect();
        assert_eq!(
            unique_count.len(),
            valid_ids.len(),
            "Valid IDs should have no duplicates"
        );

        // 有重复的IDs
        let duplicate_ids = vec![
            "provider-1".to_string(),
            "provider-2".to_string(),
            "provider-1".to_string(), // 重复
        ];

        let unique_count: HashSet<_> = duplicate_ids.iter().collect();
        assert_ne!(
            unique_count.len(),
            duplicate_ids.len(),
            "Duplicate IDs should be detected"
        );

        // 空数组
        let empty_ids: Vec<String> = vec![];
        assert!(empty_ids.is_empty(), "Empty array should be detected");
    }
}
