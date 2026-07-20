#![deny(unsafe_code)]

pub mod cmd;
pub mod mcpserver;

pub(crate) use cmd::plugin::{read_plugin_install_receipt, read_plugin_security};

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env, fs, io,
    io::{IsTerminal, Read},
    process::ExitCode,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::cmd::output;

use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use lode_core::{
    check_file_integrity, check_path, command_names, default_config,
    default_lodepack_checksum_algorithm, global_asset_dir, global_dir, list_managed_files,
    load_global_config, load_metrics, profile_names, recipe_names, redact, scan_secrets,
    template_paths, LodeError, LodePack, LodePackFile, LodePackManifest, Process, ValidatedRoot,
};
use serde_json::{json, Value};

/// Default port for MCP HTTP mode (unused in this build).
pub(crate) const MCP_HTTP_PORT: u16 = 3333;

pub(crate) use cmd::types::*;
fn main() -> ExitCode {
    run_entrypoint()
}

#[cfg(windows)]
fn run_entrypoint() -> ExitCode {
    match thread::Builder::new()
        .name("lode-cli".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
    {
        Ok(handle) => match handle.join() {
            Ok(result) => result_to_exit_code(result),
            Err(_) => {
                eprintln!("error: lode command panicked");
                ExitCode::from(lode_core::ExitCode::Error as u8)
            }
        },
        Err(error) => {
            eprintln!(
                "error: failed to start lode command: {}",
                redact(&error.to_string())
            );
            ExitCode::from(lode_core::ExitCode::Error as u8)
        }
    }
}

#[cfg(not(windows))]
fn run_entrypoint() -> ExitCode {
    result_to_exit_code(run())
}

fn result_to_exit_code(result: lode_core::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::from(lode_core::ExitCode::Ok as u8),
        Err(error) => {
            let msg = redact(&error.to_string());
            eprintln!("error: {msg}");
            ExitCode::from(error.exit_code() as u8)
        }
    }
}

fn run() -> lode_core::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Setup {
            defaults: _,
            output,
        } => cmd::setup::setup_with_output(output)?,
        Command::Init(args) => cmd::init::init(args)?,
        Command::Adopt {
            path,
            apply,
            dry_run,
            output,
        } => cmd::adopt::adopt_command(path, apply, dry_run, output)?,
        Command::Add {
            component,
            dry_run,
            overwrite,
        } => cmd::add::add_component(&component, dry_run, overwrite)?,
        Command::Sync {
            dry_run,
            force,
            section,
        } => cmd::sync::sync(dry_run, force, section.as_deref())?,
        Command::Info { output } => cmd::info::info_with_output(output)?,
        Command::Config { command } => cmd::config::config_command(command)?,
        Command::Template { command } => {
            cmd::template::library_command("templates", command, template_paths())?
        }
        Command::TemplateBundle { command } => {
            cmd::template_bundle::template_bundle_command(command)?
        }
        Command::Profile { command } => cmd::profile::profile_command(command)?,
        Command::Recipe { command } => cmd::recipe::recipe_command(command)?,
        Command::Commands { command } => cmd::commands::commands(command)?,
        Command::Plugin { command } => cmd::plugin::plugin_command(command)?,
        Command::Mcp {
            http,
            port,
            list_tools,
            list_resources,
            list_prompts,
        } => cmd::mcp::mcp_command(http, port, list_tools, list_resources, list_prompts)?,
        Command::Lsp {
            stdio,
            capabilities,
        } => cmd::lsp::lsp_command(stdio, capabilities)?,
        Command::Snippet { command } => cmd::snippet::snippet_command(command)?,
        Command::Task { target, no_store } => cmd::task::task_command(target, no_store)?,
        Command::Dev => run_make("dev")?,
        Command::Build => run_make("build")?,
        Command::Test => run_make("test")?,
        Command::Fmt => run_make("fmt")?,
        Command::Lint => run_make("lint")?,
        Command::Check(args) => cmd::check::convention_check_with_output(args)?,
        Command::Agent { command } => cmd::agent::agent_command(command)?,
        Command::AgentSim { command } => cmd::agent_sim::agent_sim_command(command)?,
        Command::Archetype { command } => cmd::archetype::archetype_command(command)?,
        Command::Cache { command } => cmd::cache::cache_command(command)?,
        Command::EnvSnapshot { command } => cmd::env_snapshot::env_snapshot_command(command)?,
        Command::Assets { command } => cmd::assets::assets_command(command)?,
        Command::Pack { command } => cmd::pack::pack_command(command)?,
        Command::Plan { command } => cmd::plan::plan_command(command)?,
        Command::Policy { command } => cmd::policy::policy_command(command)?,
        Command::Project { command } => cmd::project::project_command(command)?,
        Command::Lock { command } => cmd::lock::lock_command(command)?,
        Command::Receipts { command } => cmd::receipts::receipt_command(command)?,
        Command::Context { command } => cmd::context::context_command(command)?,
        Command::Handoff { command } => cmd::handoff::handoff_command(command)?,
        Command::Diagnose { command } => cmd::diagnose::diagnose_command(command)?,
        Command::Docs { command } => cmd::docs::docs_command(command)?,
        Command::DepGraph { command } => cmd::depgraph::depgraph_command(command)?,
        Command::Fix { path } => cmd::fix::convention_fix(path)?,
        Command::Rename { path, to } => cmd::rename::rename_path(path, to)?,
        Command::Rules { command } => cmd::rules::rules(command)?,
        Command::Sign {
            path,
            ext,
            force,
            dry_run,
        } => cmd::sign::sign_path(path, ext, force, dry_run)?,
        Command::Stamp {
            path,
            ext,
            license,
            dry_run,
        } => cmd::stamp::stamp_path(path, ext, license, dry_run)?,
        Command::Sandbox { command } => cmd::sandbox::sandbox_command(command)?,
        Command::Verify { changed, output } => verify_command(changed, output)?,
        Command::Clean => run_make("clean")?,
        Command::Fresh => {
            run_make("clean")?;
            run_make("install")?;
        }
        Command::Ship => {
            run_make("verify")?;
            run_make("release")?;
        }
        Command::Release {
            version,
            bump,
            dry_run,
            rollback,
        } => cmd::release::release(version, bump, dry_run, rollback)?,
        Command::Health { output } | Command::Audit { output } => {
            cmd::audit::health_with_output(output)?
        }
        Command::Explain => explain(),
        Command::Doctor { fix, output } => cmd::doctor::doctor_with_output(fix, output)?,
        Command::Scan { command } => cmd::scan::scan(command)?,
        Command::SecretVault { command } => cmd::secret_vault::secret_vault_command(command)?,
        Command::Git { command } => cmd::git::git(command)?,
        Command::Hooks { command } => cmd::hooks::hooks(command)?,
        Command::Env { command } => cmd::env::env_command(command)?,
        Command::License { command } => cmd::license::license(command)?,
        Command::File { command } => cmd::file::file_command(command)?,
        Command::Projects { command } => cmd::projects::projects(command)?,
        Command::Toolchain { command } => cmd::toolchain::toolchain(command)?,
        Command::Pkg { command } => cmd::pkg::pkg(command)?,
        Command::Time { command } => cmd::time::time_command(command)?,
        Command::Metrics { command } => cmd::metrics::metrics(command)?,
        Command::Migration { command } => cmd::migration::migration_command(command)?,
        Command::Workspace { command } => cmd::workspace::workspace(command)?,
        Command::Daemon { command } => cmd::daemon::daemon(command),
        Command::Log { command } => cmd::log::log_command(command)?,
        Command::Export {
            out,
            no_plugins,
            no_templates,
            no_snippets,
            no_licenses,
            no_recipes,
            no_commands,
            include_metrics,
        } => cmd::export::export_lodepack(
            out,
            ExportOptions {
                no_plugins,
                no_templates,
                no_snippets,
                no_licenses,
                no_recipes,
                no_commands,
                include_metrics,
            },
        )?,
        Command::Import {
            path,
            no_merge,
            force,
        } => cmd::export::import_lodepack(path, no_merge, force)?,
        Command::Serve {
            no_color,
            no_live,
            pane,
            refresh: _refresh,
            theme: _theme,
        } => cmd::serve::serve_impl(no_color, no_live, pane.as_deref())?,
        Command::Mc { command } => mc_command(&command)?,
        Command::Tauri { command } => tauri_command(&command)?,
        Command::Gha { command, name } => gha_command(&command, name.as_deref())?,
        Command::Cp {
            command,
            problem,
            lang,
        } => cp_command(&command, problem.as_deref(), lang.as_deref())?,
        Command::SelfCmd { command } => cmd::self_cmd::self_command(command)?,
        Command::Upgrade {
            check,
            manifest,
            dry_run,
            rollback,
        } => cmd::upgrade::upgrade(check, manifest, dry_run, rollback)?,
        Command::Completions {
            shell,
            install,
            dry_run,
            out,
        } => cmd::completions::completions(&shell, install, dry_run, out)?,
        Command::Version => println!("{}", env!("CARGO_PKG_VERSION")),
        Command::External(args) => external_command(args)?,
    }

    Ok(())
}

pub(crate) fn init_git_project(
    project_dir: &Utf8PathBuf,
    git: &lode_core::config::GitConfig,
    identity: &lode_core::config::IdentityConfig,
    project_name: &str,
) -> lode_core::Result<()> {
    if project_dir.join(".git").exists() {
        println!("git repository already exists");
        return Ok(());
    }

    let init_status = run_process_status(
        "git",
        &[
            "init".to_string(),
            "-b".to_string(),
            git.initial_branch.clone(),
        ],
        Some(project_dir),
    )?;
    if !init_status.success() {
        let fallback_status = run_process_status("git", &["init".to_string()], Some(project_dir))?;
        if !fallback_status.success() {
            return Err(LodeError::Message(format!(
                "git init failed with {fallback_status}"
            )));
        }
        let branch_status = run_process_status(
            "git",
            &[
                "checkout".to_string(),
                "-B".to_string(),
                git.initial_branch.clone(),
            ],
            Some(project_dir),
        )?;
        if !branch_status.success() {
            return Err(LodeError::Message(format!(
                "git branch setup failed with {branch_status}"
            )));
        }
    }
    println!("git initialised on {}", git.initial_branch);

    if git.initial_commit {
        if identity.author.trim().is_empty() || identity.email.trim().is_empty() {
            println!("git initial commit skipped: missing author identity");
        } else {
            run_git_in(project_dir, ["config", "user.name", &identity.author])?;
            run_git_in(project_dir, ["config", "user.email", &identity.email])?;
            run_git_in(project_dir, ["add", "."])?;
            let message = git
                .initial_commit_msg
                .replace("{project}", project_name)
                .replace("{org}", &identity.org);
            run_git_in(project_dir, ["commit", "-m", &message])?;
            println!("git initial commit: {message}");
        }
    }
    Ok(())
}

pub(crate) fn run_git_in<const N: usize>(
    project_dir: &Utf8PathBuf,
    args: [&str; N],
) -> lode_core::Result<()> {
    let args = args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    let status = run_process_status("git", &args, Some(project_dir))?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "git command failed with {status}"
        )))
    }
}

pub(crate) fn run_process_status(
    program: &str,
    args: &[String],
    current_dir: Option<&Utf8PathBuf>,
) -> lode_core::Result<std::process::ExitStatus> {
    run_process_status_with_env(program, args, current_dir, &[])
}

pub(crate) fn run_process_status_with_env(
    program: &str,
    args: &[String],
    current_dir: Option<&Utf8PathBuf>,
    envs: &[(&str, String)],
) -> lode_core::Result<std::process::ExitStatus> {
    let mut command = Process::new(program)?;
    command.args(args);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir.as_str());
    }
    command.envs(envs.iter().map(|(key, value)| (*key, value)));
    command.status()
}

fn run_process_output(program: &str, args: &[String]) -> lode_core::Result<std::process::Output> {
    Process::new(program)?.args(args).output()
}

fn run_current_lode_status(args: &[String]) -> lode_core::Result<std::process::ExitStatus> {
    Process::current_executable()?.args(args).status()
}

pub(crate) fn open_editor(path: impl AsRef<std::ffi::OsStr>) -> lode_core::Result<()> {
    // VISUAL takes precedence over EDITOR (Unix convention); defaults to notepad on Windows
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "notepad".to_string());
    let status = Process::new(&editor)?.args([path.as_ref()]).status()?;
    if !status.success() {
        return Err(LodeError::Message(format!(
            "editor exited with status: {status}"
        )));
    }
    Ok(())
}
pub(crate) fn safe_relative_path(path: &str) -> lode_core::Result<Utf8PathBuf> {
    let std_path = std::path::Path::new(path);
    for component in std_path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(LodeError::Message(format!("unsafe relative path: {path}")));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(LodeError::Message(format!("unsafe relative path: {path}")));
            }
            std::path::Component::Normal(_) | std::path::Component::CurDir => {}
        }
    }
    Ok(Utf8PathBuf::from(path))
}

pub(crate) fn print_simple_diff(current: &str, default: &str) {
    let current_lines = current.lines().collect::<Vec<_>>();
    let default_lines = default.lines().collect::<Vec<_>>();
    let max = current_lines.len().max(default_lines.len());
    for index in 0..max {
        match (current_lines.get(index), default_lines.get(index)) {
            (Some(left), Some(right)) if left == right => {}
            (Some(left), Some(right)) => {
                println!("- {left}");
                println!("+ {right}");
            }
            (Some(left), None) => println!("- {left}"),
            (None, Some(right)) => println!("+ {right}"),
            (None, None) => {}
        }
    }
}

