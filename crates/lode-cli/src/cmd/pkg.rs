#![deny(unsafe_code)]

use crate::{
    cmd, current_dir, detect_package_manager, package_audit_args, package_command,
    package_dependencies, package_manifest_inventory, package_outdated_args, package_update_args,
    run_process_status, OutputFormat, PackageDependency, PackageOperationPlan, PkgCommand,
    ScanCommand,
};
use camino::Utf8PathBuf;
use lode_core::{LodeError, ValidatedRoot};
use serde_json::json;

pub(crate) fn pkg(command: PkgCommand) -> lode_core::Result<()> {
    let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
    match command {
        PkgCommand::List { format } => print_package_inventory(&format)?,
        PkgCommand::Outdated { dry_run, format } => run_or_print_package_operation(
            &PackageOperationPlan::new("outdated", &manager, package_outdated_args(&manager)?),
            dry_run,
            &format,
        )?,
        PkgCommand::Update { name, dry_run } => {
            let args = package_update_args(&manager, name.as_deref())?;
            if dry_run {
                println!(
                    "would run: {} {}",
                    package_command(&manager),
                    args.join(" ")
                );
            } else {
                run_package_manager(&manager, args)?;
            }
        }
        PkgCommand::Audit {
            dry_run,
            format,
            fail_on,
        } => {
            let plan = PackageOperationPlan::new(
                "audit",
                &manager,
                package_audit_args(&manager, fail_on.as_deref())?,
            );
            run_or_print_package_operation(&plan, dry_run, &format)?;
            if dry_run {
                println!("would run: lode scan secrets {}", current_dir()?);
            } else {
                cmd::scan::scan(ScanCommand::Secrets {
                    path: Some(current_dir()?),
                    staged: false,
                    output: OutputFormat::Table,
                    quiet: false,
                })?;
            }
        }
        PkgCommand::Why {
            name,
            dry_run,
            format,
        } => package_explain("why", &manager, &name, dry_run, &format)?,
        PkgCommand::Info {
            name,
            dry_run,
            format,
        } => package_explain("info", &manager, &name, dry_run, &format)?,
        PkgCommand::Lock { dry_run } => {
            let args = package_lock_args(&manager)?;
            if dry_run {
                println!(
                    "would run: {} {}",
                    package_command(&manager),
                    args.join(" ")
                );
            } else {
                run_package_manager(&manager, args)?;
            }
        }
        PkgCommand::Graph { format } => package_graph(&format)?,
        PkgCommand::Clean { dry_run } => {
            for path in [
                "target",
                "node_modules",
                ".pytest_cache",
                "__pycache__",
                "dist",
                "build",
            ] {
                let path = Utf8PathBuf::from(path);
                if path.exists() {
                    if dry_run {
                        println!("would remove {path}");
                    } else if path.is_dir() {
                        ValidatedRoot::new(current_dir()?)?.remove_dir_all(&path)?;
                        println!("removed {path}");
                    }
                }
            }
        }
    }
    Ok(())
}

fn run_package_manager(manager: &str, args: Vec<String>) -> lode_core::Result<()> {
    if manager == "unknown" {
        return Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        ));
    }
    let command = package_command(manager);
    let status = run_process_status(command, &args, None)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "{command} failed with {status}"
        )))
    }
}

fn print_package_inventory(format: &str) -> lode_core::Result<()> {
    let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
    let manifests = package_manifest_inventory();
    match format {
        "table" => {
            println!("manager: {manager}");
            for manifest in &manifests {
                println!(
                    "{}\t{}\t{} dependencies",
                    manifest.file,
                    manifest.kind,
                    manifest.dependencies.len()
                );
                for dependency in &manifest.dependencies {
                    let version = dependency.version.as_deref().unwrap_or("*");
                    println!("  {} {} ({})", dependency.name, version, dependency.scope);
                }
            }
            Ok(())
        }
        "json" => {
            let inventory = json!({
                "manager": manager,
                "manifests": manifests,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&inventory)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
            Ok(())
        }
        other => Err(LodeError::Message(format!(
            "unsupported package output format: {other}"
        ))),
    }
}

fn package_explain(
    operation: &str,
    manager: &str,
    name: &str,
    dry_run: bool,
    format: &str,
) -> lode_core::Result<()> {
    let matches = package_dependencies()
        .into_iter()
        .filter(|dependency| package_name_matches(&dependency.name, name))
        .collect::<Vec<_>>();
    if !matches.is_empty() {
        print_package_matches(operation, manager, name, &matches, format)?;
        if dry_run {
            let args = match operation {
                "why" => package_why_args(manager, name)?,
                "info" => package_info_args(manager, name)?,
                _ => Vec::new(),
            };
            println!("would run: {} {}", package_command(manager), args.join(" "));
        }
        return Ok(());
    }
    let args = match operation {
        "why" => package_why_args(manager, name)?,
        "info" => package_info_args(manager, name)?,
        _ => Vec::new(),
    };
    run_or_print_package_manager(manager, args, dry_run)
}

fn print_package_matches(
    operation: &str,
    manager: &str,
    name: &str,
    matches: &[PackageDependency],
    format: &str,
) -> lode_core::Result<()> {
    match format {
        "table" => {
            println!("{operation}: {name}");
            println!("manager: {manager}");
            for dependency in matches {
                let version = dependency.version.as_deref().unwrap_or("*");
                println!(
                    "project -> {} -> {} {} ({})",
                    dependency.manifest, dependency.name, version, dependency.scope
                );
            }
            Ok(())
        }
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": operation,
                    "manager": manager,
                    "query": name,
                    "matches": matches,
                }))
                .map_err(|error| LodeError::Message(error.to_string()))?
            );
            Ok(())
        }
        other => Err(LodeError::Message(format!(
            "unsupported package output format: {other}"
        ))),
    }
}

