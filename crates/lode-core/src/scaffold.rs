use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::{
    assets, config,
    config::assets::EvaluateContext,
    fs_safety::ValidatedRoot,
    global_asset_dir, global_dir,
    template::{
        render_template, render_template_with_resolver, slug_to_class, slug_to_ident, RenderContext,
    },
    LodeError, Result,
};

const LODE_PROJECT_SUBDIRS: &[&str] = &["notes", "decisions", "plans"];

const GLOBAL_AGENT_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "CODEX.md",
    ".cursorrules",
    ".windsurfrules",
    ".mcp.json",
    "PLAN.md",
    "CONSTRAINTS.md",
    "REVIEW.md",
    "TASKS.md",
    "MEMORY.md",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitRequest {
    pub name: String,
    pub base_path: Utf8PathBuf,
    pub config: config::LodeConfig,
    pub profile: Option<String>,
    pub components: Vec<String>,
    pub dry_run: bool,
    pub overwrite: bool,
    pub lang: Option<String>,
    pub preset: Option<String>,
    pub license: Option<String>,
    pub in_place: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldReport {
    pub project_dir: Utf8PathBuf,
    pub planned_paths: Vec<Utf8PathBuf>,
    pub wrote_paths: Vec<Utf8PathBuf>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddRequest {
    pub project_dir: Utf8PathBuf,
    pub name: String,
    pub config: config::LodeConfig,
    pub component: String,
    pub dry_run: bool,
    pub overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub schema_version: u32,
    pub project: ProjectSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    pub created_by: String,
    pub created_at: String,
    pub profile: String,
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<ProjectDependency>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDependency {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldLock {
    pub schema_version: u32,
    pub generated_by: String,
    pub project: String,
    pub entries: Vec<ScaffoldLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldLockEntry {
    pub template: String,
    pub destination: Utf8PathBuf,
    pub content_hash: String,
}

#[derive(Debug, Clone)]
struct ManifestItem {
    template: &'static str,
    destination: Utf8PathBuf,
}

pub fn init_project(request: InitRequest) -> Result<ScaffoldReport> {
    let base_root = ValidatedRoot::new(&request.base_path)?;
    let project_dir = if request.in_place {
        let dir = request.base_path.clone();
        let config_path = dir.join(".lode").join("project.toml");
        if config_path.exists() && !request.overwrite {
            return Err(LodeError::AlreadyInitialised {
                path: PathBuf::from(config_path.as_str()),
            });
        }
        dir
    } else {
        let dir =
            Utf8PathBuf::from_path_buf(base_root.resolve(&request.name)?).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
        let config_path = dir.join(".lode").join("project.toml");
        if config_path.exists() && !request.overwrite {
            return Err(LodeError::AlreadyInitialised {
                path: PathBuf::from(config_path.as_str()),
            });
        }
        dir
    };

    let profile = request.profile.as_deref().unwrap_or("core/bare");
    let context = RenderContext::new()
        .with("project", &request.name)
        .with("project_ident", slug_to_ident(&request.name))
        .with("project_class", slug_to_class(&request.name))
        .with("author", &request.config.identity.author)
        .with("org", &request.config.identity.org)
        .with("license", &request.config.identity.license)
        .with("year", crate::current_year())
        .with("profile", profile);
    let mut manifest = scaffold_manifest(Some(profile), &request.components);

    let mut eval_ctx = EvaluateContext::from_env().with_profile(profile);
    eval_ctx.features_from_profile(profile);
    manifest.retain(|item| {
        let entry = request.config.assets.templates.get(item.template);
        match entry {
            Some(e) => {
                let active = e.is_active(&eval_ctx);
                if !active {
                    eprintln!(
                        "lode: skipping template '{}' (disabled in config)",
                        item.template
                    );
                }
                active
            }
            None => true,
        }
    });

    let mut planned_paths = Vec::new();
    planned_paths.push(project_dir.clone());
    for dir in &request.config.scaffold.always_dirs {
        planned_paths.push(project_dir.join(dir));
    }
    for item in &manifest {
        let rendered_destination =
            Utf8PathBuf::from(render_template(item.destination.as_str(), &context));
        planned_paths.push(project_dir.join(&rendered_destination));
    }

    if request.dry_run {
        return Ok(ScaffoldReport {
            project_dir,
            planned_paths,
            wrote_paths: Vec::new(),
            dry_run: true,
        });
    }

    let mut wrote_paths = Vec::new();
    if !request.in_place {
        base_root.create_dir_all(&request.name)?;
    }
    let root = ValidatedRoot::new(&project_dir)?;
    wrote_paths.push(project_dir.clone());

    for dir in &request.config.scaffold.always_dirs {
        let path = project_dir.join(dir);
        root.create_dir_all(dir)?;
        wrote_paths.push(path);
    }

    let lode_dir = project_dir.join(".lode");
    for subdir in LODE_PROJECT_SUBDIRS {
        let path = lode_dir.join(subdir);
        if !path.exists() {
            root.create_dir_all(path.strip_prefix(&project_dir).unwrap())?;
        }
        wrote_paths.push(path);
    }

    let mut lock_entries = Vec::new();
    for item in manifest {
        let rendered_destination =
            Utf8PathBuf::from(render_template(item.destination.as_str(), &context));
        let destination = project_dir.join(&rendered_destination);
        if destination.exists() && !request.overwrite {
            continue;
        }
        if let Some(parent) = rendered_destination.parent() {
            root.create_dir_all(parent)?;
        }
        let contents = if item.template == "lode/project.toml" {
            project_config_toml(&request.name, profile, &request.components)?
        } else {
            render_project_template(&project_dir, item.template, &context)
        };
        root.write_atomic(&rendered_destination, &contents)?;
        lock_entries.push(ScaffoldLockEntry {
            template: item.template.to_string(),
            destination: destination
                .strip_prefix(&project_dir)
                .unwrap_or(destination.as_ref())
                .to_path_buf(),
            content_hash: content_hash(&contents),
        });
        wrote_paths.push(destination);
    }
    write_scaffold_lock(
        &root,
        ScaffoldLock {
            schema_version: config::SCHEMA_VERSION,
            generated_by: "lode".to_string(),
            project: request.name.clone(),
            entries: lock_entries,
        },
    )?;

    if let Ok(agent_dir) = global_asset_dir("agents") {
        if agent_dir.exists() {
            let agent_path = agent_dir.as_std_path();
            for file_name in GLOBAL_AGENT_FILES {
                let src = agent_path.join(file_name);
                let dst = project_dir.as_std_path().join(file_name);
                if src.exists() && (!dst.exists() || request.overwrite) {
                    if let Ok(contents) = std::fs::read_to_string(&src) {
                        let _ = root.write_atomic(file_name, &contents);
                    }
                }
            }
            for subdir in &["skills", "prompts", "plans"] {
                let agent_sub = agent_path.join(subdir);
                if agent_sub.exists() {
                    let dest_dir = project_dir.join(".agents").join(subdir);
                    let dest_path = dest_dir.as_std_path();
                    let _ = std::fs::create_dir_all(dest_path);
                    if let Ok(entries) = std::fs::read_dir(&agent_sub) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    let dest_file = dest_path.join(name);
                                    if !dest_file.exists() || request.overwrite {
                                        let _ = std::fs::copy(&path, &dest_file);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ScaffoldReport {
        project_dir,
        planned_paths,
        wrote_paths,
        dry_run: false,
    })
}

pub fn add_component_to_project(request: AddRequest) -> Result<ScaffoldReport> {
    let context = RenderContext::new()
        .with("project", &request.name)
        .with("project_ident", slug_to_ident(&request.name))
        .with("project_class", slug_to_class(&request.name))
        .with("author", &request.config.identity.author)
        .with("org", &request.config.identity.org)
        .with("license", &request.config.identity.license)
        .with("year", crate::current_year())
        .with("profile", "core/bare");
    let mut manifest = Vec::new();
    add_component_items(&request.component, &mut manifest);

    let mut planned_paths = Vec::new();
    for item in &manifest {
        let rendered_destination =
            Utf8PathBuf::from(render_template(item.destination.as_str(), &context));
        planned_paths.push(request.project_dir.join(&rendered_destination));
    }

    if request.dry_run {
        return Ok(ScaffoldReport {
            project_dir: request.project_dir,
            planned_paths,
            wrote_paths: Vec::new(),
            dry_run: true,
        });
    }

    let root = ValidatedRoot::new(&request.project_dir)?;
    let mut wrote_paths = Vec::new();
    let mut lock = load_scaffold_lock(&request.project_dir).unwrap_or_else(|_| ScaffoldLock {
        schema_version: config::SCHEMA_VERSION,
        generated_by: "lode".to_string(),
        project: request.name.clone(),
        entries: Vec::new(),
    });
    for item in manifest {
        let rendered_destination =
            Utf8PathBuf::from(render_template(item.destination.as_str(), &context));
        let destination = request.project_dir.join(&rendered_destination);
        if destination.exists() && !request.overwrite {
            continue;
        }
        if let Some(parent) = rendered_destination.parent() {
            root.create_dir_all(parent)?;
        }
        let contents = render_project_template(&request.project_dir, item.template, &context);
        root.write_atomic(&rendered_destination, &contents)?;
        let relative = destination
            .strip_prefix(&request.project_dir)
            .unwrap_or(destination.as_ref())
            .to_path_buf();
        lock.entries.retain(|entry| entry.destination != relative);
        lock.entries.push(ScaffoldLockEntry {
            template: item.template.to_string(),
            destination: relative,
            content_hash: content_hash(&contents),
        });
        wrote_paths.push(destination);
    }
    write_scaffold_lock(&root, lock)?;

    Ok(ScaffoldReport {
        project_dir: request.project_dir,
        planned_paths,
        wrote_paths,
        dry_run: false,
    })
}

pub fn sync_project(
    project_dir: Utf8PathBuf,
    config: config::LodeConfig,
    force: bool,
    dry_run: bool,
) -> Result<ScaffoldReport> {
    let project_config = load_project_config(&project_dir)?;
    let root = ValidatedRoot::new(&project_dir)?;
    let profile = project_config.project.profile;
    let components = project_config.project.components;
    let context = RenderContext::new()
        .with("project", &project_config.project.name)
        .with("project_ident", slug_to_ident(&project_config.project.name))
        .with("project_class", slug_to_class(&project_config.project.name))
        .with("author", &config.identity.author)
        .with("org", &config.identity.org)
        .with("license", &config.identity.license)
        .with("year", crate::current_year())
        .with("profile", &profile);
    let manifest = scaffold_manifest(Some(&profile), &components);
    let mut planned_paths = Vec::new();
    let mut wrote_paths = Vec::new();
    let mut lock_entries = Vec::new();

    for item in manifest {
        if item.template == "lode/project.toml" {
            continue;
        }
        let rendered_destination =
            Utf8PathBuf::from(render_template(item.destination.as_str(), &context));
        let destination = project_dir.join(&rendered_destination);
        planned_paths.push(destination.clone());

        let mut contents = render_project_template(&project_dir, item.template, &context);
        if destination.exists() {
            let existing = fs::read_to_string(&destination).map_err(|source| LodeError::Io {
                path: PathBuf::from(destination.as_str()),
                source,
            })?;
            if !force {
                lock_entries.push(ScaffoldLockEntry {
                    template: item.template.to_string(),
                    destination: destination
                        .strip_prefix(&project_dir)
                        .unwrap_or(destination.as_ref())
                        .to_path_buf(),
                    content_hash: content_hash(&existing),
                });
                continue;
            }
            contents = preserve_user_content(&existing, &contents);
        }

        if dry_run {
            continue;
        }
        if let Some(parent) = rendered_destination.parent() {
            root.create_dir_all(parent)?;
        }
        root.write_atomic(&rendered_destination, &contents)?;
        lock_entries.push(ScaffoldLockEntry {
            template: item.template.to_string(),
            destination: destination
                .strip_prefix(&project_dir)
                .unwrap_or(destination.as_ref())
                .to_path_buf(),
            content_hash: content_hash(&contents),
        });
        wrote_paths.push(destination);
    }

    if !dry_run {
        write_scaffold_lock(
            &root,
            ScaffoldLock {
                schema_version: config::SCHEMA_VERSION,
                generated_by: "lode".to_string(),
                project: project_config.project.name,
                entries: lock_entries,
            },
        )?;
    }

    Ok(ScaffoldReport {
        project_dir,
        planned_paths,
        wrote_paths,
        dry_run,
    })
}

pub fn scaffold_lock_path(project_dir: &Utf8PathBuf) -> Utf8PathBuf {
    project_dir.join(".lode").join("scaffold.lock")
}

pub fn load_scaffold_lock(project_dir: &Utf8PathBuf) -> Result<ScaffoldLock> {
    let path = scaffold_lock_path(project_dir);
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
        path: PathBuf::from(path.as_str()),
        source: Box::new(source),
    })
}

fn write_scaffold_lock(root: &ValidatedRoot, mut lock: ScaffoldLock) -> Result<()> {
    lock.entries
        .sort_by(|left, right| left.destination.cmp(&right.destination));
    root.create_dir_all(".lode")?;
    let raw = toml::to_string_pretty(&lock)?;
    root.write_atomic(".lode/scaffold.lock", &raw)?;
    Ok(())
}

pub fn load_project_config(project_dir: &Utf8PathBuf) -> Result<ProjectConfig> {
    let path = project_dir.join(".lode").join("project.toml");
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    let config: ProjectConfig =
        toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
            path: PathBuf::from(path.as_str()),
            source: Box::new(source),
        })?;
    if config.schema_version != config::SCHEMA_VERSION {
        return Err(LodeError::SchemaMismatch {
            expected: config::SCHEMA_VERSION,
            found: config.schema_version,
        });
    }
    Ok(config)
}

fn resolve_template(project_dir: &Utf8PathBuf, template: &str, context: &RenderContext) -> String {
    let project_template = project_dir.join(".lode").join("templates").join(template);
    if let Ok(contents) = fs::read_to_string(&project_template) {
        return contents;
    }
    if let Ok(root) = global_dir() {
        let global_template = root.join("templates").join(template);
        if let Ok(contents) = fs::read_to_string(&global_template) {
            return contents;
        }
    }
    assets::template_contents(template, context)
}

fn render_project_template(
    project_dir: &Utf8PathBuf,
    template: &str,
    context: &RenderContext,
) -> String {
    let source = resolve_template(project_dir, template, context);
    render_template_with_resolver(&source, context, &|include| {
        Some(resolve_template(project_dir, include, context))
    })
}

fn content_hash(contents: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(contents.as_bytes());
    format!("{:064x}", hasher.finalize())
}

fn preserve_user_content(existing: &str, generated: &str) -> String {
    let markers: &[(&str, &str)] = &[
        ("<!-- lode:user-content -->", "<!-- /lode:user-content -->"),
        ("// lode:user-content", "// /lode:user-content"),
        ("# lode:user-content", "# /lode:user-content"),
        ("; lode:user-content", "; /lode:user-content"),
        ("-- lode:user-content", "-- /lode:user-content"),
    ];
    for &(begin, end) in markers {
        let has_existing = existing.contains(begin) && existing.contains(end);
        let has_generated = generated.contains(begin) && generated.contains(end);
        if has_existing && has_generated {
            let existing_start = existing.find(begin).unwrap();
            let existing_end_relative = existing[existing_start..].find(end).unwrap();
            let existing_end = existing_start + existing_end_relative + end.len();
            let generated_start = generated.find(begin).unwrap();
            let generated_end_relative = generated[generated_start..].find(end).unwrap();
            let generated_end = generated_start + generated_end_relative + end.len();
            return format!(
                "{}{}{}",
                &generated[..generated_start],
                &existing[existing_start..existing_end],
                &generated[generated_end..]
            );
        }
    }
    eprintln!("lode: warning: user content markers not found, preserving generated content");
    generated.to_string()
}

fn scaffold_manifest(profile: Option<&str>, components: &[String]) -> Vec<ManifestItem> {
    let mut items = vec![
        item("lode/project.toml", ".lode/project.toml"),
        item("root/README.md", "README.md"),
        item("root/CHANGELOG.md", "CHANGELOG.md"),
        item("root/CONTRIBUTING.md", "CONTRIBUTING.md"),
        item("root/LICENSE", "LICENSE"),
        item("root/Makefile", "Makefile"),
        item("dotfiles/env.example", ".env.example"),
        item("dotfiles/gitignore", ".gitignore"),
        item("dotfiles/gitattributes", ".gitattributes"),
        item("dotfiles/editorconfig", ".editorconfig"),
        item("ref/ARCHITECTURE.md", "_ref_/ARCHITECTURE.md"),
        item("ref/DECISIONS.md", "_ref_/DECISIONS.md"),
        item("ref/GLOSSARY.md", "_ref_/GLOSSARY.md"),
        item("ref/CONVENTIONS.md", "_ref_/CONVENTIONS.md"),
        item("ctx/PROJECT.md", "_ctx_/PROJECT.md"),
        item("ctx/ROADMAP.md", "_ctx_/ROADMAP.md"),
        item("ctx/STACK.md", "_ctx_/STACK.md"),
        item("ctx/RISKS.md", "_ctx_/RISKS.md"),
        item("ctx/NOTES.md", "_ctx_/NOTES.md"),
        item("ctx/TASKS.md", "_ctx_/TASKS.md"),
        item("agent/AGENTS.md", "AGENTS.md"),
        item("agent/CODEX.md", "CODEX.md"),
        item("agent/PLAN.md", ".lode/context/PLAN.md"),
        item("agent/CONSTRAINTS.md", ".lode/context/CONSTRAINTS.md"),
        item("agent/TASKS.md", ".lode/context/TASKS.md"),
    ];

    if let Some(profile) = profile {
        add_profile_items(profile, &mut items);
    }
    for component in components {
        add_component_items(component, &mut items);
    }
    items
}

fn add_profile_items(profile: &str, items: &mut Vec<ManifestItem>) {
    if profile.contains("rust") {
        items.extend([
            item("rust/Cargo.toml", "Cargo.toml"),
            item("rust/rust-toolchain.toml", "rust-toolchain.toml"),
            item("rust/src/main.rs", "src/main.rs"),
            item("rust/src/lib.rs", "src/lib.rs"),
            item("rust/tests/integration.rs", "tests/integration.rs"),
            item("ci/rust.yml", ".github/workflows/ci.yml"),
        ]);
    } else if profile.contains("go") {
        items.extend([
            item("go/go.mod", "go.mod"),
            item("go/cmd/project/main.go", "cmd/{{ project }}/main.go"),
            item("ci/go.yml", ".github/workflows/ci.yml"),
        ]);
    } else if profile.contains("python") || profile.contains("django") {
        items.extend([
            item("python/pyproject.toml", "pyproject.toml"),
            item("python/.python-version", ".python-version"),
            item(
                "python/src/project/__init__.py",
                "src/{{ project_ident }}/__init__.py",
            ),
            item(
                "python/src/project/main.py",
                "src/{{ project_ident }}/main.py",
            ),
            item("ci/python.yml", ".github/workflows/ci.yml"),
        ]);
        if profile.contains("django") {
            items.push(item("django/manage.py", "manage.py"));
        }
    } else if profile.contains("node")
        || profile.contains("react")
        || profile.contains("next")
        || profile.contains("astro")
        || profile.contains("svelte")
    {
        items.extend([
            item("node/package.json", "package.json"),
            item("node/tsconfig.json", "tsconfig.json"),
            item("node/src/index.ts", "src/index.ts"),
            item("ci/node.yml", ".github/workflows/ci.yml"),
        ]);
    } else if profile.contains("tauri") {
        items.extend([
            item("tauri/package.json", "package.json"),
            item("tauri/src-tauri/Cargo.toml", "src-tauri/Cargo.toml"),
            item("ci/node.yml", ".github/workflows/ci.yml"),
        ]);
    } else if profile.contains("java") || profile.contains("minecraft") {
        items.extend([
            item("java/build.gradle", "build.gradle"),
            item("java/settings.gradle", "settings.gradle"),
            item(
                "java/src/main/java/app/Main.java",
                "src/main/java/app/Main.java",
            ),
        ]);
        if profile.contains("minecraft") {
            items.push(item("minecraft/fabric/build.gradle", "fabric/build.gradle"));
        }
    } else if profile.contains("cpp") || profile.contains("competitive-cpp") {
        items.extend([
            item("cpp/CMakeLists.txt", "CMakeLists.txt"),
            item("cpp/src/main.cpp", "src/main.cpp"),
        ]);
    } else if profile.contains("c-app") || profile.contains("c-lib") {
        items.extend([
            item("c/CMakeLists.txt", "CMakeLists.txt"),
            item("c/src/main.c", "src/main.c"),
        ]);
    } else if profile.contains("zig") {
        items.extend([
            item("zig/build.zig", "build.zig"),
            item("zig/src/main.zig", "src/main.zig"),
        ]);
    } else if profile.contains("competitive") {
        items.extend([
            item("competitive/problems/a/main.cpp", "problems/a/main.cpp"),
            item("competitive/templates/cpp.cpp", "templates/cpp.cpp"),
            item("competitive/scripts/run.sh", "scripts/run.sh"),
        ]);
    }
}

fn add_component_items(component: &str, items: &mut Vec<ManifestItem>) {
    match component {
        "ci" | "github-actions" => {
            items.push(item("github/workflows/ci.yml", ".github/workflows/ci.yml"));
        }
        "security" => items.push(item(
            "github/workflows/security.yml",
            ".github/workflows/security.yml",
        )),
        "release" => items.push(item(
            "github/workflows/release.yml",
            ".github/workflows/release.yml",
        )),
        "docker" => {
            items.push(item("docker/Dockerfile", "Dockerfile"));
            items.push(item("docker/compose.yml", "compose.yml"));
            items.push(item("dotfiles/dockerignore", ".dockerignore"));
        }
        "devcontainer" => items.push(item(
            "devcontainer/devcontainer.json",
            ".devcontainer/devcontainer.json",
        )),
        "vscode" => {
            items.push(item("vscode/settings.json", ".vscode/settings.json"));
            items.push(item("vscode/extensions.json", ".vscode/extensions.json"));
            items.push(item("vscode/tasks.json", ".vscode/tasks.json"));
            items.push(item("vscode/launch.json", ".vscode/launch.json"));
        }
        "zed" => {
            items.push(item("zed/settings.json", ".zed/settings.json"));
            items.push(item("zed/tasks.json", ".zed/tasks.json"));
        }
        "nvim" | "neovim" => {
            items.push(item("neovim/lode.lua", ".config/nvim/lua/lode.lua"));
            items.push(item(
                "neovim/snippets.lua",
                ".config/nvim/lua/lode/snippets.lua",
            ));
        }
        "agent" | "agent-all" => {
            items.push(item("agent/CLAUDE.md", "CLAUDE.md"));
            items.push(item("agent/.cursorrules", ".cursorrules"));
            items.push(item("agent/.windsurfrules", ".windsurfrules"));
            items.push(item("agent/.mcp.json", ".mcp.json"));
        }
        "docs" => {
            items.push(item("docs/index.md", "docs/index.md"));
            items.push(item("docs/getting-started.md", "docs/getting-started.md"));
            items.push(item("docs/usage.md", "docs/usage.md"));
        }
        _ => {}
    }
}

fn item(template: &'static str, destination: &str) -> ManifestItem {
    ManifestItem {
        template,
        destination: Utf8PathBuf::from(destination),
    }
}

fn project_config_toml(name: &str, profile: &str, components: &[String]) -> Result<String> {
    let config = ProjectConfig {
        schema_version: config::SCHEMA_VERSION,
        project: ProjectSection {
            name: name.to_string(),
            created_by: "lode".to_string(),
            created_at: created_at(),
            profile: profile.to_string(),
            components: components.to_vec(),
            language: None,
            toolchain: None,
            assets: None,
            dependencies: None,
        },
    };

    Ok(toml::to_string_pretty(&config)?)
}

fn created_at() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_writes_nothing() {
        let temp = tempfile::tempdir().unwrap();
        let base_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();

        let report = init_project(InitRequest {
            name: "my-app".to_string(),
            base_path: base_path.clone(),
            config: config::default_config(),
            profile: None,
            components: Vec::new(),
            dry_run: true,
            overwrite: false,
            lang: None,
            preset: None,
            license: None,
            in_place: false,
        })
        .unwrap();

        assert!(report.dry_run);
        assert!(report.wrote_paths.is_empty());
        assert!(!base_path.join("my-app").exists());
    }

    #[test]
    fn init_rejects_project_name_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let base_path = Utf8PathBuf::from_path_buf(temp.path().join("base")).unwrap();
        fs::create_dir(&base_path).unwrap();

        let error = init_project(InitRequest {
            name: "../escape".to_string(),
            base_path,
            config: config::default_config(),
            profile: None,
            components: Vec::new(),
            dry_run: false,
            overwrite: false,
            lang: None,
            preset: None,
            license: None,
            in_place: false,
        })
        .unwrap_err();

        assert!(
            matches!(error, LodeError::Message(message) if message.contains("parent traversal"))
        );
        assert!(!temp.path().join("escape").exists());
    }

    #[test]
    fn existing_project_config_returns_exists() {
        let temp = tempfile::tempdir().unwrap();
        let base_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let lode_dir = base_path.join("my-app").join(".lode");
        fs::create_dir_all(&lode_dir).unwrap();
        fs::write(lode_dir.join("project.toml"), "").unwrap();

        let error = init_project(InitRequest {
            name: "my-app".to_string(),
            base_path,
            config: config::default_config(),
            profile: None,
            components: Vec::new(),
            dry_run: false,
            overwrite: false,
            lang: None,
            preset: None,
            license: None,
            in_place: false,
        })
        .unwrap_err();

        assert!(matches!(error, LodeError::AlreadyInitialised { .. }));
    }

    #[test]
    fn init_writes_scaffold_lock_and_uses_project_template_override() {
        let temp = tempfile::tempdir().unwrap();
        let base_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let project = base_path.join("my-app");
        let override_path = project
            .join(".lode")
            .join("templates")
            .join("root")
            .join("README.md");
        fs::create_dir_all(override_path.parent().unwrap()).unwrap();
        fs::write(&override_path, "# {{ project }} override\n").unwrap();

        init_project(InitRequest {
            name: "my-app".to_string(),
            base_path,
            config: config::default_config(),
            profile: None,
            components: Vec::new(),
            dry_run: false,
            overwrite: true,
            lang: None,
            preset: None,
            license: None,
            in_place: false,
        })
        .unwrap();

        assert!(scaffold_lock_path(&project).exists());
        assert_eq!(
            fs::read_to_string(project.join("README.md")).unwrap(),
            "# my-app override\n"
        );
        assert!(load_scaffold_lock(&project)
            .unwrap()
            .entries
            .iter()
            .any(|entry| entry.destination == Utf8PathBuf::from("README.md")));
    }

    #[test]
    fn project_templates_can_include_partials() {
        let temp = tempfile::tempdir().unwrap();
        let base_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let project = base_path.join("my-app");
        let templates = project.join(".lode").join("templates");
        let readme_override = templates.join("root").join("README.md");
        let partial = templates.join("partials").join("badge.md");
        fs::create_dir_all(readme_override.parent().unwrap()).unwrap();
        fs::create_dir_all(partial.parent().unwrap()).unwrap();
        fs::write(
            &readme_override,
            "# {{ project }}\n{% include \"partials/badge.md\" %}\n",
        )
        .unwrap();
        fs::write(&partial, "badge={{ project | kebab }}\n").unwrap();

        init_project(InitRequest {
            name: "my-app".to_string(),
            base_path,
            config: config::default_config(),
            profile: None,
            components: Vec::new(),
            dry_run: false,
            overwrite: true,
            lang: None,
            preset: None,
            license: None,
            in_place: false,
        })
        .unwrap();

        let readme = fs::read_to_string(project.join("README.md")).unwrap();
        assert!(readme.contains("# my-app"));
        assert!(readme.contains("badge=my-app"));
    }

    #[test]
    fn sync_force_preserves_user_content_regions() {
        let temp = tempfile::tempdir().unwrap();
        let base_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let project = base_path.join("my-app");

        init_project(InitRequest {
            name: "my-app".to_string(),
            base_path,
            config: config::default_config(),
            profile: None,
            components: Vec::new(),
            dry_run: false,
            overwrite: false,
            lang: None,
            preset: None,
            license: None,
            in_place: false,
        })
        .unwrap();

        let override_path = project
            .join(".lode")
            .join("templates")
            .join("root")
            .join("README.md");
        fs::create_dir_all(override_path.parent().unwrap()).unwrap();
        fs::write(
            &override_path,
            "# {{ project }} v2\n<!-- lode:user-content -->\nnew\n<!-- /lode:user-content -->\n",
        )
        .unwrap();
        fs::write(
            project.join("README.md"),
            "# my-app custom\n<!-- lode:user-content -->\nkeep me\n<!-- /lode:user-content -->\n",
        )
        .unwrap();

        sync_project(project.clone(), config::default_config(), true, false).unwrap();

        let readme = fs::read_to_string(project.join("README.md")).unwrap();
        assert!(readme.contains("# my-app v2"));
        assert!(readme.contains("keep me"));
    }
}