pub(crate) fn mcp_command_names() -> Vec<String> {
    let mut names = command_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<BTreeSet<_>>();
    for dir in [Utf8PathBuf::from(".lode").join("commands")]
        .into_iter()
        .chain(global_asset_dir("commands").ok())
    {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = Utf8PathBuf::from_path_buf(entry.path());
                let Ok(path) = path else {
                    continue;
                };
                if path.extension() == Some("toml") {
                    if let Some(stem) = path.file_stem() {
                        names.insert(stem.to_string());
                    }
                }
            }
        }
    }
    names.into_iter().collect()
}

pub(crate) fn json_pretty(value: &Value) -> lode_core::Result<String> {
    serde_json::to_string_pretty(value).map_err(|error| LodeError::Message(error.to_string()))
}

pub(crate) fn run_mcp_stdio() -> lode_core::Result<()> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|source| LodeError::Io {
            path: "stdin".into(),
            source,
        })?;
    for line in input.lines().filter(|line| !line.trim().is_empty()) {
        let request: Value = serde_json::from_str(line)
            .map_err(|error| LodeError::Message(format!("invalid MCP request: {error}")))?;
        println!("{}", mcp_handle_request(&request));
    }
    Ok(())
}

pub(crate) fn run_lsp_stdio() -> lode_core::Result<()> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|source| LodeError::Io {
            path: "stdin".into(),
            source,
        })?;
    for line in input.lines().filter(|line| !line.trim().is_empty()) {
        let request: Value = serde_json::from_str(line)
            .map_err(|error| LodeError::Message(format!("invalid LSP request: {error}")))?;
        if let Some(response) = lsp_handle_request(&request) {
            println!("{response}");
        }
    }
    Ok(())
}

pub(crate) fn lsp_handle_request(request: &Value) -> Option<String> {
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match method {
        "initialize" => Some(
            json!({
                "jsonrpc": "2.0",
                "id": request.get("id").cloned().unwrap_or(Value::Null),
                "result": {
                    "serverInfo": { "name": "lode-lsp", "version": env!("CARGO_PKG_VERSION") },
                    "capabilities": lsp_capabilities()
                }
            })
            .to_string(),
        ),
        "shutdown" => Some(
            json!({
                "jsonrpc": "2.0",
                "id": request.get("id").cloned().unwrap_or(Value::Null),
                "result": Value::Null
            })
            .to_string(),
        ),
        "textDocument/didOpen" | "textDocument/didSave" => {
            let uri = request
                .pointer("/params/textDocument/uri")
                .and_then(Value::as_str)
                .unwrap_or("file:///unknown");
            let text = request
                .pointer("/params/textDocument/text")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Some(
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri,
                        "diagnostics": lsp_diagnostics(uri, text)
                    }
                })
                .to_string(),
            )
        }
        "initialized" | "exit" => None,
        _ => Some(
            json!({
                "jsonrpc": "2.0",
                "id": request.get("id").cloned().unwrap_or(Value::Null),
                "error": { "code": -32601, "message": format!("method not found: {method}") }
            })
            .to_string(),
        ),
    }
}

pub(crate) fn lsp_capabilities() -> Value {
    json!({
        "textDocumentSync": {
            "openClose": true,
            "change": 1,
            "save": { "includeText": true }
        },
        "diagnosticProvider": {
            "interFileDependencies": false,
            "workspaceDiagnostics": false
        }
    })
}

pub(crate) fn lsp_diagnostics(uri: &str, text: &str) -> Vec<Value> {
    let mut diagnostics = Vec::new();
    if should_require_signature(uri)
        && !text
            .lines()
            .take(20)
            .any(|line| line.contains("@file") || line.contains("@project"))
    {
        diagnostics.push(json!({
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 1 }
            },
            "severity": 3,
            "source": "lode",
            "message": "file is missing a lode signature header"
        }));
    }
    for (line_index, line) in text.lines().enumerate() {
        if line.contains("ghp_") || line.contains("github_pat_") {
            diagnostics.push(json!({
                "range": {
                    "start": { "line": line_index, "character": 0 },
                    "end": { "line": line_index, "character": line.len() }
                },
                "severity": 1,
                "source": "lode",
                "message": "possible secret token"
            }));
        }
    }
    diagnostics
}

pub(crate) fn should_require_signature(uri: &str) -> bool {
    ["rs", "ts", "js", "py", "go", "java", "c", "cpp", "h", "hpp"]
        .iter()
        .any(|extension| uri.ends_with(&format!(".{extension}")))
}

static MCP_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub(crate) fn mcp_handle_request(request: &Value) -> String {
    let method = request["method"].as_str().unwrap_or("");
    let id = request.get("id").cloned().unwrap_or(json!(null));
    if method == "initialize" {
        MCP_INITIALIZED.store(true, Ordering::SeqCst);
    } else if method != "notifications/initialized" && !MCP_INITIALIZED.load(Ordering::SeqCst) {
        return json!({"jsonrpc":"2.0","error":{"code":-32000,"message":"not initialized"},"id":id}).to_string();
    }
    match method {
        "tools/list" => json!({"jsonrpc":"2.0","result":mcp_tools(),"id":id}).to_string(),
        "tools/call" => match mcp_call_tool(request) {
            Ok(v) => json!({"jsonrpc":"2.0","result":v,"id":id}).to_string(),
            Err((c, m)) => json!({"jsonrpc":"2.0","error":{"code":c,"message":m},"id":id}).to_string(),
        },
        "resources/list" => json!({"jsonrpc":"2.0","result":mcp_resources(),"id":id}).to_string(),
        "resources/read" => match mcp_read_resource(request) {
            Ok(v) => json!({"jsonrpc":"2.0","result":v,"id":id}).to_string(),
            Err((c, m)) => json!({"jsonrpc":"2.0","error":{"code":c,"message":m},"id":id}).to_string(),
        },
        "prompts/list" => json!({"jsonrpc":"2.0","result":mcp_prompts(),"id":id}).to_string(),
        "initialize" => json!({"jsonrpc":"2.0","result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{},"resources":{"listChanged":false},"prompts":{"listChanged":false}},"serverInfo":{"name":"lode","version":env!("CARGO_PKG_VERSION")}},"id":id}).to_string(),
        _ => mcpserver::handle_request(request).to_string(),
    }
}

pub(crate) fn mcp_tools() -> Value {
    let mut tools: Vec<Value> = mcpserver::tools::register_all_tools().iter().map(|t| json!({"name": t.name, "description": t.description, "inputSchema": t.input_schema})).collect();
    tools.push(json!({"name": "lode_scan_foreign", "description": "Analyse a non-Lode project and return a local adoption/migration report.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string", "description": "Project path to inspect."}},"required": ["path"]}}));
    tools.push(json!({"name": "lode_profile_list", "description": "List embedded/default profile names.", "inputSchema": {"type": "object", "properties": {}}}));
    tools.push(json!({"name": "lode_recipe_list", "description": "List embedded/default recipe names.", "inputSchema": {"type": "object", "properties": {}}}));
    tools.push(json!({"name": "lode_metrics_show", "description": "Return the latest project metrics snapshot if available.", "inputSchema": {"type": "object", "properties": {}}}));
    for slug in mcp_command_names() {
        tools.push(json!({"name": format!("lode_custom_{slug}"), "description": format!("Discover local custom command `{slug}`."), "inputSchema": {"type": "object", "properties": {}}}));
    }
    json!({"tools": tools})
}

pub(crate) fn mcp_resources() -> Value {
    json!({"resources": mcpserver::resources::list_resources()})
}

pub(crate) fn mcp_prompts() -> Value {
    json!({"prompts": mcpserver::prompts::list_prompts()})
}

pub(crate) fn mcp_call_tool(request: &Value) -> std::result::Result<Value, (i64, String)> {
    let name = request
        .pointer("/params/name")
        .and_then(Value::as_str)
        .ok_or_else(|| (-32602, "missing params.name".to_string()))?;
    let args = request
        .pointer("/params/arguments")
        .cloned()
        .unwrap_or(json!({}));
    let is_cli_tool = matches!(
        name,
        "lode_scan_foreign"
            | "lode_profile_list"
            | "lode_recipe_list"
            | "lode_metrics_show"
            | "lode_pkg_audit"
            | "lode_pkg_outdated"
            | "lode_pkg_update"
    );
    if !name.starts_with("lode_custom_") && !is_cli_tool {
        let validator = mcpserver::ToolInputValidator::new(&mcpserver::tools::register_all_tools());
        validator.validate(name, &args).map_err(|e| (-32602, e))?;
    }
    let value = match name {
        "lode_scan_foreign" => {
            let path = args
                .get("path")
                .and_then(Value::as_str)
                .map(Utf8PathBuf::from)
                .map(Ok)
                .unwrap_or_else(current_dir)
                .map_err(|e| (-32603, e.to_string()))?;
            serde_json::to_value(scan_foreign_project(&path).map_err(|e| (-32603, e.to_string()))?)
                .map_err(|e| (-32603, e.to_string()))?
        }
        "lode_pkg_outdated" => {
            let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
            serde_json::to_value(PackageOperationPlan::new(
                "outdated",
                &manager,
                package_outdated_args(&manager).map_err(|e| (-32603, e.to_string()))?,
            ))
            .map_err(|e| (-32603, e.to_string()))?
        }
        "lode_pkg_audit" => {
            let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
            let fail_on = args.get("fail_on").and_then(Value::as_str);
            serde_json::to_value(PackageOperationPlan::new(
                "audit",
                &manager,
                package_audit_args(&manager, fail_on).map_err(|e| (-32603, e.to_string()))?,
            ))
            .map_err(|e| (-32603, e.to_string()))?
        }
        "lode_pkg_update" => {
            let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
            let name = args.get("name").and_then(Value::as_str);
            serde_json::to_value(PackageOperationPlan::new(
                "update",
                &manager,
                package_update_args(&manager, name).map_err(|e| (-32603, e.to_string()))?,
            ))
            .map_err(|e| (-32603, e.to_string()))?
        }
        "lode_profile_list" => json!(profile_names()),
        "lode_recipe_list" => json!(recipe_names()),
        "lode_metrics_show" => {
            let cwd = current_dir().map_err(|e| (-32603, e.to_string()))?;
            serde_json::to_value(load_metrics(&cwd).map_err(|e| (-32603, e.to_string()))?)
                .map_err(|e| (-32603, e.to_string()))?
        }
        other if other.starts_with("lode_custom_") => {
            mcp_custom_command_value(other.trim_start_matches("lode_custom_"))
                .map_err(|e| (-32603, e.to_string()))?
        }
        _ => mcpserver::tools::dispatch_tool(name, &args)
            .map_err(|e| (-32603, lode_core::redact(&e)))?,
    };
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
    Ok(json!({"content": [{"type": "text", "text": lode_core::redact(&text)}]}))
}

pub(crate) fn mcp_read_resource(request: &Value) -> std::result::Result<Value, (i64, String)> {
    let uri = request
        .pointer("/params/uri")
        .and_then(Value::as_str)
        .ok_or_else(|| (-32602, "missing params.uri".to_string()))?;
    let text = match uri {
        "lode://snippets" => serde_json::to_string_pretty(&snippet_inventory())
            .map_err(|e| (-32603, e.to_string()))?,
        "lode://project/time" => serde_json::to_string_pretty(&load_time_log().unwrap_or_default())
            .map_err(|e| (-32603, e.to_string()))?,
        "lode://project/config" => {
            toml::to_string_pretty(&load_global_config().unwrap_or_else(|_| default_config()))
                .map_err(|e| (-32603, e.to_string()))?
        }
        "lode://project/conventions" => {
            let config = load_global_config().unwrap_or_else(|_| default_config());
            serde_json::to_string_pretty(&config.convention).map_err(|e| (-32603, e.to_string()))?
        }
        _ => {
            let contents =
                mcpserver::resources::read_resource(uri).map_err(|e| (e.code(), e.to_string()))?;
            return Ok(json!({"contents": contents}));
        }
    };
    Ok(
        json!({"contents": [{"uri": uri, "mimeType": if uri == "lode://project/config" { "application/toml" } else { "application/json" }, "text": text}]}),
    )
}

pub(crate) fn mcp_custom_command_value(slug: &str) -> lode_core::Result<Value> {
    safe_relative_path(slug)?;
    let path = resolve_command_path(slug)?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    Ok(json!({
        "slug": slug,
        "path": path,
        "description": value.get("description").and_then(toml::Value::as_str),
        "steps": value.get("steps").and_then(toml::Value::as_array).map(|steps| {
            steps.iter().map(|step| json!({
                "kind": step.get("kind").and_then(toml::Value::as_str).unwrap_or("shell"),
                "run": step.get("run").and_then(toml::Value::as_str).unwrap_or_default()
            })).collect::<Vec<_>>()
        }).unwrap_or_default()
    }))
}

pub(crate) fn snippet_inventory() -> Vec<String> {
    let mut snippets = Vec::new();
    if let Ok(root) = global_asset_dir("snippets") {
        let _ = crate::cmd::snippet::collect_snippet_assets(&root, &mut snippets);
    }
    snippets
        .into_iter()
        .map(|snippet| format!("{}/{}", snippet.lang, snippet.name))
        .collect()
}

