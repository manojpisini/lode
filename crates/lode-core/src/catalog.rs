use std::collections::HashMap;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::{assets, config::LodeConfig, install::global_asset_dir, LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCatalogEntry {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub intents: Vec<String>,
    pub languages: Vec<String>,
    pub project_types: Vec<String>,
    pub maturity: String,
    pub requires: Vec<String>,
    pub recommends: Vec<String>,
    pub conflicts: Vec<String>,
    pub verification: Vec<String>,
    pub tags: Vec<String>,
    /// Lifecycle: experimental, preview, stable, deprecated, retired
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub quality_score: Option<u32>,
    #[serde(default)]
    pub last_verified: Option<String>,
    #[serde(default)]
    pub verification_tests: Option<u32>,
    #[serde(default)]
    pub verification_fixtures: Option<u32>,
    #[serde(default)]
    pub verification_last_result: Option<String>,
    #[serde(default)]
    pub deprecation_replacement: Option<String>,
    #[serde(default)]
    pub deprecation_remove_after: Option<String>,
    #[serde(default)]
    pub deprecation_migration: Option<String>,
}

fn default_status() -> String {
    "stable".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCatalog {
    pub schema_version: u32,
    pub lode_version: String,
    pub entries: Vec<AssetCatalogEntry>,
    pub intents_index: HashMap<String, Vec<String>>,
    pub language_index: HashMap<String, Vec<String>>,
}

fn detect_maturity(path: &str) -> &'static str {
    if path.starts_with("core/") || path.starts_with("systems/") {
        "production"
    } else if path.starts_with("frontend/") || path.starts_with("backend/") {
        "stable"
    } else if path.starts_with("desktop/") || path.starts_with("games/") {
        "beta"
    } else {
        "stable"
    }
}

fn detect_status(path: &str, summary: &str) -> &'static str {
    let p = path.to_lowercase();
    let s = summary.to_lowercase();
    if p.contains("deprecated") || s.contains("deprecated") {
        "deprecated"
    } else if p.contains("experimental") || p.contains("alpha") || p.contains("preview-") {
        "experimental"
    } else if p.starts_with("core/") || p.starts_with("systems/") {
        "stable"
    } else if p.starts_with("frontend/") || p.starts_with("backend/") {
        "preview"
    } else {
        "stable"
    }
}

fn compute_quality_score(path: &str, kind: &str, maturity: &str) -> Option<u32> {
    let mut score: u32 = 85;
    if kind == "profile" || kind == "template" {
        score = score.saturating_add(5);
    }
    if maturity == "production" {
        score = score.saturating_add(10);
    } else if maturity == "beta" {
        score = score.saturating_sub(10);
    }
    if path.starts_with("core/") {
        score = score.saturating_add(5);
    }
    Some(score.min(100))
}

fn detect_languages(_kind: &str, path: &str) -> Vec<String> {
    let p = path.to_lowercase();
    if p.contains("rust") || p.contains("cargo") {
        vec!["rust".to_string()]
    } else if p.contains("python") || p.contains("django") || p.contains("fastapi") {
        vec!["python".to_string()]
    } else if p.contains("node")
        || p.contains("typescript")
        || p.contains("javascript")
        || p.contains("ts")
        || p.contains("js")
    {
        vec!["typescript".to_string(), "javascript".to_string()]
    } else if p.contains("go") {
        vec!["go".to_string()]
    } else if p.contains("c-app") || p.contains("c-lib") {
        vec!["c".to_string()]
    } else if p.contains("cpp") || p.contains("c++") || p.contains("competitive-cpp") {
        vec!["cpp".to_string()]
    } else if p.contains("zig") {
        vec!["zig".to_string()]
    } else if p.contains("java")
        || p.contains("gradle")
        || p.contains("maven")
        || p.contains("minecraft")
    {
        vec!["java".to_string()]
    } else if p.contains("tauri") {
        vec!["rust".to_string(), "typescript".to_string()]
    } else if p.contains("competitive") {
        vec![
            "cpp".to_string(),
            "rust".to_string(),
            "python".to_string(),
            "java".to_string(),
        ]
    } else {
        vec!["*".to_string()]
    }
}

