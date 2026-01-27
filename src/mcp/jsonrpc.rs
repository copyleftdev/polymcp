use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSONRPC_VERSION: &str = "2.0";

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResultResponse {
    pub jsonrpc: &'static str,
    pub id: RequestId,
    pub result: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    pub error: JsonRpcError,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
}

impl JsonRpcResultResponse {
    pub fn new(id: RequestId, result: impl Serialize) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            id,
            result: serde_json::to_value(result).unwrap_or(Value::Null),
        }
    }
}

impl JsonRpcErrorResponse {
    pub fn new(id: Option<RequestId>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            id,
            error: JsonRpcError {
                code,
                message: message.into(),
                data: None,
            },
        }
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(None, PARSE_ERROR, message)
    }

    pub fn invalid_request(id: Option<RequestId>, message: impl Into<String>) -> Self {
        Self::new(id, INVALID_REQUEST, message)
    }

    pub fn method_not_found(id: RequestId, method: &str) -> Self {
        Self::new(
            Some(id),
            METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )
    }

    pub fn invalid_params(id: RequestId, message: impl Into<String>) -> Self {
        Self::new(Some(id), INVALID_PARAMS, message)
    }

    pub fn internal_error(id: RequestId, message: impl Into<String>) -> Self {
        Self::new(Some(id), INTERNAL_ERROR, message)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Result(JsonRpcResultResponse),
    Error(JsonRpcErrorResponse),
}

impl From<JsonRpcResultResponse> for JsonRpcResponse {
    fn from(value: JsonRpcResultResponse) -> Self {
        JsonRpcResponse::Result(value)
    }
}

impl From<JsonRpcErrorResponse> for JsonRpcResponse {
    fn from(value: JsonRpcErrorResponse) -> Self {
        JsonRpcResponse::Error(value)
    }
}

pub fn parse_message(line: &str) -> Result<JsonRpcMessage, JsonRpcErrorResponse> {
    serde_json::from_str(line).map_err(|e| JsonRpcErrorResponse::parse_error(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let msg = parse_message(json).unwrap();
        match msg {
            JsonRpcMessage::Request(req) => {
                assert_eq!(req.method, "ping");
                assert_eq!(req.id, RequestId::Number(1));
            }
            _ => panic!("Expected Request"),
        }
    }

    #[test]
    fn parses_string_id() {
        let json = r#"{"jsonrpc":"2.0","id":"abc-123","method":"test"}"#;
        let msg = parse_message(json).unwrap();
        match msg {
            JsonRpcMessage::Request(req) => {
                assert_eq!(req.id, RequestId::String("abc-123".to_string()));
            }
            _ => panic!("Expected Request"),
        }
    }

    #[test]
    fn parses_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, JsonRpcMessage::Notification(_)));
    }

    #[test]
    fn rejects_malformed_json() {
        let result = parse_message("{not valid json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error.code, PARSE_ERROR);
    }
}