pub(crate) fn agent_sync() -> lode_core::Result<()> {
    let project_root = ValidatedRoot::new(current_dir()?)?;
    let context_dir = Utf8PathBuf::from(".lode").join("context");
    project_root.create_dir_all(&context_dir)?;
    let mut summary = String::from("# Agent Context Index\n\n");
    for root in ["_ref_", "_ctx_"] {
        let path = Utf8PathBuf::from(root);
        summary.push_str(&format!("## {root}\n"));
        if path.exists() {
            collect_context_index(&path, &mut summary)?;
        } else {
            summary.push_str("- missing\n");
        }
        summary.push('\n');
    }
    let output = context_dir.join("INDEX.md");
    project_root.write_atomic(&output, summary)?;
    println!("agent context synced to {output}");
    Ok(())
}

pub(crate) fn collect_context_index(
    path: &Utf8PathBuf,
    output: &mut String,
) -> lode_core::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            collect_context_index(&child, output)?;
        }
    } else {
        output.push_str(&format!("- {path}\n"));
    }
    Ok(())
}

pub(crate) fn agent_status() -> lode_core::Result<()> {
    let index = Utf8PathBuf::from(".lode").join("context").join("INDEX.md");
    let plan = agent_plan_path();
    println!("context\t{}", status_bool(index.exists()));
    println!("plan\t{}", status_bool(plan.exists()));
    Ok(())
}

pub(crate) fn agent_export(out: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    let output = out.unwrap_or_else(|| Utf8PathBuf::from("agent-context.lodepack"));
    let mut pack = LodePack {
        version: 1,
        manifest: LodePackManifest {
            schema_version: 3,
            lode_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now_timestamp(),
            file_count: 0,
            checksum_algorithm: default_lodepack_checksum_algorithm(),
        },
        files: Vec::new(),
    };
    let root = current_dir()?;
    for path in ["AGENTS.md", "CODEX.md", "CLAUDE.md", ".lode/context"] {
        collect_pack_files(&root, &root.join(path), &mut pack)?;
    }
    pack.manifest.file_count = pack.files.len();
    let raw = serde_json::to_string_pretty(&pack)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    write_validated_output(&output, raw)?;
    println!("exported agent context to {output}");
    Ok(())
}

pub(crate) fn agent_plan(command: AgentPlanCommand) -> lode_core::Result<()> {
    match command {
        AgentPlanCommand::Init => {
            save_agent_plan(&AgentPlan {
                next_id: 1,
                tasks: Vec::new(),
            })?;
            println!("agent plan initialised");
        }
        AgentPlanCommand::Add { task, branch } => {
            let mut plan = load_agent_plan()?;
            if plan.next_id == 0 {
                plan.next_id = 1;
            }
            let id = plan.next_id;
            plan.next_id += 1;
            plan.tasks.push(AgentTask {
                id,
                task,
                branch,
                done: false,
            });
            save_agent_plan(&plan)?;
            println!("added task {id}");
        }
        AgentPlanCommand::Done { id } => {
            let mut plan = load_agent_plan()?;
            let task = plan
                .tasks
                .iter_mut()
                .find(|task| task.id == id)
                .ok_or_else(|| LodeError::Message(format!("agent task not found: {id}")))?;
            task.done = true;
            save_agent_plan(&plan)?;
            println!("completed task {id}");
        }
        AgentPlanCommand::Show => {
            let plan = load_agent_plan()?;
            for task in plan.tasks {
                println!(
                    "{}\t{}\t{}\t{}",
                    task.id,
                    if task.done { "done" } else { "open" },
                    task.branch.unwrap_or_else(|| "-".to_string()),
                    task.task
                );
            }
        }
        AgentPlanCommand::Clear => {
            let path = agent_plan_path();
            if path.exists() {
                ValidatedRoot::new(current_dir()?)?.remove_file(&path)?;
            }
            println!("agent plan cleared");
        }
    }
    Ok(())
}

pub(crate) fn agent_plan_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("agent-plan.json")
}

pub(crate) fn load_agent_plan() -> lode_core::Result<AgentPlan> {
    let path = agent_plan_path();
    if !path.exists() {
        return Ok(AgentPlan {
            next_id: 1,
            tasks: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub(crate) fn save_agent_plan(plan: &AgentPlan) -> lode_core::Result<()> {
    let path = agent_plan_path();
    let project_root = ValidatedRoot::new(current_dir()?)?;
    if let Some(parent) = path.parent() {
        project_root.create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(plan)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    project_root.write_atomic(&path, raw).map(|_| ())
}

pub(crate) fn resolve_command_path(slug: &str) -> lode_core::Result<Utf8PathBuf> {
    let candidates = [
        Utf8PathBuf::from(".lode")
            .join("commands")
            .join(format!("{slug}.toml")),
        global_asset_dir("commands")?.join(format!("{slug}.toml")),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(LodeError::Message(format!("command not found: {slug}")))
}

fn external_command(args: Vec<String>) -> lode_core::Result<()> {
    let Some(slug) = args.first() else {
        return Err(LodeError::Message("missing command slug".to_string()));
    };
    if env::var_os("LODE_NO_CUSTOM_COMMANDS").is_some() {
        return Err(LodeError::Message(
            "custom commands are disabled by LODE_NO_CUSTOM_COMMANDS".to_string(),
        ));
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return show_command_macro_help(slug);
    }
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let forwarded_args = args
        .iter()
        .skip(1)
        .filter(|arg| arg.as_str() != "--dry-run")
        .cloned()
        .collect::<Vec<_>>();
    if !forwarded_args.is_empty() {
        println!("custom command args: {}", forwarded_args.join(" "));
    }
    run_command_macro(slug, dry_run)
}

fn show_command_macro_help(slug: &str) -> lode_core::Result<()> {
    let path = resolve_command_path(slug)?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    println!("lode {slug}");
    if let Some(description) = value.get("description").and_then(toml::Value::as_str) {
        println!("{description}");
    }
    if let Some(steps) = value.get("steps").and_then(toml::Value::as_array) {
        println!("steps:");
        for (index, step) in steps.iter().enumerate() {
            let kind = step
                .get("kind")
                .and_then(toml::Value::as_str)
                .unwrap_or("shell");
            let run = step.get("run").and_then(toml::Value::as_str).unwrap_or("");
            println!("  {}. [{kind}] {run}", index + 1);
        }
    }
    println!("flags:");
    println!("  --dry-run    preview steps without executing");
    println!("  --help       show this command definition");
    Ok(())
}

fn run_command_macro(slug: &str, dry_run: bool) -> lode_core::Result<()> {
    let path = resolve_command_path(slug)?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    let Some(steps) = value.get("steps").and_then(toml::Value::as_array) else {
        println!("command {slug} has no steps");
        return Ok(());
    };

    for (index, step) in steps.iter().enumerate() {
        let kind = step
            .get("kind")
            .and_then(toml::Value::as_str)
            .unwrap_or("shell");
        let run = step
            .get("run")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                LodeError::Message(format!("command {slug} step {} missing run", index + 1))
            })?;
        let continue_on_error = step
            .get("continue_on_error")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        println!("step {} [{kind}] {run}", index + 1);
        if dry_run {
            continue;
        }
        let result = match kind {
            "make" => run_make(run),
            "lode" => run_lode_step(run),
            "shell" => run_shell_step(run),
            other => Err(LodeError::Message(format!(
                "unsupported step kind: {other}"
            ))),
        };
        if let Err(error) = result {
            if continue_on_error {
                eprintln!("warning: {error}");
            } else {
                return Err(error);
            }
        }
    }
    Ok(())
}

pub(crate) fn run_command_macro_loaded(
    slug: &str,
    value: &toml::Value,
    args: &HashMap<String, String>,
    dry_run: bool,
) -> lode_core::Result<()> {
    let Some(steps) = value.get("steps").and_then(toml::Value::as_array) else {
        println!("command {slug} has no steps");
        return Ok(());
    };

    if let Some(description) = value.get("description").and_then(toml::Value::as_str) {
        println!("{slug}: {description}");
    } else {
        println!("{slug}");
    }

    for (index, step) in steps.iter().enumerate() {
        let kind = step
            .get("kind")
            .and_then(toml::Value::as_str)
            .unwrap_or("shell");
        let run_raw = step
            .get("run")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                LodeError::Message(format!("command {slug} step {} missing run", index + 1))
            })?;
        let show_output = step
            .get("show_output")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let continue_on_error = step
            .get("continue_on_error")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let skip_if = step
            .get("skip_if")
            .and_then(toml::Value::as_str)
            .unwrap_or("");

        // Substitute {{ args.X }} and {{ X }} patterns
        let mut run = run_raw.to_string();
        for (key, val) in args {
            run = run.replace(&format!("{{{{ args.{key} }}}}"), val);
            run = run.replace(&format!("{{{{ args.{key} }}}}"), val);
            run = run.replace(&format!("{{{{{key}}}}}"), val);
            run = run.replace(&format!("{{{{{key}}}}}"), val);
        }

        // Check skip condition
        if let Some(condition) = skip_if.strip_prefix("if_exists:") {
            if Utf8PathBuf::from(condition).exists() {
                if !dry_run {
                    println!(
                        "  {} {} [skip] {}",
                        output::dim("−"),
                        output::dim(&format!("{}.", index + 1)),
                        output::dim(&run)
                    );
                }
                continue;
            }
        }

        println!(
            "  {} {} [{}] {}",
            output::cyan("▶"),
            output::dim(&format!("{}.", index + 1)),
            output::cyan(kind),
            run
        );
        if dry_run {
            continue;
        }

        let result = match kind {
            "make" => run_make(&run),
            "lode" => run_lode_step(&run),
            "shell" | "npm" | "pnpm" | "cargo" | "python3" | "python" | "node" | "just"
            | "docker" => {
                let mut parts = shell_split(&run)
                    .ok_or_else(|| LodeError::Message(format!("unable to parse command: {run}")))?;
                if parts.is_empty() {
                    continue;
                }
                if kind != "shell" {
                    parts.insert(0, kind.to_string());
                }
                let step_ok = if show_output {
                    let output = run_process_output(&parts[0], &parts[1..])?;
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                    output.status.success()
                } else {
                    run_process_status(&parts[0], &parts[1..], None)?.success()
                };
                if step_ok {
                    println!(
                        "  {} {} [{}] {}",
                        output::green("✔"),
                        output::dim(&format!("{}.", index + 1)),
                        output::cyan(kind),
                        output::dim("done")
                    );
                } else {
                    if continue_on_error {
                        eprintln!(
                            "  {} {} [{}] {}",
                            output::yellow("⚠"),
                            output::dim(&format!("{}.", index + 1)),
                            output::cyan(kind),
                            output::dim("failed")
                        );
                    } else {
                        return Err(LodeError::Message(format!(
                            "step {} failed: {run}",
                            index + 1
                        )));
                    }
                }
                Ok(())
            }
            other => Err(LodeError::Message(format!(
                "unsupported step kind: {other}"
            ))),
        };
        if let Err(error) = result {
            if continue_on_error {
                eprintln!("  warning: {error}");
            } else {
                return Err(error);
            }
        }
    }
    Ok(())
}

fn run_lode_step(run: &str) -> lode_core::Result<()> {
    let mut parts = run.split_whitespace();
    let Some(command) = parts.next() else {
        return Ok(());
    };
    let mut args = vec![command.to_string()];
    for part in parts {
        args.push(part.to_string());
    }
    let status = run_current_lode_status(&args)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "lode {run} failed with {status}"
        )))
    }
}

fn run_shell_step(run: &str) -> lode_core::Result<()> {
    let parts = shell_split(run)
        .ok_or_else(|| LodeError::Message(format!("unable to parse command: {run}")))?;
    if parts.is_empty() {
        return Ok(());
    }
    let status = run_process_status(&parts[0], &parts[1..], None)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "shell step failed with {status}"
        )))
    }
}

/// Split a command string into arguments, respecting single and double quotes.
fn shell_split(input: &str) -> Option<Vec<String>> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if in_single || in_double {
        return None;
    }
    if !current.is_empty() {
        args.push(current);
    }
    Some(args)
}

