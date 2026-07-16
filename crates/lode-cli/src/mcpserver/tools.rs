use serde_json::{json, Value};
use std::collections::HashMap;

use super::schema::{bool_schema, optional_string_schema, string_schema, tool_input_schema};
use super::Tool;

fn load_config(root: &camino::Utf8Path) -> Result<lode_core::config::LodeConfig, String> {
    let p = root.join(".lode").join("project.toml");
    if !p.exists() {
        return Ok(lode_core::config::default_config());
    }
    toml::from_str(&std::fs::read_to_string(&p).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn validated_root(path: &str) -> Result<std::path::PathBuf, String> {
    let p = std::path::PathBuf::from(path);
    let p = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map(|cwd| cwd.join(&p)).unwrap_or(p)
    };
    lode_core::ValidatedRoot::new(&p)
        .map(|r| r.path().to_path_buf())
        .map_err(|e| format!("invalid path '{path}': {e}"))
}

fn find_manifest_dir(path: &std::path::Path) -> std::path::PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else if path.is_file() {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn today_date_string() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days = duration.as_secs() / 86400;
    let g = days as i64 + 719468;
    let year = (10000 * g + 14780) / 3652425;
    let doy = g - (365 * year + year / 4 - year / 100 + year / 400);
    let mi = (100 * doy + 52) / 3060;
    let month = (mi + 2) % 12 + 1;
    let year_out = year + (mi + 2) / 12;
    let day = doy - (mi * 306 + 5) / 10 + 1;
    format!("{year_out:04}-{month:02}-{day:02}")
}

pub fn register_all_tools() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(lifecycle_tools());
    tools.extend(convention_tools());
    tools.extend(signature_tools());
    tools.extend(env_tools());
    tools.extend(git_tools());
    tools.extend(health_tools());
    tools.extend(pkg_tools());
    tools.extend(secrets_tools());
    tools.extend(release_tools());
    tools.extend(time_tools());
    tools.extend(registry_tools());
    tools.extend(agent_tools());
    tools.extend(config_tools());
    tools.extend(template_tools());
    tools.extend(template_bundle_tools());
    tools.extend(toolchain_tools());
    tools
}

pub fn dispatch_tool(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "lode_init" => lode_init(args),
        "lode_add" => lode_add(args),
        "lode_sync" => lode_sync(args),
        "lode_info" => lode_info(args),
        "lode_check" => lode_check(args),
        "lode_fix" => lode_fix(args),
        "lode_rename" => lode_rename(args),
        "lode_sign" => lode_sign(args),
        "lode_stamp" => lode_stamp(args),
        "lode_env_check" => lode_env_check(args),
        "lode_env_add" => lode_env_add(args),
        "lode_env_sync" => lode_env_sync(args),
        "lode_git_branch" => lode_git_branch(args),
        "lode_git_commit" => lode_git_commit(args),
        "lode_git_changelog" => lode_git_changelog(args),
        "lode_git_tag" => lode_git_tag(args),
        "lode_audit" => lode_audit(args),
        "lode_metrics" => lode_metrics(args),
        "lode_pkg_outdated" => lode_pkg_outdated(args),
        "lode_pkg_audit" => lode_pkg_audit(args),
        "lode_pkg_update" => lode_pkg_update(args),
        "lode_pkg_list" => lode_pkg_list(args),
        "lode_pkg_clean" => lode_pkg_clean(args),
        "lode_scan_secrets" => lode_scan_secrets(args),
        "lode_release" => lode_release(args),
        "lode_time_today" => lode_time_today(args),
        "lode_time_report" => lode_time_report(args),
        "lode_projects_list" => lode_projects_list(args),
        "lode_projects_health" => lode_projects_health(args),
        "lode_agent_sync" => lode_agent_sync(args),
        "lode_agent_plan" => lode_agent_plan(args),
        "lode_config_show" => lode_config_show(args),
        "lode_config_set" => lode_config_set(args),
        "lode_config_validate" => lode_config_validate(args),
        "lode_template_list" => lode_template_list(args),
        "lode_template_show" => lode_template_show(args),
        "lode_template_bundle_list" => lode_template_bundle_list(args),
        "lode_template_bundle_show" => lode_template_bundle_show(args),
        "lode_template_bundle_validate" => lode_template_bundle_validate(args),
        "lode_template_bundle_preview" => lode_template_bundle_preview(args),
        "lode_template_bundle_apply" => lode_template_bundle_apply(args),
        "lode_template_bundle_capture" => lode_template_bundle_capture(args),
        "lode_toolchain_status" => lode_toolchain_status(args),
        "lode_toolchain_pin" => lode_toolchain_pin(args),
        _ => Err(format!("Unknown tool: {name}")),
    }
}

