//! MCP 2025-11-25 stdio protocol, a bounded static catalog of admitted tools.
use crate::{
    Error,
    audit::Receipt,
    runtime::{Outcome, PreparedTool},
    storage::append_audit,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

pub const MAX_TOOLS: usize = 16;
const PAGE_SIZE: usize = 8;

pub const PROTOCOL_VERSION: &str = "2025-11-25";
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

#[derive(Default)]
enum Phase {
    #[default]
    New,
    Negotiated,
    Ready,
}

pub struct Server {
    tools: BTreeMap<String, PreparedTool>,
    phase: Phase,
    audit: PathBuf,
}

fn error(id: Value, code: i32, message: &str) -> Value {
    let mut response = json!({"jsonrpc":"2.0", "error":{"code":code,"message":message}});
    // MCP error responses omit the ID when no valid request ID is available.
    if id.is_string() || id.is_i64() || id.is_u64() {
        response["id"] = id;
    }
    response
}

fn tool_error(message: &str) -> Value {
    json!({"content":[{"type":"text","text":message}],"isError":true})
}

impl Server {
    pub fn new(tool: PreparedTool, audit: PathBuf) -> Self {
        Self {
            tools: [(tool.manifest().name.clone(), tool)].into(),
            phase: Phase::New,
            audit,
        }
    }

    /// Admit a static catalog with unique signed names; serving never changes it.
    pub fn with_tools(tools: Vec<PreparedTool>, audit: PathBuf) -> Result<Self, Error> {
        if tools.is_empty() || tools.len() > MAX_TOOLS {
            return Err(Error::Format);
        }
        let count = tools.len();
        let tools: BTreeMap<_, _> = tools
            .into_iter()
            .map(|tool| (tool.manifest().name.clone(), tool))
            .collect();
        if tools.len() != count {
            return Err(Error::Format);
        }
        Ok(Self {
            tools,
            phase: Phase::New,
            audit,
        })
    }

    /// Notifications never execute a tool and never receive a response.
    pub async fn handle(&mut self, bytes: &[u8]) -> Option<Value> {
        if bytes.len() > MAX_FRAME_BYTES {
            return Some(error(Value::Null, -32600, "Request too large"));
        }
        let request: Value = match serde_json::from_slice(bytes) {
            Ok(value) => value,
            Err(_) => return Some(error(Value::Null, -32700, "Parse error")),
        };
        let object = match request.as_object() {
            Some(object) => object,
            None => return Some(error(Value::Null, -32600, "Invalid request")),
        };
        let id = object.get("id").cloned();
        let method = object.get("method").and_then(Value::as_str);
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
            || method.is_none()
            || id
                .as_ref()
                .is_some_and(|id| !(id.is_string() || id.is_i64() || id.is_u64()))
        {
            return Some(error(id.unwrap_or(Value::Null), -32600, "Invalid request"));
        }
        let method = method.unwrap();
        let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
        if id.is_none() {
            if method == "notifications/initialized"
                && matches!(self.phase, Phase::Negotiated)
                && params.is_object()
            {
                self.phase = Phase::Ready;
            }
            return None;
        }
        let id = id.unwrap();
        if !params.is_object() {
            return Some(error(id, -32602, "Invalid params"));
        }
        let result = match method {
            "ping" => json!({}),
            "initialize" => {
                if !matches!(self.phase, Phase::New) {
                    return Some(error(id, -32600, "Already initialized"));
                }
                if params
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .is_none()
                    || !params.get("capabilities").is_some_and(Value::is_object)
                    || !params.get("clientInfo").is_some_and(|v| {
                        v.get("name").is_some_and(Value::is_string)
                            && v.get("version").is_some_and(Value::is_string)
                    })
                {
                    return Some(error(id, -32602, "Invalid initialize params"));
                }
                self.phase = Phase::Negotiated;
                json!({"protocolVersion":PROTOCOL_VERSION,"capabilities":{"tools":{"listChanged":false}},
                    "serverInfo":{"name":"toolcapsule","version":env!("CARGO_PKG_VERSION")}})
            }
            _ if !matches!(self.phase, Phase::Ready) => {
                return Some(error(id, -32000, "Server not initialized"));
            }
            "tools/list" => {
                let offset = match params.get("cursor") {
                    None => 0,
                    Some(Value::String(cursor))
                        if cursor == "tools:8" && self.tools.len() > PAGE_SIZE =>
                    {
                        PAGE_SIZE
                    }
                    _ => return Some(error(id, -32602, "Invalid cursor")),
                };
                let descriptors: Vec<_> = self.tools.values().skip(offset).take(PAGE_SIZE).map(|tool| {
                    json!({"name":tool.manifest().name,"description":tool.manifest().description,
                    "inputSchema":{"type":"object","properties":{"input":{"type":"string"},"seed":{"type":"integer","minimum":0,"maximum":9007199254740991_u64}},
                    "required":["input"],"additionalProperties":false},
                    "annotations":{"readOnlyHint":true,"destructiveHint":false,"openWorldHint":false}})
                }).collect();
                let mut page = json!({"tools":descriptors});
                if offset + PAGE_SIZE < self.tools.len() {
                    page["nextCursor"] = json!("tools:8");
                }
                page
            }
            "tools/call" => {
                let Some(tool) = params
                    .get("name")
                    .and_then(Value::as_str)
                    .and_then(|name| self.tools.get(name))
                else {
                    return Some(error(id, -32602, "Unknown tool"));
                };
                self.call(tool, params.get("arguments").unwrap_or(&Value::Null))
                    .await
            }
            _ => return Some(error(id, -32601, "Method not found")),
        };
        Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
    }

    async fn call(&self, tool: &PreparedTool, args: &Value) -> Value {
        let Some(object) = args.as_object() else {
            return tool_error("Expected tool arguments");
        };
        if object.keys().any(|key| key != "input" && key != "seed") {
            return tool_error("Unknown tool argument");
        }
        let Some(input) = object.get("input").and_then(Value::as_str) else {
            return tool_error("input must be a string");
        };
        let seed = match object.get("seed") {
            None => 0,
            Some(value) => match value.as_u64().filter(|seed| *seed <= 9_007_199_254_740_991) {
                Some(seed) => seed,
                None => return tool_error("seed must be a nonnegative safe integer"),
            },
        };
        let result = match tool.run(input.as_bytes(), seed).await {
            Ok(result) => result,
            Err(error) => return tool_error(&error.to_string()),
        };
        let receipt = Receipt::new(tool, input.as_bytes(), seed, &result);
        if append_audit(&self.audit, receipt).is_err() {
            return tool_error("Audit persistence failed");
        }
        let text = match String::from_utf8(result.stdout) {
            Ok(text) => text,
            Err(_) => return tool_error("Tool output is not UTF-8"),
        };
        if result.outcome != Outcome::Success {
            return tool_error(&format!("Tool execution failed: {:?}", result.outcome));
        }
        json!({"content":[{"type":"text","text":text}],"isError":false})
    }
}
