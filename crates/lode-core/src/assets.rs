use camino::Utf8PathBuf;

use crate::template::{slug_to_class, slug_to_ident, RenderContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Template,
    Profile,
    Snippet,
    Command,
    Recipe,
    License,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetSpec {
    pub kind: AssetKind,
    pub path: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedAsset {
    pub kind: AssetKind,
    pub path: Utf8PathBuf,
    pub contents: String,
}

const CORE_PROFILES: &[&str] = &[
    "core/bare",
    "core/app",
    "core/lib",
    "core/cli",
    "core/service",
    "core/web-app",
    "core/docs-site",
    "core/workspace",
    "core/hackathon",
];

const SYSTEM_PROFILES: &[&str] = &[
    "systems/c-app",
    "systems/c-lib",
    "systems/cpp-app",
    "systems/cpp-lib",
    "systems/rust-bin",
    "systems/rust-lib",
    "systems/rust-cli",
    "systems/rust-service",
    "systems/rust-workspace",
    "systems/go-cli",
    "systems/go-service",
    "systems/go-lib",
    "systems/zig-app",
    "systems/zig-lib",
    "systems/zig-cli",
];

const OTHER_PROFILES: &[&str] = &[
    "backend/node-app",
    "backend/node-lib",
    "backend/express-api",
    "backend/fastify-api",
    "backend/nest-api",
    "frontend/vite-app",
    "frontend/react-app",
    "frontend/next-app",
    "frontend/astro-site",
    "frontend/sveltekit-app",
    "python/python-app",
    "python/python-lib",
    "python/python-cli",
    "python/django-app",
    "python/django-api",
    "java/java-app",
    "java/java-lib",
    "java/java-gradle",
    "java/java-maven",
    "desktop/tauri-app",
    "desktop/tauri-react",
    "desktop/tauri-vite",
    "desktop/tauri-vanilla",
    "games/minecraft-fabric-mod",
    "games/minecraft-forge-mod",
    "games/minecraft-neoforge-mod",
    "games/minecraft-paper-plugin",
    "challenge/competitive-cpp",
    "challenge/competitive-rust",
    "challenge/competitive-python",
    "challenge/competitive-java",
    "challenge/competitive-mixed",
];

const TEMPLATE_PATHS: &[&str] = &[
    "root/README.md",
    "root/CHANGELOG.md",
    "root/CONTRIBUTING.md",
    "root/CODE_OF_CONDUCT.md",
    "root/LICENSE",
    "root/Makefile",
    "root/justfile",
    "root/SECURITY.md",
    "root/SUPPORT.md",
    "root/NOTICE",
    "dotfiles/gitignore",
    "dotfiles/gitattributes",
    "dotfiles/editorconfig",
    "dotfiles/env.example",
    "dotfiles/env.local.example",
    "dotfiles/env.test.example",
    "dotfiles/dockerignore",
    "ref/ARCHITECTURE.md",
    "ref/DECISIONS.md",
    "ref/GLOSSARY.md",
    "ref/CONVENTIONS.md",
    "ref/DEPENDENCIES.md",
    "ref/SECURITY_MODEL.md",
    "ctx/PROJECT.md",
    "ctx/ROADMAP.md",
    "ctx/STACK.md",
    "ctx/RISKS.md",
    "ctx/NOTES.md",
    "ctx/TASKS.md",
    "ctx/OPEN_QUESTIONS.md",
    "docs/index.md",
    "docs/getting-started.md",
    "docs/usage.md",
    "docs/configuration.md",
    "docs/commands.md",
    "docs/api.md",
    "docs/deployment.md",
    "docs/troubleshooting.md",
    "docs/faq.md",
    "github/workflows/ci.yml",
    "github/workflows/security.yml",
    "github/workflows/release.yml",
    "github/ISSUE_TEMPLATE/bug_report.md",
    "github/ISSUE_TEMPLATE/feature_request.md",
    "github/PULL_REQUEST_TEMPLATE.md",
    "github/dependabot.yml",
    "github/renovate.json",
    "github/CODEOWNERS",
    "gitlab/gitlab-ci.yml",
    "ci/generic.yml",
    "ci/rust.yml",
    "ci/node.yml",
    "ci/python.yml",
    "ci/go.yml",
    "docker/Dockerfile",
    "docker/compose.yml",
    "devcontainer/devcontainer.json",
    "vscode/settings.json",
    "vscode/extensions.json",
    "vscode/tasks.json",
    "zed/settings.json",
    "neovim/lode.lua",
    "agent/AGENTS.md",
    "agent/CLAUDE.md",
    "agent/CODEX.md",
    "agent/.cursorrules",
    "agent/.windsurfrules",
    "agent/.mcp.json",
    "agent/PLAN.md",
    "agent/CONSTRAINTS.md",
    "agent/REVIEW.md",
    "agent/TASKS.md",
    "agent/MEMORY.md",
    "os/common/bootstrap.sh",
    "os/windows/bootstrap.ps1",
    "c/CMakeLists.txt",
    "c/src/main.c",
    "cpp/CMakeLists.txt",
    "cpp/src/main.cpp",
    "rust/Cargo.toml",
    "rust/rust-toolchain.toml",
    "rust/src/main.rs",
    "rust/src/lib.rs",
    "rust/tests/integration.rs",
    "go/go.mod",
    "go/cmd/project/main.go",
    "zig/build.zig",
    "zig/src/main.zig",
    "java/build.gradle",
    "java/settings.gradle",
    "java/src/main/java/app/Main.java",
    "node/package.json",
    "node/tsconfig.json",
    "node/src/index.ts",
    "python/pyproject.toml",
    "python/.python-version",
    "python/src/project/__init__.py",
    "python/src/project/main.py",
    "django/manage.py",
    "tauri/package.json",
    "tauri/src-tauri/Cargo.toml",
    "minecraft/fabric/build.gradle",
    "competitive/problems/a/main.cpp",
    "competitive/templates/cpp.cpp",
    "competitive/scripts/run.sh",
];

const SNIPPET_GROUPS: &[(&str, &[&str])] = &[
    ("any", &["todo", "fixme", "note", "invariant", "example"]),
    ("md", &["adr", "risk", "task-list", "release-notes"]),
    (
        "rs",
        &["main", "error-enum", "result-alias", "serde-struct", "test"],
    ),
    ("go", &["main", "handler", "table-test"]),
    ("ts", &["fn", "async-fn", "interface", "test"]),
    ("py", &["main-guard", "dataclass", "pytest-test"]),
    ("sh", &["strict-header", "die", "parse-args"]),
    ("yaml", &["github-job", "docker-compose-service"]),
    ("toml", &["table", "lode-command", "lode-profile"]),
    (
        "cp",
        &["fast-io-cpp", "dijkstra", "union-find", "segment-tree"],
    ),
];

const COMMANDS: &[&str] = &[
    "health",
    "verify",
    "fresh",
    "setup-dev",
    "doctor-fix",
    "clean-all",
    "fmt-all",
    "lint-all",
    "test-all",
    "run",
    "dev",
    "build",
    "release-local",
    "ship",
    "explain",
    "cp-new",
    "cp-run",
    "cp-stress",
    "hackathon-demo",
    "hackathon-package",
    "tauri-dev",
    "tauri-build",
    "mc-run-client",
    "mc-run-server",
    "gha-validate",
];

const RECIPES: &[&str] = &[
    "docker-basic",
    "devcontainer-basic",
    "github-actions-basic",
    "github-actions-release",
    "release-basic",
    "security-basic",
    "workspace-basic",
    "agent-basic",
    "docs-basic",
    "django-postgres",
    "django-rest-framework",
    "tauri-updater",
    "minecraft-fabric",
    "minecraft-forge",
    "minecraft-paper",
    "competitive-coding",
    "hackathon-demo",
];

const LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "MIT OR Apache-2.0",
    "BSD-3-Clause",
    "ISC",
];

pub fn profile_names() -> Vec<&'static str> {
    CORE_PROFILES
        .iter()
        .chain(SYSTEM_PROFILES)
        .chain(OTHER_PROFILES)
        .copied()
        .collect()
}