pub(crate) fn build_doctor_report(fixed: bool) -> DoctorReport {
    let mut checks = Vec::new();
    match load_global_config() {
        Ok(config) => checks.push(doctor_check(
            "config",
            "ok",
            &format!("schema v{}", config.schema_version),
        )),
        Err(error) => checks.push(doctor_check("config", "fail", &error.to_string())),
    }

    match global_dir() {
        Ok(root) => {
            checks.push(doctor_check("global_dir", "ok", root.as_str()));
            for name in [
                "templates",
                "profiles",
                "snippets",
                "licenses",
                "recipes",
                "plugins",
                "commands",
            ] {
                let path = root.join(name);
                let status = if path.exists() { "ok" } else { "warn" };
                checks.push(doctor_check(name, status, path.as_str()));
            }
        }
        Err(error) => checks.push(doctor_check("global_dir", "fail", &error.to_string())),
    }

    let project_config = Utf8PathBuf::from(".lode").join("project.toml");
    checks.push(doctor_check(
        "project",
        if project_config.exists() {
            "ok"
        } else {
            "warn"
        },
        if project_config.exists() {
            ".lode/project.toml"
        } else {
            "no project config in current directory"
        },
    ));

    let package_manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
    checks.push(doctor_check(
        "package_manager",
        if package_manager == "unknown" {
            "warn"
        } else {
            "ok"
        },
        &package_manager,
    ));

    let required = required_tools_for_project();
    if required.is_empty() {
        checks.push(doctor_check("toolchain", "ok", "no project tools required"));
    } else {
        for tool in required {
            let installed = command_version(tool).is_some();
            checks.push(doctor_check(
                &format!("tool:{tool}"),
                if installed { "ok" } else { "warn" },
                if installed {
                    "available"
                } else {
                    "not found on PATH"
                },
            ));
        }
    }

    match load_daemon_runtime_state() {
        Ok(state) => checks.push(doctor_check(
            "daemon",
            if state.active { "ok" } else { "warn" },
            &format!(
                "active={} paused={} watchers={}",
                state.active,
                state.paused,
                state.watchers.join(",")
            ),
        )),
        Err(error) => checks.push(doctor_check("daemon", "warn", &error.to_string())),
    }

    let upgrade_state = upgrade_state_path()
        .ok()
        .filter(|path| path.exists())
        .map(|path| path.to_string())
        .unwrap_or_else(|| "none".to_string());
    checks.push(doctor_check(
        "upgrade",
        if upgrade_state == "none" {
            "ok"
        } else {
            "warn"
        },
        &upgrade_state,
    ));

    match cmd::hooks::discover_hooks() {
        Ok(hooks) => checks.push(doctor_check(
            "hooks",
            "ok",
            &format!("{} discovered", hooks.len()),
        )),
        Err(error) => checks.push(doctor_check("hooks", "warn", &error.to_string())),
    }

    let status = if checks.iter().any(|check| check.status == "fail") {
        "fail"
    } else if checks.iter().any(|check| check.status == "warn") {
        "warn"
    } else {
        "ok"
    };
    DoctorReport {
        status: status.to_string(),
        fixed,
        checks,
    }
}

pub(crate) fn doctor_check(name: &str, status: &str, detail: &str) -> DoctorCheck {
    DoctorCheck {
        name: name.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    }
}

fn explain() {
    println!("Lode keeps project structure, defaults, commands, snippets, and context consistent.");
    println!("Start with `lode init <name> --profile systems/rust-cli --with ci,vscode`.");
}

pub(crate) fn scan_foreign_project(path: &Utf8PathBuf) -> lode_core::Result<ForeignScanReport> {
    let config = default_config();
    let convention = check_path(path, &config)?;
    let secrets = scan_secrets(path)?;
    let manifests = project_manifests(path);
    let lode_project = path.join(".lode").join("project.toml").exists();
    let package_manager = detect_package_manager_in(path);
    let mut migration_actions = Vec::new();
    if !lode_project {
        let name = path.file_name().unwrap_or("project");
        let parent = path
            .parent()
            .map(|parent| parent.to_string())
            .unwrap_or_else(|| ".".to_string());
        migration_actions.push(format!("run lode init {name} --path {parent}"));
    }
    if !path.join(".editorconfig").exists() {
        migration_actions.push("add editorconfig defaults with lode add editorconfig".to_string());
    }
    if !path.join(".gitignore").exists() {
        migration_actions
            .push("add root gitignore defaults with lode sync --section templates".to_string());
    }
    if convention.violations.is_empty() && secrets.findings.is_empty() {
        migration_actions.push("project is ready for lode adoption review".to_string());
    }
    Ok(ForeignScanReport {
        path: path.clone(),
        lode_project,
        package_manager,
        manifests,
        convention_checked: convention.checked,
        convention_violations: convention.violations.len(),
        secret_findings: secrets.findings.len(),
        migration_actions,
    })
}

pub(crate) fn project_manifests(path: &Utf8PathBuf) -> Vec<String> {
    [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "build.gradle",
        "settings.gradle",
        "pom.xml",
        "Gemfile",
    ]
    .iter()
    .filter(|manifest| path.join(manifest).exists())
    .map(|manifest| (*manifest).to_string())
    .collect()
}

pub(crate) fn stamp_files(
    path: &Utf8PathBuf,
    extensions: &[String],
    text: &str,
    force: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            if should_skip_walk(&child) {
                continue;
            }
            stamp_files(&child, extensions, text, force, dry_run)?;
        }
        return Ok(());
    }

    if !path.exists() || !matches_extension(path, extensions) {
        return Ok(());
    }
    let Some(header) = comment_header(path, text) else {
        return Ok(());
    };
    let contents = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    if !force && contents.contains(text) {
        return Ok(());
    }
    if dry_run {
        println!("would stamp {path}");
        return Ok(());
    }
    let updated = if contents.starts_with("#!") {
        if let Some((first, rest)) = contents.split_once('\n') {
            format!("{first}\n{header}{rest}")
        } else {
            format!("{contents}\n{header}")
        }
    } else {
        format!("{header}{contents}")
    };
    write_validated_output(path, updated)?;
    println!("stamped {path}");
    Ok(())
}

fn should_skip_walk(path: &Utf8PathBuf) -> bool {
    path.file_name()
        .map(|name| {
            matches!(
                name,
                ".git" | "target" | "node_modules" | ".venv" | "__pycache__"
            )
        })
        .unwrap_or(false)
}

fn matches_extension(path: &Utf8PathBuf, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return comment_prefix(path).is_some();
    }
    let ext = path.extension().unwrap_or_default();
    extensions
        .iter()
        .map(|value| value.trim_start_matches('.'))
        .any(|value| value == ext)
}

fn comment_header(path: &Utf8PathBuf, text: &str) -> Option<String> {
    let prefix = comment_prefix(path)?;
    if prefix == "<!--" {
        Some(format!("<!-- {text} -->\n\n"))
    } else {
        Some(format!("{prefix} {text}\n\n"))
    }
}

fn comment_prefix(path: &Utf8PathBuf) -> Option<&'static str> {
    match path.extension().unwrap_or_default() {
        "rs" | "c" | "h" | "cpp" | "hpp" | "cc" | "ts" | "tsx" | "js" | "jsx" | "go" | "zig"
        | "java" | "kt" | "swift" => Some("//"),
        "py" | "sh" | "ps1" | "toml" | "yml" | "yaml" | "ini" | "env" => Some("#"),
        "md" | "html" | "xml" => Some("<!--"),
        _ => None,
    }
}

pub(crate) fn add_license(
    id: &str,
    file: Option<Utf8PathBuf>,
    text: Option<&str>,
) -> lode_core::Result<()> {
    let asset_dir = global_asset_dir("licenses")?;
    let root = ValidatedRoot::new(&asset_dir)?;
    let relative = safe_relative_path(&format!("{id}.txt"))?;
    let path = asset_dir.join(&relative);
    if path.exists() {
        return Err(LodeError::Message(format!("license already exists: {id}")));
    }
    let contents = if let Some(file) = file {
        fs::read_to_string(&file).map_err(|source| LodeError::Io {
            path: file.as_str().into(),
            source,
        })?
    } else {
        text.unwrap_or(id).to_string()
    };
    if let Some(parent) = relative.parent() {
        root.create_dir_all(parent)?;
    }
    root.write_atomic(relative, contents)?;
    println!("added license {id}");
    Ok(())
}

pub(crate) fn license_path(id: &str) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&format!("{id}.txt"))?;
    Ok(global_asset_dir("licenses")?.join(relative))
}

pub(crate) fn project_license_id() -> lode_core::Result<Option<String>> {
    let path = Utf8PathBuf::from(".lode").join("project.toml");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    Ok(value
        .get("project")
        .and_then(|project| project.get("license"))
        .and_then(toml::Value::as_str)
        .map(str::to_string))
}

pub(crate) fn read_license(id: &str) -> lode_core::Result<String> {
    let candidates = [
        global_asset_dir("licenses")?.join(format!("{id}.txt")),
        global_asset_dir("licenses")?.join(id),
    ];
    for path in candidates {
        if path.exists() {
            return fs::read_to_string(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            });
        }
    }
    Err(LodeError::Message(format!("license not found: {id}")))
}

pub(crate) fn toolchain_store_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("toolchains.toml")
}

pub(crate) fn load_toolchain_store() -> lode_core::Result<ToolchainStore> {
    let path = toolchain_store_path();
    if !path.exists() {
        return Ok(ToolchainStore::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub(crate) fn save_toolchain_store(store: &ToolchainStore) -> lode_core::Result<()> {
    let path = toolchain_store_path();
    let root = ValidatedRoot::new(current_dir()?)?;
    if let Some(parent) = path.parent() {
        root.create_dir_all(parent)?;
    }
    let raw = toml::to_string_pretty(store)?;
    root.write_atomic(path, raw).map(|_| ())
}

pub(crate) fn pin_runtime(runtime: &str, version: &str) -> lode_core::Result<()> {
    let root = ValidatedRoot::new(current_dir()?)?;
    match runtime {
        "rust" | "rustc" | "cargo" => {
            root.write_atomic(
                "rust-toolchain.toml",
                format!("[toolchain]\nchannel = \"{version}\"\n"),
            )?;
        }
        "node" | "npm" | "pnpm" | "yarn" | "bun" => {
            root.write_atomic(".nvmrc", format!("{version}\n"))?;
        }
        "python" | "uv" => {
            root.write_atomic(".python-version", format!("{version}\n"))?;
        }
        "go" => {
            root.write_atomic("go.env", format!("GOTOOLCHAIN=go{version}\n"))?;
        }
        other => {
            root.write_atomic(
                format!(".toolchain-{other}"),
                format!("{other}={version}\n"),
            )?;
        }
    }
    Ok(())
}

fn package_manifest_inventory() -> Vec<PackageManifest> {
    let root = Utf8PathBuf::from(".");
    let mut manifests = Vec::new();
    for (file, kind, manager) in [
        ("Cargo.toml", "cargo", "cargo"),
        ("package.json", "node", "npm"),
        ("pyproject.toml", "python", "uv"),
        ("requirements.txt", "python", "pip"),
        ("go.mod", "go", "go"),
        ("Gemfile", "ruby", "bundler"),
        ("build.gradle", "gradle", "gradle"),
        ("settings.gradle", "gradle", "gradle"),
        ("pom.xml", "maven", "maven"),
    ] {
        let path = root.join(file);
        if !path.exists() {
            continue;
        }
        let raw = fs::read_to_string(&path).unwrap_or_default();
        let dependencies = match file {
            "Cargo.toml" => parse_cargo_dependencies(file, &raw),
            "package.json" => parse_node_dependencies(file, &raw),
            "pyproject.toml" => parse_pyproject_dependencies(file, &raw),
            "requirements.txt" => parse_requirements_dependencies(file, &raw),
            "go.mod" => parse_go_dependencies(file, &raw),
            "Gemfile" => parse_gemfile_dependencies(file, &raw),
            "build.gradle" => parse_gradle_dependencies(file, &raw),
            "pom.xml" => parse_maven_dependencies(file, &raw),
            _ => Vec::new(),
        };
        manifests.push(PackageManifest {
            file: file.to_string(),
            kind: kind.to_string(),
            manager: manager.to_string(),
            dependencies,
        });
    }
    manifests
}

pub(crate) fn package_dependencies() -> Vec<PackageDependency> {
    package_manifest_inventory()
        .into_iter()
        .flat_map(|manifest| manifest.dependencies)
        .collect()
}

fn parse_cargo_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    let value = match raw.parse::<toml::Value>() {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let mut dependencies = Vec::new();
    for scope in [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "workspace.dependencies",
    ] {
        if let Some(table) = toml_table_path(&value, scope) {
            for (name, value) in table {
                dependencies.push(PackageDependency {
                    name: name.to_string(),
                    version: toml_dependency_version(value),
                    scope: scope.to_string(),
                    manifest: manifest.to_string(),
                });
            }
        }
    }
    dependencies
}

fn toml_table_path<'a>(
    value: &'a toml::Value,
    path: &str,
) -> Option<&'a toml::map::Map<String, toml::Value>> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    current.as_table()
}

fn toml_dependency_version(value: &toml::Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            value
                .get("version")
                .and_then(toml::Value::as_str)
                .map(str::to_string)
        })
        .or_else(|| {
            value
                .get("path")
                .and_then(toml::Value::as_str)
                .map(|path| format!("path:{path}"))
        })
        .or_else(|| {
            value
                .get("git")
                .and_then(toml::Value::as_str)
                .map(|git| format!("git:{git}"))
        })
}

fn parse_node_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    let value: Value = match serde_json::from_str(raw) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let mut dependencies = Vec::new();
    for scope in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        if let Some(object) = value.get(scope).and_then(Value::as_object) {
            for (name, version) in object {
                dependencies.push(PackageDependency {
                    name: name.to_string(),
                    version: version.as_str().map(str::to_string),
                    scope: scope.to_string(),
                    manifest: manifest.to_string(),
                });
            }
        }
    }
    dependencies
}