fn detect_project_types(path: &str) -> Vec<String> {
    let p = path.to_lowercase();
    if p.contains("cli") {
        vec!["cli".to_string()]
    } else if p.contains("service") || p.contains("api") {
        vec!["api".to_string(), "service".to_string()]
    } else if p.contains("lib") || p.contains("library") {
        vec!["library".to_string()]
    } else if p.contains("app") || p.contains("application") {
        vec!["application".to_string()]
    } else if p.contains("web")
        || p.contains("frontend")
        || p.contains("react")
        || p.contains("vue")
        || p.contains("svelte")
        || p.contains("next")
        || p.contains("astro")
    {
        vec!["web-app".to_string()]
    } else if p.contains("backend")
        || p.contains("express")
        || p.contains("fastify")
        || p.contains("nest")
        || p.contains("django")
    {
        vec!["api".to_string(), "web-app".to_string()]
    } else if p.contains("workspace") {
        vec!["workspace".to_string()]
    } else if p.contains("competitive") || p.contains("cp") || p.contains("challenge") {
        vec!["competitive".to_string()]
    } else if p.contains("minecraft")
        || p.contains("fabric")
        || p.contains("forge")
        || p.contains("papermc")
        || p.contains("paper")
    {
        vec!["minecraft-plugin".to_string()]
    } else if p.contains("tauri") || p.contains("desktop") {
        vec!["desktop".to_string()]
    } else if p.contains("docs") || p.contains("documentation") {
        vec!["documentation".to_string()]
    } else if p.contains("hackathon") {
        vec!["hackathon".to_string()]
    } else {
        vec!["*".to_string()]
    }
}

fn asset_intents(kind: &str, path: &str, summary: &str) -> Vec<String> {
    let mut intents = Vec::new();
    let p = path.to_lowercase();
    let s = summary.to_lowercase();

    intents.push(format!("{}:{}", kind, path));

    let fragments = p.split('/').collect::<Vec<_>>();
    if fragments.len() > 1 {
        intents.push(fragments.last().unwrap().to_string());
    }

    for word in s.split_whitespace() {
        let w = word.trim_matches(|c: char| !c.is_alphanumeric());
        if w.len() > 3 {
            intents.push(w.to_string());
        }
    }

    intents
}

fn profile_summary(path: &str) -> String {
    let name = path.split('/').next_back().unwrap_or(path);
    let parts: Vec<&str> = name.split('-').collect();
    let readable = parts
        .iter()
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!("{readable} project profile")
}

fn profile_tags(path: &str) -> Vec<String> {
    let mut tags = vec![path.to_string()];
    let p = path.to_lowercase();
    if p.starts_with("core/") {
        tags.push("core".to_string());
    } else if p.starts_with("systems/") {
        tags.push("systems".to_string());
    } else if p.starts_with("frontend/") {
        tags.push("frontend".to_string());
    } else if p.starts_with("backend/") {
        tags.push("backend".to_string());
    } else if p.starts_with("python/") {
        tags.push("python".to_string());
    } else if p.starts_with("java/") {
        tags.push("java".to_string());
    } else if p.starts_with("desktop/") {
        tags.push("desktop".to_string());
    } else if p.starts_with("games/") {
        tags.push("games".to_string());
    } else if p.starts_with("challenge/") {
        tags.push("competitive".to_string());
    }
    let lang = detect_languages("profile", path);
    tags.extend(lang);
    tags
}

fn template_summary(path: &str) -> String {
    let name = path.split('/').next_back().unwrap_or(path);
    let readable = name.replace(['-', '_'], " ");
    format!("{readable} template")
}

fn template_tags(kind: &str, path: &str) -> Vec<String> {
    let mut tags = vec![path.to_string(), kind.to_string()];
    let prefix = path.split('/').next().unwrap_or("");
    if !prefix.is_empty() && prefix != path {
        tags.push(prefix.to_string());
    }
    tags
}

fn command_summary(path: &str) -> String {
    let name = path.split('/').next_back().unwrap_or(path);
    let readable = name.replace(['-', '_'], " ");
    format!("{readable} command")
}

fn recipe_summary(path: &str) -> String {
    let name = path.split('/').next_back().unwrap_or(path);
    let readable = name.replace(['-', '_'], " ");
    format!("{readable} recipe")
}

fn snippet_summary(path: &str) -> String {
    let name = path.split('/').next_back().unwrap_or(path);
    format!("{name} snippet")
}

fn license_summary(path: &str) -> String {
    format!("{path} license")
}

fn build_catalog_entry(kind: &str, id: &str, path: &str, summary: &str) -> AssetCatalogEntry {
    let intents = asset_intents(kind, path, summary);
    let languages = detect_languages(kind, path);
    let project_types = detect_project_types(path);
    let maturity = detect_maturity(path);
    let status = detect_status(path, summary);
    let quality_score = compute_quality_score(path, kind, maturity);
    let tags = match kind {
        "profile" => profile_tags(path),
        "template" => template_tags(kind, path),
        _ => vec![path.to_string(), kind.to_string()],
    };

    AssetCatalogEntry {
        id: id.to_string(),
        kind: kind.to_string(),
        summary: summary.to_string(),
        intents,
        languages,
        project_types,
        maturity: maturity.to_string(),
        requires: Vec::new(),
        recommends: Vec::new(),
        conflicts: Vec::new(),
        verification: Vec::new(),
        tags,
        status: status.to_string(),
        quality_score,
        last_verified: None,
        verification_tests: None,
        verification_fixtures: None,
        verification_last_result: None,
        deprecation_replacement: None,
        deprecation_remove_after: None,
        deprecation_migration: None,
    }
}

