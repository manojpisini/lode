#![deny(unsafe_code)]

use std::{collections::BTreeSet, fs, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use lode_core::{
    build_catalog, build_search_index, default_config, ensure_global_workspace, export_catalog,
    global_asset_dir, global_dir, load_search_index, save_search_index, test_assets,
    AssetCatalogEntry, LodeError, ValidatedRoot,
};
use serde::{Deserialize, Serialize};

use crate::output;
use crate::AssetsCommand;
use crate::OutputFormat;

pub(crate) fn assets_command(command: AssetsCommand) -> lode_core::Result<()> {
    match command {
        AssetsCommand::Test {
            id,
            changed: _,
            output,
        } => {
            let config = default_config();
            let report = test_assets(&config, id.as_deref())?;
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
                );
            } else {
                println!("{}  Asset Contract Tests", output::bold("Asset Tests"));
                if report.total == 0 {
                    println!("  {}  No assets to test", output::dim("~"));
                    return Ok(());
                }
                for result in &report.results {
                    if result.passed {
                        println!(
                            "  {}  {}  {}",
                            output::green("✔"),
                            result.id,
                            output::dim("pass")
                        );
                    } else {
                        println!(
                            "  {}  {}  {}",
                            output::red("✘"),
                            result.id,
                            output::red("FAIL")
                        );
                        for f in &result.failures {
                            println!("       {}", f);
                        }
                    }
                    for w in &result.warnings {
                        println!("       {}  {}", output::yellow("⚠"), w);
                    }
                }
                println!(
                    "\n  {}  {}/{} passed",
                    if report.failed == 0 {
                        output::green("✔")
                    } else {
                        output::red("✘")
                    },
                    report.passed,
                    report.total,
                );
            }
            Ok(())
        }
        AssetsCommand::Search {
            query,
            kind,
            status,
            min_quality,
            output,
        } => search_assets(
            &query,
            kind.as_deref(),
            status.as_deref(),
            min_quality,
            output,
        ),
        AssetsCommand::Show { id, output } => show_asset(&id, output),
        AssetsCommand::Add {
            source,
            asset,
            all,
            global,
            project,
            yes,
        } => add_assets(&source, asset.as_deref(), all, global, project, yes),
        AssetsCommand::List {
            output,
            global,
            project,
        } => list_assets(output, global, project),
        AssetsCommand::Remove {
            name,
            global,
            project,
            yes,
        } => remove_asset(&name, global, project, yes),
        AssetsCommand::Update {
            name,
            global,
            project,
            yes,
        } => update_assets(&name, global, project, yes),
        AssetsCommand::Init { name } => init_asset_manifest(&name),
        AssetsCommand::Catalog { out } => export_catalog_file(out),
        AssetsCommand::Index {
            rebuild,
            stats,
            output,
        } => index_command(rebuild, stats, output),
    }
}

fn index_command(rebuild: bool, stats: bool, output: OutputFormat) -> lode_core::Result<()> {
    let config = default_config();

    if rebuild {
        let idx = build_search_index(&config)?;
        save_search_index(&idx)?;
        if output.should_use_json() {
            println!(
                "{}",
                serde_json::to_string_pretty(&idx)
                    .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
            );
        } else {
            println!(
                "{}  Built search index: {} entries, {} terms",
                output::green("✔"),
                idx.total_entries,
                idx.word_index.len(),
            );
        }
        return Ok(());
    }

    if stats || !rebuild {
        match load_search_index() {
            Ok(idx) => {
                if output.should_use_json() {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&idx)
                            .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
                    );
                } else {
                    println!("{}  Search Index Status", output::bold("Search Index"));
                    println!(
                        "  {}  {} entries indexed",
                        output::cyan("ℹ"),
                        idx.total_entries
                    );
                    println!(
                        "  {}  {} unique terms",
                        output::cyan("ℹ"),
                        idx.word_index.len()
                    );
                    println!("  {}  Last indexed: {}", output::dim(" "), idx.indexed_at);
                }
            }
            Err(_) => {
                println!(
                    "{}  No search index found. Run `lode assets index --rebuild` to create one.",
                    output::dim("~")
                );
            }
        }
    }
    Ok(())
}