fn parse_pyproject_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    let value = match raw.parse::<toml::Value>() {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let mut dependencies = Vec::new();
    if let Some(items) = value
        .get("project")
        .and_then(|project| project.get("dependencies"))
        .and_then(toml::Value::as_array)
    {
        for item in items.iter().filter_map(toml::Value::as_str) {
            dependencies.push(python_dependency(manifest, "dependencies", item));
        }
    }
    if let Some(groups) = value
        .get("project")
        .and_then(|project| project.get("optional-dependencies"))
        .and_then(toml::Value::as_table)
    {
        for (group, items) in groups {
            if let Some(items) = items.as_array() {
                for item in items.iter().filter_map(toml::Value::as_str) {
                    dependencies.push(python_dependency(
                        manifest,
                        &format!("optional-dependencies.{group}"),
                        item,
                    ));
                }
            }
        }
    }
    dependencies
}

fn parse_requirements_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    raw.lines()
        .filter_map(|line| {
            let line = line.split('#').next()?.trim();
            if line.is_empty() || line.starts_with('-') {
                return None;
            }
            Some(python_dependency(manifest, "requirements", line))
        })
        .collect()
}

fn python_dependency(manifest: &str, scope: &str, spec: &str) -> PackageDependency {
    let split_at = spec
        .char_indices()
        .find(|(_, character)| matches!(character, '<' | '>' | '=' | '!' | '~' | '[' | ';' | ' '))
        .map(|(index, _)| index)
        .unwrap_or(spec.len());
    let name = spec[..split_at].trim().to_string();
    let version = spec[split_at..].trim();
    PackageDependency {
        name,
        version: (!version.is_empty()).then(|| version.to_string()),
        scope: scope.to_string(),
        manifest: manifest.to_string(),
    }
}

fn parse_go_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    let mut dependencies = Vec::new();
    let mut in_require_block = false;
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("require (") {
            in_require_block = true;
            continue;
        }
        if in_require_block && line == ")" {
            in_require_block = false;
            continue;
        }
        let require = if in_require_block {
            line
        } else if let Some(require) = line.strip_prefix("require ") {
            require
        } else {
            continue;
        };
        let mut parts = require.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let version = parts.next().map(str::to_string);
        dependencies.push(PackageDependency {
            name: name.to_string(),
            version,
            scope: "require".to_string(),
            manifest: manifest.to_string(),
        });
    }
    dependencies
}

fn parse_gemfile_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("gem ") {
                return None;
            }
            let quoted = extract_quoted_strings(trimmed);
            let name = quoted.first()?.to_string();
            let version = quoted.get(1).map(|value| (*value).to_string());
            Some(PackageDependency {
                name,
                version,
                scope: "gem".to_string(),
                manifest: manifest.to_string(),
            })
        })
        .collect()
}

fn parse_gradle_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    let scopes = [
        "implementation",
        "api",
        "compileOnly",
        "runtimeOnly",
        "testImplementation",
        "testRuntimeOnly",
    ];
    let mut dependencies = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        let Some(scope) = scopes
            .iter()
            .find(|scope| trimmed.starts_with(**scope) || trimmed.contains(&format!(" {scope} ")))
        else {
            continue;
        };
        let quoted = extract_quoted_strings(trimmed);
        let Some(spec) = quoted.first() else {
            continue;
        };
        let mut parts = spec.split(':');
        let group = parts.next().unwrap_or_default();
        let artifact = parts.next().unwrap_or_default();
        let version = parts.next().map(str::to_string);
        if group.is_empty() || artifact.is_empty() {
            continue;
        }
        dependencies.push(PackageDependency {
            name: format!("{group}:{artifact}"),
            version,
            scope: (*scope).to_string(),
            manifest: manifest.to_string(),
        });
    }
    dependencies
}

fn parse_maven_dependencies(manifest: &str, raw: &str) -> Vec<PackageDependency> {
    let mut dependencies = Vec::new();
    for block in raw.split("<dependency>").skip(1) {
        let block = block.split("</dependency>").next().unwrap_or_default();
        let Some(group) = xml_tag_text(block, "groupId") else {
            continue;
        };
        let Some(artifact) = xml_tag_text(block, "artifactId") else {
            continue;
        };
        dependencies.push(PackageDependency {
            name: format!("{group}:{artifact}"),
            version: xml_tag_text(block, "version"),
            scope: xml_tag_text(block, "scope").unwrap_or_else(|| "compile".to_string()),
            manifest: manifest.to_string(),
        });
    }
    dependencies
}

fn extract_quoted_strings(line: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut start = None;
    let mut quote = '\0';
    for (index, character) in line.char_indices() {
        if let Some(start_index) = start {
            if character == quote {
                values.push(&line[start_index..index]);
                start = None;
            }
        } else if character == '"' || character == '\'' {
            quote = character;
            start = Some(index + character.len_utf8());
        }
    }
    values
}

fn xml_tag_text(raw: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    Some(
        raw.split_once(&open)?
            .1
            .split_once(&close)?
            .0
            .trim()
            .to_string(),
    )
}

pub(crate) fn package_command(manager: &str) -> &str {
    match manager {
        "maven" => "mvn",
        other => other,
    }
}

pub(crate) fn package_outdated_args(manager: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["outdated".into()]),
        "npm" | "pnpm" | "yarn" | "bun" => Ok(vec!["outdated".into()]),
        "uv" => Ok(vec!["pip".into(), "list".into(), "--outdated".into()]),
        "pip" => Ok(vec!["list".into(), "--outdated".into()]),
        "go" => Ok(vec!["list".into(), "-m".into(), "-u".into(), "all".into()]),
        "bundler" => Ok(vec!["outdated".into()]),
        "gradle" => Ok(vec![
            "dependencyUpdates".into(),
            "-Drevision=release".into(),
        ]),
        "maven" => Ok(vec![
            "versions:display-dependency-updates".into(),
            "versions:display-plugin-updates".into(),
        ]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

pub(crate) fn package_update_args(
    manager: &str,
    name: Option<&str>,
) -> lode_core::Result<Vec<String>> {
    let mut args = match manager {
        "cargo" => vec!["update".to_string()],
        "npm" => vec!["update".to_string()],
        "pnpm" => vec!["update".to_string()],
        "yarn" => vec!["upgrade".to_string()],
        "bun" => vec!["update".to_string()],
        "uv" => vec!["lock".to_string(), "--upgrade".to_string()],
        "pip" => vec!["install".to_string(), "-U".to_string()],
        "go" => vec!["get".to_string(), "-u".to_string()],
        "bundler" => vec!["update".to_string()],
        "gradle" => vec!["--refresh-dependencies".to_string()],
        "maven" => vec![
            "versions:use-latest-releases".to_string(),
            "-DgenerateBackupPoms=false".to_string(),
        ],
        _ => {
            return Err(LodeError::Message(
                "no supported package manager files found".to_string(),
            ))
        }
    };
    if let Some(name) = name {
        if manager == "maven" {
            args.push(format!("-Dincludes={name}"));
        } else {
            args.push(name.to_string());
        }
    } else if manager == "go" {
        args.push("./...".to_string());
    } else if manager == "pip" {
        return Err(LodeError::Message(
            "pip update requires a package name".to_string(),
        ));
    }
    Ok(args)
}

pub(crate) fn package_audit_args(
    manager: &str,
    fail_on: Option<&str>,
) -> lode_core::Result<Vec<String>> {
    let fail_on = fail_on.map(validate_package_severity).transpose()?;
    match manager {
        "cargo" => {
            let mut args = vec!["audit".into()];
            if let Some(severity) = fail_on {
                args.push("--deny".into());
                args.push(severity.into());
            }
            Ok(args)
        }
        "npm" => {
            let mut args = vec!["audit".into()];
            if let Some(severity) = fail_on {
                args.push("--audit-level".into());
                args.push(severity.into());
            }
            Ok(args)
        }
        "pnpm" | "yarn" | "bun" => {
            let mut args = vec!["audit".into()];
            if let Some(severity) = fail_on {
                args.push("--audit-level".into());
                args.push(severity.into());
            }
            Ok(args)
        }
        "uv" => Ok(vec!["pip".into(), "check".into()]),
        "pip" => {
            let mut args = vec!["audit".into()];
            if let Some(severity) = fail_on {
                args.push("--severity".into());
                args.push(severity.into());
            }
            Ok(args)
        }
        "go" => Ok(vec!["vulncheck".into(), "./...".into()]),
        "bundler" => Ok(vec!["audit".into(), "check".into()]),
        "gradle" => {
            let mut args = vec!["dependencyCheckAnalyze".into()];
            if let Some(severity) = fail_on {
                args.push(format!(
                    "-DfailBuildOnCVSS={}",
                    severity_cvss_threshold(severity)
                ));
            }
            Ok(args)
        }
        "maven" => {
            let mut args = vec!["org.owasp:dependency-check-maven:check".into()];
            if let Some(severity) = fail_on {
                args.push(format!(
                    "-DfailBuildOnCVSS={}",
                    severity_cvss_threshold(severity)
                ));
            }
            Ok(args)
        }
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

pub(crate) fn validate_package_severity(severity: &str) -> lode_core::Result<&str> {
    match severity {
        "low" | "medium" | "high" | "critical" => Ok(severity),
        other => Err(LodeError::Message(format!(
            "unsupported package audit severity: {other}"
        ))),
    }
}

pub(crate) fn severity_cvss_threshold(severity: &str) -> &'static str {
    match severity {
        "low" => "0",
        "medium" => "4",
        "high" => "7",
        "critical" => "9",
        _ => "7",
    }
}

pub(crate) fn time_log_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(current_dir()?.join(".lode").join("time-log.json"))
}

pub(crate) fn load_time_log() -> lode_core::Result<TimeLog> {
    let path = time_log_path()?;
    if !path.exists() {
        return Ok(TimeLog::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub(crate) fn save_time_log(log: &TimeLog) -> lode_core::Result<()> {
    let root = ValidatedRoot::new(current_dir()?)?;
    root.create_dir_all(".lode")?;
    let raw =
        serde_json::to_string_pretty(log).map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic(".lode/time-log.json", raw).map(|_| ())
}

pub(crate) fn filter_time_sessions(
    sessions: Vec<TimeSession>,
    since: Option<&str>,
) -> Vec<TimeSession> {
    let Some(since) = since else {
        return sessions;
    };
    let key = resolve_since_key(since).unwrap_or_else(|| since.to_string());
    sessions
        .into_iter()
        .filter(|session| session.started_at.as_str() >= key.as_str())
        .collect()
}

pub(crate) fn resolve_since_key(value: &str) -> Option<String> {
    let days = value.strip_suffix('d')?.parse::<u64>().ok()?;
    let today = today_days_since_epoch();
    let target = today.saturating_sub(days);
    let (year, month, day) = civil_from_days(target as i64);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

pub(crate) fn print_time_sessions(
    label: &str,
    sessions: &[TimeSession],
    format: &str,
) -> lode_core::Result<()> {
    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(sessions)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
        }
        "markdown" | "md" => {
            print!("{}", render_time_sessions_markdown(label, sessions));
        }
        "table" => {
            println!("{label}\t{}", format_seconds(total_seconds(sessions)));
            for session in sessions {
                println!(
                    "{}\t{}\t{}",
                    session.started_at,
                    format_seconds(session.seconds),
                    session
                        .task
                        .as_deref()
                        .or(session.file.as_deref())
                        .or(session.project.as_deref())
                        .unwrap_or("-")
                );
            }
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported time output format: {other}"
            )))
        }
    }
    Ok(())
}

pub(crate) fn print_time_summary(
    sessions: &[TimeSession],
    by: &str,
    format: &str,
) -> lode_core::Result<()> {
    let mut groups = std::collections::BTreeMap::<String, u64>::new();
    for session in sessions {
        let key = match by {
            "day" => session.started_at.chars().take(10).collect::<String>(),
            "project" => session
                .project
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            "file" => session
                .file
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            "task" => session
                .task
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            other => {
                return Err(LodeError::Message(format!(
                    "unsupported time grouping: {other}"
                )))
            }
        };
        *groups.entry(key).or_insert(0) += session.seconds;
    }

    match format {
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&groups)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
        }
        "markdown" | "md" => {
            println!("# Time Summary\n");
            println!("| {by} | duration |");
            println!("| --- | --- |");
            for (key, seconds) in groups {
                println!("| {key} | {} |", format_seconds(seconds));
            }
        }
        "table" => {
            for (key, seconds) in groups {
                println!("{key}\t{}", format_seconds(seconds));
            }
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported time output format: {other}"
            )))
        }
    }
    Ok(())
}

pub(crate) fn render_time_report(
    sessions: &[TimeSession],
    format: &str,
) -> lode_core::Result<String> {
    match format {
        "json" => serde_json::to_string_pretty(&sessions)
            .map_err(|error| LodeError::Message(error.to_string())),
        "markdown" | "md" => Ok(render_time_sessions_markdown("time report", sessions)),
        "table" => {
            let mut output = format!("total\t{}\n", format_seconds(total_seconds(sessions)));
            for session in sessions {
                output.push_str(&format!(
                    "{}\t{}\t{}\n",
                    session.started_at,
                    format_seconds(session.seconds),
                    session
                        .task
                        .as_deref()
                        .or(session.file.as_deref())
                        .or(session.project.as_deref())
                        .unwrap_or("-")
                ));
            }
            Ok(output)
        }
        other => Err(LodeError::Message(format!(
            "unsupported time report format: {other}"
        ))),
    }
}