pub fn build_catalog(_config: &LodeConfig) -> AssetCatalog {
    let mut entries = Vec::new();

    for profile in assets::profile_names() {
        let summary = profile_summary(profile);
        entries.push(build_catalog_entry(
            "profile",
            &format!("profile://{profile}"),
            profile,
            &summary,
        ));
    }

    for tmpl in assets::template_paths() {
        let summary = template_summary(tmpl);
        entries.push(build_catalog_entry(
            "template",
            &format!("template://{tmpl}"),
            tmpl,
            &summary,
        ));
    }

    for cmd in assets::command_names() {
        let summary = command_summary(cmd);
        entries.push(build_catalog_entry(
            "command",
            &format!("command://{cmd}"),
            cmd,
            &summary,
        ));
    }

    for recipe in assets::recipe_names() {
        let summary = recipe_summary(recipe);
        entries.push(build_catalog_entry(
            "recipe",
            &format!("recipe://{recipe}"),
            recipe,
            &summary,
        ));
    }

    for (group, snippets) in builtin_snippet_map() {
        for name in snippets {
            let path = format!("{group}/{name}");
            let summary = snippet_summary(&path);
            entries.push(build_catalog_entry(
                "snippet",
                &format!("snippet://{path}"),
                &path,
                &summary,
            ));
        }
    }

    let licenses = [
        "MIT",
        "Apache-2.0",
        "BSD-3-Clause",
        "ISC",
        "GPL-3.0-only",
        "MPL-2.0",
        "Unlicense",
    ];
    for lic in &licenses {
        let summary = license_summary(lic);
        entries.push(build_catalog_entry(
            "license",
            &format!("license://{lic}"),
            lic,
            &summary,
        ));
    }

    let mut intents_index: HashMap<String, Vec<String>> = HashMap::new();
    let mut language_index: HashMap<String, Vec<String>> = HashMap::new();

    for entry in &entries {
        for intent in &entry.intents {
            intents_index
                .entry(intent.clone())
                .or_default()
                .push(entry.id.clone());
        }
        for lang in &entry.languages {
            language_index
                .entry(lang.clone())
                .or_default()
                .push(entry.id.clone());
        }
    }

    AssetCatalog {
        schema_version: 1,
        lode_version: env!("CARGO_PKG_VERSION").to_string(),
        entries,
        intents_index,
        language_index,
    }
}

fn builtin_snippet_map() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("any", vec!["todo", "fixme", "note", "invariant", "example"]),
        ("md", vec!["adr", "risk", "task-list", "release-notes"]),
        (
            "rs",
            vec!["main", "error-enum", "result-alias", "serde-struct", "test"],
        ),
        ("go", vec!["main", "handler", "table-test"]),
        ("ts", vec!["fn", "async-fn", "interface", "test"]),
        ("py", vec!["main-guard", "dataclass", "pytest-test"]),
        ("sh", vec!["strict-header", "die", "parse-args"]),
        ("yaml", vec!["github-job", "docker-compose-service"]),
        ("toml", vec!["table", "lode-command", "lode-profile"]),
        (
            "cp",
            vec!["fast-io-cpp", "dijkstra", "union-find", "segment-tree"],
        ),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapInfo {
    pub lode_version: String,
    pub asset_api_version: u32,
    pub config_path: String,
    pub active_profile: Option<String>,
    pub project: Option<ProjectInfo>,
    pub context_files: Vec<String>,
    pub available_commands: usize,
    pub available_profiles: usize,
    pub available_recipes: usize,
    pub available_templates: usize,
    pub available_snippets: usize,
    pub available_licenses: usize,
    pub recommended: RecommendedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub profile: Option<String>,
    pub language: Option<String>,
    pub maturity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub next: String,
}