fn search_assets(
    query: &str,
    kind_filter: Option<&str>,
    status_filter: Option<&str>,
    min_quality: Option<u32>,
    output: OutputFormat,
) -> lode_core::Result<()> {
    let config = default_config();

    let results = if let Ok(idx_results) = lode_core::search_index(&lode_core::SearchQuery {
        query: query.to_string(),
        kind: kind_filter.map(|s| s.to_string()),
        status: status_filter.map(|s| s.to_string()),
        min_quality,
        limit: 30,
    }) {
        // Use indexed search
        idx_results
    } else {
        // Fallback to catalog scan
        let catalog = build_catalog(&config);
        let query_lower = query.to_lowercase();

        let mut cat_results: Vec<lode_core::SearchResult> = catalog
            .entries
            .iter()
            .filter(|entry| {
                if let Some(k) = kind_filter {
                    if entry.kind != k {
                        return false;
                    }
                }
                if let Some(s) = status_filter {
                    if entry.status != s {
                        return false;
                    }
                }
                if let Some(mq) = min_quality {
                    if entry.quality_score.unwrap_or(0) < mq {
                        return false;
                    }
                }

                let q = &query_lower;
                entry.summary.to_lowercase().contains(q)
                    || entry.id.to_lowercase().contains(q)
                    || entry.intents.iter().any(|i| i.to_lowercase().contains(q))
                    || entry.tags.iter().any(|t| t.to_lowercase().contains(q))
                    || entry.languages.iter().any(|l| l.to_lowercase().contains(q))
            })
            .map(|entry| lode_core::SearchResult {
                id: entry.id.clone(),
                kind: entry.kind.clone(),
                summary: entry.summary.clone(),
                status: entry.status.clone(),
                maturity: entry.maturity.clone(),
                quality_score: entry.quality_score,
                languages: entry.languages.clone(),
                relevance: relevance_score(entry, query_lower.as_str()),
            })
            .collect();

        cat_results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        cat_results.truncate(30);
        cat_results
    };

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&results)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        if results.is_empty() {
            println!("No assets found matching '{query}'");
            return Ok(());
        }
        println!("Found {} matching assets:\n", results.len());
        for entry in &results {
            let status_tag = match entry.status.as_str() {
                "experimental" => " [exp]",
                "preview" => " [pre]",
                "deprecated" => " [dep]",
                "retired" => " [ret]",
                _ => "",
            };
            println!("  {}{}  {}", entry.id, status_tag, entry.summary);
            if let Some(qs) = entry.quality_score {
                println!("       Quality: {}/100", qs);
            }
            if !entry.languages.is_empty() {
                println!("       Languages: {}", entry.languages.join(", "));
            }
            println!();
        }
    }
    Ok(())
}

fn relevance_score(entry: &AssetCatalogEntry, query: &str) -> f64 {
    let mut score = 0.0;
    if entry.summary.to_lowercase().contains(query) {
        score += 5.0;
    }
    if entry.id.to_lowercase().contains(query) {
        score += 4.0;
    }
    for intent in &entry.intents {
        if intent.to_lowercase().contains(query) {
            score += 2.0;
        }
    }
    for lang in &entry.languages {
        if lang.to_lowercase().contains(query) || query.contains(&lang.to_lowercase()) {
            score += 1.0;
        }
    }
    score
}

