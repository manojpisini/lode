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
pub mod time;
pub mod toolchain;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
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
    tools.extend(toolchain::tools());
    tools
}

pub fn dispatch_tool(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        // lifecycle
        "lode_init" => lifecycle::lode_init(args),
        "lode_add" => lifecycle::lode_add(args),
        "lode_sync" => lifecycle::lode_sync(args),
        "lode_info" => lifecycle::lode_info(args),

        // convention
        "lode_check" => convention::lode_check(args),
        "lode_fix" => convention::lode_fix(args),
        "lode_rename" => convention::lode_rename(args),

        // signature
        "lode_sign" => signature::lode_sign(args),
        "lode_stamp" => signature::lode_stamp(args),

        // env
        "lode_env_check" => env::lode_env_check(args),
        "lode_env_add" => env::lode_env_add(args),
        "lode_env_sync" => env::lode_env_sync(args),

        // git
        "lode_git_branch" => git::lode_git_branch(args),
        "lode_git_commit" => git::lode_git_commit(args),
        "lode_git_changelog" => git::lode_git_changelog(args),
        "lode_git_tag" => git::lode_git_tag(args),

        // health
        "lode_audit" => health::lode_audit(args),
        "lode_metrics" => health::lode_metrics(args),

        // pkg
        "lode_pkg_outdated" => pkg::lode_pkg_outdated(args),
        "lode_pkg_audit" => pkg::lode_pkg_audit(args),
        "lode_pkg_update" => pkg::lode_pkg_update(args),
        "lode_pkg_list" => pkg::lode_pkg_list(args),
        "lode_pkg_clean" => pkg::lode_pkg_clean(args),

        // secrets
        "lode_scan_secrets" => secrets::lode_scan_secrets(args),

        // release
        "lode_release" => release::lode_release(args),

        // time
        "lode_time_today" => time::lode_time_today(args),
        "lode_time_report" => time::lode_time_report(args),

        // registry
        "lode_projects_list" => registry::lode_projects_list(args),
        "lode_projects_health" => registry::lode_projects_health(args),

        // agent
        "lode_agent_sync" => agent::lode_agent_sync(args),
        "lode_agent_plan" => agent::lode_agent_plan(args),

        // config
        "lode_config_show" => config::lode_config_show(args),
        "lode_config_set" => config::lode_config_set(args),
        "lode_config_validate" => config::lode_config_validate(args),

        // template
        "lode_template_list" => template::lode_template_list(args),
        "lode_template_show" => template::lode_template_show(args),

        // toolchain
        "lode_toolchain_status" => toolchain::lode_toolchain_status(args),
        "lode_toolchain_pin" => toolchain::lode_toolchain_pin(args),

        _ => Err(format!("Unknown tool: {name}")),
    }
}