impl BootstrapInfo {
    pub fn new(
        lode_version: &str,
        config: &LodeConfig,
        project_dir: Option<&camino::Utf8Path>,
    ) -> Self {
        let asset_counts = (
            assets::profile_names().len(),
            assets::command_names().len(),
            assets::recipe_names().len(),
            assets::template_paths().len(),
            builtin_snippet_map()
                .iter()
                .map(|(_, v)| v.len())
                .sum::<usize>(),
            7usize,
        );

        let project = project_dir.map(|dir| {
            let name = dir
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            ProjectInfo {
                name,
                profile: config.active_profile.clone(),
                language: config.stack.languages.first().cloned(),
                maturity: "development".to_string(),
            }
        });

        let context_files = vec![
            "_ctx_/CURRENT_STATE.md".to_string(),
            "_ctx_/QUALITY_GATES.md".to_string(),
            "AGENTS.md".to_string(),
        ];

        let next = if project.is_some() {
            r#"lode agent resolve --intent "describe project" --json"#.to_string()
        } else {
            "lode init <project-name>".to_string()
        };

        Self {
            lode_version: lode_version.to_string(),
            asset_api_version: 1,
            config_path: crate::install::global_config_path()
                .map(|p| p.to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            active_profile: config.active_profile.clone(),
            project,
            context_files,
            available_commands: asset_counts.1,
            available_profiles: asset_counts.0,
            available_recipes: asset_counts.2,
            available_templates: asset_counts.3,
            available_snippets: asset_counts.4,
            available_licenses: asset_counts.5,
            recommended: RecommendedAction { next },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResolution {
    pub profile: Option<String>,
    pub recipes: Vec<String>,
    pub commands: Vec<String>,
    pub templates: Vec<String>,
    pub warnings: Vec<String>,
    pub estimated_files: usize,
    pub plan_id: String,
}

pub fn resolve_intent(
    intent: &str,
    catalog: &AssetCatalog,
    _config: &LodeConfig,
) -> IntentResolution {
    let query = intent.to_lowercase();
    let query_words: Vec<&str> = query.split_whitespace().collect();

    let mut match_scores: Vec<(f64, &AssetCatalogEntry)> = catalog
        .entries
        .iter()
        .map(|entry| {
            let mut score = 0.0;

            for word in &query_words {
                let w = word.trim_matches(|c: char| !c.is_alphanumeric());

                for intent_phrase in &entry.intents {
                    if intent_phrase.contains(w) {
                        score += 2.0;
                    }
                }

                for lang in &entry.languages {
                    if lang.contains(w) || w.contains(&lang.to_lowercase()) {
                        score += 3.0;
                    }
                }

                for tag in &entry.tags {
                    if tag.contains(w) {
                        score += 1.0;
                    }
                }
            }

            if entry.summary.to_lowercase().contains(&query) {
                score += 5.0;
            }

            (score, entry)
        })
        .collect();

    match_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let top: Vec<_> = match_scores
        .into_iter()
        .take(20)
        .filter(|(s, _)| *s > 0.0)
        .collect();

    let profile = top
        .iter()
        .find(|(_, e)| e.kind == "profile")
        .map(|(_, e)| e.id.trim_start_matches("profile://").to_string());

    let recipes: Vec<String> = top
        .iter()
        .filter(|(_, e)| e.kind == "recipe")
        .take(5)
        .map(|(_, e)| e.id.trim_start_matches("recipe://").to_string())
        .collect();

    let commands: Vec<String> = top
        .iter()
        .filter(|(_, e)| e.kind == "command")
        .take(5)
        .map(|(_, e)| e.id.trim_start_matches("command://").to_string())
        .collect();

    let templates: Vec<String> = top
        .iter()
        .filter(|(_, e)| e.kind == "template")
        .take(5)
        .map(|(_, e)| e.id.trim_start_matches("template://").to_string())
        .collect();

    let warnings = Vec::new();
    let estimated_files = top
        .iter()
        .map(|(_, e)| if e.kind == "template" { 1 } else { 0 })
        .sum();

    IntentResolution {
        profile,
        recipes,
        commands,
        templates,
        warnings,
        estimated_files,
        plan_id: format!("plan_{:x}", rand_seed()),
    }
}

fn rand_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

pub fn export_catalog(config: &LodeConfig, path: &camino::Utf8Path) -> Result<()> {
    let catalog = build_catalog(config);
    let json =
        serde_json::to_string_pretty(&catalog).map_err(|e| LodeError::Message(e.to_string()))?;
    crate::fs_safety::ValidatedRoot::new(
        path.parent()
            .ok_or_else(|| LodeError::Message("no parent directory".to_string()))?,
    )?
    .write_atomic(
        path.file_name()
            .ok_or_else(|| LodeError::Message("no file name".to_string()))?,
        json,
    )?;
    Ok(())
}

pub fn catalog_path() -> Result<Utf8PathBuf> {
    Ok(global_asset_dir("templates")?
        .join(".lode")
        .join("asset-catalog.json"))
}
