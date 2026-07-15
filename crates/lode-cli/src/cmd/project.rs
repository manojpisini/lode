#![deny(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use camino::Utf8PathBuf;
use lode_core::{load_project_config, load_scaffold_lock, LodeError, ProjectConfig, ScaffoldLock};

use crate::ProjectCommand;

pub(crate) use crate::OutputFormat;

pub(crate) fn project_command(command: ProjectCommand) -> lode_core::Result<()> {
    match command {
        ProjectCommand::Plan { intent, output } => project_plan(intent, output),
        ProjectCommand::Apply {
            plan_id,
            dry_run,
            output,
        } => project_apply(&plan_id, dry_run, output),
        ProjectCommand::Diff { output } => project_diff(output),
        ProjectCommand::Reconcile { dry_run, output } => project_reconcile(dry_run, output),
        ProjectCommand::Explain { output } => project_explain(output),
    }
}

fn project_dir() -> lode_core::Result<Utf8PathBuf> {
    let dir = std::env::current_dir().map_err(|e| LodeError::Message(e.to_string()))?;
    Utf8PathBuf::try_from(dir).map_err(|_| LodeError::Message("invalid path".to_string()))
}

fn load_manifest(dir: &Utf8PathBuf) -> lode_core::Result<Option<ProjectConfig>> {
    let path = dir.join(".lode/project.toml");
    if !path.exists() {
        return Ok(None);
    }
    load_project_config(dir).map(Some)
}

fn load_lock(dir: &Utf8PathBuf) -> lode_core::Result<Option<ScaffoldLock>> {
    let path = dir.join(".lode/scaffold.lock");
    if !path.exists() {
        return Ok(None);
    }
    load_scaffold_lock(dir).map(Some)
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{secs}")
}

fn project_plan(intent: Option<String>, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?;
    let lock = load_lock(&dir)?;

    if output.should_use_json() {
        let plan = serde_json::json!({
            "project_dir": dir.as_str(),
            "has_manifest": manifest.is_some(),
            "has_lock": lock.is_some(),
            "intent": intent,
            "timestamp": timestamp(),
        });
        println!("{}", serde_json::to_string_pretty(&plan).unwrap());
    } else {
        println!("project plan for: {}", dir);
        if let Some(ref cfg) = manifest {
            println!("  manifest: {} v{}", cfg.project.name, cfg.schema_version);
        } else {
            println!("  manifest: none (not a lode project)");
        }
        println!(
            "  lock: {}",
            if lock.is_some() { "present" } else { "absent" }
        );
        println!(
            "  intent: {}",
            intent.as_deref().unwrap_or("sync project state")
        );
        println!();
        println!("use `lode project diff` to see differences");
        if manifest.is_some() {
            println!("use `lode project apply <plan-id>` to execute a plan");
        }
    }
    Ok(())
}

fn project_apply(plan_id: &str, dry_run: bool, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?.ok_or_else(|| {
        LodeError::Message("no project manifest found -- run `lode init` first".to_string())
    })?;

    if output.should_use_json() {
        let report = serde_json::json!({
            "plan_id": plan_id,
            "project": manifest.project.name,
            "dry_run": dry_run,
            "status": "not_implemented",
            "timestamp": timestamp(),
        });
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        if dry_run {
            println!(
                "[dry-run] would apply plan {plan_id} to project {}",
                manifest.project.name
            );
        } else {
            println!(
                "applying plan {plan_id} to project {}",
                manifest.project.name
            );
            println!("  (plan application not yet implemented -- use `lode plan apply {plan_id}` instead)");
        }
    }
    Ok(())
}

