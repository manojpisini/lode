use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_init".to_string(),
            description: "Initialise a new LODE project in the given directory".to_string(),
            input_schema: tool_input_schema(vec![
                ("name", "Project name", string_schema()),
                (
                    "path",
                    "Base directory to create project in",
                    string_schema(),
                ),
                ("author", "Author name", optional_string_schema()),
                ("org", "Organisation / namespace", optional_string_schema()),
                (
                    "profile",
                    "Scaffold profile (e.g. core/app, systems/rust-cli)",
                    optional_string_schema(),
                ),
                (
                    "components",
                    "Comma-separated list of components to include",
                    optional_string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_add".to_string(),
            description: "Add a component to an existing LODE project".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("name", "Project name", string_schema()),
                (
                    "component",
                    "Component name (e.g. ci, docker, agent)",
                    string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_sync".to_string(),
            description: "Synchronise scaffold state with the filesystem".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_info".to_string(),
            description: "Show project info from the LODE manifest".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

fn load_config(root: &camino::Utf8Path) -> Result<lode_core::config::LodeConfig, String> {
    let project_toml = root.join(".lode").join("project.toml");
    if !project_toml.exists() {
        return Ok(lode_core::config::default_config());
    }
    let raw = std::fs::read_to_string(&project_toml).map_err(|e| e.to_string())?;
    let config: lode_core::config::LodeConfig = toml::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(config)
}

pub fn lode_init(args: &Value) -> Result<Value, String> {
    let name = args["name"]
        .as_str()
        .ok_or("Missing required argument: name")?;
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let author = args["author"].as_str().unwrap_or("");
    let org = args["org"].as_str().unwrap_or("");
    let profile = args["profile"].as_str();
    let components = args["components"]
        .as_str()
        .map(|s| {
            s.split(',')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let base_path = camino::Utf8PathBuf::from(path);
    let mut config = lode_core::config::default_config();
    if !author.is_empty() {
        config.identity.author = author.to_string();
    }
    if !org.is_empty() {
        config.identity.org = org.to_string();
    }

    let request = lode_core::InitRequest {
        name: name.to_string(),
        base_path,
        config,
        profile: profile.map(|s| s.to_string()),
        components,
        dry_run: false,
        overwrite: false,
        lang: None,
        preset: None,
        license: None,
    };

    match lode_core::init_project(request) {
        Ok(report) => Ok(json!({
            "status": "ok",
            "project_dir": report.project_dir.to_string(),
            "wrote_paths": report.wrote_paths.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            "dry_run": report.dry_run,
        })),
        Err(e) => Err(e.to_string()),
    }
}

pub fn lode_add(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let name = args["name"]
        .as_str()
        .ok_or("Missing required argument: name")?;
    let component = args["component"]
        .as_str()
        .ok_or("Missing required argument: component")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let project_dir = camino::Utf8PathBuf::from(path);
    let config = load_config(&project_dir)?;

    let request = lode_core::AddRequest {
        project_dir: project_dir.clone(),
        name: name.to_string(),
        config,
        component: component.to_string(),
        dry_run: false,
        overwrite: false,
    };

    match lode_core::add_component_to_project(request) {
        Ok(report) => Ok(json!({
            "status": "ok",
            "wrote_paths": report.wrote_paths.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            "dry_run": report.dry_run,
        })),
        Err(e) => Err(e.to_string()),
    }
}

pub fn lode_sync(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let project_dir = camino::Utf8PathBuf::from(path);
    let config = load_config(&project_dir)?;

    match lode_core::sync_project(project_dir, config, false, false) {
        Ok(report) => Ok(json!({
            "status": "ok",
            "project_dir": report.project_dir.to_string(),
            "wrote_paths": report.wrote_paths.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            "dry_run": report.dry_run,
        })),
        Err(e) => Err(e.to_string()),
    }
}

pub fn lode_info(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;

    let _validated =
        lode_core::ValidatedRoot::new(path).map_err(|e| format!("Invalid project root: {e}"))?;

    let root = camino::Utf8PathBuf::from(path);
    let project_toml = root.join(".lode").join("project.toml");

    if !project_toml.exists() {
        return Err(format!("No LODE project found at {path}"));
    }

    let raw = std::fs::read_to_string(&project_toml).map_err(|e| e.to_string())?;
    let project_config: lode_core::ProjectConfig =
        toml::from_str(&raw).map_err(|e| e.to_string())?;

    Ok(json!({
        "path": path,
        "name": project_config.project.name,
        "created_by": project_config.project.created_by,
        "created_at": project_config.project.created_at,
        "profile": project_config.project.profile,
        "components": project_config.project.components,
        "schema_version": project_config.schema_version,
    }))
}
