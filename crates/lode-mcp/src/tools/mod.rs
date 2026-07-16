pub mod agent;
pub mod config;
pub mod convention;
pub mod env;
pub mod git;
pub mod health;
pub mod lifecycle;
pub mod pkg;
pub mod registry;
pub mod release;
pub mod secrets;
pub mod signature;
pub mod template;
pub mod template_bundle;
pub mod time;
pub mod toolchain;

use serde_json::Value;

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

        let schema = &tool.input_schema;
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|r| r.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
            .unwrap_or_default();

        for field in &required {
            if args.get(field).filter(|v| !v.is_null()).is_none() {
                return Err(format!("Missing required argument: {field}"));
            }
        }

        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
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

pub fn register_all_tools() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(lifecycle::tools());
    tools.extend(convention::tools());
    tools.extend(signature::tools());
    tools.extend(env::tools());
    tools.extend(git::tools());
    tools.extend(health::tools());
    tools.extend(pkg::tools());
    tools.extend(secrets::tools());
    tools.extend(release::tools());
    tools.extend(time::tools());
    tools.extend(registry::tools());
    tools.extend(agent::tools());
    tools.extend(config::tools());
    tools.extend(template::tools());
    tools.extend(template_bundle::tools());
    tools.extend(toolchain::tools());
    tools
}

type ToolResult<T> = Result<T, String>;

pub fn dispatch_tool(name: &str, args: &Value) -> ToolResult<Value> {
    match name {
        "lode_init" => lifecycle::lode_init(args),
        "lode_add" => lifecycle::lode_add(args),
        "lode_sync" => lifecycle::lode_sync(args),
        "lode_info" => lifecycle::lode_info(args),
        "lode_check" => convention::lode_check(args),
        "lode_fix" => convention::lode_fix(args),
        "lode_rename" => convention::lode_rename(args),
        "lode_sign" => signature::lode_sign(args),
        "lode_stamp" => signature::lode_stamp(args),
        "lode_env_check" => env::lode_env_check(args),
        "lode_env_add" => env::lode_env_add(args),
        "lode_env_sync" => env::lode_env_sync(args),
        "lode_git_branch" => git::lode_git_branch(args),
        "lode_git_commit" => git::lode_git_commit(args),
        "lode_git_changelog" => git::lode_git_changelog(args),
        "lode_git_tag" => git::lode_git_tag(args),
        "lode_audit" => health::lode_audit(args),
        "lode_metrics" => health::lode_metrics(args),
        "lode_pkg_outdated" => pkg::lode_pkg_outdated(args),
        "lode_pkg_audit" => pkg::lode_pkg_audit(args),
        "lode_pkg_update" => pkg::lode_pkg_update(args),
        "lode_pkg_list" => pkg::lode_pkg_list(args),
        "lode_pkg_clean" => pkg::lode_pkg_clean(args),
        "lode_scan_secrets" => secrets::lode_scan_secrets(args),
        "lode_release" => release::lode_release(args),
        "lode_time_today" => time::lode_time_today(args),
        "lode_time_report" => time::lode_time_report(args),
        "lode_projects_list" => registry::lode_projects_list(args),
        "lode_projects_health" => registry::lode_projects_health(args),
        "lode_agent_sync" => agent::lode_agent_sync(args),
        "lode_agent_plan" => agent::lode_agent_plan(args),
        "lode_config_show" => config::lode_config_show(args),
        "lode_config_set" => config::lode_config_set(args),
        "lode_config_validate" => config::lode_config_validate(args),
        "lode_template_list" => template::lode_template_list(args),
        "lode_template_show" => template::lode_template_show(args),
        "lode_template_bundle_list" => template_bundle::lode_template_bundle_list(args),
        "lode_template_bundle_show" => template_bundle::lode_template_bundle_show(args),
        "lode_template_bundle_validate" => template_bundle::lode_template_bundle_validate(args),
        "lode_template_bundle_preview" => template_bundle::lode_template_bundle_preview(args),
        "lode_template_bundle_apply" => template_bundle::lode_template_bundle_apply(args),
        "lode_template_bundle_capture" => template_bundle::lode_template_bundle_capture(args),
        "lode_toolchain_status" => toolchain::lode_toolchain_status(args),
        "lode_toolchain_pin" => toolchain::lode_toolchain_pin(args),
        _ => Err(format!("Unknown tool: {name}")),
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;

    #[test]
    fn register_all_tools_returns_expected_count() {
        let tools = register_all_tools();
        assert_eq!(tools.len(), 44);
    }

    #[test]
    fn all_tools_have_non_empty_names() {
        let tools = register_all_tools();
        for tool in &tools {
            assert!(!tool.name.is_empty(), "tool has empty name");
        }
    }

    #[test]
    fn all_tools_have_non_empty_descriptions() {
        let tools = register_all_tools();
        for tool in &tools {
            assert!(
                !tool.description.is_empty(),
                "{} has empty description",
                tool.name
            );
        }
    }

    #[test]
    fn all_tools_have_valid_input_schema() {
        let tools = register_all_tools();
        for tool in &tools {
            assert_eq!(
                tool.input_schema["type"], "object",
                "{} has invalid schema type",
                tool.name
            );
            assert!(
                tool.input_schema["properties"].is_object(),
                "{} has no properties",
                tool.name
            );
        }
    }

    #[test]
    fn no_duplicate_tool_names() {
        let tools = register_all_tools();
        let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), tools.len(), "duplicate tool names found");
    }

    #[test]
    fn dispatch_unknown_tool_returns_error() {
        let args = serde_json::json!({});
        let result = dispatch_tool("nonexistent_tool", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    #[test]
    fn dispatch_known_tool_with_missing_args_returns_error() {
        let args = serde_json::json!({});
        let result = dispatch_tool("lode_init", &args);
        assert!(result.is_err());
    }

    #[test]
    fn dispatch_lode_git_branch_works() {
        let args = serde_json::json!({
            "kind": "feat",
            "description": "add-new-feature"
        });
        let result = dispatch_tool("lode_git_branch", &args).unwrap();
        assert!(result["branch"].as_str().unwrap().contains("feat"));
    }

    #[test]
    fn dispatch_lode_config_show_returns_config() {
        let args = serde_json::json!({});
        let result = dispatch_tool("lode_config_show", &args).unwrap();
        assert!(result["config"].as_str().unwrap().contains("[identity]"));
    }
}
