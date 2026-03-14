use std::io::{self, BufRead, Write};

use crate::protocol::ProtocolHandler;
use crate::types::JsonRpcRequest;

/// Stdio transport for MCP protocol.
pub struct StdioTransport {
    handler: ProtocolHandler,
}

impl StdioTransport {
    pub fn new(handler: ProtocolHandler) -> Self {
        Self { handler }
    }

    /// Run the stdio transport loop.
    pub async fn run(&self) -> io::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();

        let reader = stdin.lock();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    let error_response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        }
                    });
                    let mut out = stdout.lock();
                    writeln!(out, "{}", error_response)?;
                    out.flush()?;
                    continue;
                }
            };

            let response = self.handler.handle_request(request).await;
            let response_json = serde_json::to_string(&response)
                .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Serialization failed"}}"#.to_string());

            let mut out = stdout.lock();
            writeln!(out, "{}", response_json)?;
            out.flush()?;
        }

        Ok(())
    }
}
