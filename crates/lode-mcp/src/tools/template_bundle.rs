use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::schema::{optional_string_schema, string_schema, tool_input_schema};

use super::Tool;

fn validated(path: &str) -> Result<std::path::PathBuf, String> {
    let p = PathBuf::from(path);
    let p = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map(|cwd| cwd.join(&p)).unwrap_or(p)
    };
    lode_core::ValidatedRoot::new(&p)
        .map(|r| r.path().to_path_buf())
        .map_err(|e| format!("invalid path '{}': {e}", path))
}

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "lode_template_bundle_list".to_string(),
            description: "List available template bundles in a directory".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Directory to scan for bundles (default: global templates dir)",
                optional_string_schema(),
            )]),
        },
        Tool {
            name: "lode_template_bundle_show".to_string(),
            description: "Show TOML manifest of a template bundle".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Path to the template bundle directory or its manifest",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_template_bundle_validate".to_string(),
            description: "Validate a template bundle's manifest and assets directory".to_string(),
            input_schema: tool_input_schema(vec![(
                "path",
                "Path to the template bundle directory",
                string_schema(),
            )]),
        },
        Tool {
            name: "lode_template_bundle_preview".to_string(),
            description: "Preview a directory capture without writing (classify files, detect secrets, infer variables)".to_string(),
            input_schema: tool_input_schema(vec![
                ("source", "Source directory to preview", string_schema()),
                ("mode", "Capture mode: minimal, source (default), development, complete", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_template_bundle_apply".to_string(),
            description: "Apply/render a template bundle into the target directory".to_string(),
            input_schema: tool_input_schema(vec![
                ("path", "Path to the template bundle directory", string_schema()),
                ("variables", "key=value pairs for template variables", optional_string_schema()),
                ("overwrite", "Overwrite policy: skip, error (default), replace", optional_string_schema()),
                ("dry_run", "If true, report what would happen without writing", optional_string_schema()),
                ("target", "Target directory (default: current dir)", optional_string_schema()),
            ]),
        },
        Tool {
            name: "lode_template_bundle_capture".to_string(),
            description: "Capture a directory as a template bundle".to_string(),
            input_schema: tool_input_schema(vec![
                ("source", "Source directory to capture", string_schema()),
                ("dest", "Destination path for the bundle directory", string_schema()),
                ("mode", "Capture mode: minimal, source (default), development, complete", optional_string_schema()),
                ("name", "Template name/ID override", optional_string_schema()),
                ("dry_run", "If true, preview only without writing", optional_string_schema()),
                ("no_redact", "If true, do not redact secrets in captured content", optional_string_schema()),
            ]),
        },
    ]
}

fn find_manifest_dir(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else if path.is_file() {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

pub fn lode_template_bundle_list(args: &Value) -> Result<Value, String> {
    let search_dir: PathBuf = match args.get("path").and_then(|v| v.as_str()) {
        Some(s) => validated(s)?,
        None => lode_core::global_dir()
            .ok()
            .map(|g| g.into_std_path_buf().join("templates"))
            .unwrap_or_else(|| PathBuf::from(".")),
    };

    if !search_dir.exists() {
        return Ok(json!({
            "bundles": [],
            "count": 0,
            "search_dir": search_dir.to_string_lossy(),
        }));
    }

    let mut bundles = Vec::new();
    for entry in std::fs::read_dir(&search_dir).map_err(|e| format!("read dir: {e}"))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let p = entry.path();
        if p.is_dir() {
            let dirname = p
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
                .to_string();
            let manifest_path = p.join(format!("{dirname}.toml"));
            if manifest_path.exists() {
                bundles.push(json!({
                    "path": p.to_string_lossy(),
                    "manifest": manifest_path.to_string_lossy(),
                }));
            }
        }
    }

    Ok(json!({
        "bundles": bundles,
        "count": bundles.len(),
        "search_dir": search_dir.to_string_lossy(),
    }))
}

pub fn lode_template_bundle_show(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let validated_path = validated(path)?;
    let bundle_dir = find_manifest_dir(&validated_path);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)
        .map_err(|e| format!("load bundle: {e}"))?;
    let toml_str = toml::to_string_pretty(&manifest).map_err(|e| format!("serialize: {e}"))?;
    Ok(json!({
        "manifest": toml_str,
        "path": bundle_dir.to_string_lossy(),
    }))
}

pub fn lode_template_bundle_validate(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let validated_path = validated(path)?;
    let bundle_dir = find_manifest_dir(&validated_path);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)
        .map_err(|e| format!("load bundle: {e}"))?;
    let errors = manifest.validate(&bundle_dir);
    let assets_dir = bundle_dir.join("assets");
    let assets_exist = if !manifest.assets.is_empty() {
        assets_dir.exists()
    } else {
        true
    };

    Ok(json!({
        "valid": errors.is_empty() && assets_exist,
        "errors": errors,
        "assets_dir_exists": assets_exist,
        "path": bundle_dir.to_string_lossy(),
    }))
}