fn show_asset(id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let config = default_config();
    let catalog = build_catalog(&config);

    let entry = catalog
        .entries
        .iter()
        .find(|e| e.id == id || e.id.trim_start_matches("profile://") == id);

    match entry {
        Some(e) => {
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(e)
                        .map_err(|err| lode_core::LodeError::Message(err.to_string()))?
                );
            } else {
                println!("ID:           {}", e.id);
                println!("Kind:         {}", e.kind);
                println!("Summary:      {}", e.summary);
                println!("Status:       {}", e.status);
                println!("Maturity:     {}", e.maturity);
                if let Some(qs) = e.quality_score {
                    println!("Quality:      {}/100", qs);
                }
                if let Some(lv) = &e.last_verified {
                    println!("Last Verified: {}", lv);
                }
                if let Some(vr) = &e.verification_last_result {
                    println!("Verification:  {}", vr);
                }
                if !e.languages.is_empty() && e.languages != ["*"] {
                    println!("Languages: {}", e.languages.join(", "));
                }
                if !e.project_types.is_empty() && e.project_types != ["*"] {
                    println!("Types:    {}", e.project_types.join(", "));
                }
                if !e.intents.is_empty() {
                    println!("Intents:");
                    for intent in &e.intents {
                        println!("  - {intent}");
                    }
                }
                if !e.requires.is_empty() {
                    println!("Requires: {}", e.requires.join(", "));
                }
                if !e.recommends.is_empty() {
                    println!("Recommends: {}", e.recommends.join(", "));
                }
                if e.status == "deprecated" {
                    if let Some(r) = &e.deprecation_replacement {
                        println!("Replacement:  {}", r);
                    }
                    if let Some(r) = &e.deprecation_remove_after {
                        println!("Remove After: {}", r);
                    }
                    if let Some(m) = &e.deprecation_migration {
                        println!("Migration:    {}", m);
                    }
                }
                if !e.verification.is_empty() {
                    println!("Verification:");
                    for v in &e.verification {
                        println!("  - {v}");
                    }
                }
            }
            Ok(())
        }
        None => {
            eprintln!("Asset not found: {id}");
            std::process::exit(1);
        }
    }
}

