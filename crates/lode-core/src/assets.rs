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
    "ci/c.yml",
    "ci/cpp.yml",
    "ci/rust.yml",
    "ci/go.yml",
    "ci/zig.yml",
    "ci/java-gradle.yml",
    "ci/java-maven.yml",
    "ci/node.yml",
    "ci/python.yml",
    "ci/django.yml",
    "ci/tauri.yml",
    "ci/minecraft-gradle.yml",
    "ci/docs.yml",
    "ci/release.yml",
    "ci/security.yml",
    "docker/Dockerfile",
    "docker/compose.yml",
    "devcontainer/devcontainer.json",
    "vscode/settings.json",
    "vscode/extensions.json",
    "vscode/tasks.json",
    "vscode/launch.json",
    "zed/settings.json",
    "zed/tasks.json",
    "neovim/lode.lua",
    "neovim/snippets.lua",
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
    "mc-run-client",
    "mc-run-server",
    "gha-validate",
    "tauri-dev",
    "tauri-build",
    "commit",
    "git-commit",
    "git-tag",
    "git-changelog",
    "git-install-hooks",
    "git-sign-setup",
    "git-remote-setup",
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
    "GPL-3.0-only",
    "MPL-2.0",
    "Unlicense",
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

pub fn known_asset_paths_by_kind(kind: AssetKind) -> Vec<String> {
    match kind {
        AssetKind::Profile => profile_names().iter().map(|s| s.to_string()).collect(),
        AssetKind::Template => TEMPLATE_PATHS.iter().map(|s| s.to_string()).collect(),
        AssetKind::Snippet => SNIPPET_GROUPS
            .iter()
            .flat_map(|(lang, names)| names.iter().map(move |name| format!("{lang}/{name}")))
            .collect(),
        AssetKind::Command => COMMANDS.iter().map(|s| s.to_string()).collect(),
        AssetKind::Recipe => RECIPES.iter().map(|s| s.to_string()).collect(),
        AssetKind::License => LICENSES.iter().map(|s| s.to_string()).collect(),
    }
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

fn vscode_settings() -> String {
    r#"{
  "editor.formatOnSave": true,
  "editor.tabSize": 2,
  "editor.rulers": [100],
  "editor.trimAutoWhitespace": true,
  "files.insertFinalNewline": true,
  "files.trimTrailingWhitespace": true,
  "files.exclude": {
    "**/node_modules": true,
    "**/target": true,
    "**/.git": true,
    "**/dist": true
  },
  "search.exclude": {
    "**/node_modules": true,
    "**/target": true,
    "**/*.lock": true
  },
  "lode.binaryPath": "lode",
  "lode.startDaemonOnOpen": true,
  "lode.stampOnSave": true,
  "lode.enforceRenameOnSave": true,
  "lode.showStatusBar": true,
  "lode.showDiagnostics": true,
  "lode.useLsp": true,
  "lode.checkOnSave": false,
  "lode.mcpPort": 3847
}
"#
    .to_string()
}

fn vscode_extensions() -> String {
    r#"{
  "recommendations": [
    "lode-rs.lode-vscode",
    "EditorConfig.EditorConfig",
    "tamasfe.even-better-toml",
    "usernamehw.errorlens"
  ]
}
"#
    .to_string()
}

fn vscode_tasks() -> String {
    r#"{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "lode: check conventions",
      "type": "shell",
      "command": "lode check",
      "group": "test",
      "problemMatcher": []
    },
    {
      "label": "lode: fix conventions",
      "type": "shell",
      "command": "lode fix",
      "problemMatcher": []
    },
    {
      "label": "lode: sign file",
      "type": "shell",
      "command": "lode sign \"${file}\"",
      "problemMatcher": []
    },
    {
      "label": "lode: stamp file",
      "type": "shell",
      "command": "lode stamp \"${file}\"",
      "problemMatcher": []
    },
    {
      "label": "lode: health",
      "type": "shell",
      "command": "lode health",
      "group": "test",
      "problemMatcher": []
    },
    {
      "label": "lode: audit",
      "type": "shell",
      "command": "lode audit",
      "problemMatcher": []
    },
    {
      "label": "lode: scan secrets",
      "type": "shell",
      "command": "lode scan secrets",
      "problemMatcher": []
    },
    {
      "label": "lode: time today",
      "type": "shell",
      "command": "lode time today",
      "problemMatcher": []
    },
    {
      "label": "lode: open dashboard",
      "type": "shell",
      "command": "lode serve",
      "presentation": { "reveal": "always", "panel": "dedicated" },
      "problemMatcher": []
    },
    {
      "label": "lode: agent sync",
      "type": "shell",
      "command": "lode agent sync",
      "problemMatcher": []
    },
    {
      "label": "lode: config show",
      "type": "shell",
      "command": "lode config show --format json",
      "problemMatcher": []
    },
    {
      "label": "lode: daemon start",
      "type": "shell",
      "command": "lode daemon start",
      "problemMatcher": []
    },
    {
      "label": "lode: daemon stop",
      "type": "shell",
      "command": "lode daemon stop",
      "problemMatcher": []
    },
    {
      "label": "lode: release rollback preview",
      "type": "shell",
      "command": "lode release --rollback --dry-run",
      "problemMatcher": []
    }
  ]
}
"#
    .to_string()
}

fn vscode_launch() -> String {
    r#"{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Lode: serve dashboard",
      "type": "node-terminal",
      "request": "launch",
      "command": "lode serve"
    },
    {
      "name": "Lode: run tests",
      "type": "node-terminal",
      "request": "launch",
      "command": "lode test"
    },
    {
      "name": "Lode: language server",
      "type": "node-terminal",
      "request": "launch",
      "command": "lode lsp --stdio"
    },
    {
      "name": "Lode: MCP server",
      "type": "node-terminal",
      "request": "launch",
      "command": "lode mcp --http --port 3847"
    }
  ]
}
"#
    .to_string()
}

fn zed_settings() -> String {
    r#"{
  "tab_size": 2,
  "hard_tabs": false,
  "format_on_save": "on",
  "formatter": "language_server",
  "lode": {
    "binary": "lode",
    "start_daemon_on_open": true,
    "stamp_on_save": true,
    "use_lsp": true,
    "mcp_port": 3847,
    "snippet_export": "lode snippet export --format zed --out ~/.config/zed/snippets.json"
  }
}
"#
    .to_string()
}

fn zed_tasks() -> String {
    r#"[
  { "label": "lode: check conventions", "command": "lode", "args": ["check"] },
  { "label": "lode: fix conventions", "command": "lode", "args": ["fix"] },
  { "label": "lode: sign file", "command": "lode", "args": ["sign", "$ZED_FILE"] },
  { "label": "lode: stamp file", "command": "lode", "args": ["stamp", "$ZED_FILE"] },
  { "label": "lode: health", "command": "lode", "args": ["health"] },
  { "label": "lode: audit", "command": "lode", "args": ["audit"] },
  { "label": "lode: scan secrets", "command": "lode", "args": ["scan", "secrets"] },
  { "label": "lode: time today", "command": "lode", "args": ["time", "today"] },
  { "label": "lode: config show", "command": "lode", "args": ["config", "show", "--format", "json"] },
  { "label": "lode: snippet export", "command": "lode", "args": ["snippet", "export", "--format", "zed"] },
  { "label": "lode: open dashboard", "command": "lode", "args": ["serve"] },
  { "label": "lode: git commit", "command": "lode", "args": ["git", "commit"] },
  { "label": "lode: release", "command": "lode", "args": ["release"] },
  { "label": "lode: release rollback preview", "command": "lode", "args": ["release", "--rollback", "--dry-run"] },
  { "label": "lode: agent sync", "command": "lode", "args": ["agent", "sync"] },
  { "label": "lode: daemon start", "command": "lode", "args": ["daemon", "start"] },
  { "label": "lode: daemon status", "command": "lode", "args": ["daemon", "status"] },
  { "label": "lode: daemon stop", "command": "lode", "args": ["daemon", "stop"] }
]
"#
    .to_string()
}

