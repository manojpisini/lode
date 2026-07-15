use std::{env, fs, path::PathBuf};

use camino::Utf8PathBuf;

use crate::{
    assets, assets::AssetKind, config, fs_safety::ValidatedRoot, template::RenderContext,
    LodeError, Result,
};

const GLOBAL_DIRS: &[&str] = &[
    "profiles",
    "templates",
    "snippets",
    "licenses",
    "recipes",
    "plugins",
    "cache",
    "logs",
    "commands",
    "agents",
];

const GLOBAL_AGENT_FILES: &[(&str, &str)] = &[
    (
        "AGENTS.md",
        "# LODE Agent Bootstrap Contract\n\n## What Is LODE\n\nLODE is a local-first personal development control plane.\nIt converts personal development preferences and proven engineering patterns\ninto reusable capabilities that agents can discover and execute.\n\n## First Command\n\nRun this to discover what LODE already knows about this project:\n\n```\nlode agent bootstrap --json\n```\n\n## Core Principles\n\n1. **Discover before create** — Always ask LODE first if a profile, template,\n   recipe, snippet, or command already covers the task.\n2. **Reuse before compose** — Prefer composing existing recipes over writing\n   new ones from scratch.\n3. **Compose before customize** — Use recipe composition to combine\n   capabilities before modifying them.\n4. **Create only when necessary** — If no asset matches, create the minimum\n   needed and consider promoting it back to LODE.\n\n## Contract\n\n- LODE owns: project scaffolding, templates, profiles, recipes, snippets,\n  commands, licenses, git hooks, env management, secret scanning,\n  convention checking, metrics, time tracking, releases\n- Agent owns: implementation logic, business code, testing strategy,\n  architecture decisions within the scaffold\n- Both own: agent context files (_ctx_/*.md), build configuration,\n  CI workflows, documentation\n\n## Protocol\n\nSee `LODE_AGENT_PROTOCOL.md` for the full operating protocol.\n\n## Quick Start\n\n```\nlode agent bootstrap --json      # Discover project state\nlode assets search \"database\"    # Find relevant assets\nlode agent resolve --intent \"...\" --json  # Resolve intent to capabilities\nlode config show                 # View merged configuration\nlode plan create --intent \"...\"  # Create an execution plan\n```\n",
    ),
    (
        "LODE_AGENT_PROTOCOL.md",
        "# LODE Agent Operating Protocol\n\n## 1. Bootstrap\n\n```\nlode agent bootstrap [--json]\n```\n\nReturns LODE version, active profile, project type, available assets,\ncontext files, and recommended next action.\n\n## 2. Inspect Project Context\n\n```\nls .lode/          # Project config and state\nls _ctx_/          # AI agent context files (if synced)\nls AGENTS.md       # This bootstrap contract\n```\n\n## 3. Resolve Capabilities\n\n```\nlode agent resolve --intent \"<natural language intent>\" [--json]\n```\n\nReturns matched profile, recipes, commands, templates, warnings,\nand estimated file count. Deterministic matching first, LLM only for\nambiguous intent translation.\n\n## 4. Search Assets\n\n```\nlode assets search <query> [--kind profile|template|recipe|command|snippet|license] [--format json|table]\nlode assets show <asset-id> [--json]\nlode assets list [--format json|table]\n```\n\n## 5. Discover Through JSON\n\nEvery operation that can output JSON accepts `--json` or `--format json`.\n\n```\nlode profile show <name> --json\nlode recipe show <name> --json\nlode commands show <name> --json\nlode config show --json\n```\n\n## 6. Apply Assets\n\nProfiles define project scaffolding. Recipes compose files and steps.\nCommands execute workflow macros. Templates render files from variables.\n\n```\nlode profile use <name>\nlode recipe apply <name> [--dry-run]\nlode commands run <name> [--dry-run]\nlode sync\n```\n\n## 7. Execute Workflows\n\n```\nlode verify          # Dynamic quality gates per active profile\nlode build           # Build the project\nlode test            # Run tests\nlode fmt             # Format code\nlode lint            # Lint code\nlode check           # Convention compliance\nlode commit          # Stage and commit with conventional message\n```\n\n## 8. Verify Quality Gates\n\n```\nlode check           # Naming conventions\nlode scan secrets    # Secret leakage\nlode scan foreign    # Foreign project detection\nlode doctor          # System diagnostics\nlode health          # Project health audit\nlode sign            # Content hash verification\n```\n\n## 9. Record Decisions and Provenance\n\n```\nlode agent plan init\nlode agent plan add --task \"description\"\nlode agent plan done --id <n>\nlode agent plan show\n```\n\nThe agent plan persists across sessions in `.lode/agent_plan.json`.\n\n## 10. Compact Handoff\n\nWhen handing off to another agent:\n\n1. Run `lode agent plan show` to capture task state\n2. Record key decisions in `_ctx_/ACTIVE_DECISIONS.md`\n3. Summarize verification results\n4. List remaining risks and next action\n5. Reference stable IDs (plan IDs, decision records, paths)\n\n---\n\n## Schema Versions\n\n- Config schema: 3\n- Asset API: 1\n- Plan schema: 1\n\n## Safety Modes\n\nConsult `config show --section agents` for permission boundaries.\nFuture: `safe` (require confirm for writes), `power` (autonomous within\nproject), `locked` (read-only diagnostics).\n\n## Preference Hierarchy\n\n1. Locked global policy\n2. Personal preferences (`~/.lode/preferences.toml`)\n3. Project preferences (`.lode/preferences.toml`)\n4. Session decisions (not persisted)\n\n## Context Files\n\n```\n_ctx_/CURRENT_STATE.md      # Current task and status\n_ctx_/QUALITY_GATES.md      # Pass/fail gates\n_ctx_/ARCHITECTURE_MAP.md   # Module boundaries\n_ctx_/ACTIVE_DECISIONS.md   # Key decisions with rationale\n_ctx_/OPEN_RISKS.md         # Identified risks\n_ctx_/RECENT_CHANGES.md     # Recent file changes\n```\n",
    ),
    (
        "CLAUDE.md",
        "# Claude Context\n\n# Place Claude-specific instructions, preferences, and behavioral guidelines here.\n",
    ),
    (
        "CODEX.md",
        "# Codex Context\n\n# Place Codex-specific instructions, focused patterns, and code generation preferences here.\n",
    ),
    (
        ".cursorrules",
        "# Cursor Rules\n\n# Define cursor-specific editing rules and code generation preferences here.\n",
    ),
    (
        ".windsurfrules",
        "# Windsurf Rules\n\n# Define Windsurf-specific AI editing rules here.\n",
    ),
    (
        ".mcp.json",
        "{\n  \"servers\": {}\n}\n",
    ),
    (
        "PLAN.md",
        "# Plan\n\n# Outline implementation plans, milestones, and task breakdowns here.\n",
    ),
    (
        "CONSTRAINTS.md",
        "# Constraints\n\n# Document project constraints, boundaries, and non-negotiables here.\n",
    ),
    (
        "REVIEW.md",
        "# Review Notes\n\n# Record code review findings, suggestions, and follow-up items here.\n",
    ),
    (
        "TASKS.md",
        "# Tasks\n\n# Track actionable tasks, their status, and ownership here.\n",
    ),
    (
        "MEMORY.md",
        "# Memory\n\n# Capture durable project notes, decisions, and cross-session context here.\n",
    ),
];

