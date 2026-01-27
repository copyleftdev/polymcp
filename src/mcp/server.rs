use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tracing::{debug, info, warn};

use super::error::McpError;
use super::jsonrpc::{
    JsonRpcErrorResponse, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    JsonRpcResultResponse, RequestId,
};
use super::transport::StdioTransport;
use super::types::{
    CallToolParams, CallToolResult, DynTool, Implementation, InitializeParams, InitializeResult,
    ListToolsResult, PROTOCOL_VERSION, ServerCapabilities, ToolsCapability,
};

pub struct McpServerBuilder {
    name: String,
    version: String,
    description: Option<String>,
    instructions: Option<String>,
    tools: HashMap<String, DynTool>,
}

impl McpServerBuilder {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            instructions: None,
            tools: HashMap::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    pub fn tool(mut self, tool: impl super::types::Tool + 'static) -> Self {
        let def = tool.definition();
        self.tools.insert(def.name.clone(), Arc::new(tool));
        self
    }

    pub fn build(self) -> McpServer {
        McpServer {
            server_info: Implementation {
                name: self.name,
                version: self.version,
                title: None,
                description: self.description,
            },
            instructions: self.instructions,
            tools: self.tools,
            initialized: false,
        }
    }
}

pub struct McpServer {
    server_info: Implementation,
    instructions: Option<String>,
    tools: HashMap<String, DynTool>,
    initialized: bool,
}

impl McpServer {
    pub fn builder(name: impl Into<String>, version: impl Into<String>) -> McpServerBuilder {
        McpServerBuilder::new(name, version)
    }

    pub async fn run(mut self) -> Result<(), McpError> {
        let (transport, mut receiver) = StdioTransport::spawn();

        info!(name = %self.server_info.name, version = %self.server_info.version, "server starting");

        while let Some(msg_result) = receiver.recv().await {
            match msg_result {
                Ok(msg) => {
                    if let Some(response) = self.handle_message(msg).await {
                        transport.send(&response).await?;
                    }
                }
                Err(err_json) => {
                    transport.send_raw(err_json).await?;
                }
            }
        }

        info!("server shutting down");
        Ok(())
    }

    async fn handle_message(&mut self, msg: JsonRpcMessage) -> Option<JsonRpcResponse> {
        match msg {
            JsonRpcMessage::Request(req) => Some(self.handle_request(req).await),
            JsonRpcMessage::Notification(notif) => {
                self.handle_notification(notif);
                None
            }
        }
    }

    async fn handle_request(&mut self, req: JsonRpcRequest) -> JsonRpcResponse {
        debug!(method = %req.method, "handling request");

        if req.jsonrpc != "2.0" {
            return JsonRpcErrorResponse::invalid_request(Some(req.id), "Invalid JSON-RPC version")
                .into();
        }

        match req.method.as_str() {
            "initialize" => self.handle_initialize(req.id, req.params),
            "ping" => self.handle_ping(req.id),
            "tools/list" => self.handle_tools_list(req.id),
            "tools/call" => self.handle_tools_call(req.id, req.params).await,
            _ => JsonRpcErrorResponse::method_not_found(req.id, &req.method).into(),
        }
    }

    fn handle_notification(&mut self, notif: JsonRpcNotification) {
        debug!(method = %notif.method, "handling notification");

        match notif.method.as_str() {
            "notifications/initialized" => {
                info!("client initialized");
                self.initialized = true;
            }
            "notifications/cancelled" => {
                debug!("received cancellation");
            }
            _ => {
                warn!(method = %notif.method, "unknown notification");
            }
        }
    }

