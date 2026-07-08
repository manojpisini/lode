use serde_json::{json, Value};

use crate::schema::{string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_template_list".to_string(),
            description: "List available project template paths".to_string(),
            input_schema: tool_input_schema(vec![]),
        },
        Tool {
            name: "lode_template_show".to_string(),
            description: "Show details of a specific template path".to_string(),
            input_schema: tool_input_schema(vec![(
                "template",
                "Template path to inspect",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_template_list(_args: &Value) -> Result<Value, String> {
    let templates = lode_core::template_paths();

    let items: Vec<Value> = templates
        .iter()
        .map(|name| {
            json!({
                "name": name,
            })
        })
        .collect();

    Ok(json!({
        "total": items.len(),
        "templates": items,
    }))
}

pub fn lode_template_show(args: &Value) -> Result<Value, String> {
    let template = args["template"]
        .as_str()
        .ok_or("Missing required argument: template")?;

    let templates = lode_core::template_paths();

    for name in templates {
        if *name == template {
            return Ok(json!({
                "name": name,
            }));
        }
    }

    Err(format!("Template not found: {template}"))
}
