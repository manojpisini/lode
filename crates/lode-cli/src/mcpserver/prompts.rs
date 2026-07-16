use serde_json::{json, Value};

use super::error::McpError;

type PromptResult<T> = Result<T, McpError>;

pub fn list_prompts() -> Vec<Value> {
    vec![
        json!({"name": "lode-project-review", "description": "Review a LODE project: config, conventions, health, and suggestions", "arguments": [{"name": "path", "description": "Path to the LODE project root", "required": true}]}),
        json!({"name": "lode-scaffold-plan", "description": "Generate a scaffold plan for a new or existing project", "arguments": [{"name": "path", "description": "Path to the project root", "required": true}, {"name": "recipe", "description": "Component to add (optional, e.g. ci, docker, agent)", "required": false}]}),
        json!({"name": "lode-convention-check", "description": "Check project naming conventions and suggest fixes", "arguments": [{"name": "path", "description": "Path to the project root", "required": true}]}),
    ]
}

pub fn get_prompt(name: &str, args: &Value) -> PromptResult<Value> {
    match name {
        "lode-project-review" => project_review(args),
        "lode-scaffold-plan" => scaffold_plan(args),
        "lode-convention-check" => convention_check(args),
        _ => Err(McpError::NotFound(format!("Unknown prompt: {name}"))),
    }
}

fn load_config(root: &camino::Utf8Path) -> Result<lode_core::config::LodeConfig, String> {
    let p = root.join(".lode").join("project.toml");
    if !p.exists() {
        return Ok(lode_core::config::default_config());
    }
    toml::from_str(&std::fs::read_to_string(&p).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn project_review(args: &Value) -> PromptResult<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing required argument: path".to_string()))?;
    let validated = lode_core::ValidatedRoot::new(path)?;
    let root = camino::Utf8PathBuf::from_path_buf(validated.path().to_path_buf())
        .map_err(|_| McpError::Internal("non-utf8 path".to_string()))?;
    let project_toml = root.join(".lode").join("project.toml");
    let mut sections = Vec::new();
    if project_toml.exists() {
        sections.push(format!(
            "## Project Configuration\n{}",
            std::fs::read_to_string(&project_toml)?
        ));
        let config = load_config(&root).map_err(McpError::Internal)?;
        let audit = lode_core::audit_project(&root, &config)?;
        sections.push(format!("## Health\n- Score: {}/100\n- Convention violations: {}\n- Secret findings: {}\n- License: {}\n- README: {}", audit.score, audit.convention_violations, audit.secret_findings, audit.license_present, audit.readme_present));
    } else {
        sections.push("No LODE project found at the given path.".to_string());
    }
    Ok(
        json!({"description": format!("Project review for {path}"), "messages": [{"role": "user", "content": {"type": "text", "text": sections.join("\n\n")}}]}),
    )
}

fn scaffold_plan(args: &Value) -> PromptResult<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing required argument: path".to_string()))?;
    let recipe = args["recipe"].as_str().unwrap_or("none");
    let text = format!("Scaffold plan for {path}:\n\n1. Validate project structure\n2. Add component: {recipe}\n3. Sync scaffold state\n4. Run convention check\n5. Commit changes");
    Ok(
        json!({"description": format!("Scaffold plan for {path}"), "messages": [{"role": "user", "content": {"type": "text", "text": text}}]}),
    )
}

fn convention_check(args: &Value) -> PromptResult<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing required argument: path".to_string()))?;
    let validated = lode_core::ValidatedRoot::new(path)?;
    let root = camino::Utf8PathBuf::from_path_buf(validated.path().to_path_buf())
        .map_err(|_| McpError::Internal("non-utf8 path".to_string()))?;
    let project_toml = root.join(".lode").join("project.toml");
    let mut sections = Vec::new();
    if project_toml.exists() {
        let config = load_config(&root).map_err(McpError::Internal)?;
        let report = lode_core::check_path(&root, &config)?;
        sections.push(format!(
            "Convention rule: {}",
            config.convention.default_case
        ));
        sections.push(format!("Checked: {} items", report.checked));
        sections.push(format!("Violations found: {}", report.violations.len()));
        for v in &report.violations {
            sections.push(format!("  - {} -> {}", v.path, v.expected_name));
        }
    } else {
        sections.push("No LODE project found at the given path.".to_string());
    }
    Ok(
        json!({"description": format!("Convention check for {path}"), "messages": [{"role": "user", "content": {"type": "text", "text": sections.join("\n")}}]}),
    )
}