pub fn lode_template_bundle_preview(args: &Value) -> Result<Value, String> {
    let source = args["source"]
        .as_str()
        .ok_or("Missing required argument: source")?;
    let validated_source = validated(source)?;
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("source");

    let capture_mode = match mode {
        "minimal" => lode_core::template_bundle_capture::CaptureMode::Minimal,
        "source" => lode_core::template_bundle_capture::CaptureMode::Source,
        "development" => lode_core::template_bundle_capture::CaptureMode::Development,
        "complete" => lode_core::template_bundle_capture::CaptureMode::Complete,
        other => return Err(format!("unknown capture mode: {other}")),
    };

    let config = lode_core::template_bundle_capture::CaptureConfig {
        mode: capture_mode,
        ..Default::default()
    };

    let preview = lode_core::template_bundle_capture::capture_preview(&validated_source, &config)
        .map_err(|e| format!("preview: {e}"))?;

    let classifications: Vec<Value> = preview
        .file_classifications
        .iter()
        .map(|fc| {
            json!({
                "path": fc.path.to_string(),
                "kind": format!("{:?}", fc.classification),
                "size_bytes": fc.size_bytes,
            })
        })
        .collect();

    Ok(json!({
        "source": preview.source.to_string_lossy(),
        "template_id": preview.template_id,
        "template_name": preview.template_name,
        "inline_count": preview.inline_count,
        "asset_count": preview.asset_count,
        "directory_count": preview.directory_count,
        "estimated_size_kb": preview.estimated_size_kb,
        "variables": preview.variables,
        "secrets_found": preview.secrets_found,
        "classifications": classifications,
        "warnings": preview.warnings,
        "mode": mode,
    }))
}

pub fn lode_template_bundle_apply(args: &Value) -> Result<Value, String> {
    let path = args["path"]
        .as_str()
        .ok_or("Missing required argument: path")?;
    let validated_path = validated(path)?;
    let variables_val = args.get("variables").and_then(|v| v.as_str()).unwrap_or("");
    let overwrite = args
        .get("overwrite")
        .and_then(|v| v.as_str())
        .unwrap_or("error");
    let dry_run = args
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .map(|s| validated(s))
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let bundle_dir = find_manifest_dir(&validated_path);
    if !bundle_dir.exists() {
        return Err(format!(
            "bundle directory not found: {}",
            bundle_dir.display()
        ));
    }

    let values: HashMap<String, String> = if variables_val.is_empty() {
        HashMap::new()
    } else {
        variables_val
            .split(',')
            .filter_map(|pair| {
                let eq = pair.find('=')?;
                Some((
                    pair[..eq].trim().to_string(),
                    pair[eq + 1..].trim().to_string(),
                ))
            })
            .collect()
    };

    let report = lode_core::template_bundle_apply::apply_bundle(
        &bundle_dir,
        &target,
        &values,
        overwrite,
        dry_run,
    )
    .map_err(|e| format!("apply: {e}"))?;

    Ok(json!({
        "files_written": report.files_written,
        "assets_copied": report.assets_copied,
        "directories_created": report.directories_created,
        "files_skipped": report.files_skipped,
        "assets_skipped": report.assets_skipped,
        "warnings": report.warnings,
        "errors": report.errors,
        "dry_run": dry_run,
    }))
}

pub fn lode_template_bundle_capture(args: &Value) -> Result<Value, String> {
    let source = args["source"]
        .as_str()
        .ok_or("Missing required argument: source")?;
    let validated_source = validated(source)?;
    let dest = args["dest"]
        .as_str()
        .ok_or("Missing required argument: dest")?;
    let validated_dest = validated(dest)?;
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("source");
    let name = args.get("name").and_then(|v| v.as_str());
    let dry_run = args
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let no_redact = args
        .get("no_redact")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let capture_mode = match mode {
        "minimal" => lode_core::template_bundle_capture::CaptureMode::Minimal,
        "source" => lode_core::template_bundle_capture::CaptureMode::Source,
        "development" => lode_core::template_bundle_capture::CaptureMode::Development,
        "complete" => lode_core::template_bundle_capture::CaptureMode::Complete,
        other => return Err(format!("unknown capture mode: {other}")),
    };

    let dest_path = validated_dest.clone();

    let config = lode_core::template_bundle_capture::CaptureConfig {
        mode: capture_mode,
        template_id: name.map(|n| n.to_string()),
        template_name: name.map(|n| n.to_string()),
        destination: Some(dest_path.clone()),
        project: false,
        dry_run,
        redact_secrets: !no_redact,
        ..Default::default()
    };

    if dry_run {
        let preview =
            lode_core::template_bundle_capture::capture_preview(&validated_source, &config)
                .map_err(|e| format!("preview: {e}"))?;
        let classifications: Vec<Value> = preview
            .file_classifications
            .iter()
            .map(|fc| {
                json!({
                    "path": fc.path.to_string(),
                    "kind": format!("{:?}", fc.classification),
                    "size_bytes": fc.size_bytes,
                })
            })
            .collect();
        return Ok(json!({
            "dry_run": true,
            "source": preview.source.to_string_lossy(),
            "inline_count": preview.inline_count,
            "asset_count": preview.asset_count,
            "directory_count": preview.directory_count,
            "estimated_size_kb": preview.estimated_size_kb,
            "variables": preview.variables,
            "secrets_found": preview.secrets_found,
            "classifications": classifications,
            "warnings": preview.warnings,
        }));
    }

    // Ensure parent dir exists
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dest dir: {e}"))?;
    }

    let receipt = lode_core::template_bundle_capture::capture_template(&validated_source, &config)
        .map_err(|e| format!("capture: {e}"))?;

    Ok(json!({
        "dry_run": false,
        "source": receipt.source.to_string_lossy(),
        "destination": receipt.destination.to_string_lossy(),
        "inline_files": receipt.inline_files.len(),
        "assets_copied": receipt.assets_copied.len(),
        "excluded_paths": receipt.excluded_paths.len(),
        "secret_findings": receipt.secret_findings.len(),
        "operation_id": receipt.operation_id,
    }))
}