    fn handle_initialize(&mut self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        let params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcErrorResponse::invalid_params(id, e.to_string()).into();
                }
            },
            None => {
                return JsonRpcErrorResponse::invalid_params(id, "Missing params").into();
            }
        };

        info!(
            client = %params.client_info.name,
            client_version = %params.client_info.version,
            protocol = %params.protocol_version,
            "client connecting"
        );

        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                logging: None,
            },
            server_info: self.server_info.clone(),
            instructions: self.instructions.clone(),
        };

        JsonRpcResultResponse::new(id, result).into()
    }

    fn handle_ping(&self, id: RequestId) -> JsonRpcResponse {
        JsonRpcResultResponse::new(id, serde_json::json!({})).into()
    }

    fn handle_tools_list(&self, id: RequestId) -> JsonRpcResponse {
        let tools: Vec<_> = self.tools.values().map(|t| t.definition()).collect();

        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };

        JsonRpcResultResponse::new(id, result).into()
    }

    async fn handle_tools_call(&self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        let params: CallToolParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcErrorResponse::invalid_params(id, e.to_string()).into();
                }
            },
            None => {
                return JsonRpcErrorResponse::invalid_params(id, "Missing params").into();
            }
        };

        let tool = match self.tools.get(&params.name) {
            Some(t) => t,
            None => {
                return JsonRpcErrorResponse::invalid_params(
                    id,
                    format!("Tool not found: {}", params.name),
                )
                .into();
            }
        };

        let arguments = params.arguments.unwrap_or_default();

        match tool.call(arguments).await {
            Ok(result) => JsonRpcResultResponse::new(id, result).into(),
            Err(e) => {
                let result = CallToolResult::error(e.to_string());
                JsonRpcResultResponse::new(id, result).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::{ToolCallFuture, ToolDefinition, ToolInputSchema};

    struct EchoTool;

    impl super::super::types::Tool for EchoTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition::new("echo")
                .with_description("Echo the input")
                .with_schema(ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: Some(HashMap::from([(
                        "message".to_string(),
                        serde_json::json!({"type": "string"}),
                    )])),
                    required: Some(vec!["message".to_string()]),
                })
        }

        fn call(&self, arguments: HashMap<String, Value>) -> ToolCallFuture {
            let message = arguments
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("no message")
                .to_string();

            Box::pin(async move { Ok(CallToolResult::text(message)) })
        }
    }

    fn build_test_server() -> McpServer {
        McpServer::builder("test-server", "0.1.0")
            .description("Test server")
            .tool(EchoTool)
            .build()
    }

    #[tokio::test]
    async fn handles_initialize_request() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Result(res) => {
                assert_eq!(res.result["protocolVersion"], "2025-11-25");
                assert_eq!(res.result["serverInfo"]["name"], "test-server");
                assert!(res.result["capabilities"]["tools"].is_object());
            }
            JsonRpcResponse::Error(e) => panic!("Expected result, got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn handles_ping_request() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::String("ping-1".to_string()),
            method: "ping".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Result(res) => {
                assert_eq!(res.id, RequestId::String("ping-1".to_string()));
            }
            JsonRpcResponse::Error(e) => panic!("Expected result, got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn handles_tools_list() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(2),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Result(res) => {
                let tools = res.result["tools"].as_array().unwrap();
                assert_eq!(tools.len(), 1);
                assert_eq!(tools[0]["name"], "echo");
                assert_eq!(tools[0]["description"], "Echo the input");
                assert_eq!(tools[0]["inputSchema"]["type"], "object");
            }
            JsonRpcResponse::Error(e) => panic!("Expected result, got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn handles_tools_call() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(3),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "echo",
                "arguments": {
                    "message": "Hello, world!"
                }
            })),
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Result(res) => {
                let content = &res.result["content"];
                assert_eq!(content[0]["type"], "text");
                assert_eq!(content[0]["text"], "Hello, world!");
            }
            JsonRpcResponse::Error(e) => panic!("Expected result, got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn returns_error_for_unknown_tool() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(4),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "nonexistent",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("Tool not found"));
            }
            JsonRpcResponse::Result(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn returns_error_for_unknown_method() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(5),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32601);
                assert!(e.error.message.contains("Method not found"));
            }
            JsonRpcResponse::Result(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn returns_error_for_invalid_jsonrpc_version() {
        let mut server = build_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            id: RequestId::Number(6),
            method: "ping".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        match response {
            JsonRpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32600);
            }
            JsonRpcResponse::Result(_) => panic!("Expected error"),
        }
    }
}
