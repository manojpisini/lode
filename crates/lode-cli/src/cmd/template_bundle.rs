#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::TemplateBundleCommand;

fn print_apply_report(report: &lode_core::template_bundle_apply::ApplyReport) {
    for d in &report.directories_created {
        println!("  dir  {}", d);
    }
    for f in &report.files_written {
        println!("  file {}", f);
    }
    for a in &report.assets_copied {
        println!("  asset {}", a);
    }
    for f in &report.files_skipped {
        println!("  skip {}", f);
    }
    for a in &report.assets_skipped {
        println!("  skip asset {}", a);
    }
    for w in &report.warnings {
        println!("  warn {}", w);
    }
    for e in &report.errors {
        println!("  error {}", e);
    }
    println!(
        "result: {} files, {} assets, {} dirs, {} errors",
        report.files_written.len(),
        report.assets_copied.len(),
        report.directories_created.len(),
        report.errors.len()
    );
}

pub(crate) fn template_bundle_command(command: TemplateBundleCommand) -> lode_core::Result<()> {
    match command {
        TemplateBundleCommand::Apply {
            path,
            variables,
            overwrite,
            dry_run,
        } => apply(
            &path,
            variables,
            overwrite.as_deref().unwrap_or("error"),
            dry_run,
        ),
        TemplateBundleCommand::Capture {
            source,
            dest,
            mode,
            dry_run,
            no_redact,
            name,
            kind,
        } => capture(
            &source,
            &dest,
            mode.as_deref(),
            dry_run,
            !no_redact,
            name.as_deref(),
            kind.as_deref(),
        ),
        TemplateBundleCommand::Preview { source, mode } => preview(&source, mode.as_deref()),
        TemplateBundleCommand::List { path } => list(path.as_ref()),
        TemplateBundleCommand::Show { path } => show(&path),
        TemplateBundleCommand::Validate { path } => validate(&path),
        TemplateBundleCommand::Verify { path } => verify(&path),
    }
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

fn apply(
    path: &PathBuf,
    variables: Vec<String>,
    overwrite: &str,
    dry_run: bool,
) -> lode_core::Result<()> {
    let bundle_dir = find_manifest_dir(path);
    if !bundle_dir.exists() {
        return Err(lode_core::LodeError::Message(format!(
            "bundle directory not found: {}",
            bundle_dir.display()
        )));
    }

    let values = parse_variables(&variables);
    let target = std::env::current_dir()
        .map_err(|e| lode_core::LodeError::Message(format!("current dir: {e}")))?;

    let report = lode_core::template_bundle_apply::apply_bundle(
        &bundle_dir,
        &target,
        &values,
        overwrite,
        dry_run,
    )?;

    print_apply_report(&report);
    Ok(())
}

fn capture(
    source: &PathBuf,
    dest: &PathBuf,
    mode: Option<&str>,
    dry_run: bool,
    redact_secrets: bool,
    name: Option<&str>,
    _kind: Option<&str>,
) -> lode_core::Result<()> {
    let capture_mode = match mode.unwrap_or("source") {
        "minimal" => lode_core::template_bundle_capture::CaptureMode::Minimal,
        "source" => lode_core::template_bundle_capture::CaptureMode::Source,
        "development" => lode_core::template_bundle_capture::CaptureMode::Development,
        "complete" => lode_core::template_bundle_capture::CaptureMode::Complete,
        other => {
            return Err(lode_core::LodeError::Message(format!(
                "unknown capture mode: {other} (use minimal, source, development, complete)"
            )));
        }
    };

    let config = lode_core::template_bundle_capture::CaptureConfig {
        mode: capture_mode,
        template_id: name.map(|n| n.to_string()),
        template_name: name.map(|n| n.to_string()),
        destination: Some(dest.clone()),
        project: false,
        dry_run,
        redact_secrets,
        ..Default::default()
    };

    if dry_run {
        let preview = lode_core::template_bundle_capture::capture_preview(source, &config)?;
        println!("preview for {}:", preview.source.display());
        println!("  inline files:   {}", preview.inline_count);
        println!("  assets:         {}", preview.asset_count);
        println!("  directories:    {}", preview.directory_count);
        println!("  estimated size: {} KB", preview.estimated_size_kb);
        println!("  variables:      {}", preview.variables.join(", "));
        if !preview.secrets_found.is_empty() {
            println!("  secrets found:  {}", preview.secrets_found.len());
        }
        for w in &preview.warnings {
            println!("  warning: {w}");
        }
        return Ok(());
    }

    let receipt = lode_core::template_bundle_capture::capture_template(source, &config)?;

    // Write receipt to destination
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| lode_core::LodeError::Message(format!("create dest dir: {e}")))?;
    }
    let receipt_path = dest.join("capture-receipt.json");
    let receipt_json = serde_json::to_string_pretty(&receipt)
        .map_err(|e| lode_core::LodeError::Message(e.to_string()))?;
    std::fs::write(&receipt_path, receipt_json)
        .map_err(|e| lode_core::LodeError::Message(format!("write receipt: {e}")))?;

    println!("captured {} -> {}", source.display(), dest.display());
    println!("  inline files: {}", receipt.inline_files.len());
    println!("  assets copied: {}", receipt.assets_copied.len());
    println!("  excluded: {}", receipt.excluded_paths.len());
    println!("  secrets: {}", receipt.secret_findings.len());
    println!("  receipt: {}", receipt_path.display());
    Ok(())
}