// lifecycle
fn lifecycle_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_init".into(),
            description: "Initialise a new LODE project in the given directory".into(),
            input_schema: tool_input_schema(vec![
                ("name", "Project name", string_schema()),
                (
                    "path",
                    "Base directory to create project in",
                    string_schema(),
                ),
                ("author", "Author name", optional_string_schema()),
                ("org", "Organisation / namespace", optional_string_schema()),
                ("profile", "Scaffold profile", optional_string_schema()),
                (
                    "components",
                    "Comma-separated list of components to include",
                    optional_string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_add".into(),
            description: "Add a component to an existing LODE project".into(),
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
            name: "lode_sync".into(),
            description: "Synchronise scaffold state with the filesystem".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_info".into(),
            description: "Show project info from the LODE manifest".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_init(args: &Value) -> Result<Value, String> {
    let request = lode_core::InitRequest {
        name: args["name"]
            .as_str()
            .ok_or("Missing required: name")?
            .to_string(),
        base_path: camino::Utf8PathBuf::from_path_buf(validated_root(
            args["path"].as_str().ok_or("Missing required: path")?,
        )?)
        .map_err(|_| "non-utf8 path".to_string())?,
        config: {
            let mut c = lode_core::config::default_config();
            if let Some(a) = args["author"].as_str() {
                if !a.is_empty() {
                    c.identity.author = a.to_string();
                }
            }
            if let Some(o) = args["org"].as_str() {
                if !o.is_empty() {
                    c.identity.org = o.to_string();
                }
            }
            c
        },
        profile: args["profile"].as_str().map(|s| s.to_string()),
        components: args["components"]
            .as_str()
            .map(|s| {
                s.split(',')
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        dry_run: false,
        overwrite: false,
        lang: None,
        preset: None,
        license: None,
        in_place: false,
    };
    lode_core::init_project(request).map(|r| json!({"status":"ok","project_dir":r.project_dir.to_string(),"wrote_paths":r.wrote_paths.iter().map(|p|p.to_string()).collect::<Vec<_>>(),"dry_run":r.dry_run})).map_err(|e| e.to_string())
}

pub fn lode_add(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from_path_buf(validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .map_err(|_| "non-utf8 path".to_string())?;
    let request = lode_core::AddRequest {
        project_dir: root.clone(),
        name: args["name"]
            .as_str()
            .ok_or("Missing required: name")?
            .to_string(),
        config: load_config(&root)?,
        component: args["component"]
            .as_str()
            .ok_or("Missing required: component")?
            .to_string(),
        dry_run: false,
        overwrite: false,
    };
    lode_core::add_component_to_project(request).map(|r| json!({"status":"ok","wrote_paths":r.wrote_paths.iter().map(|p|p.to_string()).collect::<Vec<_>>(),"dry_run":r.dry_run})).map_err(|e| e.to_string())
}

pub fn lode_sync(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from_path_buf(validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .map_err(|_| "non-utf8 path".to_string())?;
    let config = load_config(&root)?;
    lode_core::sync_project(root, config, false, false).map(|r| json!({"status":"ok","project_dir":r.project_dir.to_string(),"wrote_paths":r.wrote_paths.iter().map(|p|p.to_string()).collect::<Vec<_>>(),"dry_run":r.dry_run})).map_err(|e| e.to_string())
}

pub fn lode_info(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from_path_buf(validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .map_err(|_| "non-utf8 path".to_string())?;
    let p = root.join(".lode").join("project.toml");
    if !p.exists() {
        return Err(format!("No LODE project found at {}", root));
    }
    let cfg: lode_core::ProjectConfig =
        toml::from_str(&std::fs::read_to_string(&p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(
        json!({"path":root.as_str(),"name":cfg.project.name,"created_by":cfg.project.created_by,"created_at":cfg.project.created_at,"profile":cfg.project.profile,"components":cfg.project.components,"schema_version":cfg.schema_version}),
    )
}

// convention
fn convention_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_check".into(),
            description: "Check project for convention violations".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_fix".into(),
            description: "Automatically fix convention violations where possible".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_rename".into(),
            description: "Rename a file or directory to match conventions".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "target",
                    "Path to rename (relative to project root)",
                    string_schema(),
                ),
                (
                    "new_name",
                    "New name for the file/directory",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_check(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    lode_core::check_path(&root, &load_config(&root)?).map(|r| json!({"path":root.as_str(),"checked":r.checked,"violations_count":r.violations.len(),"violations":r.violations.iter().map(|v|json!({"path":v.path.to_string(),"expected_name":v.expected_name})).collect::<Vec<_>>(),"renamed":r.renamed.len()})).map_err(|e| e.to_string())
}

pub fn lode_fix(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    lode_core::fix_path(&root, &load_config(&root)?).map(|r| json!({"path":root.as_str(),"checked":r.checked,"remaining_violations":r.violations.len(),"renamed":r.renamed.len(),"renamed_files":r.renamed.iter().map(|(f,t)|json!({"from":f.to_string(),"to":t.to_string()})).collect::<Vec<_>>()})).map_err(|e| e.to_string())
}

pub fn lode_rename(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from(args["path"].as_str().ok_or("Missing required: path")?);
    let config = load_config(&root)?;
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let target_path = validated
        .resolve(args["target"].as_str().ok_or("Missing required: target")?)
        .map_err(|e| e.to_string())?;
    let parent = camino::Utf8Path::new(args["target"].as_str().ok_or("Missing required: target")?)
        .parent()
        .unwrap_or_else(|| camino::Utf8Path::new(""))
        .to_path_buf();
    let name = if let Some(n) = args["new_name"].as_str() {
        if n.is_empty() {
            let stem = target_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("Cannot determine file name")?;
            lode_core::normalize_name(stem, &config)
        } else {
            n.to_string()
        }
    } else {
        let stem = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Cannot determine file name")?;
        lode_core::normalize_name(stem, &config)
    };
    validated
        .rename_entry(
            args["target"].as_str().ok_or("Missing required: target")?,
            parent.join(&name),
        )
        .map_err(|e| e.to_string())?;
    Ok(json!({"status":"ok","from":args["target"].as_str().ok_or("")?,"to":name}))
}

// signature
fn signature_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_sign".into(),
            description: "Compute content hash and show signature header for a file".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("file", "Relative file path to sign", string_schema()),
            ]),
        },
        Tool {
            name: "lode_stamp".into(),
            description: "Write a signature header into a file".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("file", "Relative file path to stamp", string_schema()),
            ]),
        },
    ]
}

pub fn lode_sign(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from(args["path"].as_str().ok_or("Missing required: path")?);
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let file_path = validated
        .resolve(args["file"].as_str().ok_or("Missing required: file")?)
        .map_err(|e| e.to_string())?;
    if !file_path.exists() {
        return Err(format!(
            "File not found: {}",
            args["file"].as_str().ok_or("")?
        ));
    }
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let hash = lode_core::compute_content_hash(&content);
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let prefix = lode_core::comment_prefix_for_extension(ext).unwrap_or("//");
    Ok(
        json!({"file":args["file"].as_str().ok_or("")?,"hash":hash,"header":format!("{prefix} lode:sha256={hash}"),"has_signature":lode_core::has_signature_header(&content)}),
    )
}

pub fn lode_stamp(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from(args["path"].as_str().ok_or("Missing required: path")?);
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let file_path = validated
        .resolve(args["file"].as_str().ok_or("Missing required: file")?)
        .map_err(|e| e.to_string())?;
    if !file_path.exists() {
        return Err(format!(
            "File not found: {}",
            args["file"].as_str().ok_or("")?
        ));
    }
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let hash = lode_core::compute_content_hash(&content);
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let prefix = lode_core::comment_prefix_for_extension(ext).unwrap_or("//");
    validated
        .write_atomic(
            args["file"].as_str().ok_or("")?,
            format!("{prefix} lode:sha256={hash}\n{content}"),
        )
        .map_err(|e| e.to_string())?;
    Ok(json!({"status":"ok","file":args["file"].as_str().ok_or("")?,"hash":hash}))
}

// env
fn env_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_env_check".into(),
            description: "Check environment variables for drift or missing values".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_env_add".into(),
            description: "Add a new environment variable to the .env config".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("key", "Environment variable name", string_schema()),
                ("value", "Default value", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_env_sync".into(),
            description: "Synchronise .env file with the env config".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_env_check(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    let drifts = lode_core::check_env_drift(root.as_std_path(), &lode_core::EnvConfig::default())
        .map_err(|e| e.to_string())?;
    Ok(
        json!({"path":root.as_str(),"drift_count":drifts.len(),"drifts":drifts.iter().map(|d|json!({"key":d.key,"issue":d.issue})).collect::<Vec<_>>(),"status":if drifts.is_empty(){"ok"}else{"drift"}}),
    )
}

pub fn lode_env_add(args: &Value) -> Result<Value, String> {
    let key = args["key"].as_str().ok_or("Missing required: key")?;
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "Invalid key '{key}': must contain only alphanumeric characters and underscores"
        ));
    }
    let value = args["value"].as_str().unwrap_or("");
    if value.contains('\n') || value.contains('\r') || value.contains('\0') {
        return Err("Invalid .env value: must not contain newlines or null bytes".to_string());
    }
    let root = camino::Utf8PathBuf::from(args["path"].as_str().ok_or("Missing required: path")?);
    let validated = lode_core::ValidatedRoot::new(root.as_std_path()).map_err(|e| e.to_string())?;
    let env_path = validated.resolve(".env").map_err(|e| e.to_string())?;
    let entry = format!("{key}={value}\n");
    if env_path.exists() {
        validated
            .write_atomic(
                ".env",
                format!(
                    "{}{entry}",
                    std::fs::read_to_string(&env_path).map_err(|e| e.to_string())?
                ),
            )
            .map_err(|e| e.to_string())?;
    } else {
        validated
            .write_atomic(".env", entry)
            .map_err(|e| e.to_string())?;
    }
    Ok(json!({"status":"ok","key":key,"added":true}))
}

pub fn lode_env_sync(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    lode_core::generate_env(
        root.as_std_path(),
        &lode_core::EnvConfig::default(),
        root.file_name().unwrap_or("project"),
    )
    .map_err(|e| e.to_string())?;
    Ok(json!({"status":"ok","path":root.as_str()}))
}

// git
fn git_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_git_branch".into(),
            description: "Generate a conventional branch name from a description".into(),
            input_schema: tool_input_schema(vec![
                (
                    "kind",
                    "Branch kind (feat, fix, chore, etc.)",
                    string_schema(),
                ),
                ("description", "Branch description", string_schema()),
            ]),
        },
        Tool {
            name: "lode_git_commit".into(),
            description: "Stage all changes and create a conventional commit".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("message", "Commit message", string_schema()),
            ]),
        },
        Tool {
            name: "lode_git_changelog".into(),
            description: "Generate a changelog from git log".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("from_tag", "Start from this tag", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_git_tag".into(),
            description: "Create a git tag for the current HEAD".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("tag", "Tag name (e.g. v1.0.0)", string_schema()),
                (
                    "message",
                    "Tag message (optional)",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_git_branch(args: &Value) -> Result<Value, String> {
    Ok(
        json!({"branch":lode_core::branch_name(args["kind"].as_str().ok_or("Missing required: kind")?,args["description"].as_str().ok_or("Missing required: description")?)}),
    )
}

pub fn lode_git_commit(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    if !lode_core::is_git_repo(&root) {
        return Err("Not a git repository".to_string());
    }
    let msg = args["message"]
        .as_str()
        .ok_or("Missing required: message")?;
    lode_core::git_add_all(&root).map_err(|e| e.to_string())?;
    lode_core::git_commit(&root, msg).map_err(|e| e.to_string())?;
    Ok(json!({"status":"ok","message":msg}))
}

pub fn lode_git_changelog(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    if !lode_core::is_git_repo(&root) {
        return Err("Not a git repository".to_string());
    }
    lode_core::git_changelog(&root, args["from_tag"].as_str())
        .map(|c| json!({"path":root.display().to_string(),"changelog":c}))
        .map_err(|e| e.to_string())
}

pub fn lode_git_tag(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    if !lode_core::is_git_repo(&root) {
        return Err("Not a git repository".to_string());
    }
    lode_core::git_tag(
        &root,
        args["tag"].as_str().ok_or("Missing required: tag")?,
        args["message"].as_str(),
    )
    .map_err(|e| e.to_string())?;
    Ok(json!({"status":"ok","tag":args["tag"].as_str().ok_or("")?}))
}

// health
fn health_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_audit".into(),
            description: "Run a project health audit (conventions, secrets, files)".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_metrics".into(),
            description: "Show project metrics (audit report from .lode/metrics.json)".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_audit(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    if !root.join(".lode").exists() {
        return Err(format!("No LODE project found at {}", root));
    }
    lode_core::audit_project(&root, &lode_core::config::default_config()).map(|r|json!({"path":root.as_str(),"score":r.score,"convention_violations":r.convention_violations,"secret_findings":r.secret_findings,"license_present":r.license_present,"env_example_present":r.env_example_present,"readme_present":r.readme_present})).map_err(|e|e.to_string())
}

pub fn lode_metrics(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    lode_core::load_metrics(&root).map(|r|json!({"path":root.as_str(),"score":r.score,"convention_violations":r.convention_violations,"secret_findings":r.secret_findings,"license_present":r.license_present,"env_example_present":r.env_example_present,"readme_present":r.readme_present})).map_err(|e|e.to_string())
}

// pkg
fn pkg_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_pkg_outdated".into(),
            description: "List outdated dependencies".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_pkg_audit".into(),
            description: "Audit dependencies for known vulnerabilities".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_pkg_update".into(),
            description: "Update dependencies".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "package",
                    "Package name to update (optional)",
                    optional_string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_pkg_list".into(),
            description: "Detect the package manager for a project".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_pkg_clean".into(),
            description: "Show clean command for detected package manager".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_pkg_outdated(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let pm = lode_core::detect_package_manager(&root).ok_or("No package manager detected")?;
    Ok(
        json!({"path":root.display().to_string(),"package_manager":pm,"args":lode_core::package_outdated_args(&pm).map_err(|e|e.to_string())?}),
    )
}

pub fn lode_pkg_audit(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let pm = lode_core::detect_package_manager(&root).ok_or("No package manager detected")?;
    Ok(
        json!({"path":root.display().to_string(),"package_manager":pm,"args":lode_core::package_audit_args(&pm,None).map_err(|e|e.to_string())?}),
    )
}

pub fn lode_pkg_update(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let pm = lode_core::detect_package_manager(&root).ok_or("No package manager detected")?;
    Ok(
        json!({"path":root.display().to_string(),"package_manager":pm,"args":lode_core::package_update_args(&pm,args["package"].as_str()).map_err(|e|e.to_string())?}),
    )
}

pub fn lode_pkg_list(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    Ok(
        json!({"path":root.display().to_string(),"package_manager":lode_core::detect_package_manager(&root)}),
    )
}

pub fn lode_pkg_clean(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let pm = lode_core::detect_package_manager(&root).ok_or("No package manager detected")?;
    Ok(
        json!({"path":root.display().to_string(),"package_manager":pm,"command":format!("{pm} clean")}),
    )
}

// secrets
fn secrets_tools() -> Vec<Tool> {
    vec![Tool {
        name: "lode_scan_secrets".into(),
        description: "Scan project files for leaked secrets, API keys, and tokens".into(),
        input_schema: tool_input_schema(vec![
            ("path", "Project root directory", string_schema()),
            (
                "pattern",
                "Optional regex pattern to filter findings",
                optional_string_schema(),
            ),
        ]),
    }]
}

pub fn lode_scan_secrets(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    let report = lode_core::scan_secrets(&root).map_err(|e| e.to_string())?;
    Ok(
        json!({"path":root.as_str(),"checked_files":report.checked_files,"total_findings":report.findings.len(),"findings":report.findings.iter().map(|f|json!({"file":f.path.to_string(),"line":f.line,"kind":f.kind})).collect::<Vec<_>>(),"status":if report.findings.is_empty(){"clean"}else{"findings"}}),
    )
}

// release
fn release_tools() -> Vec<Tool> {
    vec![Tool {
        name: "lode_release".into(),
        description: "Bump version and prepare a release".into(),
        input_schema: tool_input_schema(vec![
            ("path", "Project root directory", string_schema()),
            ("bump", "Bump type: major, minor, or patch", string_schema()),
            ("dry_run", "Preview without making changes", bool_schema()),
        ]),
    }]
}

pub fn lode_release(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let bump = args["bump"].as_str().ok_or("Missing required: bump")?;
    if !["major", "minor", "patch"].contains(&bump) {
        return Err(format!(
            "Invalid bump type '{bump}'. Must be one of: major, minor, patch"
        ));
    }
    lode_core::create_release(&root, &lode_core::ReleaseConfig::default(), args["dry_run"].as_bool().unwrap_or(false)).map(|r|json!({"status":"ok","path":root.display().to_string(),"old_version":r.old_version,"new_version":r.new_version,"tag":r.tag,"files_updated":r.files_updated.iter().map(|p|p.to_string_lossy().to_string()).collect::<Vec<_>>(),"dry_run":r.dry_run})).map_err(|e|e.to_string())
}

// time
fn time_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_time_today".into(),
            description: "Show today's time tracking summary".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_time_report".into(),
            description: "Show time tracking sessions".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "limit",
                    "Max number of sessions to show",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_time_today(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    let total_seconds = lode_core::time_today(&root).map_err(|e| e.to_string())?;
    let log = lode_core::load_time_log(&root).map_err(|e| e.to_string())?;
    let today = today_date_string();
    let today_sessions: Vec<_> = log
        .sessions
        .iter()
        .filter(|s| s.ended_at.starts_with(&today))
        .cloned()
        .collect();
    Ok(
        json!({"path":root.as_str(),"total_seconds":total_seconds,"sessions":today_sessions.iter().map(|s|json!({"started_at":s.started_at,"ended_at":s.ended_at,"seconds":s.seconds,"project":s.project,"file":s.file,"task":s.task})).collect::<Vec<_>>()}),
    )
}

pub fn lode_time_report(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8Path::from_path(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .ok_or_else(|| "non-utf8 path".to_string())?
    .to_path_buf();
    let limit = args["limit"]
        .as_str()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(1000);
    let log = lode_core::load_time_log(&root).map_err(|e| e.to_string())?;
    let sessions: Vec<_> = log.sessions.iter().rev().take(limit).cloned().collect();
    Ok(
        json!({"path":root.as_str(),"total_sessions":log.sessions.len(),"showing":sessions.len(),"sessions":sessions.iter().map(|s|json!({"started_at":s.started_at,"ended_at":s.ended_at,"seconds":s.seconds,"project":s.project,"file":s.file,"task":s.task})).collect::<Vec<_>>()}),
    )
}

// registry
fn registry_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_projects_list".into(),
            description: "List all registered LODE projects".into(),
            input_schema: tool_input_schema(vec![]),
        },
        Tool {
            name: "lode_projects_health".into(),
            description: "Show health status for all registered projects".into(),
            input_schema: tool_input_schema(vec![]),
        },
    ]
}

pub fn lode_projects_list(_args: &Value) -> Result<Value, String> {
    let registry = lode_core::load_registry().map_err(|e| e.to_string())?;
    Ok(
        json!({"total":registry.projects.len(),"projects":registry.projects.iter().map(|p|json!({"name":p.name,"path":p.path.to_string(),"profile":p.profile,"last_seen":p.last_seen})).collect::<Vec<_>>()}),
    )
}

pub fn lode_projects_health(_args: &Value) -> Result<Value, String> {
    let registry = lode_core::load_registry().map_err(|e| e.to_string())?;
    let results: Vec<Value> = registry.projects.iter().map(|p|json!({"name":p.name,"path":p.path.to_string(),"healthy":p.path.join(".lode").join("project.toml").exists()})).collect();
    Ok(json!({"total":results.len(),"projects":results}))
}

// agent
fn agent_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_agent_sync".into(),
            description: "Show agent configuration sync status for a project".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_agent_plan".into(),
            description: "Generate an execution plan for a task".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                ("task", "Task description", string_schema()),
            ]),
        },
    ]
}