fn list_assets(output: OutputFormat, global: bool, project: bool) -> lode_core::Result<()> {
    if global || project {
        return list_installed_assets(output, global, project);
    }
    let config = default_config();
    let catalog = build_catalog(&config);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&catalog)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!(
            "LODE Asset Catalog ({} total entries)\n",
            catalog.entries.len()
        );

        let mut by_kind: std::collections::BTreeMap<&str, Vec<&AssetCatalogEntry>> =
            std::collections::BTreeMap::new();
        for entry in &catalog.entries {
            by_kind.entry(&entry.kind).or_default().push(entry);
        }

        for (kind, entries) in &by_kind {
            println!("  {} ({})", kind, entries.len());
            for entry in entries.iter().take(10) {
                println!("    {}  {}", entry.id, entry.summary);
            }
            if entries.len() > 10 {
                println!("    ... and {} more", entries.len() - 10);
            }
            println!();
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct LodeAssetManifest {
    name: String,
    description: Option<String>,
    version: Option<String>,
    assets: Vec<LodeAssetEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct LodeAssetEntry {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    path: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledAssetsFile {
    schema_version: u32,
    assets: Vec<InstalledAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledAsset {
    name: String,
    kind: String,
    source: String,
    manifest_name: String,
    manifest_version: Option<String>,
    description: Option<String>,
    destination_root: String,
    destination_relative: String,
    installed_at: String,
}

impl Default for InstalledAssetsFile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            assets: Vec::new(),
        }
    }
}

struct AssetScope {
    label: &'static str,
    metadata_path: Utf8PathBuf,
}

struct SourceCheckout {
    path: Utf8PathBuf,
    _temp: Option<tempfile::TempDir>,
}

fn add_assets(
    source: &str,
    asset: Option<&str>,
    all: bool,
    global: bool,
    project: bool,
    _yes: bool,
) -> lode_core::Result<()> {
    let scope = selected_scope(global, project)?;
    let checkout = resolve_source(source)?;
    let manifest = load_lode_asset_manifest(&checkout.path)?;
    let selected = select_manifest_assets(&manifest, asset, all)?;
    install_manifest_assets(&scope, &checkout.path, source, &manifest, &selected, false)?;
    println!(
        "{}  installed {} asset(s) into {} scope",
        output::green("OK"),
        selected.len(),
        scope.label
    );
    Ok(())
}

fn list_installed_assets(
    output: OutputFormat,
    global: bool,
    project: bool,
) -> lode_core::Result<()> {
    let scope = selected_scope(global, project)?;
    let installed = read_installed_assets(&scope)?;
    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&installed)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
        return Ok(());
    }
    if installed.assets.is_empty() {
        println!("No assets installed in {} scope", scope.label);
        return Ok(());
    }
    println!("Installed assets ({})", scope.label);
    for asset in &installed.assets {
        println!(
            "  {}  {}  {}",
            asset.name,
            asset.kind,
            asset.description.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

fn remove_asset(name: &str, global: bool, project: bool, yes: bool) -> lode_core::Result<()> {
    if !yes {
        return Err(LodeError::Message(
            "asset removal requires -y/--yes in non-interactive mode".to_string(),
        ));
    }
    let scope = selected_scope(global, project)?;
    let mut installed = read_installed_assets(&scope)?;
    let index = installed
        .assets
        .iter()
        .position(|asset| asset.name == name)
        .ok_or_else(|| LodeError::Message(format!("asset is not installed: {name}")))?;
    let removed = installed.assets.remove(index);
    remove_installed_payload(&removed)?;
    write_installed_assets(&scope, &installed)?;
    println!("{}  removed {name}", output::green("OK"));
    Ok(())
}

fn update_assets(
    names: &[String],
    global: bool,
    project: bool,
    yes: bool,
) -> lode_core::Result<()> {
    if !yes {
        return Err(LodeError::Message(
            "asset update requires -y/--yes in non-interactive mode".to_string(),
        ));
    }
    let scope = selected_scope(global, project)?;
    let installed = read_installed_assets(&scope)?;
    let requested: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    let targets = installed
        .assets
        .iter()
        .filter(|asset| requested.is_empty() || requested.contains(asset.name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Err(LodeError::Message(
            "no matching installed assets to update".to_string(),
        ));
    }
    for target in &targets {
        let checkout = resolve_source(&target.source)?;
        let manifest = load_lode_asset_manifest(&checkout.path)?;
        let selected = select_manifest_assets(&manifest, Some(&target.name), false)?;
        install_manifest_assets(
            &scope,
            &checkout.path,
            &target.source,
            &manifest,
            &selected,
            true,
        )?;
    }
    println!(
        "{}  updated {} asset(s) in {} scope",
        output::green("OK"),
        targets.len(),
        scope.label
    );
    Ok(())
}

fn init_asset_manifest(name: &str) -> lode_core::Result<()> {
    let cwd = crate::current_dir()?;
    let path = cwd.join("lode.json");
    if path.exists() {
        return Err(LodeError::Message("lode.json already exists".to_string()));
    }
    let root = ValidatedRoot::new(&cwd)?;
    let manifest = serde_json::json!({
        "name": name,
        "description": "",
        "version": "0.1.0",
        "assets": []
    });
    let raw =
        serde_json::to_string_pretty(&manifest).map_err(|e| LodeError::Message(e.to_string()))?;
    root.write_atomic("lode.json", format!("{raw}\n"))?;
    println!("{}  wrote lode.json", output::green("OK"));
    Ok(())
}

fn selected_scope(global: bool, project: bool) -> lode_core::Result<AssetScope> {
    if global && project {
        return Err(LodeError::Message(
            "choose either --global or --project, not both".to_string(),
        ));
    }
    if global {
        ensure_global_workspace()?;
        let state_dir = global_dir()?.join("state");
        fs::create_dir_all(&state_dir).map_err(|e| LodeError::Io {
            path: state_dir.as_std_path().to_path_buf(),
            source: e,
        })?;
        return Ok(AssetScope {
            label: "global",
            metadata_path: state_dir.join("assets-installed.json"),
        });
    }
    let cwd = crate::current_dir()?;
    let lode_dir = cwd.join(".lode");
    fs::create_dir_all(&lode_dir).map_err(|e| LodeError::Io {
        path: lode_dir.as_std_path().to_path_buf(),
        source: e,
    })?;
    Ok(AssetScope {
        label: "project",
        metadata_path: lode_dir.join("assets-installed.json"),
    })
}

fn resolve_source(source: &str) -> lode_core::Result<SourceCheckout> {
    let local = Utf8PathBuf::from(source);
    if local.exists() {
        let path = local.canonicalize_utf8().map_err(|e| LodeError::Io {
            path: local.as_std_path().to_path_buf(),
            source: e,
        })?;
        return Ok(SourceCheckout { path, _temp: None });
    }

    let url = github_clone_url(source)?;
    let temp = tempfile::tempdir().map_err(|e| LodeError::Message(e.to_string()))?;
    let checkout = temp.path().join("source");
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &url,
            checkout.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| LodeError::Message(format!("failed to run git clone: {e}")))?;
    if !status.success() {
        return Err(LodeError::Message(format!(
            "failed to clone asset source: {}",
            lode_core::redact(source)
        )));
    }
    let path = Utf8PathBuf::from_path_buf(checkout).map_err(|path| {
        LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
    })?;
    Ok(SourceCheckout {
        path,
        _temp: Some(temp),
    })
}

fn github_clone_url(source: &str) -> lode_core::Result<String> {
    if source.contains("/tree/") || source.contains("/blob/") {
        return Err(LodeError::Message(
            "direct repo tree/path sources are not supported in assets v1".to_string(),
        ));
    }
    if source.starts_with("https://github.com/") {
        return Ok(source.trim_end_matches('/').to_string());
    }
    if source.starts_with("http://") || source.starts_with("https://") || source.contains('@') {
        return Err(LodeError::Message(
            "assets v1 supports local paths, owner/repo shorthand, and GitHub HTTPS URLs"
                .to_string(),
        ));
    }
    let parts = source.split('/').collect::<Vec<_>>();
    if parts.len() == 2 && parts.iter().all(|part| is_github_segment(part)) {
        return Ok(format!("https://github.com/{}/{}.git", parts[0], parts[1]));
    }
    Err(LodeError::Message(format!(
        "asset source not found or unsupported: {}",
        lode_core::redact(source)
    )))
}

fn is_github_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('.')
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn load_lode_asset_manifest(root: &Utf8Path) -> lode_core::Result<LodeAssetManifest> {
    let path = root.join("lode.json");
    let raw = fs::read_to_string(&path).map_err(|e| LodeError::Io {
        path: path.as_std_path().to_path_buf(),
        source: e,
    })?;
    let manifest: LodeAssetManifest = serde_json::from_str(&raw)
        .map_err(|e| LodeError::Message(format!("invalid lode.json: {e}")))?;
    validate_lode_asset_manifest(root, &manifest)?;
    Ok(manifest)
}

fn validate_lode_asset_manifest(
    root: &Utf8Path,
    manifest: &LodeAssetManifest,
) -> lode_core::Result<()> {
    if manifest.name.trim().is_empty() {
        return Err(LodeError::Message("lode.json name is required".to_string()));
    }
    let mut names = BTreeSet::new();
    for asset in &manifest.assets {
        if asset.name.trim().is_empty() {
            return Err(LodeError::Message("asset name is required".to_string()));
        }
        if !names.insert(asset.name.as_str()) {
            return Err(LodeError::Message(format!(
                "duplicate asset name in lode.json: {}",
                asset.name
            )));
        }
        asset_kind_dir(&asset.kind)?;
        let relative = crate::safe_relative_path(&asset.path)?;
        let source_path = root.join(relative);
        if !source_path.exists() {
            return Err(LodeError::Message(format!(
                "asset path does not exist: {}",
                asset.path
            )));
        }
    }
    Ok(())
}

fn select_manifest_assets(
    manifest: &LodeAssetManifest,
    asset: Option<&str>,
    all: bool,
) -> lode_core::Result<Vec<LodeAssetEntry>> {
    if all && asset.is_some() {
        return Err(LodeError::Message(
            "choose either --all or --asset, not both".to_string(),
        ));
    }
    if all {
        return Ok(manifest.assets.clone());
    }
    if let Some(name) = asset {
        let found = manifest
            .assets
            .iter()
            .find(|entry| entry.name == name)
            .cloned()
            .ok_or_else(|| LodeError::Message(format!("asset not found in lode.json: {name}")))?;
        return Ok(vec![found]);
    }
    match manifest.assets.as_slice() {
        [single] => Ok(vec![single.clone()]),
        [] => Err(LodeError::Message(
            "lode.json contains no assets".to_string(),
        )),
        _ => Err(LodeError::Message(
            "source contains multiple assets; pass --all or --asset <name>".to_string(),
        )),
    }
}

fn install_manifest_assets(
    scope: &AssetScope,
    source_dir: &Utf8Path,
    source: &str,
    manifest: &LodeAssetManifest,
    selected: &[LodeAssetEntry],
    replace_existing: bool,
) -> lode_core::Result<()> {
    let mut installed = read_installed_assets(scope)?;
    for entry in selected {
        if let Some(existing_index) = installed
            .assets
            .iter()
            .position(|asset| asset.name == entry.name)
        {
            if !replace_existing {
                return Err(LodeError::Message(format!(
                    "asset already installed in {} scope: {}",
                    scope.label, entry.name
                )));
            }
            let existing = installed.assets.remove(existing_index);
            let _ = remove_installed_payload(&existing);
        }
        let destination = copy_manifest_asset(scope, source_dir, entry)?;
        installed.assets.push(InstalledAsset {
            name: entry.name.clone(),
            kind: entry.kind.clone(),
            source: source.to_string(),
            manifest_name: manifest.name.clone(),
            manifest_version: manifest.version.clone(),
            description: entry
                .description
                .clone()
                .or_else(|| manifest.description.clone()),
            destination_root: destination.0,
            destination_relative: destination.1,
            installed_at: crate::now_timestamp(),
        });
    }
    installed.assets.sort_by(|a, b| a.name.cmp(&b.name));
    write_installed_assets(scope, &installed)
}

fn copy_manifest_asset(
    scope: &AssetScope,
    source_dir: &Utf8Path,
    entry: &LodeAssetEntry,
) -> lode_core::Result<(String, String)> {
    let kind_dir = asset_kind_dir(&entry.kind)?;
    let destination_root = if scope.label == "global" {
        global_asset_dir(kind_dir)?
    } else {
        crate::current_dir()?.join(".lode").join(kind_dir)
    };
    fs::create_dir_all(&destination_root).map_err(|e| LodeError::Io {
        path: destination_root.as_std_path().to_path_buf(),
        source: e,
    })?;

    let source_root = ValidatedRoot::new(source_dir)?;
    let destination_root_validated = ValidatedRoot::new(&destination_root)?;
    let source_relative = crate::safe_relative_path(&entry.path)?;
    let source_path = source_root.resolve(&source_relative)?;
    let destination_relative = if source_path.is_dir() {
        crate::safe_relative_path(&entry.name)?
    } else {
        let file_name = Utf8Path::new(&entry.path).file_name().ok_or_else(|| {
            LodeError::Message(format!("asset path has no file name: {}", entry.path))
        })?;
        crate::safe_relative_path(file_name)?
    };
    let destination_path = destination_root.join(&destination_relative);
    if destination_path.exists() {
        return Err(LodeError::Message(format!(
            "asset destination already exists: {}",
            destination_path
        )));
    }
    copy_asset_payload(
        &source_root,
        &destination_root_validated,
        &source_relative,
        &destination_relative,
    )?;
    Ok((
        destination_root.to_string(),
        destination_relative.to_string(),
    ))
}

fn read_installed_assets(scope: &AssetScope) -> lode_core::Result<InstalledAssetsFile> {
    if !scope.metadata_path.exists() {
        return Ok(InstalledAssetsFile::default());
    }
    let raw = fs::read_to_string(&scope.metadata_path).map_err(|e| LodeError::Io {
        path: scope.metadata_path.as_std_path().to_path_buf(),
        source: e,
    })?;
    serde_json::from_str(&raw)
        .map_err(|e| LodeError::Message(format!("invalid installed assets metadata: {e}")))
}

fn write_installed_assets(
    scope: &AssetScope,
    installed: &InstalledAssetsFile,
) -> lode_core::Result<()> {
    let parent = scope
        .metadata_path
        .parent()
        .ok_or_else(|| LodeError::Message("installed assets metadata has no parent".to_string()))?;
    fs::create_dir_all(parent).map_err(|e| LodeError::Io {
        path: parent.as_std_path().to_path_buf(),
        source: e,
    })?;
    let root = ValidatedRoot::new(parent)?;
    let file_name = scope.metadata_path.file_name().ok_or_else(|| {
        LodeError::Message("installed assets metadata has no file name".to_string())
    })?;
    let raw =
        serde_json::to_string_pretty(installed).map_err(|e| LodeError::Message(e.to_string()))?;
    root.write_atomic(file_name, format!("{raw}\n"))?;
    Ok(())
}

fn copy_asset_payload(
    source_root: &ValidatedRoot,
    destination_root: &ValidatedRoot,
    source_relative: &Utf8Path,
    destination_relative: &Utf8Path,
) -> lode_core::Result<()> {
    let source = source_root.resolve(source_relative)?;
    if source.is_dir() {
        destination_root.create_dir_all(destination_relative)?;
        for entry in fs::read_dir(&source).map_err(|e| LodeError::Io {
            path: source.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| LodeError::Io {
                path: source.clone(),
                source: e,
            })?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let child_source =
                crate::safe_relative_path(Utf8PathBuf::from(source_relative).join(&name).as_str())?;
            let child_destination = crate::safe_relative_path(
                Utf8PathBuf::from(destination_relative).join(&name).as_str(),
            )?;
            copy_asset_payload(
                source_root,
                destination_root,
                &child_source,
                &child_destination,
            )?;
        }
        return Ok(());
    }

    if let Some(parent) = destination_relative.parent() {
        if !parent.as_str().is_empty() {
            destination_root.create_dir_all(parent)?;
        }
    }
    let bytes = fs::read(&source).map_err(|e| LodeError::Io {
        path: source,
        source: e,
    })?;
    destination_root.write_atomic(destination_relative, bytes)?;
    Ok(())
}

fn remove_installed_payload(asset: &InstalledAsset) -> lode_core::Result<()> {
    let root = Utf8PathBuf::from(&asset.destination_root);
    if !root.exists() {
        return Ok(());
    }
    let validated = ValidatedRoot::new(&root)?;
    let relative = crate::safe_relative_path(&asset.destination_relative)?;
    let path = root.join(&relative);
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        validated.remove_dir_all(relative)?;
    } else {
        validated.remove_file(relative)?;
    }
    Ok(())
}

fn asset_kind_dir(kind: &str) -> lode_core::Result<&'static str> {
    match kind {
        "template" => Ok("templates"),
        "snippet" => Ok("snippets"),
        "recipe" => Ok("recipes"),
        "command" => Ok("commands"),
        "profile" => Ok("profiles"),
        "plugin" => Ok("plugins"),
        "license" => Ok("licenses"),
        other => Err(LodeError::Message(format!("invalid asset type: {other}"))),
    }
}