fn preview(source: &PathBuf, mode: Option<&str>) -> lode_core::Result<()> {
    let capture_mode = match mode.unwrap_or("source") {
        "minimal" => lode_core::template_bundle_capture::CaptureMode::Minimal,
        "source" => lode_core::template_bundle_capture::CaptureMode::Source,
        "development" => lode_core::template_bundle_capture::CaptureMode::Development,
        "complete" => lode_core::template_bundle_capture::CaptureMode::Complete,
        other => {
            return Err(lode_core::LodeError::Message(format!(
                "unknown capture mode: {other}"
            )));
        }
    };

    let config = lode_core::template_bundle_capture::CaptureConfig {
        mode: capture_mode,
        ..Default::default()
    };

    let preview = lode_core::template_bundle_capture::capture_preview(source, &config)?;
    println!("source: {}", preview.source.display());
    println!("template id:   {}", preview.template_id);
    println!("template name: {}", preview.template_name);
    println!("inline files:  {}", preview.inline_count);
    println!("assets:        {}", preview.asset_count);
    println!("directories:   {}", preview.directory_count);
    println!("estimated:     {} KB", preview.estimated_size_kb);
    println!();
    println!("classifications:");
    for fc in &preview.file_classifications {
        println!("  {:?} {:>8} {}", fc.classification, fc.size_bytes, fc.path);
    }
    if !preview.variables.is_empty() {
        println!();
        println!("inferred variables: {}", preview.variables.join(", "));
    }
    if !preview.secrets_found.is_empty() {
        println!();
        println!("secrets: {}", preview.secrets_found.len());
        for s in &preview.secrets_found {
            println!("  {}:{}", s.path, s.line);
        }
    }
    for w in &preview.warnings {
        println!("warning: {w}");
    }
    Ok(())
}

fn list(path: Option<&PathBuf>) -> lode_core::Result<()> {
    let search_dir: PathBuf = if let Some(p) = path {
        p.clone()
    } else {
        lode_core::global_dir()?
            .into_std_path_buf()
            .join("templates")
    };

    if !search_dir.exists() {
        println!("no template bundles found in {}", search_dir.display());
        return Ok(());
    }

    let mut found = false;
    for entry in std::fs::read_dir(&search_dir)
        .map_err(|e| lode_core::LodeError::Message(format!("read dir: {e}")))?
    {
        let entry = entry.map_err(|e| lode_core::LodeError::Message(format!("entry: {e}")))?;
        let p = entry.path();
        if p.is_dir() {
            // Look for .toml with matching name
            let dirname = p
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            let manifest_path = p.join(format!("{dirname}.toml"));
            let manifest_path2 = p.join(format!(
                "{}.toml",
                p.file_name().unwrap_or_default().to_string_lossy()
            ));
            if manifest_path.exists() || manifest_path2.exists() {
                println!("{}", p.display());
                found = true;
            }
        }
    }
    if !found {
        println!("no template bundles found in {}", search_dir.display());
    }
    Ok(())
}

fn show(path: &PathBuf) -> lode_core::Result<()> {
    let bundle_dir = find_manifest_dir(path);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)?;
    let toml_str = toml::to_string_pretty(&manifest)
        .map_err(|e| lode_core::LodeError::Message(e.to_string()))?;
    println!("{}", toml_str);
    Ok(())
}

fn validate(path: &PathBuf) -> lode_core::Result<()> {
    let bundle_dir = find_manifest_dir(path);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)?;
    let errors = manifest.validate(&bundle_dir);
    if errors.is_empty() {
        println!("template bundle is valid: {}", bundle_dir.display());
    } else {
        for e in &errors {
            println!("error: {e}");
        }
        return Err(lode_core::LodeError::Message(format!(
            "template bundle has {} validation error(s)",
            errors.len()
        )));
    }

    if !manifest.assets.is_empty() {
        let assets_dir = bundle_dir.join("assets");
        if !assets_dir.exists() {
            println!(
                "warning: assets/ directory not found ({} assets declared)",
                manifest.assets.len()
            );
        }
    }
    Ok(())
}

fn verify(path: &PathBuf) -> lode_core::Result<()> {
    let bundle_dir = find_manifest_dir(path);
    let manifest = lode_core::template_bundle::load_template_bundle(&bundle_dir)?;
    let errors = manifest.validate(&bundle_dir);
    if !errors.is_empty() {
        for e in &errors {
            println!("validation error: {e}");
        }
        return Err(lode_core::LodeError::Message(format!(
            "template bundle has {} validation error(s)",
            errors.len()
        )));
    }

    let mut all_ok = true;

    for (i, file) in manifest.files.iter().enumerate() {
        let fpath = bundle_dir.join(format!("files/file_{i}"));
        if fpath.exists() {
            println!("  file[{}] {} -> exists", i, file.path);
        } else {
            println!("  file[{}] {} -> embedded content (ok)", i, file.path);
        }
    }

    let assets_dir = bundle_dir.join("assets");
    for asset in &manifest.assets {
        let asset_path = assets_dir.join(&asset.source);
        if asset_path.exists() {
            println!("  asset {} -> exists", asset.source);
        } else {
            println!("  asset {} -> MISSING", asset.source);
            all_ok = false;
        }
    }

    for hook in &manifest.hooks {
        let hook_name = hook.kind.as_deref().unwrap_or("shell");
        println!("  hook [{hook_name}] -> {}", hook.run);
    }

    if all_ok {
        println!("all files and assets verified: {}", bundle_dir.display());
    } else {
        println!("some files/assets are missing");
    }
    Ok(())
}

/// Parse `key=value` pairs into a HashMap
fn parse_variables(vars: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for v in vars {
        if let Some(eq) = v.find('=') {
            let key = v[..eq].trim().to_string();
            let val = v[eq + 1..].trim().to_string();
            map.insert(key, val);
        }
    }
    map
}