pub fn command_names() -> &'static [&'static str] {
    COMMANDS
}

pub fn recipe_names() -> &'static [&'static str] {
    RECIPES
}

pub fn template_paths() -> &'static [&'static str] {
    TEMPLATE_PATHS
}

pub fn default_assets(context: &RenderContext) -> Vec<RenderedAsset> {
    let mut assets = Vec::new();
    for profile in profile_names() {
        assets.push(RenderedAsset {
            kind: AssetKind::Profile,
            path: Utf8PathBuf::from(format!("{profile}.toml")),
            contents: profile_contents(profile),
        });
    }
    for path in TEMPLATE_PATHS {
        assets.push(RenderedAsset {
            kind: AssetKind::Template,
            path: Utf8PathBuf::from(path),
            contents: template_contents(path, context),
        });
    }
    for (lang, names) in SNIPPET_GROUPS {
        for name in *names {
            assets.push(RenderedAsset {
                kind: AssetKind::Snippet,
                path: Utf8PathBuf::from(format!("{lang}/{name}.snippet")),
                contents: snippet_contents(lang, name),
            });
        }
    }
    for command in COMMANDS {
        assets.push(RenderedAsset {
            kind: AssetKind::Command,
            path: Utf8PathBuf::from(format!("{command}.toml")),
            contents: command_contents(command),
        });
    }
    for recipe in RECIPES {
        assets.push(RenderedAsset {
            kind: AssetKind::Recipe,
            path: Utf8PathBuf::from(format!("{recipe}.toml")),
            contents: recipe_contents(recipe),
        });
    }
    for license in LICENSES {
        assets.push(RenderedAsset {
            kind: AssetKind::License,
            path: Utf8PathBuf::from(format!("{license}.txt")),
            contents: license_contents(license),
        });
    }
    assets
}

