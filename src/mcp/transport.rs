use std::io::{self, BufRead, Write};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error};

use super::error::McpError;
use super::jsonrpc::{JsonRpcMessage, JsonRpcResponse, parse_message};

pub struct StdioTransport {
    tx: mpsc::Sender<String>,
}

impl StdioTransport {
    pub fn spawn() -> (Self, mpsc::Receiver<Result<JsonRpcMessage, String>>) {
        let (input_tx, input_rx) = mpsc::channel::<Result<JsonRpcMessage, String>>(100);
        let (output_tx, mut output_rx) = mpsc::channel::<String>(100);

        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("stdin closed");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        debug!(message = %trimmed, "received");
                        match parse_message(trimmed) {
                            Ok(msg) => {
                                if input_tx.send(Ok(msg)).await.is_err() {
                                    break;
                                }
                            }
                            Err(err) => {
                                let err_json = serde_json::to_string(&err).unwrap_or_else(|_| {
                                    r#"{"error":"serialization failed"}"#.to_string()
                                });
                                if input_tx.send(Err(err_json)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "stdin read error");
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut stdout = tokio::io::stdout();
            while let Some(msg) = output_rx.recv().await {
                debug!(message = %msg, "sending");
                if let Err(e) = stdout.write_all(msg.as_bytes()).await {
                    error!(error = %e, "stdout write error");
                    break;
                }
                if let Err(e) = stdout.write_all(b"\n").await {
                    error!(error = %e, "stdout write error");
                    break;
                }
                if let Err(e) = stdout.flush().await {
                    error!(error = %e, "stdout flush error");
                    break;
                }
            }
        });

        (Self { tx: output_tx }, input_rx)
    }

    pub async fn send(&self, response: &JsonRpcResponse) -> Result<(), McpError> {
        let json = serde_json::to_string(response)?;
        self.tx
            .send(json)
            .await
            .map_err(|_| McpError::TransportClosed)
    }

    pub async fn send_raw(&self, json: String) -> Result<(), McpError> {
        self.tx
            .send(json)
            .await
            .map_err(|_| McpError::TransportClosed)
    }
}

pub struct SyncStdioTransport;

impl SyncStdioTransport {
    pub fn read_line() -> Result<Option<String>, McpError> {
        let stdin = io::stdin();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    Self::read_line()
                } else {
                    Ok(Some(trimmed))
                }
            }
            Err(e) => Err(McpError::Io(e)),
        }
    }

    pub fn write_line(json: &str) -> Result<(), McpError> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "{}", json)?;
        handle.flush()?;
        Ok(())
    }

    pub fn send(response: &JsonRpcResponse) -> Result<(), McpError> {
        let json = serde_json::to_string(response)?;
        Self::write_line(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::jsonrpc::{JsonRpcResultResponse, RequestId};

    #[test]
    fn sync_transport_serializes_response() {
        let response =
            JsonRpcResultResponse::new(RequestId::Number(1), serde_json::json!({"status": "ok"}));
        let json = serde_json::to_string(&JsonRpcResponse::Result(response)).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
    }
}