fn global_dir_env(child: &str) -> Option<&'static str> {
    match child {
        "templates" => Some("LODE_TEMPLATES"),
        "profiles" => Some("LODE_PROFILES"),
        "snippets" => Some("LODE_SNIPPETS"),
        "licenses" => Some("LODE_LICENSES"),
        "plugins" => Some("LODE_PLUGINS"),
        "recipes" => Some("LODE_RECIPES"),
        "commands" => Some("LODE_COMMANDS_DIR"),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupReport {
    pub global_dir: Utf8PathBuf,
    pub config_path: Utf8PathBuf,
    pub created_dirs: Vec<Utf8PathBuf>,
    pub wrote_config: bool,
}

pub fn global_dir() -> Result<Utf8PathBuf> {
    if let Ok(path) = env::var("LODE_CONFIG") {
        let path = Utf8PathBuf::from(path);
        return path.parent().map(Utf8PathBuf::from).ok_or_else(|| {
            LodeError::Message("LODE_CONFIG must include a parent directory".into())
        });
    }

    let home = dirs::home_dir()
        .ok_or_else(|| LodeError::Message("could not determine home directory".into()))?;
    let dir = home.join(".lode");
    Utf8PathBuf::from_path_buf(dir)
        .map_err(|path| LodeError::Message(format!("path is not valid UTF-8: {}", path.display())))
}

pub fn global_config_path() -> Result<Utf8PathBuf> {
    if let Ok(path) = env::var("LODE_CONFIG") {
        return Ok(Utf8PathBuf::from(path));
    }

    Ok(global_dir()?.join("config.toml"))
}

pub fn global_asset_dir(child: &str) -> Result<Utf8PathBuf> {
    if let Some(var) = global_dir_env(child) {
        if let Ok(path) = env::var(var) {
            return Ok(Utf8PathBuf::from(path));
        }
    }
    Ok(global_dir()?.join(child))
}

pub fn ensure_global_workspace() -> Result<()> {
    let dir = global_dir()?;
    let root = trusted_root(&dir)?;

    for child in GLOBAL_DIRS {
        let path = global_asset_dir(child)?;
        if path.starts_with(&dir) {
            root.create_dir_all(
                path.strip_prefix(&dir)
                    .map_err(|_| LodeError::Message(format!("expected {path} under {dir}")))?,
            )?;
        } else {
            trusted_root(&path)?;
        }
    }

    Ok(())
}

pub fn setup_defaults(overwrite: bool) -> Result<SetupReport> {
    let dir = global_dir()?;
    let config_path = global_config_path()?;
    let mut created_dirs = Vec::new();

    if !dir.exists() {
        created_dirs.push(dir.clone());
    }
    let root = trusted_root(&dir)?;

    for child in GLOBAL_DIRS {
        let path = global_asset_dir(child)?;
        if !path.exists() {
            created_dirs.push(path.clone());
        }
        if path.starts_with(&dir) {
            root.create_dir_all(
                path.strip_prefix(&dir)
                    .map_err(|_| LodeError::Message(format!("expected {path} under {dir}")))?,
            )?;
        } else {
            trusted_root(&path)?;
        }
    }

    let wrote_config = overwrite || !config_path.exists();
    if wrote_config {
        let encoded = toml::to_string_pretty(&config::default_config())?;
        root.write_atomic(
            config_path.strip_prefix(&dir).map_err(|_| {
                LodeError::Message(format!("expected config path {config_path} under {dir}"))
            })?,
            encoded,
        )?;
    }

    let context = RenderContext::new()
        .with("project", "project")
        .with("author", "Your Name")
        .with("year", crate::current_year());
    for asset in assets::default_assets(&context) {
        let root = match asset.kind {
            assets::AssetKind::Template => "templates",
            assets::AssetKind::Profile => "profiles",
            assets::AssetKind::Snippet => "snippets",
            assets::AssetKind::Command => "commands",
            assets::AssetKind::Recipe => "recipes",
            assets::AssetKind::License => "licenses",
        };
        let asset_dir = global_asset_dir(root)?;
        let destination = asset_dir.join(&asset.path);
        if overwrite || !destination.exists() {
            let root = trusted_root(&asset_dir)?;
            if let Some(parent) = asset.path.parent() {
                root.create_dir_all(parent)?;
            }
            root.write_atomic(&asset.path, asset.contents)?;
        }
    }

    let agent_dir = global_asset_dir("agents")?;
    let agent_root = trusted_root(&agent_dir)?;
    for (name, contents) in GLOBAL_AGENT_FILES {
        let dest = agent_dir.join(name);
        if overwrite || !dest.exists() {
            agent_root.write_atomic(name, contents)?;
        }
    }
    for subdir in &["skills", "prompts", "plans"] {
        let path = agent_dir.join(subdir);
        if !path.exists() {
            agent_root.create_dir_all(subdir)?;
        }
    }

    if let Ok(mut config) = load_global_config() {
        if config.assets.auto_register {
            let _ = auto_register_assets(&mut config);
        }
    }

    Ok(SetupReport {
        global_dir: dir,
        config_path,
        created_dirs,
        wrote_config,
    })
}

pub fn load_global_config() -> Result<config::LodeConfig> {
    ensure_global_workspace()?;

    let path = global_config_path()?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    let raw = crate::config::migrations::migrate_config_source_if_needed(&path, &raw)?;
    let config: config::LodeConfig =
        toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
            path: PathBuf::from(path.as_str()),
            source: Box::new(source),
        })?;
    config::validate_schema(&config)?;
    Ok(config)
}

