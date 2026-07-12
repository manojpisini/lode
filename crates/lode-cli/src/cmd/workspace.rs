#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{LodeError, ValidatedRoot};

use crate::{current_dir, json_pretty, run_make, run_process_status, safe_relative_path, WorkspaceCommand};

pub(crate) fn workspace(command: WorkspaceCommand) -> lode_core::Result<()> {
    match command {
        WorkspaceCommand::Init => workspace_init()?,
        WorkspaceCommand::List { format } => workspace_list(&format)?,
        WorkspaceCommand::Add { name } => workspace_add(&name)?,
        WorkspaceCommand::Remove { name, confirm } => workspace_remove(&name, confirm)?,
        WorkspaceCommand::Run {
            target,
            pkg,
            changed,
            parallel,
            dry_run,
        } => workspace_run(&target, pkg.as_deref(), &changed, parallel, dry_run)?,
        WorkspaceCommand::Graph { format } => workspace_graph(&format)?,
    }
    Ok(())
}

fn workspace_file() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("workspace.toml")
}

fn workspace_init() -> lode_core::Result<()> {
    let path = workspace_file();
    let root = ValidatedRoot::new(current_dir()?)?;
    if let Some(parent) = path.parent() {
        root.create_dir_all(parent)?;
    }
    if !path.exists() {
        root.write_atomic(&path, "members = []\n")?;
    }
    println!("workspace initialised");
    Ok(())
}

fn workspace_members() -> lode_core::Result<Vec<String>> {
    let path = workspace_file();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    Ok(value
        .get("members")
        .and_then(toml::Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default())
}

fn save_workspace_members(members: &[String]) -> lode_core::Result<()> {
    let path = workspace_file();
    let root = ValidatedRoot::new(current_dir()?)?;
    if let Some(parent) = path.parent() {
        root.create_dir_all(parent)?;
    }
    let quoted = members
        .iter()
        .map(|member| format!("\"{member}\""))
        .collect::<Vec<_>>()
        .join(", ");
    root.write_atomic(path, format!("members = [{quoted}]\n"))
        .map(|_| ())
}

fn workspace_add(name: &str) -> lode_core::Result<()> {
    validate_workspace_member(name)?;
    let mut members = workspace_members()?;
    if !members.iter().any(|member| member == name) {
        members.push(name.to_string());
        members.sort();
        save_workspace_members(&members)?;
    }
    ValidatedRoot::new(current_dir()?)?.create_dir_all(safe_relative_path(name)?)?;
    println!("workspace member added: {name}");
    Ok(())
}

fn workspace_remove(name: &str, confirm: bool) -> lode_core::Result<()> {
    validate_workspace_member(name)?;
    if !confirm {
        return Err(LodeError::Message(
            "refusing to remove workspace member without --confirm".to_string(),
        ));
    }
    let mut members = workspace_members()?;
    let before = members.len();
    members.retain(|member| member != name);
    if members.len() == before {
        return Err(LodeError::Message(format!(
            "workspace member not found: {name}"
        )));
    }
    save_workspace_members(&members)?;
    println!("workspace member removed: {name}");
    Ok(())
}

fn validate_workspace_member(name: &str) -> lode_core::Result<()> {
    let relative = safe_relative_path(name)?;
    if relative.as_str().is_empty() || relative.as_str().starts_with(".lode") {
        return Err(LodeError::Message(format!(
            "unsafe workspace member path: {name}"
        )));
    }
    Ok(())
}

fn workspace_list(format: &str) -> lode_core::Result<()> {
    let members = workspace_members()?;
    match format {
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&members)
                .map_err(|error| LodeError::Message(error.to_string()))?
        ),
        "table" => {
            if members.is_empty() {
                println!("workspace has no members");
            } else {
                for member in members {
                    println!("{member}");
                }
            }
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported workspace list format: {other}"
            )))
        }
    }
    Ok(())
}

fn workspace_run(
    target: &str,
    pkg: Option<&str>,
    changed: &[String],
    parallel: Option<usize>,
    dry_run: bool,
) -> lode_core::Result<()> {
    let mut members = workspace_members()?;
    if let Some(pkg) = pkg {
        members.retain(|member| member == pkg || member.ends_with(&format!("/{pkg}")));
    }
    if !changed.is_empty() {
        let affected = affected_workspace_members(&members, changed);
        if affected.is_empty() {
            println!("no workspace members affected by changed path(s)");
            return Ok(());
        }
        members = affected;
    }
    if members.is_empty() {
        if dry_run {
            println!("would run make {target}");
            return Ok(());
        }
        return run_make(target);
    }
    if let Some(parallel) = parallel {
        println!("parallel requested: {parallel}");
    }
    for member in members {
        println!("==> {member}: {target}");
        let makefile = Utf8PathBuf::from(&member).join("Makefile");
        if dry_run {
            println!("would run: make -C {member} {target}");
            continue;
        }
        if makefile.exists() {
            let args = vec!["-C".to_string(), member.clone(), target.to_string()];
            let status = run_process_status("make", &args, None)?;
            if !status.success() {
                return Err(LodeError::Message(format!(
                    "workspace member {member} target {target} failed with {status}"
                )));
            }
        } else {
            println!("skip {member}: no Makefile");
        }
    }
    Ok(())
}

fn affected_workspace_members(members: &[String], changed: &[String]) -> Vec<String> {
    members
        .iter()
        .filter(|member| {
            let normalized_member = normalize_workspace_path(member);
            changed.iter().any(|path| {
                let normalized_path = normalize_workspace_path(path);
                normalized_path == normalized_member
                    || normalized_path.starts_with(&format!("{normalized_member}/"))
            })
        })
        .cloned()
        .collect()
}

fn normalize_workspace_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn workspace_graph(format: &str) -> lode_core::Result<()> {
    let members = workspace_members()?;
    match format {
        "ascii" => {
            println!("workspace");
            for member in members {
                println!("  -> {member}");
            }
        }
        "dot" => {
            println!("digraph workspace {{");
            println!("  root [label=\"workspace\"];");
            for member in members {
                println!("  root -> \"{member}\";");
            }
            println!("}}");
        }
        "json" => {
            let graph = serde_json::json!({
                "root": "workspace",
                "members": members,
                "edges": members.iter().map(|member| serde_json::json!({"from": "workspace", "to": member})).collect::<Vec<_>>()
            });
            println!("{}", json_pretty(&graph)?);
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported workspace graph format: {other}"
            )))
        }
    }
    Ok(())
}
