use crate::tools::ToolRegistry;
use crate::types::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, ServerCapabilities,
    INTERNAL_ERROR, METHOD_NOT_FOUND,
};

/// MCP protocol handler.
pub struct ProtocolHandler {
    registry: ToolRegistry,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Handle a JSON-RPC request.
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "tools/list" => self.handle_tools_list(request),
            "tools/call" => self.handle_tools_call(request).await,
            "resources/list" => self.handle_resources_list(request),
            "prompts/list" => self.handle_prompts_list(request),
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: METHOD_NOT_FOUND,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let capabilities = ServerCapabilities::default();
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": capabilities,
                "serverInfo": {
                    "name": "agentic-workflow-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = self.registry.tool_definitions();
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    async fn handle_tools_call(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tool_name = request.params.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let arguments = request.params.get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match self.registry.call_tool(tool_name, arguments).await {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(result).unwrap_or_default()),
                error: None,
            },
            Err((code, message)) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code,
                    message,
                    data: None,
                }),
            },
        }
    }

    fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({ "resources": [] })),
            error: None,
        }
    }

    fn handle_prompts_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({ "prompts": [] })),
            error: None,
        }
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}
