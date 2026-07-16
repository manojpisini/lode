use serde_json::{json, Value};

use crate::error::McpError;
use crate::prompts;
use crate::resources;
use crate::tools::{dispatch_tool, register_all_tools, Tool, ToolInputValidator};

type McpResult<T> = Result<T, McpError>;

pub struct McpServer {
    tools: Vec<Tool>,
    validator: ToolInputValidator,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    pub fn new() -> Self {
        let tools = register_all_tools();
        let validator = ToolInputValidator::new(&tools);
        Self { tools, validator }
    }

    pub fn handle_request(&self, request: &Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let id = request.get("id").cloned().unwrap_or(json!(null));

        let result = match method {
            "initialize" => self.handle_initialize(request),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(request),
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resources_read(request),
            "prompts/list" => self.handle_prompts_list(),
            "prompts/get" => self.handle_prompts_get(request),
            _ => {
                return json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {method}")
                    },
                    "id": id,
                });
            }
        };

        match result {
            Ok(value) => json!({
                "jsonrpc": "2.0",
                "result": value,
                "id": id,
            }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": e.code(),
                    "message": e.to_string(),
                },
                "id": id,
            }),
        }
    }

    fn handle_initialize(&self, _request: &Value) -> McpResult<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": { "listChanged": false },
                "prompts": { "listChanged": false },
            },
            "serverInfo": {
                "name": "lode-mcp",
                "version": env!("CARGO_PKG_VERSION"),
            },
        }))
    }

    fn handle_tools_list(&self) -> McpResult<Value> {
        let tools: Vec<Value> = self
            .tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema,
                })
            })
            .collect();

        Ok(json!({"tools": tools}))
    }

    fn handle_tools_call(&self, request: &Value) -> McpResult<Value> {
        let name = request["params"]["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing tool name".to_string()))?;
        let args = request["params"]
            .get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        self.validator
            .validate(name, &args)
            .map_err(McpError::InvalidParams)?;

        match dispatch_tool(name, &args) {
            Ok(value) => {
                let text = serde_json::to_string_pretty(&value).unwrap_or_default();
                let redacted = lode_core::redact(&text);
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": redacted,
                    }],
                }))
            }
            Err(e) => {
                let msg = lode_core::redact(&e.to_string());
                Err(McpError::Internal(msg))
            }
        }
    }

    fn handle_resources_list(&self) -> McpResult<Value> {
        let resources = resources::list_resources();
        Ok(json!({"resources": resources}))
    }

    fn handle_resources_read(&self, request: &Value) -> McpResult<Value> {
        let uri = request["params"]["uri"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing resource URI".to_string()))?;

        let contents = resources::read_resource(uri)?;
        Ok(json!({"contents": contents}))
    }

    fn handle_prompts_list(&self) -> McpResult<Value> {
        let prompts = prompts::list_prompts();
        Ok(json!({"prompts": prompts}))
    }

    fn handle_prompts_get(&self, request: &Value) -> McpResult<Value> {
        let name = request["params"]["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing prompt name".to_string()))?;
        let args = request["params"]
            .get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        let prompt = prompts::get_prompt(name, &args)?;
        Ok(prompt)
    }
}

#[cfg(test)]
mod mcp_server_tests {
    use super::*;

    #[test]
    fn initialize_returns_server_info() {
        let server = McpServer::new();
        let response =
            server.handle_request(&json!({"jsonrpc":"2.0","id":1,"method":"initialize"}));
        assert_eq!(response["result"]["serverInfo"]["name"], "lode-mcp");
    }

    #[test]
    fn missing_tool_name_is_structured_error() {
        let server = McpServer::new();
        let response = server
            .handle_request(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{}}));
        assert!(response.get("error").is_some());
        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn unknown_tool_is_structured_error() {
        let server = McpServer::new();
        let response = server.handle_request(
            &json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"missing_tool"}}),
        );
        assert!(response.get("error").is_some());
    }
}