fn scan_asset_directory(
    dir: &camino::Utf8Path,
    kind: &str,
) -> Vec<(String, config::assets::AssetEntry)> {
    let path = dir.as_std_path();
    if !path.exists() || !path.is_dir() {
        return Vec::new();
    }
    let mut entries = Vec::new();
    let _ = walk_dir(path, kind, "", &mut entries);
    entries
}

fn walk_dir(
    dir: &std::path::Path,
    _kind: &str,
    prefix: &str,
    entries: &mut Vec<(String, config::assets::AssetEntry)>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir).map_err(|e| LodeError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LodeError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| LodeError::Io {
            path: path.clone(),
            source: e,
        })?;
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let sub_prefix = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{prefix}/{name}")
            };
            walk_dir(&path, _kind, &sub_prefix, entries)?;
        } else if file_type.is_file() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let asset_path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{prefix}/{name}")
            };
            entries.push((asset_path, config::assets::AssetEntry::default()));
        }
    }
    Ok(())
}

pub fn auto_register_assets(config: &mut config::LodeConfig) -> Result<usize> {
    let mut total = 0usize;
    let asset_kinds: &[(AssetKind, &str)] = &[
        (AssetKind::Template, "templates"),
        (AssetKind::Profile, "profiles"),
        (AssetKind::Snippet, "snippets"),
        (AssetKind::Command, "commands"),
        (AssetKind::Recipe, "recipes"),
        (AssetKind::License, "licenses"),
    ];
    for &(akind, kind) in asset_kinds {
        let dir = match global_asset_dir(kind) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let disk_files: Vec<(String, config::assets::AssetEntry)> =
            scan_asset_directory(&dir, kind);
        let known_builtins: std::collections::HashSet<String> =
            crate::assets::known_asset_paths_by_kind(akind)
                .into_iter()
                .collect();
        for (path, entry) in &disk_files {
            if known_builtins.contains(path.as_str()) {
                continue;
            }
            let already_registered = match kind {
                "templates" => config.assets.templates.contains_key(path.as_str()),
                "profiles" => config.assets.profiles.contains_key(path.as_str()),
                "snippets" => config.assets.snippets.contains_key(path.as_str()),
                "commands" => config.assets.commands.contains_key(path.as_str()),
                "recipes" => config.assets.recipes.contains_key(path.as_str()),
                "licenses" => config.assets.licenses.contains_key(path.as_str()),
                _ => true,
            };
            if !already_registered {
                match kind {
                    "templates" => {
                        config.assets.templates.insert(path.clone(), entry.clone());
                    }
                    "profiles" => {
                        config.assets.profiles.insert(path.clone(), entry.clone());
                    }
                    "snippets" => {
                        config.assets.snippets.insert(path.clone(), entry.clone());
                    }
                    "commands" => {
                        config.assets.commands.insert(path.clone(), entry.clone());
                    }
                    "recipes" => {
                        config.assets.recipes.insert(path.clone(), entry.clone());
                    }
                    "licenses" => {
                        config.assets.licenses.insert(path.clone(), entry.clone());
                    }
                    _ => {}
                }
                total += 1;
            }
        }
    }
    if total > 0 {
        save_global_config(config)?;
    }
    Ok(total)
}