pub fn lode_agent_sync(args: &Value) -> Result<Value, String> {
    let root = camino::Utf8PathBuf::from_path_buf(validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?)
    .map_err(|_| "non-utf8 path".to_string())?;
    Ok(
        json!({"path":root.to_string(),"agents_dir":root.join(".lode").join("agents").to_string(),"exists":root.join(".lode").join("agents").exists(),"status":"ok"}),
    )
}

pub fn lode_agent_plan(args: &Value) -> Result<Value, String> {
    let path = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let task = args["task"].as_str().ok_or("Missing required: task")?;
    Ok(
        json!({"path":path.display().to_string(),"task":task,"steps":[
            json!({"step":1,"action":"analyse","description":format!("Analyse project at {}",path.display())}),
            json!({"step":2,"action":"plan","description":format!("Plan execution for: {task}")}),
            json!({"step":3,"action":"execute","description":"Execute planned steps"}),
            json!({"step":4,"action":"verify","description":"Verify results"}),
        ]}),
    )
}

// config
fn config_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_config_show".into(),
            description: "Show the default LODE configuration".into(),
            input_schema: tool_input_schema(vec![]),
        },
        Tool {
            name: "lode_config_set".into(),
            description: "Set a configuration value in the project's .lode/project.toml".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "key",
                    "Config key to set using dot notation",
                    string_schema(),
                ),
                ("value", "Value to set", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_config_validate".into(),
            description: "Validate a project configuration against the schema".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_config_show(_args: &Value) -> Result<Value, String> {
    Ok(
        json!({"config":toml::to_string_pretty(&lode_core::config::default_config()).map_err(|e|e.to_string())?}),
    )
}

pub fn lode_config_set(args: &Value) -> Result<Value, String> {
    let root =
        lode_core::ValidatedRoot::new(args["path"].as_str().ok_or("Missing required: path")?)
            .map_err(|e| format!("Invalid project root: {e}"))?;
    let key = args["key"].as_str().ok_or("Missing required: key")?;
    let value = args["value"].as_str().unwrap_or("");
    let raw = std::fs::read_to_string(root.path().join(".lode").join("project.toml"))
        .map_err(|_| format!("No LODE project at {}", args["path"].as_str().unwrap_or("")))?;
    let mut config: toml::Value = toml::from_str(&raw).map_err(|e| e.to_string())?;
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = &mut config;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(table) = current.as_table_mut() {
                table.insert(part.to_string(), toml::Value::String(value.to_string()));
            } else {
                return Err(format!("Cannot set key '{key}': parent is not a table"));
            }
        } else {
            current = current.get_mut(part).ok_or_else(|| {
                format!("Cannot set key '{key}': path segment '{part}' not found")
            })?;
        }
    }
    let new_content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    root.write_atomic(".lode/project.toml", &new_content)
        .map_err(|e| format!("Failed to write config: {e}"))?;
    Ok(json!({"status":"ok","key":key,"value":value,"config":new_content}))
}

