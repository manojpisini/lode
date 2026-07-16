use serde_json::{json, Value};

pub fn tool_input_schema(properties: Vec<(&str, &str, Value)>) -> Value {
    let mut props = serde_json::Map::new();
    let mut required = Vec::new();
    for (name, description, schema) in properties {
        let mut prop = schema;
        prop["description"] = json!(description);
        props.insert(name.to_string(), prop);
        required.push(name.to_string());
    }
    json!({
        "type": "object",
        "properties": props,
        "required": required,
    })
}

pub fn string_schema() -> Value {
    json!({"type": "string"})
}
pub fn optional_string_schema() -> Value {
    json!({"type": "string"})
}
pub fn bool_schema() -> Value {
    json!({"type": "boolean"})
}
pub fn number_schema() -> Value {
    json!({"type": "number"})
}
pub fn array_schema(items: Value) -> Value {
    json!({"type": "array", "items": items})
}
pub fn object_schema() -> Value {
    json!({"type": "object"})
}
