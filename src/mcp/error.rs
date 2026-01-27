use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Method not found: {method}")]
    MethodNotFound { method: String },

    #[error("Invalid parameters: {message}")]
    InvalidParams { message: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },

    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transport closed")]
    TransportClosed,
}

impl McpError {
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            McpError::ParseError(_) => -32700,
            McpError::InvalidRequest { .. } => -32600,
            McpError::MethodNotFound { .. } => -32601,
            McpError::InvalidParams { .. } => -32602,
            McpError::InternalError { .. } => -32603,
            McpError::ToolNotFound { .. } => -32602,
            McpError::Io(_) => -32603,
            McpError::TransportClosed => -32603,
        }
    }
}
