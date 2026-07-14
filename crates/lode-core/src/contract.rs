use std::fs;

use camino::Utf8PathBuf;
use serde::Serialize;

use crate::{
    build_catalog, config::LodeConfig, install::global_asset_dir, AssetCatalogEntry, Result,
};

#[derive(Debug, Clone, Serialize)]
pub struct AssetTestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<AssetTestResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetTestResult {
    pub id: String,
    pub kind: String,
    pub passed: bool,
    pub failures: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn test_assets(config: &LodeConfig, id_filter: Option<&str>) -> Result<AssetTestReport> {
    let catalog = build_catalog(config);
    let mut results = Vec::new();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for entry in &catalog.entries {
        if let Some(filter) = id_filter {
            if !entry.id.contains(filter) {
                continue;
            }
        }
        let result = test_single_asset(config, entry);
        if result.passed {
            passed += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }

    if results.is_empty() {
        return Ok(AssetTestReport {
            total: 0,
            passed: 0,
            failed: 0,
            results,
        });
    }

    Ok(AssetTestReport {
        total: results.len(),
        passed,
        failed,
        results,
    })
}

fn test_single_asset(config: &LodeConfig, entry: &AssetCatalogEntry) -> AssetTestResult {
    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    // Contract: must have non-empty summary
    if entry.summary.is_empty() || entry.summary == format!("{} {}", entry.kind, "entry") {
        failures.push("missing or generic summary".to_string());
    }

    // Contract: must have intents
    if entry.intents.is_empty() {
        warnings.push("no intents defined".to_string());
    }

    // Contract: status must be valid
    if !["experimental", "preview", "stable", "deprecated", "retired"]
        .contains(&entry.status.as_str())
    {
        failures.push(format!("invalid status: {}", entry.status));
    }

    // Contract: lifecycle consistency
    if entry.status == "deprecated" && entry.deprecation_replacement.is_none() {
        warnings.push("deprecated without replacement".to_string());
    }

    // Contract: maturity must be set
    if entry.maturity.is_empty() {
        warnings.push("no maturity set".to_string());
    }

    // Contract: verify source files exist for templates/recipes/profiles/commands
    if entry.kind == "template" || entry.kind == "recipe" || entry.kind == "profile" {
        let path_part = entry.id.trim_start_matches(&format!("{}://", entry.kind));
        let asset_root = global_asset_dir("templates").unwrap_or_else(|_| Utf8PathBuf::new());
        let source_dir = asset_root.join(path_part);
        if !source_dir.exists() {
            warnings.push(format!("source directory not found: {}", source_dir));
        } else {
            let has_toml = has_file_with_ext(&source_dir, "toml");
            let has_md = has_file_with_ext(&source_dir, "md");
            if !has_toml && !has_md {
                warnings.push(format!("no .toml or .md files in source: {}", source_dir));
            }
        }
    }

    // Contract: verify languages are meaningful
    if entry.languages.is_empty() {
        warnings.push("no languages defined".to_string());
    }

    let passed = failures.is_empty();
    AssetTestResult {
        id: entry.id.clone(),
        kind: entry.kind.clone(),
        passed,
        failures,
        warnings,
    }
}

fn has_file_with_ext(dir: &Utf8PathBuf, ext: &str) -> bool {
    if !dir.exists() {
        return false;
    }
    match fs::read_dir(dir.as_std_path()) {
        Ok(entries) => entries
            .flatten()
            .any(|e| e.path().extension().map(|x| x == ext).unwrap_or(false)),
        Err(_) => false,
    }
}