fn project_diff(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?;
    let lock = load_lock(&dir)?;

    let mut diffs: Vec<serde_json::Value> = Vec::new();

    // Check if project dir exists
    if !dir.join(".lode").exists() {
        diffs.push(serde_json::json!({
            "type": "missing",
            "path": ".lode",
            "expected": "directory",
            "status": "not a lode project"
        }));
    }

    // Check manifest
    if manifest.is_none() {
        diffs.push(serde_json::json!({
            "type": "missing",
            "path": ".lode/project.toml",
            "expected": "project manifest",
            "status": "absent"
        }));
    }

    // Check lock
    if lock.is_none() && manifest.is_some() {
        diffs.push(serde_json::json!({
            "type": "missing",
            "path": ".lode/scaffold.lock",
            "expected": "scaffold lock",
            "status": "absent"
        }));
    }

    // Check lock entries against filesystem
    if let Some(ref lock) = lock {
        for entry in &lock.entries {
            let full_path = dir.join(&entry.destination);
            if !full_path.exists() {
                diffs.push(serde_json::json!({
                    "type": "missing_file",
                    "path": entry.destination.as_str(),
                    "template": entry.template,
                    "status": "deleted"
                }));
            }
        }
    }

    if output.should_use_json() {
        println!("{}", serde_json::to_string_pretty(&diffs).unwrap());
    } else {
        if diffs.is_empty() {
            println!("project is up to date with manifest");
        } else {
            println!("found {} difference(s):", diffs.len());
            for diff in &diffs {
                let kind = diff.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                let path = diff.get("path").and_then(|v| v.as_str()).unwrap_or("?");
                let status = diff.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  [{kind}] {path} ({status})");
            }
        }
    }
    Ok(())
}

fn project_reconcile(dry_run: bool, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?
        .ok_or_else(|| LodeError::Message("no project manifest found".to_string()))?;
    let lock = load_lock(&dir)?;

    if output.should_use_json() {
        let result = serde_json::json!({
            "project": manifest.project.name,
            "dry_run": dry_run,
            "manifest_valid": true,
            "lock_present": lock.is_some(),
            "timestamp": timestamp(),
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("reconciling project: {}", manifest.project.name);
        if dry_run {
            println!(
                "  [dry-run] checking {} components...",
                manifest.project.components.len()
            );
            for component in &manifest.project.components {
                println!("    would verify component: {component}");
            }
            if let Some(ref lock) = lock {
                println!("    would check {} tracked files", lock.entries.len());
            }
            println!("  run without --dry-run to apply fixes");
        } else {
            println!(
                "  project is valid ({})",
                if lock.is_some() {
                    "tracked"
                } else {
                    "untracked"
                }
            );
            println!("  run `lode project diff` to see any discrepancies");
        }
    }
    Ok(())
}

fn project_explain(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?;
    let lock = load_lock(&dir)?;

    if output.should_use_json() {
        let info = serde_json::json!({
            "project_dir": dir.as_str(),
            "is_lode_project": manifest.is_some(),
            "manifest": manifest,
            "lock_entries": lock.as_ref().map(|l| l.entries.len()),
            "timestamp": timestamp(),
        });
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
    } else {
        if let Some(ref cfg) = manifest {
            println!("Project: {}", cfg.project.name);
            println!("  Profile: {}", cfg.project.profile);
            println!("  Created: {}", cfg.project.created_at);
            println!("  Components ({}):", cfg.project.components.len());
            for component in &cfg.project.components {
                println!("    - {component}");
            }
            if let Some(ref lang) = cfg.project.language {
                println!("  Language: {lang}");
            }
            if let Some(ref toolchain) = cfg.project.toolchain {
                println!("  Toolchain:");
                for tc in toolchain {
                    println!("    - {tc}");
                }
            }
            if let Some(ref assets) = cfg.project.assets {
                println!("  Assets ({}):", assets.len());
                for asset in assets {
                    println!("    - {asset}");
                }
            }
            if let Some(ref deps) = cfg.project.dependencies {
                println!("  Dependencies ({}):", deps.len());
                for dep in deps {
                    if let Some(ref ver) = dep.version {
                        println!("    - {} v{ver}", dep.name);
                    } else {
                        println!("    - {}", dep.name);
                    }
                }
            }
            if let Some(ref lock) = lock {
                println!("  Tracked files: {}", lock.entries.len());
            } else {
                println!("  Tracked files: none (no scaffold.lock)");
            }
        } else {
            println!("Not a LODE project: {}", dir);
            println!("  Run `lode init` to create a project manifest");
        }
    }
    Ok(())
}