pub(crate) fn render_time_sessions_markdown(label: &str, sessions: &[TimeSession]) -> String {
    let mut output = format!(
        "# {label}\n\nTotal: {}\n\n",
        format_seconds(total_seconds(sessions))
    );
    output.push_str("| started | duration | context |\n");
    output.push_str("| --- | --- | --- |\n");
    for session in sessions {
        let context = session
            .task
            .as_deref()
            .or(session.file.as_deref())
            .or(session.project.as_deref())
            .unwrap_or("-");
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            session.started_at,
            format_seconds(session.seconds),
            context
        ));
    }
    output
}

pub(crate) fn total_seconds(sessions: &[TimeSession]) -> u64 {
    sessions.iter().map(|session| session.seconds).sum()
}

pub(crate) fn format_seconds(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

pub(crate) fn today_utc() -> String {
    let days = today_days_since_epoch() as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

pub(crate) fn now_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let day = seconds / 86_400;
    let second_of_day = seconds % 86_400;
    let (year, month, date) = civil_from_days(day as i64);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;
    format!("{year:04}-{month:02}-{date:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub(crate) fn today_days_since_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or_default()
}

pub(crate) fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}

pub(crate) fn detect_toolchains() -> Vec<String> {
    let mut detected = Vec::new();
    for (file, tool) in [
        ("rust-toolchain.toml", "rust"),
        ("Cargo.toml", "rust"),
        ("package.json", "node"),
        (".nvmrc", "node"),
        ("pyproject.toml", "python"),
        (".python-version", "python"),
        ("go.mod", "go"),
        ("build.zig", "zig"),
        ("build.gradle", "java/gradle"),
        ("pom.xml", "java/maven"),
    ] {
        if Utf8PathBuf::from(file).exists() {
            detected.push(format!("{tool}\t{file}"));
        }
    }
    detected
}

pub(crate) fn required_tools_for_project() -> Vec<&'static str> {
    let mut tools = vec!["git"];
    if Utf8PathBuf::from("Cargo.toml").exists() {
        tools.extend(["rustc", "cargo"]);
    }
    if Utf8PathBuf::from("package.json").exists() {
        tools.push("node");
    }
    if Utf8PathBuf::from("pyproject.toml").exists() {
        tools.push("python");
    }
    if Utf8PathBuf::from("go.mod").exists() {
        tools.push("go");
    }
    if Utf8PathBuf::from("build.zig").exists() {
        tools.push("zig");
    }
    tools
}

pub(crate) fn detect_package_manager() -> Option<String> {
    detect_package_manager_in(&Utf8PathBuf::from("."))
}

pub(crate) fn detect_package_manager_in(root: &Utf8PathBuf) -> Option<String> {
    if root.join("Cargo.lock").exists() || root.join("Cargo.toml").exists() {
        Some("cargo".to_string())
    } else if root.join("bun.lockb").exists() {
        Some("bun".to_string())
    } else if root.join("pnpm-lock.yaml").exists() {
        Some("pnpm".to_string())
    } else if root.join("yarn.lock").exists() {
        Some("yarn".to_string())
    } else if root.join("package-lock.json").exists() || root.join("package.json").exists() {
        Some("npm".to_string())
    } else if root.join("uv.lock").exists() || root.join("pyproject.toml").exists() {
        Some("uv".to_string())
    } else if root.join("requirements.txt").exists() {
        Some("pip".to_string())
    } else if root.join("go.sum").exists() || root.join("go.mod").exists() {
        Some("go".to_string())
    } else if root.join("Gemfile.lock").exists() || root.join("Gemfile").exists() {
        Some("bundler".to_string())
    } else if root.join("build.gradle").exists() || root.join("settings.gradle").exists() {
        Some("gradle".to_string())
    } else if root.join("pom.xml").exists() {
        Some("maven".to_string())
    } else {
        None
    }
}

pub(crate) fn command_version(command: &str) -> Option<String> {
    let output = run_process_output(command, &["--version".to_string()]).ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.lines().next().unwrap_or("installed").to_string())
}

pub(crate) fn collect_pack_files(
    root: &Utf8PathBuf,
    path: &Utf8PathBuf,
    pack: &mut LodePack,
) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            collect_pack_files(root, &child, pack)?;
        }
        return Ok(());
    }
    let Ok(relative) = path.strip_prefix(root) else {
        return Ok(());
    };
    let contents = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let checksum = content_hash_bytes(contents.as_bytes());
    pack.files.push(LodePackFile {
        path: relative.as_str().replace('\\', "/"),
        contents,
        checksum,
    });
    Ok(())
}

pub(crate) fn metrics_baseline_path(root: &Utf8PathBuf) -> Utf8PathBuf {
    root.join(".lode").join("metrics-baseline.json")
}

pub(crate) fn save_metrics_baseline(
    root: &Utf8PathBuf,
    report: &lode_core::AuditReport,
) -> lode_core::Result<()> {
    let validated_root = ValidatedRoot::new(root)?;
    validated_root.create_dir_all(".lode")?;
    let raw = serde_json::to_string_pretty(report)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    validated_root
        .write_atomic(".lode/metrics-baseline.json", raw)
        .map(|_| ())
}

pub(crate) fn load_metrics_baseline(
    root: &Utf8PathBuf,
) -> lode_core::Result<lode_core::AuditReport> {
    let path = metrics_baseline_path(root);
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub(crate) fn status_bool(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "missing"
    }
}

pub(crate) fn daemon_result(command: DaemonCommand) -> lode_core::Result<()> {
    match command {
        DaemonCommand::Start {
            no_rename,
            no_sign,
            no_stamp,
            foreground,
            no_env_drift: _no_env_drift,
            no_license_drift: _no_license_drift,
        } => {
            let state = format!(
                "active\nforeground={foreground}\nrename={}\nsign={}\nstamp={}\n",
                !no_rename, !no_sign, !no_stamp
            );
            write_daemon_state(&state)?;
            append_daemon_log(&format!(
                "daemon started foreground={foreground} rename={} sign={} stamp={}",
                !no_rename, !no_sign, !no_stamp
            ))?;
            println!("daemon started");
            if foreground {
                run_foreground_daemon(!no_rename, !no_sign, !no_stamp)?;
            }
        }
        DaemonCommand::Stop { project } => {
            write_daemon_state("inactive")?;
            append_daemon_log(&format!(
                "daemon stopped{}",
                project
                    .as_deref()
                    .map(|project| format!(" project={project}"))
                    .unwrap_or_default()
            ))?;
            println!("daemon stopped");
        }
        DaemonCommand::Restart => {
            write_daemon_state("active")?;
            append_daemon_log("daemon restarted")?;
            println!("daemon restarted");
        }
        DaemonCommand::Pause => {
            let mut runtime = load_daemon_runtime_state()?;
            if !runtime.active {
                return Err(LodeError::Message(
                    "daemon is not active; cannot pause".to_string(),
                ));
            }
            runtime.paused = true;
            runtime.updated_at = now_timestamp();
            runtime.uptime_s = daemon_uptime_seconds(&runtime);
            write_daemon_runtime_state(&runtime)?;
            write_daemon_state_text("active\npaused=true\n")?;
            append_daemon_log("daemon paused")?;
            println!("daemon paused");
        }
        DaemonCommand::Resume => {
            let mut runtime = load_daemon_runtime_state()?;
            if !runtime.active {
                return Err(LodeError::Message(
                    "daemon is not active; cannot resume".to_string(),
                ));
            }
            runtime.paused = false;
            runtime.updated_at = now_timestamp();
            runtime.uptime_s = daemon_uptime_seconds(&runtime);
            write_daemon_runtime_state(&runtime)?;
            write_daemon_state_text("active\npaused=false\n")?;
            append_daemon_log("daemon resumed")?;
            println!("daemon resumed");
        }
        DaemonCommand::ListWatchers { output } => {
            let runtime = load_daemon_runtime_state()?;
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "active": runtime.active,
                        "paused": runtime.paused,
                        "watchers": runtime.watchers,
                    }))
                    .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else if runtime.watchers.is_empty() {
                println!("no watchers active");
            } else {
                for watcher in runtime.watchers {
                    println!(
                        "{watcher}\t{}",
                        if runtime.paused { "paused" } else { "active" }
                    );
                }
            }
        }
        DaemonCommand::Status { quiet, output } => {
            let state =
                fs::read_to_string(daemon_state_path()?).unwrap_or_else(|_| "inactive".to_string());
            let runtime = load_daemon_runtime_state()?;
            let active = runtime.active;
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string(&runtime)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else if quiet {
                println!(
                    "{}",
                    if runtime.paused {
                        "paused"
                    } else if active {
                        "active"
                    } else {
                        "inactive"
                    }
                );
            } else {
                println!("daemon status: {}", state.trim());
                println!("uptime_s: {}", runtime.uptime_s);
                println!("events: {}", runtime.events);
                println!("paused: {}", status_bool(runtime.paused));
                println!("watchers: {}", runtime.watchers.join(","));
            }
        }
        DaemonCommand::Log { tail, follow } => {
            let path = daemon_log_path()?;
            let log = fs::read_to_string(&path).unwrap_or_else(|_| "no entries\n".to_string());
            print_log_lines(&log, tail);
            if follow {
                follow_daemon_log(&path, log.len())?;
            }
        }
    }
    Ok(())
}

pub(crate) fn print_log_lines(log: &str, tail: Option<usize>) {
    let mut lines = log.lines().collect::<Vec<_>>();
    if let Some(tail) = tail {
        let start = lines.len().saturating_sub(tail);
        lines = lines[start..].to_vec();
    }
    for line in lines {
        println!("{line}");
    }
}

pub(crate) fn follow_daemon_log(path: &Utf8PathBuf, mut offset: usize) -> lode_core::Result<()> {
    let interactive = io::stdin().is_terminal();
    let max_ticks = env::var("LODE_DAEMON_FOLLOW_TICKS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(if interactive { usize::MAX } else { 3 });
    let mut ticks = 0usize;
    while ticks < max_ticks {
        thread::sleep(Duration::from_millis(250));
        let current = fs::read_to_string(path).unwrap_or_default();
        if current.len() > offset {
            let appended = &current[offset..];
            for line in appended.lines() {
                println!("{line}");
            }
            offset = current.len();
        }
        ticks = ticks.saturating_add(1);
        if interactive
            && event::poll(Duration::from_millis(1)).map_err(terminal_error)?
            && matches!(
                event::read().map_err(terminal_error)?,
                Event::Key(key) if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            )
        {
            break;
        }
    }
    Ok(())
}

pub(crate) fn run_foreground_daemon(
    rename: bool,
    sign: bool,
    stamp: bool,
) -> lode_core::Result<()> {
    let root = current_dir()?;
    let mut snapshot = snapshot_project(&root)?;
    write_project_daemon_snapshot(&root, &snapshot)?;
    let once = env::var_os("LODE_DAEMON_ONCE").is_some() || !io::stdin().is_terminal();
    let interactive = io::stdin().is_terminal();

    if interactive {
        enable_raw_mode().map_err(terminal_error)?;
    }

    println!(
        "foreground daemon watching {} rename={} sign={} stamp={}{}",
        root,
        rename,
        sign,
        stamp,
        if interactive {
            " (press q to quit)"
        } else {
            ""
        }
    );

    loop {
        if interactive
            && event::poll(Duration::from_millis(50)).map_err(terminal_error)?
            && matches!(
                event::read().map_err(terminal_error)?,
                Event::Key(key) if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            )
        {
            break;
        }

        thread::sleep(Duration::from_millis(950));
        let next = snapshot_project(&root)?;
        let changes = daemon_changes(&snapshot, &next);
        if !changes.is_empty() {
            record_daemon_activity(&root, &changes)?;
            write_project_daemon_snapshot(&root, &next)?;
            println!(
                "daemon observed {} changed file(s): {}",
                changes.paths().len(),
                changes.summary()
            );
        }
        snapshot = next;

        if once {
            break;
        }
    }

    if interactive {
        disable_raw_mode().map_err(terminal_error)?;
    }
    append_daemon_log("foreground daemon exited")?;
    Ok(())
}

pub(crate) fn snapshot_project(root: &Utf8PathBuf) -> lode_core::Result<BTreeMap<String, u64>> {
    let mut snapshot = BTreeMap::new();
    snapshot_dir(root, root, &mut snapshot)?;
    Ok(snapshot)
}

pub(crate) fn snapshot_dir(
    root: &Utf8PathBuf,
    dir: &Utf8PathBuf,
    snapshot: &mut BTreeMap<String, u64>,
) -> lode_core::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(LodeError::Io {
                path: dir.as_str().into(),
                source,
            })
        }
    };

    for entry in entries {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|path| LodeError::Message(format!("non-utf8 path: {}", path.display())))?;
        let name = path.file_name().unwrap_or_default();
        if should_skip_watch_path(name) {
            continue;
        }
        let metadata = entry.metadata().map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        if metadata.is_dir() {
            snapshot_dir(root, &path, snapshot)?;
        } else if metadata.is_file() {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or_default();
            let relative = path
                .strip_prefix(root)
                .map(|path| path.as_str().replace('\\', "/"))
                .unwrap_or_else(|_| path.as_str().replace('\\', "/"));
            snapshot.insert(relative, modified);
        }
    }
    Ok(())
}

pub(crate) fn should_skip_watch_path(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | ".venv"
            | "dist"
            | "build"
            | ".lodepack"
            | "daemon-state.json"
    )
}

