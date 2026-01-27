use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: Option<String>,
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub data: T,
    pub next_url: Option<String>,
    pub count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub error: Option<String>,
    pub message: Option<String>,
}

impl ErrorResponse {
    pub fn message(&self) -> String {
        self.error
            .clone()
            .or_else(|| self.message.clone())
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsWrapper<T> {
    #[serde(default)]
    pub results: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleResult<T> {
    #[serde(flatten)]
    pub result: T,
}