pub fn lode_config_validate(args: &Value) -> Result<Value, String> {
    let root =
        lode_core::ValidatedRoot::new(args["path"].as_str().ok_or("Missing required: path")?)
            .map_err(|e| format!("Invalid project root: {e}"))?;
    let p = root.path().join(".lode").join("project.toml");
    if !p.exists() {
        return Err(format!(
            "No LODE project at {}",
            args["path"].as_str().unwrap_or("")
        ));
    }
    let _: lode_core::config::LodeConfig =
        toml::from_str(&std::fs::read_to_string(&p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(json!({"valid":true,"path":args["path"].as_str().unwrap_or("")}))
}

// template
fn template_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_template_list".into(),
            description: "List available project template paths".into(),
            input_schema: tool_input_schema(vec![]),
        },
        Tool {
            name: "lode_template_show".into(),
            description: "Show details of a specific template path".into(),
            input_schema: tool_input_schema(vec![(
                "template",
                "Template path to inspect",
                string_schema(),
            )]),
        },
    ]
}

pub fn lode_template_list(_args: &Value) -> Result<Value, String> {
    let items: Vec<Value> = lode_core::template_paths()
        .iter()
        .map(|n| json!({"name":n}))
        .collect();
    Ok(json!({"total":items.len(),"templates":items}))
}

