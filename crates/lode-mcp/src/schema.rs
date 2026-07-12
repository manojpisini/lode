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

#[cfg(test)]
mod schema_tests {
    use super::*;

    #[test]
    fn tool_input_schema_with_no_properties() {
        let schema = tool_input_schema(vec![]);
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].as_object().unwrap().is_empty());
        assert!(schema["required"].as_array().unwrap().is_empty());
    }

    #[test]
    fn tool_input_schema_with_properties() {
        let schema = tool_input_schema(vec![
            ("name", "The name", string_schema()),
            ("count", "The count", number_schema()),
        ]);
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"]["description"]
            .as_str()
            .unwrap()
            .contains("name"));
        assert!(schema["properties"]["count"]["description"]
            .as_str()
            .unwrap()
            .contains("count"));
        assert_eq!(schema["required"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn string_schema_has_correct_type() {
        assert_eq!(string_schema()["type"], "string");
    }

    #[test]
    fn optional_string_schema_has_correct_type() {
        assert_eq!(optional_string_schema()["type"], "string");
    }

    #[test]
    fn bool_schema_has_correct_type() {
        assert_eq!(bool_schema()["type"], "boolean");
    }

    #[test]
    fn number_schema_has_correct_type() {
        assert_eq!(number_schema()["type"], "number");
    }

    #[test]
    fn array_schema_has_items() {
        let schema = array_schema(string_schema());
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
    }

    #[test]
    fn object_schema_has_correct_type() {
        assert_eq!(object_schema()["type"], "object");
    }
}