pub(crate) fn daemon_changes(
    before: &BTreeMap<String, u64>,
    after: &BTreeMap<String, u64>,
) -> DaemonChangeSet {
    let mut changes = DaemonChangeSet::default();
    for (path, modified) in after {
        match before.get(path) {
            None => changes.created.push(path.clone()),
            Some(previous) if previous != modified => changes.modified.push(path.clone()),
            _ => {}
        }
    }
    for path in before.keys() {
        if !after.contains_key(path) {
            changes.deleted.push(path.clone());
        }
    }
    changes
}

pub(crate) fn record_daemon_activity(
    root: &Utf8PathBuf,
    changes: &DaemonChangeSet,
) -> lode_core::Result<()> {
    let paths = changes.paths();
    append_daemon_event(
        "fs.batch",
        &format!(
            "changed files: {} ({})",
            paths.join(", "),
            changes.summary()
        ),
        paths.clone(),
    )?;
    let mut log = load_time_log()?;
    log.sessions.push(TimeSession {
        started_at: now_timestamp(),
        ended_at: None,
        seconds: 60,
        project: root.file_name().map(str::to_string),
        file: paths.first().cloned(),
        task: Some(format!("daemon activity: {}", changes.summary())),
    });
    save_time_log(&log)
}

pub(crate) fn write_project_daemon_snapshot(
    root: &Utf8PathBuf,
    snapshot: &BTreeMap<String, u64>,
) -> lode_core::Result<()> {
    let project_root = lode_core::ValidatedRoot::new(root)?;
    let mut files = BTreeMap::new();
    for (relative, modified_s) in snapshot {
        let file_path = root.join(relative);
        if let Ok(contents) = fs::read(&file_path) {
            files.insert(
                relative.clone(),
                ProjectDaemonFileState {
                    modified_s: *modified_s,
                    content_hash: content_hash_bytes(&contents),
                },
            );
        }
    }
    let state = ProjectDaemonState {
        schema_version: 3,
        project: root.file_name().map(str::to_string),
        updated_at: now_timestamp(),
        file_count: files.len(),
        files,
    };
    project_root.create_dir_all(".lode")?;
    let raw = serde_json::to_string_pretty(&state)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    project_root
        .write_atomic(".lode/daemon-state.json", raw)
        .map(|_| ())
}

pub(crate) fn content_hash_bytes(contents: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(contents);
    format!("{:064x}", hasher.finalize())
}

pub(crate) fn count_dir_entries(path: &Utf8PathBuf) -> lode_core::Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })? {
        entry.map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        count += 1;
    }
    Ok(count)
}

pub(crate) fn self_clean_targets() -> lode_core::Result<Vec<Utf8PathBuf>> {
    let root = global_dir()?;
    let logs = root.join("logs");
    Ok(vec![
        root.join("cache").join("upgrade"),
        daemon_state_path()?,
        daemon_runtime_state_path()?,
        logs.join("daemon.log.1"),
        logs.join("daemon.log.2"),
        logs.join("daemon.log.3"),
    ])
}

pub(crate) fn default_upgrade_manifest_path() -> Utf8PathBuf {
    global_dir()
        .map(|root| root.join("cache").join("upgrade").join("latest.json"))
        .unwrap_or_else(|_| Utf8PathBuf::from(".lode/cache/upgrade/latest.json"))
}

pub(crate) fn upgrade_state_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?
        .join("cache")
        .join("upgrade")
        .join("upgrade-state.json"))
}

pub(crate) fn read_upgrade_manifest(path: &Utf8PathBuf) -> lode_core::Result<UpgradeManifest> {
    let raw = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let manifest: UpgradeManifest = serde_json::from_str(&raw)
        .map_err(|error| LodeError::Message(format!("invalid upgrade manifest: {error}")))?;
    if manifest.schema_version != 3 {
        return Err(LodeError::Message(format!(
            "unsupported upgrade manifest schema: {}",
            manifest.schema_version
        )));
    }
    safe_relative_path(&manifest.binary)?;
    if manifest.checksum.trim().is_empty() {
        return Err(LodeError::Message(
            "upgrade manifest checksum is empty".to_string(),
        ));
    }
    Ok(manifest)
}

pub(crate) fn upgrade_candidate_path(
    manifest_path: &Utf8PathBuf,
    manifest: &UpgradeManifest,
) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&manifest.binary)?;
    Ok(manifest_path
        .parent()
        .map(|parent| parent.join(relative.clone()))
        .unwrap_or(relative))
}

pub(crate) fn file_checksum(path: &Utf8PathBuf) -> lode_core::Result<String> {
    let bytes = fs::read(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(content_hash_bytes(&bytes))
}

pub(crate) fn write_upgrade_state(state: &UpgradeState) -> lode_core::Result<()> {
    let root = daemon_global_root()?;
    root.create_dir_all("cache/upgrade")?;
    let raw = serde_json::to_string_pretty(state)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic("cache/upgrade/upgrade-state.json", raw)
        .map(|_| ())
}

pub(crate) fn read_upgrade_state() -> lode_core::Result<UpgradeState> {
    let path = upgrade_state_path()?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub(crate) fn rollback_staged_upgrade(dry_run: bool) -> lode_core::Result<()> {
    let state = read_upgrade_state()?;
    let path = upgrade_state_path()?;
    if dry_run {
        println!("would rollback staged upgrade {}", state.version);
        println!("would remove {path}");
        return Ok(());
    }
    daemon_global_root()?.remove_file("cache/upgrade/upgrade-state.json")?;
    println!("upgrade rollback cleared\t{}", state.version);
    Ok(())
}

pub(crate) fn completion_script(shell: &str) -> lode_core::Result<String> {
    let commands = command_words();
    let script = match shell {
        "bash" => format!(
            r#"# lode shell integration
_lode_chdir_hook() {{
  if [[ -f ".lode/project.toml" ]]; then
    lode daemon status --quiet 2>/dev/null || lode daemon start --foreground >/dev/null 2>&1 &
  fi
}}
case ";$PROMPT_COMMAND;" in
  *";_lode_chdir_hook;"*) ;;
  *) PROMPT_COMMAND="_lode_chdir_hook${{PROMPT_COMMAND:+; $PROMPT_COMMAND}}" ;;
esac
lp() {{ cd "$(lode projects cd "$1")"; }}
complete -W '{commands}' lode
"#
        ),
        "zsh" => format!(
            r#"#compdef lode
# lode shell integration
autoload -Uz add-zsh-hook
_lode_chdir_hook() {{
  if [[ -f ".lode/project.toml" ]]; then
    lode daemon status --quiet 2>/dev/null || lode daemon start --foreground >/dev/null 2>&1 &
  fi
}}
add-zsh-hook chpwd _lode_chdir_hook
lp() {{ cd "$(lode projects cd "$1")"; }}
_arguments '1: :(({commands}))'
"#
        ),
        "fish" => format!(
            r#"# lode shell integration
function _lode_hook --on-variable PWD
  if test -f .lode/project.toml
    lode daemon status --quiet 2>/dev/null; or lode daemon start --foreground >/dev/null 2>&1 &
  end
end
function lp
  cd (lode projects cd $argv[1])
end
{}
"#,
            commands
                .split_whitespace()
                .map(|command| format!("complete -c lode -f -a {command}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        "powershell" | "pwsh" => format!(
            r#"# lode shell integration
function Invoke-LodePromptHook {{
  if (Test-Path ".lode/project.toml") {{
    lode daemon status --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {{ Start-Process lode -ArgumentList @("daemon","start","--foreground") -WindowStyle Hidden }}
  }}
}}
function lp($Name) {{ Set-Location (lode projects cd $Name) }}
Register-ArgumentCompleter -Native -CommandName lode -ScriptBlock {{
  param($wordToComplete)
  '{commands}'.Split(' ') | Where-Object {{ $_ -like "$wordToComplete*" }}
}}
"#
        ),
        other => {
            return Err(LodeError::Message(format!(
                "unsupported completion shell: {other}"
            )))
        }
    };
    Ok(script)
}

pub(crate) fn command_words() -> String {
    let mut words = Cli::command()
        .get_subcommands()
        .flat_map(|command| {
            std::iter::once(command.get_name().to_string())
                .chain(command.get_all_aliases().map(str::to_string))
        })
        .collect::<Vec<_>>();
    words.sort();
    words.dedup();
    words.join(" ")
}

pub(crate) fn default_completion_path(shell: &str) -> lode_core::Result<Utf8PathBuf> {
    let file = match shell {
        "bash" => "lode.bash",
        "zsh" => "_lode",
        "fish" => "lode.fish",
        "powershell" | "pwsh" => "lode.ps1",
        other => {
            return Err(LodeError::Message(format!(
                "unsupported completion shell: {other}"
            )))
        }
    };
    Ok(global_dir()?.join("completions").join(file))
}

pub(crate) fn completion_source_line(shell: &str, path: &Utf8PathBuf) -> lode_core::Result<String> {
    let source = match shell {
        "bash" | "zsh" => format!("source \"{path}\""),
        "fish" => format!("source \"{path}\""),
        "powershell" | "pwsh" => format!(". \"{path}\""),
        other => {
            return Err(LodeError::Message(format!(
                "unsupported completion shell: {other}"
            )))
        }
    };
    Ok(source)
}

pub(crate) fn completion_install_hint(
    shell: &str,
    path: &Utf8PathBuf,
) -> lode_core::Result<String> {
    let hint = match shell {
        "bash" => format!("add to ~/.bashrc: source \"{path}\""),
        "zsh" => format!("copy or link into your fpath, or add to ~/.zshrc: source \"{path}\""),
        "fish" => format!(
            "copy to ~/.config/fish/completions/lode.fish or add to config.fish: source \"{path}\""
        ),
        "powershell" | "pwsh" => format!("add to $PROFILE: . \"{path}\""),
        other => {
            return Err(LodeError::Message(format!(
                "unsupported completion shell: {other}"
            )))
        }
    };
    Ok(hint)
}

pub(crate) fn write_validated_output(
    path: &Utf8PathBuf,
    contents: impl AsRef<[u8]>,
) -> lode_core::Result<()> {
    let path = if path.is_absolute() {
        path.clone()
    } else {
        current_dir()?.join(path)
    };
    let mut root_path = path.parent().ok_or_else(|| {
        LodeError::Message(format!("output path has no parent directory: {path}"))
    })?;
    while !root_path.exists() {
        root_path = root_path.parent().ok_or_else(|| {
            LodeError::Message(format!("output path has no existing ancestor: {path}"))
        })?;
    }
    let root = ValidatedRoot::new(root_path)?;
    let relative = path.strip_prefix(root_path).map_err(|_| {
        LodeError::Message(format!("output path is outside validated root: {path}"))
    })?;
    if let Some(parent) = relative.parent() {
        root.create_dir_all(parent)?;
    }
    root.write_atomic(relative, contents).map(|_| ())
}

pub(crate) fn write_completion_install_receipt(
    shell: &str,
    path: &Utf8PathBuf,
    source: &str,
    hint: &str,
) -> lode_core::Result<()> {
    let receipt = CompletionInstallReceipt {
        schema_version: 3,
        shell: shell.to_string(),
        path: path.to_string(),
        installed_at: now_timestamp(),
        source: source.to_string(),
        hint: hint.to_string(),
    };
    let raw = serde_json::to_string_pretty(&receipt)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    let root = daemon_global_root()?;
    root.create_dir_all("completions")?;
    root.write_atomic("completions/install-receipt.json", raw)
        .map(|_| ())
}

pub(crate) fn terminal_error(error: io::Error) -> LodeError {
    LodeError::Message(format!("terminal error: {error}"))
}

pub(crate) fn daemon_state_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("cache").join("daemon-state.txt"))
}

pub(crate) fn daemon_runtime_state_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("cache").join("daemon-state.json"))
}

pub(crate) fn daemon_log_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("logs").join("daemon.log"))
}

pub(crate) fn daemon_global_root() -> lode_core::Result<lode_core::ValidatedRoot> {
    lode_core::ensure_global_workspace()?;
    lode_core::ValidatedRoot::new(global_dir()?)
}

pub(crate) fn write_daemon_state(state: &str) -> lode_core::Result<()> {
    write_daemon_state_text(state)?;
    write_daemon_runtime_state(&runtime_state_from_text(state)?)
}

pub(crate) fn write_daemon_state_text(state: &str) -> lode_core::Result<()> {
    daemon_global_root()?
        .write_atomic("cache/daemon-state.txt", state)
        .map(|_| ())
}

pub(crate) fn append_daemon_log(line: &str) -> lode_core::Result<()> {
    append_daemon_event("lifecycle", line, Vec::new())
}

pub(crate) fn append_daemon_event(
    kind: &str,
    message: &str,
    files: Vec<String>,
) -> lode_core::Result<()> {
    let root = daemon_global_root()?;
    let path = root.resolve("logs/daemon.log")?;
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| LodeError::Io {
            path: path.clone(),
            source,
        })?;
    std::io::Write::write_all(&mut log, format!("{message}\n").as_bytes()).map_err(|source| {
        LodeError::Io {
            path: path.clone(),
            source,
        }
    })?;

    let mut state = load_daemon_runtime_state()?;
    state.events += 1;
    state.updated_at = now_timestamp();
    state.uptime_s = daemon_uptime_seconds(&state);
    state.recent_events.push(DaemonEvent {
        id: state.events,
        kind: kind.to_string(),
        message: message.to_string(),
        files,
        created_at: state.updated_at.clone(),
    });
    let overflow = state.recent_events.len().saturating_sub(50);
    if overflow > 0 {
        state.recent_events.drain(0..overflow);
    }
    write_daemon_runtime_state(&state)
}

