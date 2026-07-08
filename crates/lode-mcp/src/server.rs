use serde_json::{json, Value};

use crate::prompts;
use crate::resources;
use crate::tools::{dispatch_tool, register_all_tools};

pub struct McpServer {
    tools: Vec<crate::tools::Tool>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: register_all_tools(),
        }
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
                    "code": -32000,
                    "message": e,
                },
                "id": id,
            }),
        }
    }

    fn handle_initialize(&self, _request: &Value) -> Result<Value, String> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": { "listChanged": false },
                "prompts": { "listChanged": false },
            },
            "serverInfo": {
                "name": "lode-mcp",
                "version": "0.1.0",
            },
        }))
    }

    fn handle_tools_list(&self) -> Result<Value, String> {
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

    fn handle_tools_call(&self, request: &Value) -> Result<Value, String> {
        let name = request["params"]["name"]
            .as_str()
            .ok_or("Missing tool name")?;
        let args = request["params"]
            .get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        match dispatch_tool(name, &args) {
            Ok(value) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&value).unwrap_or_default(),
                }],
            })),
            Err(e) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {e}"),
                }],
                "isError": true,
            })),
        }
    }

    fn handle_resources_list(&self) -> Result<Value, String> {
        let resources = resources::list_resources();
        Ok(json!({"resources": resources}))
    }

    fn handle_resources_read(&self, request: &Value) -> Result<Value, String> {
        let uri = request["params"]["uri"]
            .as_str()
            .ok_or("Missing resource URI")?;

        let contents = resources::read_resource(uri)?;
        Ok(json!({"contents": contents}))
    }

    fn handle_prompts_list(&self) -> Result<Value, String> {
        let prompts = prompts::list_prompts();
        Ok(json!({"prompts": prompts}))
    }

    fn handle_prompts_get(&self, request: &Value) -> Result<Value, String> {
        let name = request["params"]["name"]
            .as_str()
            .ok_or("Missing prompt name")?;
        let args = request["params"]
            .get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        let prompt = prompts::get_prompt(name, &args)?;
        Ok(prompt)
    }
}