fn package_name_matches(candidate: &str, query: &str) -> bool {
    candidate == query
        || candidate.contains(query)
        || candidate
            .rsplit_once(':')
            .map(|(_, artifact)| artifact == query)
            .unwrap_or(false)
        || candidate
            .rsplit_once('/')
            .map(|(_, tail)| tail == query)
            .unwrap_or(false)
}

fn run_or_print_package_operation(
    plan: &PackageOperationPlan,
    dry_run: bool,
    format: &str,
) -> lode_core::Result<()> {
    if dry_run {
        print_package_plan(plan, format)
    } else {
        run_package_manager(&plan.manager, plan.args.clone())
    }
}

fn print_package_plan(plan: &PackageOperationPlan, format: &str) -> lode_core::Result<()> {
    match format {
        "table" => {
            println!("would run: {}", plan.command_line());
            Ok(())
        }
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(plan)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
            Ok(())
        }
        other => Err(LodeError::Message(format!(
            "unsupported package output format: {other}"
        ))),
    }
}

fn run_or_print_package_manager(
    manager: &str,
    args: Vec<String>,
    dry_run: bool,
) -> lode_core::Result<()> {
    if dry_run {
        println!("would run: {} {}", package_command(manager), args.join(" "));
        Ok(())
    } else {
        run_package_manager(manager, args)
    }
}

fn package_lock_args(manager: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["generate-lockfile".into()]),
        "npm" => Ok(vec![
            "install".into(),
            "--package-lock-only".into(),
            "--ignore-scripts".into(),
        ]),
        "pnpm" => Ok(vec!["install".into(), "--lockfile-only".into()]),
        "yarn" => Ok(vec!["install".into(), "--mode=update-lockfile".into()]),
        "bun" => Ok(vec!["install".into(), "--lockfile-only".into()]),
        "uv" => Ok(vec!["lock".into()]),
        "pip" => Ok(vec!["freeze".into()]),
        "go" => Ok(vec!["mod".into(), "tidy".into()]),
        "bundler" => Ok(vec!["lock".into()]),
        "gradle" => Ok(vec!["dependencies".into(), "--write-locks".into()]),
        "maven" => Ok(vec![
            "dependency:go-offline".into(),
            "-DgenerateBackupPoms=false".into(),
        ]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_why_args(manager: &str, name: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["tree".into(), "-i".into(), name.into()]),
        "npm" => Ok(vec!["explain".into(), name.into()]),
        "pnpm" | "yarn" => Ok(vec!["why".into(), name.into()]),
        "bun" => Ok(vec!["pm".into(), "why".into(), name.into()]),
        "uv" => Ok(vec!["pip".into(), "show".into(), name.into()]),
        "pip" => Ok(vec!["show".into(), name.into()]),
        "go" => Ok(vec!["mod".into(), "why".into(), name.into()]),
        "bundler" => Ok(vec!["why".into(), name.into()]),
        "gradle" => Ok(vec![
            "dependencyInsight".into(),
            "--dependency".into(),
            name.into(),
        ]),
        "maven" => Ok(vec!["dependency:tree".into(), format!("-Dincludes={name}")]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_info_args(manager: &str, name: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["search".into(), name.into()]),
        "npm" | "pnpm" | "yarn" | "bun" => Ok(vec!["info".into(), name.into()]),
        "uv" => Ok(vec!["pip".into(), "show".into(), name.into()]),
        "pip" => Ok(vec!["show".into(), name.into()]),
        "go" => Ok(vec!["list".into(), "-m".into(), name.into()]),
        "bundler" => Ok(vec!["info".into(), name.into()]),
        "gradle" => Ok(vec![
            "dependencyInsight".into(),
            "--dependency".into(),
            name.into(),
        ]),
        "maven" => Ok(vec!["dependency:tree".into(), format!("-Dincludes={name}")]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_graph(format: &str) -> lode_core::Result<()> {
    let manifests = [
        ("Cargo.toml", "cargo"),
        ("package.json", "node"),
        ("pyproject.toml", "python"),
        ("go.mod", "go"),
        ("build.gradle", "gradle"),
        ("settings.gradle", "gradle"),
        ("pom.xml", "maven"),
    ];
    let found = manifests
        .iter()
        .filter(|(file, _)| Utf8PathBuf::from(*file).exists())
        .map(|(file, kind)| serde_json::json!({ "file": file, "kind": kind }))
        .collect::<Vec<_>>();
    let manager = detect_package_manager();
    match format {
        "json" => {
            let graph = json!({
                "manager": manager,
                "manifests": found,
                "edges": found.iter().filter_map(|manifest| {
                    Some(json!({
                        "from": "project",
                        "to": manifest.get("kind")?.as_str()?,
                        "label": manifest.get("file")?.as_str()?
                    }))
                }).collect::<Vec<_>>()
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&graph)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
        }
        "ascii" => {
            println!(
                "project manager={}",
                manager.as_deref().unwrap_or("unknown")
            );
            for (file, kind) in manifests {
                if Utf8PathBuf::from(file).exists() {
                    println!("`- {kind} ({file})");
                }
            }
        }
        "dot" => {
            println!("digraph packages {{");
            println!(
                "  project [label=\"project\\nmanager={}\"];",
                manager.as_deref().unwrap_or("unknown")
            );
            for (file, kind) in manifests {
                if Utf8PathBuf::from(file).exists() {
                    println!("  project -> {kind} [label=\"{file}\"];");
                }
            }
            println!("}}");
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported package graph format: {other}"
            )))
        }
    }
    Ok(())
}