pub(crate) fn runtime_state_from_text(state: &str) -> lode_core::Result<DaemonRuntimeState> {
    let mut runtime = load_daemon_runtime_state().unwrap_or_default();
    let now = now_timestamp();
    let active = state.lines().next().unwrap_or("inactive") == "active";
    if active && !runtime.active {
        runtime.started_at = now.clone();
        runtime.events = 0;
        runtime.recent_events.clear();
    }
    runtime.active = active;
    runtime.paused = state
        .lines()
        .find_map(|line| line.strip_prefix("paused="))
        .map(|value| value == "true")
        .unwrap_or(false);
    runtime.updated_at = now;
    runtime.uptime_s = daemon_uptime_seconds(&runtime);
    runtime.foreground = state
        .lines()
        .find_map(|line| line.strip_prefix("foreground="))
        .map(|value| value == "true")
        .unwrap_or(false);
    let rename = state
        .lines()
        .find_map(|line| line.strip_prefix("rename="))
        .map(|value| value == "true")
        .unwrap_or(active);
    let sign = state
        .lines()
        .find_map(|line| line.strip_prefix("sign="))
        .map(|value| value == "true")
        .unwrap_or(active);
    let stamp = state
        .lines()
        .find_map(|line| line.strip_prefix("stamp="))
        .map(|value| value == "true")
        .unwrap_or(active);
    runtime.watchers = [
        (rename, "rename"),
        (sign, "headers"),
        (stamp, "path_sync"),
        (active, "env_drift"),
    ]
    .into_iter()
    .filter_map(|(enabled, name)| enabled.then_some(name.to_string()))
    .collect();
    runtime.project = current_dir()
        .ok()
        .and_then(|path| path.file_name().map(str::to_string));
    Ok(runtime)
}

pub(crate) fn load_daemon_runtime_state() -> lode_core::Result<DaemonRuntimeState> {
    let path = daemon_runtime_state_path()?;
    if !path.exists() {
        return Ok(DaemonRuntimeState::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let mut state: DaemonRuntimeState =
        serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    state.uptime_s = daemon_uptime_seconds(&state);
    Ok(state)
}

pub(crate) fn write_daemon_runtime_state(state: &DaemonRuntimeState) -> lode_core::Result<()> {
    let raw = serde_json::to_string_pretty(state)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    daemon_global_root()?
        .write_atomic("cache/daemon-state.json", raw)
        .map(|_| ())
}

pub(crate) fn daemon_uptime_seconds(state: &DaemonRuntimeState) -> u64 {
    if !state.active {
        return 0;
    }
    parse_timestamp_seconds(&state.started_at)
        .map(|started| unix_seconds().saturating_sub(started))
        .unwrap_or_default()
}

pub(crate) fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub(crate) fn parse_timestamp_seconds(timestamp: &str) -> Option<u64> {
    let date = timestamp.get(0..10)?;
    let time = timestamp.get(11..19)?;
    let mut date_parts = date.split('-').map(|part| part.parse::<i64>().ok());
    let year = date_parts.next()??;
    let month = date_parts.next()??;
    let day = date_parts.next()??;
    let mut time_parts = time.split(':').map(|part| part.parse::<u64>().ok());
    let hour = time_parts.next()??;
    let minute = time_parts.next()??;
    let second = time_parts.next()??;
    let days = days_from_civil(year, month, day)?;
    Some(days as u64 * 86_400 + hour * 3_600 + minute * 60 + second)
}

pub(crate) fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

pub(crate) fn list_make_targets() -> lode_core::Result<()> {
    let path = Utf8PathBuf::from("Makefile");
    if !path.exists() {
        for target in [
            "dev", "build", "test", "fmt", "lint", "check", "verify", "clean", "docs", "install",
            "update", "release",
        ] {
            println!("{target}");
        }
        return Ok(());
    }
    let contents = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    for line in contents.lines() {
        if let Some((target, _)) = line.split_once(':') {
            if !target.trim().is_empty()
                && target
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
            {
                println!("{}", target.trim());
            }
        }
    }
    Ok(())
}

pub(crate) fn gha_command(command: &str, name: Option<&str>) -> lode_core::Result<()> {
    match command {
        "validate" => {
            let dir = Utf8PathBuf::from(".github").join("workflows");
            if !dir.exists() {
                return Err(LodeError::Message(
                    "no GitHub workflow directory found".to_string(),
                ));
            }
            let mut count = 0usize;
            for entry in fs::read_dir(&dir).map_err(|source| LodeError::Io {
                path: dir.as_str().into(),
                source,
            })? {
                let entry = entry.map_err(|source| LodeError::Io {
                    path: dir.as_str().into(),
                    source,
                })?;
                let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                    LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
                })?;
                if matches!(path.extension(), Some("yml" | "yaml")) {
                    let contents = fs::read_to_string(&path).map_err(|source| LodeError::Io {
                        path: path.as_str().into(),
                        source,
                    })?;
                    if !contents.contains("jobs:") {
                        return Err(LodeError::Message(format!("workflow missing jobs: {path}")));
                    }
                    count += 1;
                }
            }
            println!("validated {count} workflow(s)");
        }
        "add" => {
            let name = name.unwrap_or("ci-rust");
            let relative = Utf8PathBuf::from(".github")
                .join("workflows")
                .join(format!("{name}.yml"));
            let root = ValidatedRoot::new(current_dir()?)?;
            root.create_dir_all(".github/workflows")?;
            root.write_atomic(relative, workflow_contents(name))?;
            println!("added workflow {name}");
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported gha command: {other}"
            )))
        }
    }
    Ok(())
}

fn workflow_contents(name: &str) -> String {
    let run = if name.contains("node") {
        "npm ci\n      - run: npm test"
    } else if name.contains("tauri") {
        "npm ci\n      - run: npm run build"
    } else if name.contains("minecraft") {
        "./gradlew build"
    } else {
        "cargo test --workspace"
    };
    format!(
        "name: {name}\non: [push, pull_request]\njobs:\n  verify:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: {run}\n"
    )
}

pub(crate) fn tauri_command(command: &str) -> lode_core::Result<()> {
    match command {
        "doctor" => {
            println!(
                "src-tauri\t{}",
                status_bool(Utf8PathBuf::from("src-tauri").exists())
            );
            println!(
                "package.json\t{}",
                status_bool(Utf8PathBuf::from("package.json").exists())
            );
        }
        "dev" | "build" => run_make(&format!("tauri-{command}"))?,
        other => {
            return Err(LodeError::Message(format!(
                "unsupported tauri command: {other}"
            )))
        }
    }
    Ok(())
}

pub(crate) fn mc_command(command: &str) -> lode_core::Result<()> {
    match command {
        "run-client" | "run-server" | "build" => run_make(&format!("mc-{command}"))?,
        "doctor" => {
            println!(
                "gradle\t{}",
                status_bool(
                    Utf8PathBuf::from("build.gradle").exists()
                        || Utf8PathBuf::from("settings.gradle").exists()
                )
            );
            println!(
                "minecraft sources\t{}",
                status_bool(Utf8PathBuf::from("src/main").exists())
            );
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported mc command: {other}"
            )))
        }
    }
    Ok(())
}

pub(crate) fn cp_command(
    command: &str,
    problem: Option<&str>,
    lang: Option<&str>,
) -> lode_core::Result<()> {
    match command {
        "new" => {
            let problem = problem.unwrap_or("a");
            let lang = lang.unwrap_or("cpp");
            let ext = match lang {
                "rs" | "rust" => "rs",
                "py" | "python" => "py",
                "java" => "java",
                _ => "cpp",
            };
            let relative = Utf8PathBuf::from("problems")
                .join(problem)
                .join(format!("main.{ext}"));
            let root = ValidatedRoot::new(current_dir()?)?;
            let parent = relative
                .parent()
                .ok_or_else(|| LodeError::Message("problem file path has no parent".to_string()))?;
            root.create_dir_all(parent)?;
            root.write_atomic(relative, cp_template(ext))?;
            println!("created competitive problem {problem}");
        }
        "run" | "test" | "stress" => {
            let problem = problem.unwrap_or("a");
            println!("competitive coding {command} {problem}");
        }
        "archive" => {
            let contest = problem.unwrap_or("contest");
            ValidatedRoot::new(current_dir()?)?
                .create_dir_all(Utf8PathBuf::from("archive").join(contest))?;
            println!("archived contest {contest}");
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported cp command: {other}"
            )))
        }
    }
    Ok(())
}

fn cp_template(ext: &str) -> &'static str {
    match ext {
        "rs" => "fn main() {\n    println!(\"hello\");\n}\n",
        "py" => "def main():\n    pass\n\nif __name__ == \"__main__\":\n    main()\n",
        "java" => "class Main {\n    public static void main(String[] args) {}\n}\n",
        _ => "#include <bits/stdc++.h>\nusing namespace std;\nint main(){ios::sync_with_stdio(false);cin.tie(nullptr);}\n",
    }
}

pub(crate) fn verify_command(changed: bool, output: OutputFormat) -> lode_core::Result<()> {
    if changed {
        return verify_changed(output);
    }
    run_make("verify")
}

fn verify_changed(output: OutputFormat) -> lode_core::Result<()> {
    let dir = current_dir()?;
    let manifest_path = lode_core::file_manifest_path(&dir);

    if !manifest_path.exists() {
        println!(
            "{}",
            output::dim(
                "No file manifest found. Run `lode file add` or `lode agent policy` first."
            )
        );
        return Ok(());
    }

    let results = check_file_integrity(&dir)?;
    let files = list_managed_files(&dir)?;

    let ok_count = results.iter().filter(|r| r.status == "ok").count();
    let modified_count = results.iter().filter(|r| r.status == "modified").count();
    let missing_count = results.iter().filter(|r| r.status == "missing").count();
    let untracked_count = results.iter().filter(|r| r.status == "not_tracked").count();
    let total = results.len();

    if output.should_use_json() {
        let report = serde_json::json!({
            "total_files": total,
            "ok": ok_count,
            "modified": modified_count,
            "missing": missing_count,
            "not_tracked": untracked_count,
            "results": results,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("{}", output::bold("Change-Aware Verification"));
        println!("  {} {} managed files", output::cyan("ℹ"), files.len());

        if results.is_empty() {
            println!("\n  {} No files to check.", output::dim("~"));
            return Ok(());
        }

        let mut has_issues = false;
        for result in &results {
            match result.status.as_str() {
                "ok" => {
                    println!(
                        "  {}  {}  {}",
                        output::green("✔"),
                        result.path,
                        output::dim("unchanged")
                    );
                }
                "modified" => {
                    has_issues = true;
                    println!(
                        "  {}  {}  {}",
                        output::yellow("⚠"),
                        result.path,
                        output::yellow("MODIFIED")
                    );
                }
                "missing" => {
                    has_issues = true;
                    println!(
                        "  {}  {}  {}",
                        output::red("✘"),
                        result.path,
                        output::red("MISSING")
                    );
                }
                "not_tracked" => {
                    println!(
                        "  {}  {}  {}",
                        output::cyan("ℹ"),
                        result.path,
                        output::dim("(not tracked)")
                    );
                }
                _ => {
                    has_issues = true;
                    println!("  {}  {}  {}", output::red("?"), result.path, result.status);
                }
            }
        }

        println!();
        println!("  {}  {} ok", output::green("✔"), ok_count);
        if modified_count > 0 {
            println!("  {}  {} modified", output::yellow("⚠"), modified_count);
        }
        if missing_count > 0 {
            println!("  {}  {} missing", output::red("✘"), missing_count);
        }
        if untracked_count > 0 {
            println!("  {}  {} not tracked", output::cyan("ℹ"), untracked_count);
        }

        if has_issues {
            println!(
                "\n  {} Use `lode file check` for detailed integrity checks",
                output::dim("→")
            );
        }
    }
    Ok(())
}

pub(crate) fn run_make(target: &str) -> lode_core::Result<()> {
    if !Utf8PathBuf::from("Makefile").exists() {
        println!("make target `{target}` requested, but no Makefile exists here");
        return Ok(());
    }
    let status = run_process_status("make", &[target.to_string()], None)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "make {target} failed with {status}"
        )))
    }
}

pub(crate) fn list_dir(path: Utf8PathBuf) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        println!("{}", entry.path().display());
    }
    Ok(())
}

pub(crate) fn collect_file_names(
    path: &Utf8PathBuf,
    items: &mut Vec<String>,
) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if child.is_dir() {
            collect_file_names(&child, items)?;
        } else if let Some(name) = child.file_name() {
            items.push(name.to_string());
        }
    }
    items.sort();
    Ok(())
}

pub(crate) fn current_dir() -> lode_core::Result<Utf8PathBuf> {
    let path = env::current_dir().map_err(|source| LodeError::Io {
        path: ".".into(),
        source,
    })?;
    Utf8PathBuf::from_path_buf(path)
        .map_err(|path| LodeError::Message(format!("path is not valid UTF-8: {}", path.display())))
}