pub fn lode_template_show(args: &Value) -> Result<Value, String> {
    let template = args["template"]
        .as_str()
        .ok_or("Missing required: template")?;
    for name in lode_core::template_paths() {
        if *name == template {
            return Ok(json!({"name":name}));
        }
    }
    Err(format!("Template not found: {template}"))
}

// template_bundle
fn template_bundle_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_template_bundle_list".into(),
            description: "List available template bundles in a directory".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Directory to scan for bundles (default: global templates dir)",
                optional_string_schema(),
            )]),
        },
        Tool {
            name: "lode_template_bundle_show".into(),
            description: "Show TOML manifest of a template bundle".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Path to the template bundle directory or its manifest",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_template_bundle_validate".into(),
            description: "Validate a template bundle's manifest and assets directory".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Path to the template bundle directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_template_bundle_preview".into(),
            description: "Preview a directory capture without writing".into(),
            input_schema: tool_input_schema(vec![
                ("source", "Source directory to preview", string_schema()),
                (
                    "mode",
                    "Capture mode: minimal, source (default), development, complete",
                    optional_string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_template_bundle_apply".into(),
            description: "Apply/render a template bundle into the target directory".into(),
            input_schema: tool_input_schema(vec![
                (
                    "path",
                    "Path to the template bundle directory",
                    string_schema(),
                ),
                (
                    "variables",
                    "key=value pairs for template variables",
                    optional_string_schema(),
                ),
                (
                    "overwrite",
                    "Overwrite policy: skip, error (default), replace",
                    optional_string_schema(),
                ),
                (
                    "dry_run",
                    "If true, report what would happen without writing",
                    optional_string_schema(),
                ),
                (
                    "target",
                    "Target directory (default: current dir)",
                    optional_string_schema(),
                ),
            ]),
        },
        Tool {
            name: "lode_template_bundle_capture".into(),
            description: "Capture a directory as a template bundle".into(),
            input_schema: tool_input_schema(vec![
                ("source", "Source directory to capture", string_schema()),
                (
                    "dest",
                    "Destination path for the bundle directory",
                    string_schema(),
                ),
                (
                    "mode",
                    "Capture mode: minimal, source (default), development, complete",
                    optional_string_schema(),
                ),
                (
                    "name",
                    "Template name/ID override",
                    optional_string_schema(),
                ),
                (
                    "dry_run",
                    "If true, preview only without writing",
                    optional_string_schema(),
                ),
                (
                    "no_redact",
                    "If true, do not redact secrets in captured content",
                    optional_string_schema(),
                ),
            ]),
        },
    ]
}

pub fn lode_template_bundle_list(args: &Value) -> Result<Value, String> {
    let search_dir = match args.get("path").and_then(|v| v.as_str()) {
        Some(s) => validated_root(s)?,
        None => lode_core::global_dir()
            .ok()
            .map(|g| g.into_std_path_buf().join("templates"))
            .unwrap_or_else(|| std::path::PathBuf::from(".")),
    };
    if !search_dir.exists() {
        return Ok(json!({"bundles":[],"count":0,"search_dir":search_dir.to_string_lossy()}));
    }
    let mut bundles = Vec::new();
    for entry in std::fs::read_dir(&search_dir).map_err(|e| format!("read dir: {e}"))? {
        let p = entry.map_err(|e| format!("entry: {e}"))?.path();
        if p.is_dir() {
            let dirname = p
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
                .to_string();
            if p.join(format!("{dirname}.toml")).exists() {
                bundles.push(json!({"path":p.to_string_lossy(),"manifest":p.join(format!("{dirname}.toml")).to_string_lossy()}));
            }
        }
    }
    Ok(json!({"bundles":bundles,"count":bundles.len(),"search_dir":search_dir.to_string_lossy()}))
}

pub fn lode_template_bundle_show(args: &Value) -> Result<Value, String> {
    let bundle_dir = find_manifest_dir(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)
        .map_err(|e| format!("load bundle: {e}"))?;
    Ok(
        json!({"manifest":toml::to_string_pretty(&manifest).map_err(|e|format!("serialize: {e}"))?,"path":bundle_dir.to_string_lossy()}),
    )
}

pub fn lode_template_bundle_validate(args: &Value) -> Result<Value, String> {
    let bundle_dir = find_manifest_dir(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)
        .map_err(|e| format!("load bundle: {e}"))?;
    let errors = manifest.validate(&bundle_dir);
    let assets_exist = if manifest.assets.is_empty() {
        true
    } else {
        bundle_dir.join("assets").exists()
    };
    Ok(
        json!({"valid":errors.is_empty() && assets_exist,"errors":errors,"assets_dir_exists":assets_exist,"path":bundle_dir.to_string_lossy()}),
    )
}

pub fn lode_template_bundle_preview(args: &Value) -> Result<Value, String> {
    let source = validated_root(args["source"].as_str().ok_or("Missing required: source")?)?;
    let mode_str = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("source");
    let mode = match mode_str {
        "minimal" => lode_core::template_bundle_capture::CaptureMode::Minimal,
        "source" => lode_core::template_bundle_capture::CaptureMode::Source,
        "development" => lode_core::template_bundle_capture::CaptureMode::Development,
        "complete" => lode_core::template_bundle_capture::CaptureMode::Complete,
        other => return Err(format!("unknown capture mode: {other}")),
    };
    let config = lode_core::template_bundle_capture::CaptureConfig {
        mode,
        ..Default::default()
    };
    let preview = lode_core::template_bundle_capture::capture_preview(&source, &config)
        .map_err(|e| format!("preview: {e}"))?;
    Ok(
        json!({"source":preview.source.to_string_lossy(),"template_id":preview.template_id,"template_name":preview.template_name,"inline_count":preview.inline_count,"asset_count":preview.asset_count,"directory_count":preview.directory_count,"estimated_size_kb":preview.estimated_size_kb,"variables":preview.variables,"secrets_found":preview.secrets_found,"classifications":preview.file_classifications.iter().map(|fc|json!({"path":fc.path.to_string(),"kind":format!("{:?}",fc.classification),"size_bytes":fc.size_bytes})).collect::<Vec<_>>(),"warnings":preview.warnings,"mode":mode_str}),
    )
}

pub fn lode_template_bundle_apply(args: &Value) -> Result<Value, String> {
    let bundle_dir = find_manifest_dir(&validated_root(
        args["path"].as_str().ok_or("Missing required: path")?,
    )?);
    if !bundle_dir.exists() {
        return Err(format!("bundle not found: {}", bundle_dir.display()));
    }
    let values: HashMap<String, String> = args
        .get("variables")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .split(',')
        .filter_map(|pair| {
            let eq = pair.find('=')?;
            Some((
                pair[..eq].trim().to_string(),
                pair[eq + 1..].trim().to_string(),
            ))
        })
        .collect();
    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .map(validated_root)
        .transpose()?
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });
    let report = lode_core::template_bundle_apply::apply_bundle(
        &bundle_dir,
        &target,
        &values,
        args.get("overwrite")
            .and_then(|v| v.as_str())
            .unwrap_or("error"),
        args.get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    )
    .map_err(|e| format!("apply: {e}"))?;
    Ok(
        json!({"files_written":report.files_written,"assets_copied":report.assets_copied,"directories_created":report.directories_created,"files_skipped":report.files_skipped,"assets_skipped":report.assets_skipped,"warnings":report.warnings,"errors":report.errors,"dry_run":args.get("dry_run").and_then(|v|v.as_bool()).unwrap_or(false)}),
    )
}