fn export_catalog_file(out: Option<camino::Utf8PathBuf>) -> lode_core::Result<()> {
    let config = default_config();
    let path = match out {
        Some(p) => p,
        None => lode_core::catalog::catalog_path()?,
    };
    export_catalog(&config, &path)?;
    println!("Catalog written to {}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(path: &std::path::Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, contents).expect("write file");
    }

    #[test]
    fn lode_json_valid_mixed_manifest_loads() {
        let temp = tempfile::tempdir().expect("create temp dir");
        write_file(&temp.path().join("templates/app/main.txt"), "hello");
        write_file(&temp.path().join("snippets/log.toml"), "body = 'log'");
        write_file(
            &temp.path().join("lode.json"),
            r#"{
  "name": "pack",
  "description": "test pack",
  "version": "0.1.0",
  "assets": [
    {"name":"app","type":"template","path":"templates/app"},
    {"name":"log","type":"snippet","path":"snippets/log.toml"}
  ]
}"#,
        );
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).expect("utf8 path");
        let manifest = load_lode_asset_manifest(&root).expect("load manifest");
        assert_eq!(manifest.assets.len(), 2);
    }

    #[test]
    fn lode_json_missing_required_fields_fails() {
        let temp = tempfile::tempdir().expect("create temp dir");
        write_file(&temp.path().join("lode.json"), r#"{"assets":[]}"#);
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).expect("utf8 path");
        assert!(load_lode_asset_manifest(&root).is_err());
    }

    #[test]
    fn lode_json_invalid_asset_type_fails() {
        let temp = tempfile::tempdir().expect("create temp dir");
        write_file(&temp.path().join("asset.txt"), "hello");
        write_file(
            &temp.path().join("lode.json"),
            r#"{"name":"pack","assets":[{"name":"x","type":"skill","path":"asset.txt"}]}"#,
        );
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).expect("utf8 path");
        assert!(load_lode_asset_manifest(&root).is_err());
    }

    #[test]
    fn lode_json_traversal_path_fails() {
        let temp = tempfile::tempdir().expect("create temp dir");
        write_file(
            &temp.path().join("lode.json"),
            r#"{"name":"pack","assets":[{"name":"x","type":"template","path":"../outside"}]}"#,
        );
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).expect("utf8 path");
        assert!(load_lode_asset_manifest(&root).is_err());
    }
}