fn neovim_lode_lua() -> String {
    r#"local M = {}

M.opts = {
  binary = "lode",
  daemon_auto_start = true,
  stamp_on_save = true,
  enforce_rename = true,
  use_lsp = true,
  show_statusline = true,
  sign_column = true,
  which_key_prefix = "<leader>l",
  telescope = true,
}

local function in_lode_project()
  return vim.fn.filereadable(".lode/project.toml") == 1
end

local function run(args)
  return vim.fn.jobstart(vim.list_extend({ M.opts.binary }, args), { detach = true })
end

local function run_current_file(command)
  local file = vim.fn.expand("%:p")
  if file ~= "" then
    run({ command, file })
  end
end

local function terminal(args)
  vim.cmd("terminal " .. M.opts.binary .. " " .. table.concat(args, " "))
end

local function setup_lsp()
  if not M.opts.use_lsp then
    return
  end
  local ok_configs, configs = pcall(require, "lspconfig.configs")
  local ok_lspconfig, lspconfig = pcall(require, "lspconfig")
  if not (ok_configs and ok_lspconfig) then
    return
  end
  if not configs.lode_lsp then
    configs.lode_lsp = {
      default_config = {
        cmd = { M.opts.binary, "lsp", "--stdio" },
        filetypes = { "*" },
        root_dir = function(fname)
          local marker = vim.fs.find(".lode/project.toml", { path = fname, upward = true })[1]
          if marker then
            return vim.fs.dirname(marker)
          end
          return nil
        end,
        settings = {},
      },
    }
  end
  lspconfig.lode_lsp.setup({})
end

local function setup_telescope()
  if not M.opts.telescope then
    return
  end
  local ok, telescope = pcall(require, "telescope")
  if ok then
    telescope.load_extension("lode")
  end
end

function M.daemon_toggle()
  run({ "daemon", "status", "--json" })
  run({ "daemon", "start" })
end

function M.setup(opts)
  M.opts = vim.tbl_extend("force", M.opts, opts or {})
  setup_lsp()
  setup_telescope()
  vim.api.nvim_create_autocmd("VimEnter", {
    callback = function()
      if M.opts.daemon_auto_start and in_lode_project() then
        run({ "daemon", "start" })
      end
    end,
  })
  vim.api.nvim_create_autocmd("BufWritePost", {
    callback = function()
      if M.opts.stamp_on_save and in_lode_project() then
        run_current_file("stamp")
      end
    end,
  })
  vim.keymap.set("n", M.opts.which_key_prefix .. "c", function() run({ "check" }) end, { desc = "Lode check" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "f", function() run({ "fix" }) end, { desc = "Lode fix" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "s", function() run_current_file("sign") end, { desc = "Lode sign file" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "t", function() run_current_file("stamp") end, { desc = "Lode stamp file" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "a", function() run({ "audit" }) end, { desc = "Lode audit" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "i", function() terminal({ "info" }) end, { desc = "Lode info" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "m", function() terminal({ "time", "today" }) end, { desc = "Lode time today" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "v", function() terminal({ "serve" }) end, { desc = "Lode dashboard" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "g", function() terminal({ "git", "commit" }) end, { desc = "Lode git commit" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "r", function() terminal({ "release" }) end, { desc = "Lode release" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "d", M.daemon_toggle, { desc = "Lode daemon toggle" })
  vim.keymap.set("n", M.opts.which_key_prefix .. "x", function() run({ "agent", "sync" }) end, { desc = "Lode agent sync" })

  vim.api.nvim_create_user_command("LodeCheck", function() run({ "check" }) end, {})
  vim.api.nvim_create_user_command("LodeFix", function() run({ "fix" }) end, {})
  vim.api.nvim_create_user_command("LodeAudit", function() terminal({ "audit" }) end, {})
  vim.api.nvim_create_user_command("LodeSign", function() run_current_file("sign") end, {})
  vim.api.nvim_create_user_command("LodeStamp", function() run_current_file("stamp") end, {})
  vim.api.nvim_create_user_command("LodeRelease", function() terminal({ "release" }) end, {})
  vim.api.nvim_create_user_command("LodeSecrets", function() terminal({ "scan", "secrets" }) end, {})
  vim.api.nvim_create_user_command("LodeTime", function() terminal({ "time", "today" }) end, {})
  vim.api.nvim_create_user_command("LodeServe", function() terminal({ "serve" }) end, {})
  vim.api.nvim_create_user_command("LodeSync", function() run({ "agent", "sync" }) end, {})
end

function M.statusline()
  if not in_lode_project() then
    return ""
  end
  return "lode"
end

return M
"#
    .to_string()
}

fn neovim_snippets_lua() -> String {
    r#"return {
  lode_header = {
    prefix = "lode-header",
    body = {
      "---",
      "title: ${1:title}",
      "created: ${2:today}",
      "updated: ${2:today}",
      "owner: ${3:owner}",
      "---",
      "",
      "$0",
    },
    description = "Lode metadata header",
  },
  lode_task = {
    prefix = "lode-task",
    body = { "- [ ] ${1:task}" },
    description = "Lode markdown task",
  },
}
"#
    .to_string()
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
        "ci/c.yml" => "name: C CI\non: [push, pull_request]\njobs:\n  c:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cmake -S . -B build -DCMAKE_BUILD_TYPE=Release\n      - run: cmake --build build --parallel\n      - run: ctest --test-dir build --output-on-failure\n".to_string(),
        "ci/cpp.yml" => "name: C++ CI\non: [push, pull_request]\njobs:\n  cpp:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cmake -S . -B build -DCMAKE_BUILD_TYPE=Release\n      - run: cmake --build build --parallel\n      - run: ctest --test-dir build --output-on-failure\n".to_string(),
        "ci/rust.yml" => "name: Rust CI\non: [push, pull_request]\njobs:\n  rust:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo test --workspace\n".to_string(),
        "ci/go.yml" => "name: Go CI\non: [push, pull_request]\njobs:\n  go:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: go test ./...\n".to_string(),
        "ci/zig.yml" => "name: Zig CI\non: [push, pull_request]\njobs:\n  zig:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: mlugg/setup-zig@v2\n      - run: zig fmt --check .\n      - run: zig build test\n".to_string(),
        "ci/java-gradle.yml" => "name: Java Gradle CI\non: [push, pull_request]\njobs:\n  gradle:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/setup-java@v4\n        with:\n          distribution: temurin\n          java-version: '21'\n          cache: gradle\n      - run: ./gradlew build\n".to_string(),
        "ci/java-maven.yml" => "name: Java Maven CI\non: [push, pull_request]\njobs:\n  maven:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/setup-java@v4\n        with:\n          distribution: temurin\n          java-version: '21'\n          cache: maven\n      - run: ./mvnw --batch-mode verify\n".to_string(),
        "ci/node.yml" => "name: Node CI\non: [push, pull_request]\njobs:\n  node:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: npm ci && npm test\n".to_string(),
        "ci/python.yml" => "name: Python CI\non: [push, pull_request]\njobs:\n  python:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: uv sync && uv run pytest\n".to_string(),
        "ci/django.yml" => "name: Django CI\non: [push, pull_request]\njobs:\n  django:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: astral-sh/setup-uv@v5\n      - run: uv sync --frozen\n      - run: uv run python manage.py check\n      - run: uv run pytest\n".to_string(),
        "ci/tauri.yml" => "name: Tauri CI\non: [push, pull_request]\njobs:\n  tauri:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22\n          cache: npm\n      - uses: dtolnay/rust-toolchain@stable\n      - run: npm ci\n      - run: npm run build\n      - run: cargo test --manifest-path src-tauri/Cargo.toml\n".to_string(),
        "ci/minecraft-gradle.yml" => "name: Minecraft Gradle CI\non: [push, pull_request]\njobs:\n  mod:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/setup-java@v4\n        with:\n          distribution: temurin\n          java-version: '21'\n          cache: gradle\n      - run: ./gradlew build\n      - uses: actions/upload-artifact@v4\n        with:\n          name: mod-jars\n          path: build/libs/*.jar\n".to_string(),
        "ci/docs.yml" => "name: Docs CI\non: [push, pull_request]\njobs:\n  docs:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: make docs\n".to_string(),
        "docker/Dockerfile" => "FROM alpine:3.20\nWORKDIR /app\nCOPY . .\nCMD [\"sh\"]\n".to_string(),
        "docker/compose.yml" => "services:\n  app:\n    build: .\n    env_file: .env.example\n".to_string(),
        "devcontainer/devcontainer.json" => "{\n  \"name\": \"{{ project }}\",\n  \"image\": \"mcr.microsoft.com/devcontainers/base:ubuntu\"\n}\n".replace("{{ project }}", project),
        "vscode/settings.json" => vscode_settings(),
        "vscode/extensions.json" => vscode_extensions(),
        "vscode/tasks.json" => vscode_tasks(),
        "vscode/launch.json" => vscode_launch(),
        "zed/settings.json" => zed_settings(),
        "zed/tasks.json" => zed_tasks(),
        "neovim/lode.lua" => neovim_lode_lua(),
        "neovim/snippets.lua" => neovim_snippets_lua(),
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
    let (lang, desc, checks, targets, vars): (
        &str,
        &str,
        &[(&str, &str)],
        &[(&str, &str, &str)],
        &[&str],
    ) = match profile {
        // ── Core profiles ──────────────────────────────────────────────
        "core/bare" => (
            "generic",
            "Minimal empty project skeleton with no language assumptions",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build the project"),
                ("test", "make test", "Run tests"),
                ("clean", "make clean", "Clean build artifacts"),
            ],
            &[],
        ),
        "core/app" => (
            "generic",
            "Generic application project with standard tooling",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build the project"),
                ("test", "make test", "Run tests"),
                ("dev", "make dev", "Start development server"),
                ("clean", "make clean", "Clean build artifacts"),
                ("install", "make install", "Install dependencies"),
            ],
            &["APP_ENV=development", "LOG_LEVEL=debug"],
        ),
        "core/lib" => (
            "generic",
            "Generic library project",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build the library"),
                ("test", "make test", "Run tests"),
                ("doc", "make docs", "Generate documentation"),
                ("clean", "make clean", "Clean build artifacts"),
            ],
            &[],
        ),
        "core/cli" => (
            "generic",
            "Generic CLI application project",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build the CLI"),
                ("test", "make test", "Run tests"),
                ("install", "make install", "Install the CLI binary"),
                ("clean", "make clean", "Clean build artifacts"),
            ],
            &["LOG_LEVEL=info"],
        ),
        "core/service" => (
            "generic",
            "Generic long-running service daemon",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build the service"),
                ("test", "make test", "Run tests"),
                ("dev", "make dev", "Run service in dev mode"),
                ("clean", "make clean", "Clean build artifacts"),
            ],
            &["APP_ENV=development", "PORT=8080", "LOG_LEVEL=debug"],
        ),
        "core/web-app" => (
            "generic",
            "Generic web application",
            &[("make", "make --version")],
            &[
                ("dev", "make dev", "Start the dev server"),
                ("build", "make build", "Build for production"),
                ("test", "make test", "Run tests"),
                ("lint", "make lint", "Lint the codebase"),
                ("clean", "make clean", "Clean build artifacts"),
            ],
            &["NODE_ENV=development", "PORT=3000"],
        ),
        "core/docs-site" => (
            "generic",
            "Documentation site project",
            &[("make", "make --version")],
            &[
                ("dev", "make dev", "Start docs dev server"),
                ("build", "make build", "Build static docs site"),
                ("serve", "make serve", "Serve built docs"),
            ],
            &[],
        ),
        "core/workspace" => (
            "generic",
            "Multi-crate or multi-module workspace",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build all workspace members"),
                ("test", "make test", "Test all workspace members"),
                ("clean", "make clean", "Clean all build artifacts"),
                ("lint", "make lint", "Lint all workspace members"),
            ],
            &[],
        ),
        "core/hackathon" => (
            "generic",
            "Rapid prototyping hackathon project",
            &[("make", "make --version")],
            &[
                ("dev", "make dev", "Start live-reload dev environment"),
                ("build", "make build", "Build for demo"),
                ("demo", "make demo", "Start the demo"),
            ],
            &["HACKATHON=1", "LOG_LEVEL=debug", "PORT=3000"],
        ),

        // ── System profiles: Rust ─────────────────────────────────────
        p if p.starts_with("systems/rust-") => {
            let sub = p.strip_prefix("systems/rust-").unwrap_or("app");
            let desc = match sub {
                "bin" => "Rust binary application",
                "lib" => "Rust library crate",
                "cli" => "Rust CLI application with clap argument parsing",
                "service" => "Rust service with HTTP framework",
                "workspace" => "Rust workspace with multiple crates",
                _ => "Rust project",
            };
            let targets: &[(&str, &str, &str)] = match sub {
                "lib" => &[
                    ("build", "cargo build", "Build the library"),
                    ("test", "cargo test", "Run tests"),
                    ("doc", "cargo doc --no-deps", "Generate documentation"),
                    ("fmt", "cargo fmt --check", "Check formatting"),
                    ("clippy", "cargo clippy -- -D warnings", "Lint with clippy"),
                ],
                "workspace" => &[
                    ("build", "cargo build --workspace", "Build all crates"),
                    ("test", "cargo test --workspace", "Test all crates"),
                    ("fmt", "cargo fmt --check", "Check formatting"),
                    (
                        "clippy",
                        "cargo clippy --workspace -- -D warnings",
                        "Lint all crates",
                    ),
                ],
                _ => &[
                    ("build", "cargo build", "Build the project"),
                    ("test", "cargo test", "Run tests"),
                    ("fmt", "cargo fmt --check", "Check formatting"),
                    ("clippy", "cargo clippy -- -D warnings", "Lint with clippy"),
                    ("run", "cargo run", "Run the binary"),
                ],
            };
            (
                "rust",
                desc,
                &[("rustc", "rustc --version"), ("cargo", "cargo --version")],
                targets,
                &["CARGO_TERM_COLOR=always"],
            )
        }

        // ── System profiles: Go ───────────────────────────────────────
        p if p.starts_with("systems/go-") => {
            let sub = p.strip_prefix("systems/go-").unwrap_or("app");
            let desc = match sub {
                "cli" => "Go CLI application with cobra or standard flag package",
                "service" => "Go HTTP service with standard library or chi/gin",
                "lib" => "Go library module",
                _ => "Go project",
            };
            (
                "go",
                desc,
                &[("go", "go version")],
                &[
                    ("build", "go build ./...", "Build all packages"),
                    ("test", "go test ./... -v -count=1", "Run all tests"),
                    ("vet", "go vet ./...", "Vet all packages"),
                    ("fmt", "gofmt -l .", "Check formatting"),
                ],
                &["GOPROXY=https://proxy.golang.org,direct", "GO111MODULE=on"],
            )
        }

        // ── System profiles: C ────────────────────────────────────────
        p if p == "systems/c-app" || p == "systems/c-lib" => {
            let sub = p.strip_prefix("systems/c-").unwrap_or("app");
            (
                "c",
                format!(
                    "C {} with CMake build system",
                    if sub == "app" {
                        "application"
                    } else {
                        "library"
                    }
                )
                .leak(),
                &[
                    ("gcc", "gcc --version"),
                    ("cmake", "cmake --version"),
                    ("make", "make --version"),
                ],
                &[
                    (
                        "build",
                        "cmake -S . -B build && cmake --build build",
                        "Configure and build",
                    ),
                    (
                        "test",
                        "ctest --test-dir build --output-on-failure",
                        "Run tests",
                    ),
                    (
                        "clean",
                        "cmake --build build --target clean",
                        "Clean build artifacts",
                    ),
                ],
                &[],
            )
        }

        // ── System profiles: C++ ──────────────────────────────────────
        p if p == "systems/cpp-app" || p == "systems/cpp-lib" => {
            let sub = p.strip_prefix("systems/cpp-").unwrap_or("app");
            (
                "cpp",
                format!(
                    "C++ {} with CMake build system",
                    if sub == "app" {
                        "application"
                    } else {
                        "library"
                    }
                )
                .leak(),
                &[
                    ("g++", "g++ --version"),
                    ("cmake", "cmake --version"),
                    ("make", "make --version"),
                ],
                &[
                    (
                        "build",
                        "cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build",
                        "Configure and build",
                    ),
                    (
                        "test",
                        "ctest --test-dir build --output-on-failure",
                        "Run tests",
                    ),
                    (
                        "clean",
                        "cmake --build build --target clean",
                        "Clean build artifacts",
                    ),
                ],
                &[],
            )
        }

        // ── System profiles: Zig ──────────────────────────────────────
        p if p.starts_with("systems/zig-") => {
            let sub = p.strip_prefix("systems/zig-").unwrap_or("app");
            let desc = match sub {
                "app" => "Zig application",
                "lib" => "Zig library",
                "cli" => "Zig CLI tool",
                _ => "Zig project",
            };
            (
                "zig",
                desc,
                &[("zig", "zig version")],
                &[
                    ("build", "zig build", "Build the project"),
                    ("test", "zig build test", "Run tests"),
                    ("fmt", "zig fmt --check .", "Check formatting"),
                ],
                &[],
            )
        }

        // ── Backend profiles: Node/TypeScript ─────────────────────────
        p if p.starts_with("backend/") => {
            let sub = p.strip_prefix("backend/").unwrap_or("node-app");
            let (desc, extra_targets): (&str, &[(&str, &str, &str)]) = match sub {
                "node-app" => ("Node.js application with TypeScript", &[]),
                "node-lib" => (
                    "Node.js library with TypeScript",
                    &[("doc", "npm run docs", "Generate documentation")],
                ),
                "express-api" => (
                    "Express.js REST API with TypeScript",
                    &[("dev", "npm run dev", "Start dev server with hot reload")],
                ),
                "fastify-api" => (
                    "Fastify REST API with TypeScript",
                    &[("dev", "npm run dev", "Start dev server with hot reload")],
                ),
                "nest-api" => (
                    "NestJS modular API with TypeScript",
                    &[(
                        "dev",
                        "npm run start:dev",
                        "Start dev server with watch mode",
                    )],
                ),
                _ => ("Node.js project", &[]),
            };
            let mut targets = vec![
                ("build", "npm run build", "Build TypeScript to JavaScript"),
                ("test", "npm test", "Run tests"),
                ("lint", "npm run lint", "Lint the codebase"),
                ("clean", "npm run clean", "Clean build artifacts"),
            ];
            targets.extend_from_slice(extra_targets);
            (
                "typescript",
                desc,
                &[("node", "node --version"), ("npm", "npm --version")],
                targets.leak(),
                &["NODE_ENV=development"],
            )
        }

        // ── Frontend profiles ─────────────────────────────────────────
        p if p.starts_with("frontend/") => {
            let sub = p.strip_prefix("frontend/").unwrap_or("vite-app");
            let desc = match sub {
                "vite-app" => "Vite-powered frontend project",
                "react-app" => "React single-page application with Vite",
                "next-app" => "Next.js full-stack application",
                "astro-site" => "Astro static site or content site",
                "sveltekit-app" => "SvelteKit application",
                _ => "Frontend web project",
            };
            (
                "typescript",
                desc,
                &[("node", "node --version"), ("npm", "npm --version")],
                &[
                    ("dev", "npm run dev", "Start the dev server"),
                    ("build", "npm run build", "Build for production"),
                    ("preview", "npm run preview", "Preview production build"),
                    ("test", "npm test", "Run tests"),
                    ("lint", "npm run lint", "Lint the codebase"),
                ],
                &["NODE_ENV=development"],
            )
        }

        // ── Python profiles ──────────────────────────────────────────
        p if p.starts_with("python/") => {
            let sub = p.strip_prefix("python/").unwrap_or("python-app");
            let desc = match sub {
                "python-app" => "Python application with uv/pip",
                "python-lib" => "Python library package",
                "python-cli" => "Python CLI tool with argparse or click",
                "django-app" => "Django web application",
                "django-api" => "Django REST Framework API",
                _ => "Python project",
            };
            (
                "python",
                desc,
                &[("python3", "python3 --version"), ("uv", "uv --version")],
                &[
                    ("sync", "uv sync", "Sync dependencies"),
                    ("test", "uv run pytest", "Run tests with pytest"),
                    ("lint", "uv run ruff check .", "Lint with ruff"),
                    (
                        "fmt-check",
                        "uv run ruff format --check .",
                        "Check formatting with ruff",
                    ),
                ],
                &["PYTHONDONTWRITEBYTECODE=1", "PYTHONUNBUFFERED=1"],
            )
        }

        // ── Java profiles ────────────────────────────────────────────
        p if p.starts_with("java/") => {
            let (desc, checks, targets): (&str, &[(&str, &str)], &[(&str, &str, &str)]) =
                match profile {
                    "java/java-gradle" => (
                        "Java project with Gradle build tool",
                        &[("java", "java --version"), ("gradle", "gradle --version")],
                        &[
                            ("build", "./gradlew build", "Build with Gradle"),
                            ("test", "./gradlew test", "Run tests"),
                            ("clean", "./gradlew clean", "Clean build artifacts"),
                        ],
                    ),
                    "java/java-maven" => (
                        "Java project with Maven build tool",
                        &[("java", "java --version"), ("mvn", "mvn --version")],
                        &[
                            ("build", "mvn --batch-mode compile", "Compile with Maven"),
                            ("test", "mvn --batch-mode test", "Run tests"),
                            (
                                "package",
                                "mvn --batch-mode package",
                                "Package the artifact",
                            ),
                            ("clean", "mvn --batch-mode clean", "Clean build artifacts"),
                        ],
                    ),
                    _ => (
                        "Java project",
                        &[("java", "java --version"), ("javac", "javac --version")],
                        &[
                            ("build", "make build", "Build the project"),
                            ("test", "make test", "Run tests"),
                            ("clean", "make clean", "Clean build artifacts"),
                        ],
                    ),
                };
            ("java", desc, checks, targets, &[])
        }

        // ── Desktop / Tauri profiles ──────────────────────────────────
        p if p.starts_with("desktop/tauri-") => {
            let sub = p.strip_prefix("desktop/tauri-").unwrap_or("app");
            let desc = match sub {
                "app" => "Tauri desktop application",
                "react" => "Tauri desktop app with React frontend",
                "vite" => "Tauri desktop app with Vite frontend",
                "vanilla" => "Tauri desktop app with vanilla HTML/CSS/JS",
                _ => "Tauri desktop application",
            };
            (
                "typescript",
                desc,
                &[
                    ("node", "node --version"),
                    ("npm", "npm --version"),
                    ("rustc", "rustc --version"),
                    ("cargo", "cargo --version"),
                ],
                &[
                    ("dev", "npm run tauri dev", "Start Tauri dev environment"),
                    (
                        "build",
                        "npm run tauri build",
                        "Build Tauri app for production",
                    ),
                    (
                        "test",
                        "cargo test --manifest-path src-tauri/Cargo.toml",
                        "Run Rust backend tests",
                    ),
                ],
                &["NODE_ENV=development"],
            )
        }

        // ── Games / Minecraft profiles ────────────────────────────────
        p if p.starts_with("games/minecraft-") => {
            let sub = p.strip_prefix("games/minecraft-").unwrap_or("fabric-mod");
            let desc = match sub {
                "fabric-mod" => "Minecraft Fabric mod with Gradle",
                "forge-mod" => "Minecraft Forge mod with Gradle",
                "neoforge-mod" => "Minecraft NeoForge mod with Gradle",
                "paper-plugin" => "Minecraft Paper plugin with Gradle",
                _ => "Minecraft project",
            };
            (
                "java",
                desc,
                &[("java", "java --version"), ("gradle", "gradle --version")],
                &[
                    ("build", "./gradlew build", "Build the mod/plugin JAR"),
                    ("test", "./gradlew test", "Run tests"),
                    ("clean", "./gradlew clean", "Clean build artifacts"),
                ],
                &[],
            )
        }

        // ── Challenge / Competitive Programming profiles ──────────────
        p if p.starts_with("challenge/competitive-") => {
            let sub = p.strip_prefix("challenge/competitive-").unwrap_or("cpp");
            let (lang, desc, checks, targets): (
                &str,
                &str,
                &[(&str, &str)],
                &[(&str, &str, &str)],
            ) = match sub {
                "cpp" => (
                    "cpp",
                    "Competitive programming setup for C++17/20",
                    &[("g++", "g++ --version")],
                    &[
                        (
                            "compile",
                            "g++ -std=c++20 -O2 -Wall -o solve solution.cpp",
                            "Compile a solution",
                        ),
                        ("run", "./solve < input.txt", "Run solution with input"),
                        ("stress", "make stress", "Run stress test"),
                    ],
                ),
                "rust" => (
                    "rust",
                    "Competitive programming setup for Rust",
                    &[("rustc", "rustc --version")],
                    &[
                        (
                            "build",
                            "rustc -O -o solve solution.rs",
                            "Compile a solution",
                        ),
                        ("run", "./solve < input.txt", "Run solution with input"),
                        ("stress", "make stress", "Run stress test"),
                    ],
                ),
                "python" => (
                    "python",
                    "Competitive programming setup for Python 3",
                    &[("python3", "python3 --version")],
                    &[
                        (
                            "run",
                            "python3 solution.py < input.txt",
                            "Run solution with input",
                        ),
                        ("stress", "make stress", "Run stress test"),
                    ],
                ),
                "java" => (
                    "java",
                    "Competitive programming setup for Java",
                    &[("java", "java --version"), ("javac", "javac --version")],
                    &[
                        ("compile", "javac Solution.java", "Compile solution"),
                        (
                            "run",
                            "java Solution < input.txt",
                            "Run solution with input",
                        ),
                    ],
                ),
                _ => (
                    "generic",
                    "Multi-language competitive programming workspace",
                    &[("make", "make --version")],
                    &[
                        ("run", "make run", "Run the active problem solution"),
                        ("stress", "make stress", "Stress test a solution"),
                    ],
                ),
            };
            (lang, desc, checks, targets, &[])
        }

        _ => (
            "generic",
            "Project scaffolded with Lode",
            &[("make", "make --version")],
            &[
                ("build", "make build", "Build the project"),
                ("test", "make test", "Run tests"),
                ("clean", "make clean", "Clean build artifacts"),
            ],
            &["APP_ENV=development"],
        ),
    };

    let mut out = format!(
        "schema_version = 3\nname = \"{profile}\"\nlanguage = \"{lang}\"\ndescription = \"{desc}\"\n\n[scaffold]\ninclude_core = true\ninclude_agent = true\n\n[build]\ngenerate_makefile = true\ntask_runner = \"make\"\n"
    );
    for (name, command, d) in targets {
        out.push_str(&format!(
            "\n[[build.targets]]\nname = \"{name}\"\ncommand = \"{command}\"\ndescription = \"{d}\"\n"
        ));
    }
    if !vars.is_empty() {
        out.push_str("\n[env]\nauto_create = true\n");
        let quoted: Vec<String> = vars.iter().map(|v| format!("\"{v}\"")).collect();
        out.push_str(&format!("vars = [{}]\n", quoted.join(", ")));
    }
    if !checks.is_empty() {
        out.push_str("\n[prereq]\nauto_install = false\n");
        for (n, c) in checks {
            out.push_str(&format!(
                "\n[[prereq.checks]]\nname = \"{n}\"\ncommand = \"{c}\"\n"
            ));
        }
    }
    out
}

fn snippet_contents(lang: &str, name: &str) -> String {
    let body = match (lang, name) {
        // ── any/* ───────────────────────────────────────────────────
        ("any", "todo") => "// TODO($1): $2",
        ("any", "fixme") => "// FIXME($1): $2",
        ("any", "note") => "// NOTE($1): $2",
        ("any", "invariant") => "// INVARIANT: $1",
        ("any", "example") => "// Example:\n// ```\n// $1\n// ```",

        // ── md/* ────────────────────────────────────────────────────
        ("md", "adr") => {
            "# ADR-$(date +%Y%m%d): $1\n\n## Status\n\nProposed\n\n## Context\n\n$2\n\n## Decision\n\n$3\n\n## Consequences\n\n$4\n"
        }
        ("md", "risk") => "# Risk: $1\n\n- **Severity**: $2\n- **Likelihood**: $3\n- **Mitigation**: $4\n- **Owner**: $5\n",
        ("md", "task-list") => {
            "# Tasks\n\n- [ ] $1\n- [ ] $2\n- [ ] $3\n"
        }
        ("md", "release-notes") => {
            "# Release $1\n\n## Highlights\n\n- $2\n\n## Breaking Changes\n\n- $3\n\n## Fixes\n\n- $4\n"
        }

        // ── rs/* ────────────────────────────────────────────────────
        ("rs", "main") => {
            "fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let args: Vec<String> = std::env::args().collect();\n    eprintln!(\"{:?}\", args);\n    Ok(())\n}\n"
        }
        ("rs", "error-enum") => {
            "#[derive(Debug)]\npub enum Error {\n    $1,\n}\n\nimpl std::fmt::Display for Error {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        match self {\n            Self::$1 => write!(f, \"$1\"),\n        }\n    }\n}\n\nimpl std::error::Error for Error {}\n"
        }
        ("rs", "result-alias") => "pub type Result<T> = std::result::Result<T, Error>;\n",
        ("rs", "serde-struct") => {
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\npub struct $1 {\n    pub $2: $3,\n}\n"
        }
        ("rs", "test") => {
            "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn $1() {\n        let result = $2;\n        assert!(result);\n    }\n}\n"
        }

        // ── go/* ────────────────────────────────────────────────────
        ("go", "main") => "package main\n\nimport (\n\t\"fmt\"\n\t\"os\"\n)\n\nfunc main() {\n\targs := os.Args[1:]\n\tfmt.Println(args)\n}\n",
        ("go", "handler") => {
            "func Handler(w http.ResponseWriter, r *http.Request) {\n\tw.Header().Set(\"Content-Type\", \"application/json\")\n\tw.WriteHeader(http.StatusOK)\n\tenc := json.NewEncoder(w)\n\tenc.Encode(map[string]string{\"status\": \"ok\"})\n}\n"
        }
        ("go", "table-test") => {
            "func Test$1(t *testing.T) {\n\ttests := []struct {\n\t\tname string\n\t\tinput $2\n\t\twant $3\n\t}{\n\t\t{\"$4\", $5, $6},\n\t}\n\tfor _, tt := range tests {\n\t\tt.Run(tt.name, func(t *testing.T) {\n\t\t\tgot := $7(tt.input)\n\t\t\tif got != tt.want {\n\t\t\t\tt.Errorf(\"got %v, want %v\", got, tt.want)\n\t\t\t}\n\t\t})\n\t}\n}\n"
        }

        // ── ts/* ────────────────────────────────────────────────────
        ("ts", "fn") => "export function $1($2: $3): $4 {\n    return $5;\n}\n",
        ("ts", "async-fn") => "export async function $1($2: $3): Promise<$4> {\n    return $5;\n}\n",
        ("ts", "interface") => "export interface $1 {\n    $2: $3;\n}\n",
        ("ts", "test") => {
            "import { describe, it, expect } from 'vitest';\n\ndescribe('$1', () => {\n    it('$2', () => {\n        const result = $3;\n        expect(result).toBe($4);\n    });\n});\n"
        }

        // ── py/* ────────────────────────────────────────────────────
        ("py", "main-guard") => "def main() -> None:\n    $1\n\nif __name__ == \"__main__\":\n    main()\n",
        ("py", "dataclass") => {
            "from dataclasses import dataclass\n\n\n@dataclass\nclass $1:\n    $2: $3\n\n    def $4(self) -> $5:\n        return $6\n"
        }
        ("py", "pytest-test") => "import pytest\n\n\ndef test_$1():\n    result = $2\n    assert result == $3\n",

        // ── sh/* ────────────────────────────────────────────────────
        ("sh", "strict-header") => "#!/usr/bin/env bash\nset -euo pipefail\nIFS=$'\\n\\t'\n\n$1\n",
        ("sh", "die") => {
            "die() {\n\techo >&2 \"FATAL: $*\"\n\texit 1\n}\n\n$1 || die \"$1 failed\"\n"
        }
        ("sh", "parse-args") => {
            "while [[ $# -gt 0 ]]; do\n\tcase \"$1\" in\n\t\t-h|--help)\n\t\t\techo \"Usage: $0 [options]\"\n\t\t\texit 0\n\t\t\t;;\n\t\t*)\n\t\t\techo \"Unknown option: $1\"\n\t\t\texit 1\n\t\t\t;;\n\tesac\n\tshift\ndone\n"
        }

        // ── yaml/* ──────────────────────────────────────────────────
        ("yaml", "github-job") => "  $1:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: $2\n",
        ("yaml", "docker-compose-service") => "  $1:\n    image: $2\n    ports:\n      - \"$3:$3\"\n    environment:\n      - $4\n",

        // ── toml/* ──────────────────────────────────────────────────
        ("toml", "table") => "[$1]\n$2 = $3\n",
        ("toml", "lode-command") => "slug = \"$1\"\ndescription = \"$2\"\n\n[[steps]]\nkind = \"shell\"\nrun = \"$3\"\n",
        ("toml", "lode-profile") => "schema_version = 3\nname = \"$1\"\nlanguage = \"$2\"\ndescription = \"$3\"\n\n[scaffold]\ninclude_core = true\ninclude_agent = true\n",

        // ── cp/* ────────────────────────────────────────────────────
        ("cp", "fast-io-cpp") => {
            "#include <bits/stdc++.h>\nusing namespace std;\n\nint main() {\n    ios::sync_with_stdio(false);\n    cin.tie(nullptr);\n\n    int t;\n    cin >> t;\n    while (t--) {\n        $1\n    }\n    return 0;\n}\n"
        }
        ("cp", "dijkstra") => {
            "vector<long long> dijkstra(int start, const vector<vector<pair<int, long long>>>& g) {\n    int n = (int)g.size();\n    vector<long long> dist(n, LLONG_MAX);\n    priority_queue<pair<long long, int>, vector<pair<long long, int>>, greater<>> pq;\n    dist[start] = 0;\n    pq.emplace(0, start);\n    while (!pq.empty()) {\n        auto [d, u] = pq.top(); pq.pop();\n        if (d != dist[u]) continue;\n        for (auto [v, w] : g[u]) {\n            if (dist[v] > d + w) {\n                dist[v] = d + w;\n                pq.emplace(dist[v], v);\n            }\n        }\n    }\n    return dist;\n}\n"
        }
        ("cp", "union-find") => {
            "struct DSU {\n    vector<int> p, sz;\n    DSU(int n) : p(n), sz(n, 1) { iota(p.begin(), p.end(), 0); }\n    int find(int x) { return p[x] == x ? x : p[x] = find(p[x]); }\n    bool unite(int a, int b) {\n        a = find(a); b = find(b);\n        if (a == b) return false;\n        if (sz[a] < sz[b]) swap(a, b);\n        p[b] = a;\n        sz[a] += sz[b];\n        return true;\n    }\n    bool same(int a, int b) { return find(a) == find(b); }\n};\n"
        }
        ("cp", "segment-tree") => {
            "struct SegTree {\n    int n;\n    vector<long long> t;\n    SegTree(int n_) : n(n_), t(2 * n) {}\n    void set(int pos, long long val) {\n        for (t[pos += n] = val; pos > 1; pos >>= 1)\n            t[pos >> 1] = t[pos] + t[pos ^ 1];\n    }\n    long long sum(int l, int r) {\n        long long res = 0;\n        for (l += n, r += n; l < r; l >>= 1, r >>= 1) {\n            if (l & 1) res += t[l++];\n            if (r & 1) res += t[--r];\n        }\n        return res;\n    }\n};\n"
        }

        _ => "{name} $1\n",
    };
    format!("name: {name}\nlang: {lang}\n---\n{body}")
}

fn command_contents(command: &str) -> String {
    match command {
        "commit" => {
            return r#"slug = "commit"
description = "Stage all files and commit with a message"
help = "Usage: lode commit -m \"message\""

[output]
style = "detailed"

[args.message]
short = "m"
long = "message"
help = "Commit message"
required = true

[[steps]]
kind = "shell"
run = "git add ."

[[steps]]
kind = "shell"
run = "git commit -m \"{{ args.message }}\""

[[steps]]
kind = "shell"
run = "git log -1 --oneline --format='%h %s (%cr)'"
show_output = true
"#.to_string()
        }
        "fresh" => {
            return r#"slug = "fresh"
description = "Clean build artifacts and perform a full rebuild"

[[steps]]
kind = "make"
run = "clean"

[[steps]]
kind = "make"
run = "install"
"#.to_string()
        }
        "ship" => {
            return r#"slug = "ship"
description = "Verify project and create a release"

[[steps]]
kind = "make"
run = "verify"

[[steps]]
kind = "make"
run = "release"
"#.to_string()
        }
        "explain" => {
            return r#"slug = "explain"
description = "Explain what Lode does"

[[steps]]
kind = "shell"
run = "echo \"Lode keeps project structure, defaults, commands, snippets, and context consistent.\" \"Start with `lode init <name> --profile systems/rust-cli --with ci,vscode`.\""
"#.to_string()
        }
        "setup-dev" => {
            return r#"slug = "setup-dev"
description = "Check prerequisites, install dependencies, and verify the setup"

[[steps]]
kind = "make"
run = "check-prereqs"

[[steps]]
kind = "make"
run = "install"

[[steps]]
kind = "make"
run = "build"

[[steps]]
kind = "shell"
run = "echo 'Setup complete — ready to develop'"
show_output = true
"#.to_string()
        }
        "dev" => {
            return r#"slug = "dev"
description = "Build and run the development environment"

[[steps]]
kind = "make"
run = "build"

[[steps]]
kind = "make"
run = "dev"
"#.to_string()
        }
        "build" => {
            return r#"slug = "build"
description = "Clean previous artifacts and perform a full build"

[[steps]]
kind = "make"
run = "clean"

[[steps]]
kind = "make"
run = "build"
"#.to_string()
        }
        "test-all" => {
            return r#"slug = "test-all"
description = "Build, run all tests, and show a summary"

[[steps]]
kind = "make"
run = "build"

[[steps]]
kind = "make"
run = "test"

[[steps]]
kind = "shell"
run = "echo 'All tests completed'"
show_output = true
"#.to_string()
        }
        "lint-all" => {
            return r#"slug = "lint-all"
description = "Check formatting and run the linter, then show results"

[[steps]]
kind = "make"
run = "fmt"

[[steps]]
kind = "make"
run = "lint"

[[steps]]
kind = "shell"
run = "echo 'Lint checks completed'"
show_output = true
"#.to_string()
        }
        "fmt-all" => {
            return r#"slug = "fmt-all"
description = "Format the codebase and verify formatting"

[[steps]]
kind = "make"
run = "fmt"

[[steps]]
kind = "shell"
run = "echo 'Formatting completed'"
show_output = true
"#.to_string()
        }
        "clean-all" => {
            return r#"slug = "clean-all"
description = "Remove all common build artifacts and caches"

[[steps]]
kind = "make"
run = "clean"

[[steps]]
kind = "shell"
run = "rm -rf target/ node_modules/ __pycache__/ .pytest_cache/ build/ dist/ *.pyc .ruff_cache/"
show_output = true

[[steps]]
kind = "shell"
run = "echo 'Cleanup completed'"
show_output = true
"#.to_string()
        }
        "verify" => {
            return r#"slug = "verify"
description = "Full verification pipeline: format, lint, and test"

[[steps]]
kind = "make"
run = "fmt"

[[steps]]
kind = "make"
run = "lint"

[[steps]]
kind = "make"
run = "test"
"#.to_string()
        }
        "health" => {
            return r#"slug = "health"
description = "Check environment health and report status"

[[steps]]
kind = "shell"
run = "echo '=== Environment Health ==='"
show_output = true

[[steps]]
kind = "shell"
run = "echo '--- Toolchain ---'"
show_output = true

[[steps]]
kind = "shell"
run = "which make && make --version | head -1 || echo 'make not found'"

[[steps]]
kind = "shell"
run = "echo '--- Project ---'"
show_output = true

[[steps]]
kind = "shell"
run = "test -f Makefile && echo 'Makefile found' || echo 'No Makefile'"

[[steps]]
kind = "shell"
run = "echo 'Health check complete'"
show_output = true
"#.to_string()
        }
        "release-local" => {
            return r#"slug = "release-local"
description = "Verify, build in release mode, and package the artifact"

[[steps]]
kind = "make"
run = "verify"

[[steps]]
kind = "make"
run = "release"
"#.to_string()
        }
        "cp-new" => {
            return r#"slug = "cp-new"
description = "Scaffold a new competitive programming problem directory"
help = "Usage: lode cp-new <problem-name>"

[args.name]
help = "Problem identifier (e.g. cf-1234-a)"
required = true

[[steps]]
kind = "shell"
run = "mkdir -p \"{{ args.name }}\""

[[steps]]
kind = "shell"
run = "cp templates/cpp.cpp \"{{ args.name }}/solution.cpp\""
show_output = true

[[steps]]
kind = "shell"
run = "touch \"{{ args.name }}/input.txt\" \"{{ args.name }}/brute.cpp\" \"{{ args.name }}/gen.cpp\""

[[steps]]
kind = "shell"
run = "echo 'Competitive programming problem {{ args.name }} scaffolded in {{ args.name }}/'"
show_output = true
"#.to_string()
        }
        "cp-run" => {
            return r#"slug = "cp-run"
description = "Compile and run a competitive programming solution"
help = "Usage: lode cp-run <file.cpp>"

[args.file]
help = "Path to the solution file"
required = true

[[steps]]
kind = "shell"
run = "g++ -std=c++20 -O2 -Wall -o solve \"{{ args.file }}\""
show_output = true

[[steps]]
kind = "shell"
run = "cat input.txt | ./solve"
show_output = true
"#.to_string()
        }
        "cp-stress" => {
            return r#"slug = "cp-stress"
description = "Stress test a solution against a brute force implementation"

[[steps]]
kind = "shell"
run = "g++ -std=c++20 -O2 -o solve solution.cpp && g++ -std=c++20 -O2 -o brute brute.cpp && g++ -std=c++20 -O2 -o gen gen.cpp"
show_output = true

[[steps]]
kind = "shell"
run = "for i in $(seq 1 100); do echo \"Test $i\"; ./gen > input.txt; ./solve < input.txt > out_solve.txt; ./brute < input.txt > out_brute.txt; if ! diff -q out_solve.txt out_brute.txt; then echo \"Mismatch on test $i\"; cat input.txt; exit 1; fi; done; echo 'All 100 stress tests passed'"
show_output = true
"#.to_string()
        }
        "hackathon-demo" => {
            return r#"slug = "hackathon-demo"
description = "Build and start the hackathon demo environment"

[[steps]]
kind = "make"
run = "build"

[[steps]]
kind = "make"
run = "demo"
"#.to_string()
        }
        "mc-run-client" => {
            return r#"slug = "mc-run-client"
description = "Run the Minecraft client for testing"

[[steps]]
kind = "shell"
run = "echo 'Starting Minecraft client…'"
show_output = true

[[steps]]
kind = "shell"
run = "./gradlew runClient"
show_output = true
"#.to_string()
        }
        "mc-run-server" => {
            return r#"slug = "mc-run-server"
description = "Run the Minecraft server for testing"

[[steps]]
kind = "shell"
run = "echo 'Starting Minecraft server…'"
show_output = true

[[steps]]
kind = "shell"
run = "./gradlew runServer"
show_output = true
"#.to_string()
        }
        "gha-validate" => {
            return r#"slug = "gha-validate"
description = "Validate GitHub Actions workflow files"

[[steps]]
kind = "shell"
run = "echo 'Validating GitHub Actions workflows…'"
show_output = true

[[steps]]
kind = "shell"
run = "find .github/workflows -name '*.yml' -o -name '*.yaml' | while read f; do echo \"Checking $f\"; action-validator \"$f\" 2>/dev/null || echo \"  (action-validator not installed — checking syntax only)\"; done"
show_output = true

[[steps]]
kind = "shell"
run = "echo 'GitHub Actions validation completed'"
show_output = true
"#.to_string()
        }
        "tauri-dev" => {
            return r#"slug = "tauri-dev"
description = "Start the Tauri development environment with hot reload"

[[steps]]
kind = "npm"
run = "install"

[[steps]]
kind = "npm"
run = "run"
args = ["tauri", "dev"]
"#.to_string()
        }
        "tauri-build" => {
            return r#"slug = "tauri-build"
description = "Build Tauri app for production"

[[steps]]
kind = "npm"
run = "install"

[[steps]]
kind = "npm"
run = "run"
args = ["tauri", "build"]
"#.to_string()
        }
        "git-commit" => {
            return r#"slug = "git-commit"
description = "Stage all changes and create a git commit"
help = "Usage: lode git-commit -m \"message\""

[output]
style = "detailed"

[args.message]
short = "m"
long = "message"
help = "Commit message"
required = true

[[steps]]
kind = "shell"
run = "git add ."

[[steps]]
kind = "shell"
run = "git commit -m \"{{ args.message }}\""

[[steps]]
kind = "shell"
run = "git log -1 --oneline --format='%h %s (%cr)'"
show_output = true
"#
            .to_string()
        }
        "git-tag" => {
            return r#"slug = "git-tag"
description = "Create an annotated version tag and optionally push"
help = "Usage: lode git-tag -v 1.0.0 -m 'v1.0.0' --push"

[output]
style = "detailed"

[args.version]
short = "v"
long = "version"
help = "Version to tag (e.g. 1.0.0)"
required = true

[[steps]]
kind = "shell"
run = "git tag -a \"v{{ args.version }}\" -m \"v{{ args.version }}\""

[[steps]]
kind = "shell"
run = "git push origin \"v{{ args.version }}\""
"#
            .to_string()
        }
        "git-changelog" => {
            return r#"slug = "git-changelog"
description = "Generate changelog from git log"
help = "Usage: lode git-changelog"

[output]
style = "detailed"

[[steps]]
kind = "shell"
run = "LAST=$(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD 2>/dev/null); git log --oneline --no-merges ${LAST}..HEAD > CHANGELOG.md"
show_output = true

[[steps]]
kind = "shell"
run = "echo 'Changelog written to CHANGELOG.md'"
show_output = true
"#
            .to_string()
        }
        "git-install-hooks" => {
            return r#"slug = "git-install-hooks"
description = "Install lode-managed git hook scripts"

[[steps]]
kind = "shell"
run = "mkdir -p .git/hooks"

[[steps]]
kind = "shell"
run = "printf '#!/usr/bin/env sh\n# lode-managed\nlode check .\nlode scan secrets .\n' > .git/hooks/pre-commit"

[[steps]]
kind = "shell"
run = "printf '#!/usr/bin/env sh\n# lode-managed\nlode task test\n' > .git/hooks/pre-push"
show_output = true
"#
            .to_string()
        }
        "git-sign-setup" => {
            return r#"slug = "git-sign-setup"
description = "Configure git commit signing metadata"

[[steps]]
kind = "shell"
run = "mkdir -p .lode"

[[steps]]
kind = "shell"
run = "printf 'enabled = true\nmode = \"manual\"\n' > .lode/git-signing.toml"
show_output = true
"#
            .to_string()
        }
        "git-remote-setup" => {
            return r#"slug = "git-remote-setup"
description = "Record git remote provider metadata"
help = "Usage: lode git-remote-setup --provider github --visibility private"

[args.provider]
short = "p"
long = "provider"
help = "Git provider (github, gitlab, etc.)"

[args.visibility]
short = "v"
long = "visibility"
help = "Repository visibility (public, private)"

[args.token_env]
short = "t"
long = "token-env"
help = "Environment variable with auth token"

[[steps]]
kind = "shell"
run = "mkdir -p .lode"

[[steps]]
kind = "shell"
run = "printf 'provider = \"%s\"\nvisibility = \"%s\"\ntoken_env = \"%s\"\n' \"{{ args.provider }}\" \"{{ args.visibility }}\" \"{{ args.token_env }}\" > .lode/remote.toml"
show_output = true
"#
            .to_string()
        }
        _ => {
            format!(
                "slug = \"{command}\"\ndescription = \"Default {command} command macro\"\n\n[[steps]]\nkind = \"make\"\nrun = \"{}\"\n",
                command.replace("-all", "")
            )
        }
    }
}

fn recipe_contents(recipe: &str) -> String {
    match recipe {
        "docker-basic" => r#"name = "docker-basic"
description = "Docker containerization with Dockerfile, Compose, and .dockerignore"

[[files]]
template = "docker/Dockerfile"
dest = "Dockerfile"

[[files]]
template = "docker/compose.yml"
dest = "compose.yml"

[[files]]
template = "dotfiles/dockerignore"
dest = ".dockerignore"
"#
        .to_string(),
        "devcontainer-basic" => r#"name = "devcontainer-basic"
description = "Development container configuration with VS Code integration"

[[files]]
template = "devcontainer/devcontainer.json"
dest = ".devcontainer/devcontainer.json"

[[files]]
template = "vscode/extensions.json"
dest = ".devcontainer/extensions.json"
"#
        .to_string(),
        "github-actions-basic" => r#"name = "github-actions-basic"
description = "GitHub Actions CI workflow with issue templates and Dependabot"

[[files]]
template = "github/workflows/ci.yml"
dest = ".github/workflows/ci.yml"

[[files]]
template = "github/dependabot.yml"
dest = ".github/dependabot.yml"

[[files]]
template = "github/ISSUE_TEMPLATE/bug_report.md"
dest = ".github/ISSUE_TEMPLATE/bug_report.md"

[[files]]
template = "github/ISSUE_TEMPLATE/feature_request.md"
dest = ".github/ISSUE_TEMPLATE/feature_request.md"

[[files]]
template = "github/PULL_REQUEST_TEMPLATE.md"
dest = ".github/PULL_REQUEST_TEMPLATE.md"

[[files]]
template = "github/CODEOWNERS"
dest = ".github/CODEOWNERS"
"#
        .to_string(),
        "github-actions-release" => r#"name = "github-actions-release"
description = "GitHub Actions release workflow triggered by version tags"

[[files]]
template = "github/workflows/release.yml"
dest = ".github/workflows/release.yml"
"#
        .to_string(),
        "release-basic" => r#"name = "release-basic"
description = "Basic release pipeline with changelog, version bump, and tagging"

[[files]]
template = "root/CHANGELOG.md"
dest = "CHANGELOG.md"

[[files]]
template = "github/workflows/release.yml"
dest = ".github/workflows/release.yml"

[[files]]
template = "ci/release.yml"
dest = "ci/release.yml"
"#
        .to_string(),
        "security-basic" => r#"name = "security-basic"
description = "Security policy, audit CI, and secrets scanning configuration"

[[files]]
template = "root/SECURITY.md"
dest = "SECURITY.md"

[[files]]
template = "ci/security.yml"
dest = "ci/security.yml"

[[files]]
template = "github/workflows/security.yml"
dest = ".github/workflows/security.yml"
"#
        .to_string(),
        "workspace-basic" => r#"name = "workspace-basic"
description = "Multi-package workspace configuration for monorepos"

[[files]]
template = "root/Makefile"
dest = "Makefile"

[[files]]
template = "root/justfile"
dest = "justfile"
"#
        .to_string(),
        "agent-basic" => r#"name = "agent-basic"
description = "AI agent context files for Lode, Claude, Cursor, Windsurf, and Codex"

[[files]]
template = "agent/AGENTS.md"
dest = "AGENTS.md"

[[files]]
template = "agent/CLAUDE.md"
dest = "CLAUDE.md"

[[files]]
template = "agent/CODEX.md"
dest = "CODEX.md"

[[files]]
template = "agent/PLAN.md"
dest = ".agent/PLAN.md"

[[files]]
template = "agent/CONSTRAINTS.md"
dest = ".agent/CONSTRAINTS.md"

[[files]]
template = "agent/REVIEW.md"
dest = ".agent/REVIEW.md"

[[files]]
template = "agent/TASKS.md"
dest = ".agent/TASKS.md"

[[files]]
template = "agent/MEMORY.md"
dest = ".agent/MEMORY.md"

[[files]]
template = "agent/.cursorrules"
dest = ".cursorrules"

[[files]]
template = "agent/.windsurfrules"
dest = ".windsurfrules"

[[files]]
template = "agent/.mcp.json"
dest = ".mcp.json"
"#
        .to_string(),
        "docs-basic" => r#"name = "docs-basic"
description = "Documentation site with index, getting started, usage, and reference guides"

[[files]]
template = "docs/index.md"
dest = "docs/index.md"

[[files]]
template = "docs/getting-started.md"
dest = "docs/getting-started.md"

[[files]]
template = "docs/usage.md"
dest = "docs/usage.md"

[[files]]
template = "docs/configuration.md"
dest = "docs/configuration.md"

[[files]]
template = "docs/commands.md"
dest = "docs/commands.md"

[[files]]
template = "docs/api.md"
dest = "docs/api.md"

[[files]]
template = "docs/deployment.md"
dest = "docs/deployment.md"

[[files]]
template = "docs/troubleshooting.md"
dest = "docs/troubleshooting.md"

[[files]]
template = "docs/faq.md"
dest = "docs/faq.md"
"#
        .to_string(),
        "django-postgres" => r#"name = "django-postgres"
description = "Django application with PostgreSQL database setup"

[[files]]
template = "django/manage.py"
dest = "manage.py"

[[files]]
template = "ci/django.yml"
dest = "ci/django.yml"

[[files]]
template = "docker/Dockerfile"
dest = "Dockerfile"

[[files]]
template = "docker/compose.yml"
dest = "compose.yml"
"#
        .to_string(),
        "django-rest-framework" => r#"name = "django-rest-framework"
description = "Django REST Framework API with serializers, viewsets, and routing"

[[files]]
template = "django/manage.py"
dest = "manage.py"

[[files]]
template = "ci/django.yml"
dest = "ci/django.yml"

[[files]]
template = "docker/compose.yml"
dest = "compose.yml"
"#
        .to_string(),
        "tauri-updater" => r#"name = "tauri-updater"
description = "Tauri app with built-in updater and release infrastructure"

[[files]]
template = "tauri/package.json"
dest = "package.json"

[[files]]
template = "tauri/src-tauri/Cargo.toml"
dest = "src-tauri/Cargo.toml"

[[files]]
template = "ci/tauri.yml"
dest = "ci/tauri.yml"
"#
        .to_string(),
        "minecraft-fabric" => r#"name = "minecraft-fabric"
description = "Minecraft Fabric mod with Gradle build and CI"

[[files]]
template = "minecraft/fabric/build.gradle"
dest = "build.gradle"

[[files]]
template = "ci/minecraft-gradle.yml"
dest = "ci/minecraft-gradle.yml"

[[files]]
template = "github/workflows/ci.yml"
dest = ".github/workflows/ci.yml"
"#
        .to_string(),
        "minecraft-forge" => r#"name = "minecraft-forge"
description = "Minecraft Forge mod with Gradle build and CI"

[[files]]
template = "ci/minecraft-gradle.yml"
dest = "ci/minecraft-gradle.yml"

[[files]]
template = "github/workflows/ci.yml"
dest = ".github/workflows/ci.yml"
"#
        .to_string(),
        "minecraft-paper" => r#"name = "minecraft-paper"
description = "Minecraft Paper plugin with Gradle build and CI"

[[files]]
template = "ci/minecraft-gradle.yml"
dest = "ci/minecraft-gradle.yml"

[[files]]
template = "github/ISSUE_TEMPLATE/bug_report.md"
dest = ".github/ISSUE_TEMPLATE/bug_report.md"
"#
        .to_string(),
        "competitive-coding" => r#"name = "competitive-coding"
description = "Competitive programming directory with templates, scripts, and CI"

[[files]]
template = "competitive/templates/cpp.cpp"
dest = "competitive/templates/cpp.cpp"

[[files]]
template = "competitive/problems/a/main.cpp"
dest = "competitive/problems/a/main.cpp"

[[files]]
template = "competitive/scripts/run.sh"
dest = "competitive/scripts/run.sh"
"#
        .to_string(),
        "hackathon-demo" => r#"name = "hackathon-demo"
description = "Hackathon demo environment with Docker Compose and dev server"

[[files]]
template = "docker/compose.yml"
dest = "compose.yml"

[[files]]
template = "docker/Dockerfile"
dest = "Dockerfile"

[[files]]
template = "root/Makefile"
dest = "Makefile"

[[files]]
template = "root/README.md"
dest = "README.md"
"#
        .to_string(),
        _ => {
            format!(
                "name = \"{recipe}\"\ndescription = \"Capability bundle for {recipe}\"\n\n[[files]]\ntemplate = \"docs/index.md\"\ndest = \"docs/{recipe}.md\"\n"
            )
        }
    }
}

fn license_contents(license: &str) -> String {
    match license {
        "MIT" => "MIT License\n\nCopyright (c) {{ year }} {{ author }}\n\nPermission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software.\n\nThe above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.\n".to_string(),
        "Apache-2.0" => "Apache License\nVersion 2.0, January 2004\nhttps://www.apache.org/licenses/LICENSE-2.0\n\nCopyright {{ year }} {{ author }}\n\nLicensed under the Apache License, Version 2.0. You may not use this file except in compliance with the License. You may obtain a copy of the License at the URL above.\n\nUnless required by applicable law or agreed to in writing, software distributed under the License is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n".to_string(),
        "MIT OR Apache-2.0" => "Licensed under either of\n\n- Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)\n- MIT license (https://opensource.org/license/mit)\n\nat your option.\n".to_string(),
        "BSD-3-Clause" => "BSD 3-Clause License\n\nCopyright (c) {{ year }}, {{ author }}\n\nRedistribution and use in source and binary forms, with or without modification, are permitted provided that the BSD 3-Clause License conditions are met.\n\nTHIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\" AND ANY EXPRESS OR IMPLIED WARRANTIES ARE DISCLAIMED.\n".to_string(),
        "ISC" => "ISC License\n\nCopyright (c) {{ year }} {{ author }}\n\nPermission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the copyright notice and this permission notice appear in all copies.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE.\n".to_string(),
        "GPL-3.0-only" => "GNU General Public License v3.0 only\n\nCopyright (c) {{ year }} {{ author }}\n\nThis project is licensed under the GNU General Public License version 3 only. See https://www.gnu.org/licenses/gpl-3.0.en.html for the complete license terms.\n\nSPDX-License-Identifier: GPL-3.0-only\n".to_string(),
        "MPL-2.0" => "Mozilla Public License Version 2.0\n\nCopyright (c) {{ year }} {{ author }}\n\nThis Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, you can obtain one at https://mozilla.org/MPL/2.0/.\n\nSPDX-License-Identifier: MPL-2.0\n".to_string(),
        "Unlicense" => "The Unlicense\n\nThis is free and unencumbered software released into the public domain.\n\nAnyone is free to copy, modify, publish, use, compile, sell, or distribute this software, either in source code form or as a compiled binary, for any purpose.\n\nSee https://unlicense.org/ for the complete public-domain dedication.\n".to_string(),
        other => format!("{other}\n\nDefault license placeholder. Replace with full license text if needed.\n"),
    }
}

fn makefile_contents() -> String {
    "help:\n\t@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | sort\n\ndev: ## Run development server\n\t@echo \"No dev command configured\"\n\nbuild: ## Build project\n\t@echo \"No build command configured\"\n\ntest: ## Run tests\n\t@echo \"No test command configured\"\n\ntest-watch: ## Run tests in watch mode\n\t@echo \"No watch command configured\"\n\nfmt: ## Format project\n\t@echo \"No format command configured\"\n\nlint: ## Lint project\n\t@echo \"No lint command configured\"\n\ncheck: ## Run checks\n\t$(MAKE) fmt lint test\n\nverify: ## Format, lint, test, audit\n\t$(MAKE) check\n\naudit: ## Run audit\n\t@echo \"No audit command configured\"\n\nclean: ## Clean generated files\n\t@echo \"No clean command configured\"\n\ndocs: ## Serve docs\n\t@echo \"No docs command configured\"\n\ninstall: ## Install dependencies\n\t@echo \"No install command configured\"\n\nupdate: ## Update dependencies\n\t@echo \"No update command configured\"\n\nrelease: ## Prepare release\n\t@echo \"No release command configured\"\n".to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{template_contents, template_paths};
    use crate::template::RenderContext;

    #[test]
    fn canonical_ci_assets_are_unique_and_useful() {
        let required = [
            "ci/generic.yml",
            "ci/c.yml",
            "ci/cpp.yml",
            "ci/rust.yml",
            "ci/go.yml",
            "ci/zig.yml",
            "ci/java-gradle.yml",
            "ci/java-maven.yml",
            "ci/node.yml",
            "ci/python.yml",
            "ci/django.yml",
            "ci/tauri.yml",
            "ci/minecraft-gradle.yml",
            "ci/docs.yml",
            "ci/release.yml",
            "ci/security.yml",
        ];
        let unique = required.into_iter().collect::<HashSet<_>>();
        let context = RenderContext::new().with("project", "demo");

        assert_eq!(unique.len(), required.len());
        for path in required {
            assert_eq!(
                template_paths()
                    .iter()
                    .filter(|candidate| **candidate == path)
                    .count(),
                1,
                "missing or duplicate {path}"
            );
            let contents = template_contents(path, &context);
            assert!(
                !contents.contains("Generated default asset"),
                "placeholder {path}"
            );
            assert!(contents.contains("jobs:"), "not a useful workflow: {path}");
        }
    }
}
