pub mod error;
pub mod prompts;
pub mod resources;
pub mod schema;
pub mod tools;

use serde_json::{json, Value};

use self::error::McpError;
use self::tools::register_all_tools;

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct ToolInputValidator {
    tools: Vec<Tool>,
}

impl ToolInputValidator {
    pub fn new(tools: &[Tool]) -> Self {
        Self {
            tools: tools.to_vec(),
        }
    }

    pub fn validate(&self, name: &str, args: &Value) -> Result<(), String> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| format!("Unknown tool: {name}"))?;
        let required = tool
            .input_schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|r| r.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
            .unwrap_or_default();
        for field in &required {
            if args.get(field).filter(|v| !v.is_null()).is_none() {
                return Err(format!("Missing required argument: {field}"));
            }
        }
        if let Some(properties) = tool
            .input_schema
            .get("properties")
            .and_then(|p| p.as_object())
        {
            for (key, value) in args.as_object().unwrap_or(&serde_json::Map::new()) {
                if !properties.contains_key(key) {
                    return Err(format!("Unknown argument: {key}"));
                }
                if let Some(prop_schema) = properties.get(key) {
                    if let Some(prop_type) = prop_schema.get("type").and_then(|t| t.as_str()) {
                        let valid = match prop_type {
                            "string" => value.is_string(),
                            "integer" | "number" => value.is_number(),
                            "boolean" => value.is_boolean(),
                            "array" => value.is_array(),
                            "object" => value.is_object(),
                            _ => true,
                        };
                        if !valid {
                            return Err(format!("Argument '{key}' expected type '{prop_type}'"));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

type McpResult<T> = Result<T, McpError>;

pub fn handle_request(request: &Value) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let id = request.get("id").cloned().unwrap_or(json!(null));
    let result = match method {
        "initialize" => Ok(
            json!({"protocolVersion": "2024-11-05", "capabilities": {"tools": {}, "resources": {"listChanged": false}, "prompts": {"listChanged": false}}, "serverInfo": {"name": "lode", "version": env!("CARGO_PKG_VERSION")}}),
        ),
        "tools/list" => tool_list(),
        "tools/call" => tool_call(request),
        "resources/list" => Ok(json!({"resources": resources::list_resources()})),
        "resources/read" => resource_read(request),
        "prompts/list" => Ok(json!({"prompts": prompts::list_prompts()})),
        "prompts/get" => prompt_get(request),
        _ => Err(McpError::NotFound(format!("Method not found: {method}"))),
    };
    match result {
        Ok(value) => json!({"jsonrpc": "2.0", "result": value, "id": id}),
        Err(e) => {
            json!({"jsonrpc": "2.0", "error": {"code": e.code(), "message": e.to_string()}, "id": id})
        }
    }
}

fn tool_list() -> McpResult<Value> {
    let validator = ToolInputValidator::new(&register_all_tools());
    let tools: Vec<Value> = validator.tools.iter().map(|t| json!({"name": t.name, "description": t.description, "inputSchema": t.input_schema})).collect();
    Ok(json!({"tools": tools}))
}

fn tool_call(request: &Value) -> McpResult<Value> {
    let name = request["params"]["name"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing tool name".to_string()))?;
    let args = request["params"]
        .get("arguments")
        .cloned()
        .unwrap_or(json!({}));
    let validator = ToolInputValidator::new(&register_all_tools());
    validator
        .validate(name, &args)
        .map_err(McpError::InvalidParams)?;
    match self::tools::dispatch_tool(name, &args) {
        Ok(value) => {
            let text = serde_json::to_string_pretty(&value).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": lode_core::redact(&text)}]}))
        }
        Err(e) => Err(McpError::Internal(lode_core::redact(&e))),
    }
}

fn resource_read(request: &Value) -> McpResult<Value> {
    let uri = request["params"]["uri"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing resource URI".to_string()))?;
    let contents = resources::read_resource(uri)?;
    Ok(json!({"contents": contents}))
}

fn prompt_get(request: &Value) -> McpResult<Value> {
    let name = request["params"]["name"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing prompt name".to_string()))?;
    let args = request["params"]
        .get("arguments")
        .cloned()
        .unwrap_or(json!({}));
    prompts::get_prompt(name, &args)
}