pub fn template_contents(path: &str, context: &RenderContext) -> String {
    let project = context.get("project").unwrap_or("project");
    let ident = slug_to_ident(project);
    let class = slug_to_class(project);
    match path {
        "root/README.md" => format!(
            "# {project}\n\nGenerated with Lode.\n\n## Usage\n\n```sh\nmake help\n```\n"
        ),
        "root/CHANGELOG.md" => "# Changelog\n\nAll notable changes are documented here.\n".to_string(),
        "root/CONTRIBUTING.md" => "# Contributing\n\nUse conventional commits, run `make verify`, and keep changes small.\n".to_string(),
        "root/CODE_OF_CONDUCT.md" => "# Code of Conduct\n\nBe respectful, direct, and constructive.\n".to_string(),
        "root/LICENSE" => "MIT OR Apache-2.0\n".to_string(),
        "root/Makefile" => makefile_contents(),
        "root/justfile" => "default:\n    just --list\n\nverify:\n    make verify\n".to_string(),
        "root/SECURITY.md" => "# Security\n\nReport vulnerabilities privately to the project maintainer.\n".to_string(),
        "root/SUPPORT.md" => "# Support\n\nOpen an issue with reproduction steps and environment details.\n".to_string(),
        "root/NOTICE" => format!("{project} includes generated project foundation files from Lode.\n"),
        "dotfiles/gitignore" => ".env\n.env.*\n!.env.example\ntarget/\nnode_modules/\n__pycache__/\n.DS_Store\n".to_string(),
        "dotfiles/gitattributes" => "* text=auto eol=lf\n*.ps1 text eol=crlf\n".to_string(),
        "dotfiles/editorconfig" => "root = true\n\n[*]\ncharset = utf-8\nend_of_line = lf\ninsert_final_newline = true\nindent_style = space\nindent_size = 4\n\n[*.{json,yml,yaml,toml,md}]\nindent_size = 2\n".to_string(),
        "dotfiles/env.example" => format!("APP_NAME={project}\nAPP_ENV=development\nLOG_LEVEL=debug\nPORT=3000\n"),
        "dotfiles/env.local.example" => "APP_ENV=local\nLOG_LEVEL=debug\n".to_string(),
        "dotfiles/env.test.example" => "APP_ENV=test\nLOG_LEVEL=info\n".to_string(),
        "dotfiles/dockerignore" => ".git\ntarget\nnode_modules\n.env\n".to_string(),
        "ref/ARCHITECTURE.md" => format!("# Architecture\n\n## System\n\n{project} architecture notes live here.\n"),
        "ref/DECISIONS.md" => "# Decisions\n\n## ADR-0001: Initial project foundation\n\nStatus: accepted\n\nLode generated the initial structure.\n".to_string(),
        "ref/GLOSSARY.md" => "# Glossary\n\nAdd domain terms here.\n".to_string(),
        "ref/CONVENTIONS.md" => "# Conventions\n\n- Use conventional commits.\n- Keep generated files reviewed.\n- Run `make verify` before shipping.\n".to_string(),
        "ref/DEPENDENCIES.md" => "# Dependencies\n\nTrack important dependency decisions here.\n".to_string(),
        "ref/SECURITY_MODEL.md" => "# Security Model\n\nDocument trust boundaries, secrets, and data handling.\n".to_string(),
        "ctx/PROJECT.md" => format!("# Project\n\nName: {project}\n\nPurpose: TODO\n"),
        "ctx/ROADMAP.md" => "# Roadmap\n\n- [ ] Define first milestone\n- [ ] Ship first release\n".to_string(),
        "ctx/STACK.md" => "# Stack\n\nRecord languages, frameworks, and tools here.\n".to_string(),
        "ctx/RISKS.md" => "# Risks\n\n- Unknowns should be written down early.\n".to_string(),
        "ctx/NOTES.md" => "# Notes\n\nWorking notes live here.\n".to_string(),
        "ctx/TASKS.md" => "# Tasks\n\n- [ ] Replace generated placeholders.\n".to_string(),
        "ctx/OPEN_QUESTIONS.md" => "# Open Questions\n\n- What should be decided before the first release?\n".to_string(),
        "docs/index.md" => format!("# {project} Documentation\n\nStart here.\n"),
        "docs/getting-started.md" => "# Getting Started\n\nRun `make install` then `make dev`.\n".to_string(),
        "docs/usage.md" => "# Usage\n\nDocument common workflows here.\n".to_string(),
        "docs/configuration.md" => "# Configuration\n\nDocument configuration keys here.\n".to_string(),
        "docs/commands.md" => "# Commands\n\nUse `make help`.\n".to_string(),
        "docs/api.md" => "# API\n\nDocument public API here.\n".to_string(),
        "docs/deployment.md" => "# Deployment\n\nDocument release and deployment steps.\n".to_string(),
        "docs/troubleshooting.md" => "# Troubleshooting\n\nRecord known issues and fixes.\n".to_string(),
        "docs/faq.md" => "# FAQ\n\nAdd common questions here.\n".to_string(),
        "github/workflows/ci.yml" | "ci/generic.yml" => "name: CI\non: [push, pull_request]\njobs:\n  verify:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: make verify\n".to_string(),
        "github/workflows/security.yml" | "ci/security.yml" => "name: Security\non: [push, pull_request]\njobs:\n  scan:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: make audit\n".to_string(),
        "github/workflows/release.yml" | "ci/release.yml" => "name: Release\non:\n  push:\n    tags: ['v*']\njobs:\n  release:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: make release\n".to_string(),
        "github/ISSUE_TEMPLATE/bug_report.md" => "---\nname: Bug report\n---\n\n## Expected\n\n## Actual\n\n## Reproduction\n".to_string(),
        "github/ISSUE_TEMPLATE/feature_request.md" => "---\nname: Feature request\n---\n\n## Problem\n\n## Proposal\n".to_string(),
        "github/PULL_REQUEST_TEMPLATE.md" => "## Summary\n\n## Tests\n\n## Risks\n".to_string(),
        "github/dependabot.yml" => "version: 2\nupdates: []\n".to_string(),
        "github/renovate.json" => "{\n  \"extends\": [\"config:recommended\"]\n}\n".to_string(),
        "github/CODEOWNERS" => "* @maintainers\n".to_string(),
        "gitlab/gitlab-ci.yml" => "verify:\n  script:\n    - make verify\n".to_string(),
        "ci/rust.yml" => "name: Rust CI\non: [push, pull_request]\njobs:\n  rust:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo test --workspace\n".to_string(),
        "ci/node.yml" => "name: Node CI\non: [push, pull_request]\njobs:\n  node:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: npm ci && npm test\n".to_string(),
        "ci/python.yml" => "name: Python CI\non: [push, pull_request]\njobs:\n  python:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: uv sync && uv run pytest\n".to_string(),
        "ci/go.yml" => "name: Go CI\non: [push, pull_request]\njobs:\n  go:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: go test ./...\n".to_string(),
        "docker/Dockerfile" => "FROM alpine:3.20\nWORKDIR /app\nCOPY . .\nCMD [\"sh\"]\n".to_string(),
        "docker/compose.yml" => "services:\n  app:\n    build: .\n    env_file: .env.example\n".to_string(),
        "devcontainer/devcontainer.json" => "{\n  \"name\": \"{{ project }}\",\n  \"image\": \"mcr.microsoft.com/devcontainers/base:ubuntu\"\n}\n".replace("{{ project }}", project),
        "vscode/settings.json" => "{\n  \"editor.formatOnSave\": true\n}\n".to_string(),
        "vscode/extensions.json" => "{\n  \"recommendations\": []\n}\n".to_string(),
        "vscode/tasks.json" => "{\n  \"version\": \"2.0.0\",\n  \"tasks\": [{\"label\":\"verify\",\"type\":\"shell\",\"command\":\"make verify\"}]\n}\n".to_string(),
        "zed/settings.json" => "{\n  \"format_on_save\": \"on\"\n}\n".to_string(),
        "neovim/lode.lua" => "vim.keymap.set('n', '<leader>lv', ':make verify<CR>')\n".to_string(),
        "agent/AGENTS.md" | "agent/CLAUDE.md" | "agent/CODEX.md" => format!("# Agent Context for {project}\n\nRead `_ref_` for permanent truth and `_ctx_` for current working context.\n"),
        "agent/.cursorrules" | "agent/.windsurfrules" => "Follow `_ref_/CONVENTIONS.md`; prefer small, verified changes.\n".to_string(),
        "agent/.mcp.json" => "{\n  \"servers\": {}\n}\n".to_string(),
        "agent/PLAN.md" => "# Plan\n\n- [ ] Define implementation plan.\n".to_string(),
        "agent/CONSTRAINTS.md" => "# Constraints\n\n- Preserve user changes.\n- Verify before finalizing.\n".to_string(),
        "agent/REVIEW.md" => "# Review Notes\n\nFindings go here.\n".to_string(),
        "agent/TASKS.md" => "# Agent Tasks\n\n- [ ] Keep context current.\n".to_string(),
        "agent/MEMORY.md" => "# Memory\n\nDurable project notes go here.\n".to_string(),
        "os/common/bootstrap.sh" => "#!/usr/bin/env sh\nset -eu\nprintf 'bootstrap common tools\\n'\n".to_string(),
        "os/windows/bootstrap.ps1" => "Write-Host 'bootstrap Windows tools'\n".to_string(),
        "c/CMakeLists.txt" => format!("cmake_minimum_required(VERSION 3.20)\nproject({ident})\nadd_executable({ident} src/main.c)\n"),
        "cpp/CMakeLists.txt" => format!("cmake_minimum_required(VERSION 3.20)\nproject({ident})\nadd_executable({ident} src/main.cpp)\n"),
        "c/src/main.c" => "#include <stdio.h>\n\nint main(void) {\n    puts(\"hello from {{ project }}\");\n    return 0;\n}\n".replace("{{ project }}", project),
        "cpp/src/main.cpp" => "#include <iostream>\n\nint main() {\n    std::cout << \"hello from {{ project }}\\n\";\n}\n".replace("{{ project }}", project),
        "rust/Cargo.toml" => format!("[package]\nname = \"{project}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"),
        "rust/rust-toolchain.toml" => "[toolchain]\nchannel = \"stable\"\n".to_string(),
        "rust/src/main.rs" => format!("fn main() {{\n    println!(\"hello from {project}\");\n}}\n"),
        "rust/src/lib.rs" => format!("pub fn name() -> &'static str {{\n    \"{project}\"\n}}\n"),
        "rust/tests/integration.rs" => "use {{ project }}::name;\n\n#[test]\nfn exposes_name() {\n    assert!(!name().is_empty());\n}\n".replace("{{ project }}", &ident),
        "go/go.mod" => format!("module {project}\n\ngo 1.22\n"),
        "go/cmd/project/main.go" => "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"hello from {{ project }}\")\n}\n".replace("{{ project }}", project),
        "zig/build.zig" => "const std = @import(\"std\");\npub fn build(b: *std.Build) void { _ = b; }\n".to_string(),
        "zig/src/main.zig" => "const std = @import(\"std\");\npub fn main() !void { try std.io.getStdOut().writer().print(\"hello\\n\", .{}); }\n".to_string(),
        "java/build.gradle" => "plugins { id 'application' }\nrepositories { mavenCentral() }\napplication { mainClass = 'app.Main' }\n".to_string(),
        "java/settings.gradle" => format!("rootProject.name = '{project}'\n"),
        "java/src/main/java/app/Main.java" => format!("package app;\n\npublic class Main {{\n  public static void main(String[] args) {{ System.out.println(\"hello from {project}\"); }}\n}}\n"),
        "node/package.json" => format!("{{\n  \"name\": \"{project}\",\n  \"version\": \"0.1.0\",\n  \"type\": \"module\",\n  \"scripts\": {{\"dev\":\"tsx src/index.ts\",\"build\":\"tsc\",\"test\":\"node --test\"}},\n  \"devDependencies\": {{}}\n}}\n"),
        "node/tsconfig.json" => "{\n  \"compilerOptions\": {\"target\":\"ES2022\",\"module\":\"NodeNext\",\"moduleResolution\":\"NodeNext\",\"strict\":true}\n}\n".to_string(),
        "node/src/index.ts" => format!("console.log('hello from {project}');\n"),
        "python/pyproject.toml" => format!("[project]\nname = \"{project}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\n\n[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n"),
        "python/.python-version" => "3.11\n".to_string(),
        "python/src/project/__init__.py" => format!("__all__ = [\"main\"]\n__version__ = \"0.1.0\"\nPROJECT = \"{project}\"\n"),
        "python/src/project/main.py" => format!("def main() -> None:\n    print(\"hello from {project}\")\n\nif __name__ == \"__main__\":\n    main()\n"),
        "django/manage.py" => "#!/usr/bin/env python\nfrom django.core.management import execute_from_command_line\nexecute_from_command_line()\n".to_string(),
        "tauri/package.json" => format!("{{\"name\":\"{project}\",\"version\":\"0.1.0\",\"scripts\":{{\"dev\":\"tauri dev\",\"build\":\"tauri build\"}}}}\n"),
        "tauri/src-tauri/Cargo.toml" => format!("[package]\nname = \"{ident}_app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        "minecraft/fabric/build.gradle" => "plugins { id 'fabric-loom' version '1.6-SNAPSHOT' }\n".to_string(),
        "competitive/problems/a/main.cpp" | "competitive/templates/cpp.cpp" => "#include <bits/stdc++.h>\nusing namespace std;\nint main(){ios::sync_with_stdio(false);cin.tie(nullptr);}\n".to_string(),
        "competitive/scripts/run.sh" => "#!/usr/bin/env sh\nset -eu\ng++ -std=c++20 \"$1\" && ./a.out\n".to_string(),
        _ => format!("# {}\n\nGenerated default asset for {}.\n", path, class),
    }
}

fn profile_contents(profile: &str) -> String {
    let language = if profile.contains("rust") {
        "rust"
    } else if profile.contains("go") {
        "go"
    } else if profile.contains("python") || profile.contains("django") {
        "python"
    } else if profile.contains("node")
        || profile.contains("react")
        || profile.contains("next")
        || profile.contains("astro")
        || profile.contains("svelte")
        || profile.contains("tauri")
    {
        "typescript"
    } else if profile.contains("java") || profile.contains("minecraft") {
        "java"
    } else if profile.contains("cpp") || profile.contains("competitive-cpp") {
        "cpp"
    } else if profile.contains("c-app") || profile.contains("c-lib") {
        "c"
    } else if profile.contains("zig") {
        "zig"
    } else {
        "generic"
    };
    format!(
        "schema_version = 3\nname = \"{profile}\"\nlanguage = \"{language}\"\ndescription = \"Default {profile} profile\"\n\n[scaffold]\ninclude_core = true\ninclude_agent = true\n"
    )
}

fn snippet_contents(lang: &str, name: &str) -> String {
    format!("name: {name}\nlang: {lang}\n---\n{name} $1\n")
}

fn command_contents(command: &str) -> String {
    format!(
        "slug = \"{command}\"\ndescription = \"Default {command} command macro\"\n\n[[steps]]\nkind = \"make\"\nrun = \"{}\"\n",
        command.replace("-all", "")
    )
}

fn recipe_contents(recipe: &str) -> String {
    format!(
        "name = \"{recipe}\"\ndescription = \"Default optional {recipe} capability bundle\"\n\n[[files]]\ntemplate = \"docs/index.md\"\ndest = \"docs/{recipe}.md\"\n"
    )
}

fn license_contents(license: &str) -> String {
    match license {
        "MIT" => "MIT License\n\nCopyright (c) {{ year }} {{ author }}\n\nPermission is hereby granted, free of charge, to any person obtaining a copy.\n".to_string(),
        "Apache-2.0" => "Apache License\nVersion 2.0, January 2004\nhttp://www.apache.org/licenses/\n".to_string(),
        "MIT OR Apache-2.0" => "Licensed under either of Apache License, Version 2.0 or MIT license at your option.\n".to_string(),
        other => format!("{other}\n\nDefault license placeholder. Replace with full license text if needed.\n"),
    }
}

fn makefile_contents() -> String {
    "help:\n\t@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | sort\n\ndev: ## Run development server\n\t@echo \"No dev command configured\"\n\nbuild: ## Build project\n\t@echo \"No build command configured\"\n\ntest: ## Run tests\n\t@echo \"No test command configured\"\n\ntest-watch: ## Run tests in watch mode\n\t@echo \"No watch command configured\"\n\nfmt: ## Format project\n\t@echo \"No format command configured\"\n\nlint: ## Lint project\n\t@echo \"No lint command configured\"\n\ncheck: ## Run checks\n\t$(MAKE) fmt lint test\n\nverify: ## Format, lint, test, audit\n\t$(MAKE) check\n\naudit: ## Run audit\n\t@echo \"No audit command configured\"\n\nclean: ## Clean generated files\n\t@echo \"No clean command configured\"\n\ndocs: ## Serve docs\n\t@echo \"No docs command configured\"\n\ninstall: ## Install dependencies\n\t@echo \"No install command configured\"\n\nupdate: ## Update dependencies\n\t@echo \"No update command configured\"\n\nrelease: ## Prepare release\n\t@echo \"No release command configured\"\n".to_string()
}