pub fn auto_register_global_assets() -> Result<usize> {
    let mut config = load_global_config()?;
    let count = auto_register_assets(&mut config)?;
    if count > 0 {
        save_global_config(&config)?;
    }
    Ok(count)
}

pub fn save_global_config(config: &config::LodeConfig) -> Result<()> {
    config::validate_schema(config)?;
    ensure_global_workspace()?;
    let path = global_config_path()?;
    let encoded = toml::to_string_pretty(config)?;
    let parent = path
        .parent()
        .ok_or_else(|| LodeError::Message("global config path must have a parent".into()))?;
    trusted_root(parent)?.write_atomic(
        path.file_name()
            .ok_or_else(|| LodeError::Message("global config path must name a file".into()))?,
        encoded,
    )?;
    Ok(())
}

pub(crate) fn trusted_root(path: impl AsRef<std::path::Path>) -> Result<ValidatedRoot> {
    let path = path.as_ref();
    if !path.exists() {
        let parent = path
            .parent()
            .ok_or_else(|| LodeError::Message("install root must have a parent".into()))?;
        let name = path
            .file_name()
            .ok_or_else(|| LodeError::Message("install root must name a directory".into()))?;
        ValidatedRoot::new(parent)?.create_dir_all(name)?;
    }
    ValidatedRoot::new(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::EnvGuard;

    #[test]
    fn global_workspace_path_respects_lode_config() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("custom").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config.to_str().unwrap());

        assert_eq!(
            global_dir().unwrap(),
            Utf8PathBuf::from_path_buf(config.parent().unwrap().to_path_buf()).unwrap()
        );
        assert_eq!(
            global_config_path().unwrap(),
            Utf8PathBuf::from_path_buf(config).unwrap()
        );
    }

    #[test]
    fn global_asset_dir_respects_specific_overrides() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("custom-templates");
        let _templates_guard = EnvGuard::set("LODE_TEMPLATES", templates.to_str().unwrap());

        assert_eq!(
            global_asset_dir("templates").unwrap(),
            Utf8PathBuf::from_path_buf(templates).unwrap()
        );
    }

    #[test]
    fn old_global_config_is_migrated_and_backed_up() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".lode").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config_path.to_str().unwrap());

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(
            &config_path,
            r#"
schema_version = 2
active_profile = "systems/rust-cli"

[identity]
author = "Ada"
"#,
        )
        .unwrap();

        let config = load_global_config().unwrap();

        assert_eq!(config.schema_version, config::SCHEMA_VERSION);
        assert_eq!(config.identity.author, "Ada");
        assert_eq!(config.identity.email, "you@example.com");
        assert_eq!(config.active_profile.as_deref(), Some("systems/rust-cli"));
        assert!(config_path
            .with_file_name("config.toml.bak-schema-2")
            .exists());
        assert!(fs::read_to_string(&config_path)
            .unwrap()
            .contains("schema_version = 3"));
    }

    #[test]
    fn future_global_config_schema_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".lode").join("config.toml");
        let _guard = EnvGuard::set("LODE_CONFIG", config_path.to_str().unwrap());

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        let mut config = config::default_config();
        config.schema_version = config::SCHEMA_VERSION + 1;
        fs::write(&config_path, toml::to_string_pretty(&config).unwrap()).unwrap();

        let error = load_global_config().unwrap_err();

        assert!(matches!(
            error,
            LodeError::SchemaMismatch {
                expected: config::SCHEMA_VERSION,
                found
            } if found == config::SCHEMA_VERSION + 1
        ));
    }
}