pub fn lode_template_bundle_capture(args: &Value) -> Result<Value, String> {
    let source = validated_root(args["source"].as_str().ok_or("Missing required: source")?)?;
    let dest = validated_root(args["dest"].as_str().ok_or("Missing required: dest")?)?;
    let mode_str = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("source");
    let mode = match mode_str {
        "minimal" => lode_core::template_bundle_capture::CaptureMode::Minimal,
        "source" => lode_core::template_bundle_capture::CaptureMode::Source,
        "development" => lode_core::template_bundle_capture::CaptureMode::Development,
        "complete" => lode_core::template_bundle_capture::CaptureMode::Complete,
        other => return Err(format!("unknown capture mode: {other}")),
    };
    let dry_run = args
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let config = lode_core::template_bundle_capture::CaptureConfig {
        mode,
        template_id: args
            .get("name")
            .and_then(|v| v.as_str())
            .map(|n| n.to_string()),
        template_name: args
            .get("name")
            .and_then(|v| v.as_str())
            .map(|n| n.to_string()),
        destination: Some(dest.clone()),
        project: false,
        dry_run,
        redact_secrets: true,
        ..Default::default()
    };
    if dry_run {
        let preview = lode_core::template_bundle_capture::capture_preview(&source, &config)
            .map_err(|e| format!("preview: {e}"))?;
        return Ok(
            json!({"dry_run":true,"source":preview.source.to_string_lossy(),"inline_count":preview.inline_count,"asset_count":preview.asset_count,"directory_count":preview.directory_count,"estimated_size_kb":preview.estimated_size_kb,"variables":preview.variables,"secrets_found":preview.secrets_found,"classifications":preview.file_classifications.iter().map(|fc|json!({"path":fc.path.to_string(),"kind":format!("{:?}",fc.classification),"size_bytes":fc.size_bytes})).collect::<Vec<_>>(),"warnings":preview.warnings}),
        );
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dest dir: {e}"))?;
    }
    let receipt = lode_core::template_bundle_capture::capture_template(&source, &config)
        .map_err(|e| format!("capture: {e}"))?;
    Ok(
        json!({"dry_run":false,"source":receipt.source.to_string_lossy(),"destination":receipt.destination.to_string_lossy(),"inline_files":receipt.inline_files.len(),"assets_copied":receipt.assets_copied.len(),"excluded_paths":receipt.excluded_paths.len(),"secret_findings":receipt.secret_findings.len(),"operation_id":receipt.operation_id}),
    )
}

// toolchain
fn toolchain_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_toolchain_status".into(),
            description: "Show installed toolchain versions for a project".into(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Project root directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_toolchain_pin".into(),
            description: "Pin a specific tool version in the toolchain store".into(),
            input_schema: tool_input_schema(vec![
                ("path", "Project root directory", string_schema()),
                (
                    "runtime",
                    "Runtime name (e.g. rust, node, python, go)",
                    string_schema(),
                ),
                ("version", "Version to pin", string_schema()),
            ]),
        },
    ]
}

pub fn lode_toolchain_status(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let tools: Vec<Value> = lode_core::toolchain_status(&root, &lode_core::ToolchainConfig::default()).iter().map(|s|json!({"runtime":s.runtime,"installed":s.installed,"version":s.version,"lock_version":s.lock_version,"manager":s.manager})).collect();
    Ok(json!({"path":root.display().to_string(),"tools":tools}))
}

pub fn lode_toolchain_pin(args: &Value) -> Result<Value, String> {
    let root = validated_root(args["path"].as_str().ok_or("Missing required: path")?)?;
    let runtime_name = args["runtime"]
        .as_str()
        .ok_or("Missing required: runtime")?;
    let version = args["version"]
        .as_str()
        .ok_or("Missing required: version")?;
    if version.is_empty()
        || version.contains('/')
        || version.contains('\\')
        || version.contains('\0')
        || !version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
    {
        return Err(format!(
            "Invalid version '{version}': must be a valid semver-like version string"
        ));
    }
    let toolchain = lode_core::ToolchainConfig::default();
    let runtime = toolchain
        .runtimes
        .iter()
        .find(|r| r.name == runtime_name)
        .ok_or_else(|| format!("Unknown runtime: {runtime_name}"))?;
    lode_core::pin_runtime(&root, runtime, version).map_err(|e| e.to_string())?;
    Ok(json!({"status":"ok","runtime":runtime_name,"version":version}))
}
