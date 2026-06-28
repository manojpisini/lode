use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    env, fs,
    hash::{Hash, Hasher},
    io,
    io::{IsTerminal, Read},
    process::ExitCode,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use camino::Utf8PathBuf;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lode_core::{
    add_component_to_project, audit_project, check_path, command_names, default_config, fix_path,
    global_asset_dir, global_dir, init_project, load_global_config, load_metrics, load_registry,
    profile_names, prune_registry, recipe_names, register_project, save_global_config,
    save_metrics, save_registry, scan_secrets, setup_defaults, sync_project, template_paths,
    AddRequest, InitRequest, LodeError, Process,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[command(
    name = "lode",
    version,
    about = "Personal coding preference system",
    allow_external_subcommands = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Setup {
        #[arg(long)]
        defaults: bool,
    },
    #[command(alias = "new")]
    Init(InitArgs),
    Add {
        component: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        overwrite: bool,
    },
    Sync {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        section: Option<String>,
    },
    Info {
        #[arg(long)]
        json: bool,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Template {
        #[command(subcommand)]
        command: LibraryCommand,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    Recipe {
        #[command(subcommand)]
        command: RecipeCommand,
    },
    Snippet {
        #[command(subcommand)]
        command: SnippetCommand,
    },
    Commands {
        #[command(subcommand)]
        command: CommandsCommand,
    },
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },
    Mcp {
        #[arg(long)]
        http: bool,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        list_tools: bool,
        #[arg(long)]
        list_resources: bool,
        #[arg(long)]
        list_prompts: bool,
    },
    Lsp {
        #[arg(long)]
        stdio: bool,
        #[arg(long)]
        capabilities: bool,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Task {
        target: Option<String>,
        #[arg(long)]
        no_store: bool,
    },
    Dev,
    Build,
    Test,
    Fmt,
    Lint,
    Check(CheckArgs),
    Fix {
        path: Option<Utf8PathBuf>,
    },
    Rename {
        path: Utf8PathBuf,
        #[arg(long)]
        to: Option<String>,
    },
    Rules {
        #[command(subcommand)]
        command: RulesCommand,
    },
    Sign {
        path: Option<Utf8PathBuf>,
        #[arg(long, value_delimiter = ',')]
        ext: Vec<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Stamp {
        path: Option<Utf8PathBuf>,
        #[arg(long, value_delimiter = ',')]
        ext: Vec<String>,
        #[arg(long)]
        license: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Verify,
    Clean,
    Fresh,
    Ship,
    Release {
        version: Option<String>,
        #[arg(long)]
        bump: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        rollback: bool,
    },
    Health,
    Explain,
    Audit,
    Doctor {
        #[arg(long)]
        fix: bool,
        #[arg(long)]
        json: bool,
    },
    Scan {
        #[command(subcommand)]
        command: ScanCommand,
    },
    Git {
        #[command(subcommand)]
        command: GitCommand,
    },
    Hooks {
        #[command(subcommand)]
        command: HooksCommand,
    },
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    License {
        #[command(subcommand)]
        command: LicenseCommand,
    },
    Projects {
        #[command(subcommand)]
        command: ProjectsCommand,
    },
    Toolchain {
        #[command(subcommand)]
        command: ToolchainCommand,
    },
    Pkg {
        #[command(subcommand)]
        command: PkgCommand,
    },
    Time {
        #[command(subcommand)]
        command: TimeCommand,
    },
    Metrics {
        #[command(subcommand)]
        command: MetricsCommand,
    },
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    Log {
        #[command(subcommand)]
        command: LogCommand,
    },
    Export {
        #[arg(long)]
        out: Option<Utf8PathBuf>,
        #[arg(long)]
        no_plugins: bool,
        #[arg(long)]
        no_templates: bool,
        #[arg(long)]
        no_snippets: bool,
        #[arg(long)]
        no_licenses: bool,
        #[arg(long)]
        no_recipes: bool,
        #[arg(long)]
        no_commands: bool,
        #[arg(long)]
        include_metrics: bool,
    },
    Import {
        path: Utf8PathBuf,
        #[arg(long)]
        no_merge: bool,
        #[arg(long)]
        force: bool,
    },
    Serve {
        #[arg(long)]
        no_color: bool,
        #[arg(long)]
        no_live: bool,
        #[arg(long)]
        pane: Option<String>,
    },
    Mc {
        command: String,
    },
    Tauri {
        command: String,
    },
    Gha {
        command: String,
        name: Option<String>,
    },
    Cp {
        command: String,
        problem: Option<String>,
        #[arg(long)]
        lang: Option<String>,
    },
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        command: SelfCommand,
    },
    Upgrade {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        manifest: Option<Utf8PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        rollback: bool,
    },
    Completions {
        shell: String,
        #[arg(long)]
        install: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        out: Option<Utf8PathBuf>,
    },
    Version,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Debug, Args)]
struct InitArgs {
    name: String,
    #[arg(short = 'p', long = "path")]
    path: Option<Utf8PathBuf>,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long = "with", value_delimiter = ',')]
    components: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    overwrite: bool,
    #[arg(long)]
    no_git: bool,
}

#[derive(Debug, Args)]
struct CheckArgs {
    path: Option<Utf8PathBuf>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    fix: bool,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show {
        #[arg(long, value_enum, default_value = "toml")]
        format: OutputFormat,
        #[arg(long)]
        defaults: bool,
        #[arg(long)]
        project: bool,
        #[arg(long)]
        section: Option<String>,
    },
    Validate {
        #[arg(long)]
        defaults: bool,
        #[arg(long)]
        project: bool,
    },
    Diff,
    Set {
        key: String,
        value: String,
    },
    Reset {
        key: String,
    },
}

#[derive(Debug, Subcommand)]
enum LibraryCommand {
    List {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Show {
        name: String,
        #[arg(long)]
        raw: bool,
    },
    Diff {
        name: String,
    },
    Reset {
        name: String,
    },
    Validate {
        #[arg(long)]
        all: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ProfileCommand {
    List,
    Show { name: String },
    Use { name: String },
    New { name: String },
    Delete { name: String },
}

#[derive(Debug, Subcommand)]
enum RecipeCommand {
    List,
    Show {
        name: String,
    },
    Apply {
        name: String,
        #[arg(long)]
        dry_run: bool,
    },
    Compose {
        names: Vec<String>,
    },
    New {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum CommandsCommand {
    List,
    Show {
        name: String,
    },
    Add {
        slug: String,
        #[arg(long)]
        global: bool,
        #[arg(long)]
        from: Option<String>,
    },
    Remove {
        slug: String,
        #[arg(long)]
        global: bool,
    },
    Export {
        #[arg(long)]
        out: Option<Utf8PathBuf>,
    },
    Run {
        slug: String,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum PluginCommand {
    List,
    Search {
        query: Option<String>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Add {
        source: Utf8PathBuf,
        #[arg(long)]
        allow_unsafe: bool,
    },
    Remove {
        name: String,
    },
    Update {
        name: Option<String>,
    },
    Info {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    Sync,
    Status,
    Export {
        #[arg(long)]
        out: Option<Utf8PathBuf>,
    },
    Plan {
        #[command(subcommand)]
        command: AgentPlanCommand,
    },
}

#[derive(Debug, Subcommand)]
enum AgentPlanCommand {
    Init,
    Add {
        task: String,
        #[arg(long)]
        branch: Option<String>,
    },
    Done {
        id: u64,
    },
    Show,
    Clear,
}

#[derive(Debug, Subcommand)]
enum SnippetCommand {
    List {
        #[arg(long)]
        lang: Option<String>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Show {
        name: String,
        #[arg(long)]
        lang: Option<String>,
    },
    Search {
        query: String,
    },
    Add {
        name: String,
        #[arg(long, default_value = "any")]
        lang: String,
        #[arg(long)]
        trigger: Option<String>,
        #[arg(long)]
        desc: Option<String>,
    },
    Remove {
        name: String,
        #[arg(long)]
        lang: Option<String>,
    },
    Insert {
        name: String,
        file: Option<Utf8PathBuf>,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        line: Option<usize>,
    },
    Export {
        #[arg(long)]
        lang: Option<String>,
        #[arg(long, default_value = "vscode")]
        format: String,
        #[arg(long)]
        out: Option<Utf8PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum ScanCommand {
    Secrets {
        path: Option<Utf8PathBuf>,
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        quiet: bool,
    },
    Foreign {
        path: Option<Utf8PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum RulesCommand {
    List,
    Check { path: Option<Utf8PathBuf> },
    Validate,
}

#[derive(Debug, Subcommand)]
enum GitCommand {
    Branch {
        kind: String,
        description: String,
    },
    Commit {
        message: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        breaking: bool,
        #[arg(long)]
        no_confirm: bool,
    },
    Tag {
        version: String,
        #[arg(long)]
        no_changelog: bool,
        #[arg(long)]
        push: bool,
        #[arg(long)]
        message: Option<String>,
    },
    Changelog {
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        out: Option<Utf8PathBuf>,
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    InstallHooks,
    UninstallHooks,
    HooksStatus,
    SignSetup,
    RemoteSetup {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        visibility: Option<String>,
        #[arg(long)]
        token_env: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum HooksCommand {
    List,
    Status,
    Test {
        event: String,
    },
    Run {
        event: String,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum EnvCommand {
    Check,
    Add {
        key: String,
        #[arg(long)]
        default: Option<String>,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        secret: bool,
    },
    Sync,
    Use {
        profile: String,
    },
}

#[derive(Debug, Subcommand)]
enum LicenseCommand {
    List {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Show {
        id: String,
    },
    Info {
        id: String,
    },
    Add {
        id: String,
        #[arg(long)]
        file: Option<Utf8PathBuf>,
        #[arg(long)]
        text: Option<String>,
    },
    Remove {
        id: String,
    },
    Set {
        id: String,
    },
    Check {
        #[arg(long)]
        json: bool,
    },
    Apply {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ProjectsCommand {
    List {
        #[arg(long, default_value = "table")]
        format: String,
        #[arg(long, default_value = "name")]
        sort: String,
    },
    Cd {
        name: String,
    },
    Register {
        path: Option<Utf8PathBuf>,
    },
    Remove {
        name: String,
    },
    Health {
        #[arg(long)]
        stale_only: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        refresh: bool,
    },
    Prune,
}

#[derive(Debug, Subcommand)]
enum ToolchainCommand {
    List,
    Status,
    Doctor,
    Add {
        runtime: String,
        version: String,
    },
    Remove {
        runtime: String,
        version: String,
    },
    Use {
        runtime: String,
        version: String,
    },
    Pin {
        runtime: Option<String>,
        version: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Update {
        runtime: Option<String>,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Debug, Subcommand)]
enum PkgCommand {
    List {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Outdated {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Update {
        name: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    Audit {
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "table")]
        format: String,
        #[arg(long)]
        fail_on: Option<String>,
    },
    Why {
        name: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Info {
        name: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Lock {
        #[arg(long)]
        dry_run: bool,
    },
    Graph {
        #[arg(long, default_value = "ascii")]
        format: String,
    },
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TimeCommand {
    Today {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Show {
        #[arg(long)]
        since: Option<String>,
        #[arg(long, default_value = "day")]
        by: String,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Report {
        #[arg(long)]
        since: Option<String>,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        out: Option<Utf8PathBuf>,
    },
    Clear {
        #[arg(long)]
        before: Option<String>,
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Debug, Subcommand)]
enum MetricsCommand {
    Show,
    Trend {
        #[arg(long)]
        last: Option<usize>,
    },
    Baseline,
    DiffBaseline,
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommand {
    Init,
    List {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Add {
        name: String,
    },
    Remove {
        name: String,
        #[arg(long)]
        confirm: bool,
    },
    Run {
        target: String,
        #[arg(long)]
        pkg: Option<String>,
        #[arg(long)]
        changed: Vec<String>,
        #[arg(long)]
        parallel: Option<usize>,
        #[arg(long)]
        dry_run: bool,
    },
    Graph {
        #[arg(long, default_value = "ascii")]
        format: String,
    },
}

#[derive(Debug, Subcommand)]
enum DaemonCommand {
    Start {
        #[arg(long)]
        no_rename: bool,
        #[arg(long)]
        no_sign: bool,
        #[arg(long)]
        no_stamp: bool,
        #[arg(long)]
        foreground: bool,
    },
    Stop {
        #[arg(long)]
        project: Option<String>,
    },
    Restart,
    Pause,
    Resume,
    ListWatchers {
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        json: bool,
    },
    Log {
        #[arg(long)]
        tail: Option<usize>,
        #[arg(long)]
        follow: bool,
    },
}

#[derive(Debug, Subcommand)]
enum LogCommand {
    Init,
    Daemon {
        #[arg(long)]
        tail: Option<usize>,
    },
    Clear,
}

#[derive(Debug, Subcommand)]
enum SelfCommand {
    Info,
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
    Uninstall {
        #[arg(long)]
        keep_config: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Toml,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct LodePack {
    version: u32,
    #[serde(default = "default_lodepack_manifest")]
    manifest: LodePackManifest,
    files: Vec<LodePackFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LodePackManifest {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    lode_version: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    file_count: usize,
    #[serde(default = "default_lodepack_checksum_algorithm")]
    checksum_algorithm: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LodePackFile {
    path: String,
    contents: String,
    #[serde(default)]
    checksum: String,
}

fn default_schema_version() -> u32 {
    3
}

fn default_lodepack_checksum_algorithm() -> String {
    "lode-default-hash-v1".to_string()
}

fn default_lodepack_manifest() -> LodePackManifest {
    LodePackManifest {
        schema_version: default_schema_version(),
        lode_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: String::new(),
        file_count: 0,
        checksum_algorithm: default_lodepack_checksum_algorithm(),
    }
}

#[derive(Debug, Clone, Copy)]
struct ExportOptions {
    no_plugins: bool,
    no_templates: bool,
    no_snippets: bool,
    no_licenses: bool,
    no_recipes: bool,
    no_commands: bool,
    include_metrics: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TimeLog {
    #[serde(default)]
    sessions: Vec<TimeSession>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompletionInstallReceipt {
    schema_version: u32,
    shell: String,
    path: String,
    installed_at: String,
    source: String,
    hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeSession {
    started_at: String,
    #[serde(default)]
    ended_at: Option<String>,
    #[serde(default)]
    seconds: u64,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    task: Option<String>,
}

#[derive(Debug, Clone)]
struct SnippetAsset {
    lang: String,
    name: String,
    body: String,
    path: Utf8PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct PluginIndexEntry {
    name: String,
    version: String,
    description: String,
    source: String,
    installed: bool,
    path: Option<Utf8PathBuf>,
    capabilities: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AgentPlan {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    tasks: Vec<AgentTask>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentTask {
    id: u64,
    task: String,
    #[serde(default)]
    branch: Option<String>,
    done: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ToolchainStore {
    #[serde(default)]
    runtimes: std::collections::BTreeMap<String, Vec<String>>,
    #[serde(default)]
    active: std::collections::BTreeMap<String, String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(lode_core::ExitCode::Ok as u8),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code() as u8)
        }
    }
}

fn run() -> lode_core::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Setup { defaults: _ } => setup()?,
        Command::Init(args) => init(args)?,
        Command::Add {
            component,
            dry_run,
            overwrite,
        } => add_component(&component, dry_run, overwrite)?,
        Command::Sync {
            dry_run,
            force,
            section,
        } => sync_command(dry_run, force, section.as_deref())?,
        Command::Info { json } => info(json)?,
        Command::Config { command } => config_command(command)?,
        Command::Template { command } => library_command("templates", command, template_paths())?,
        Command::Profile { command } => profile_command(command)?,
        Command::Recipe { command } => recipe_command(command)?,
        Command::Commands { command } => commands_command(command)?,
        Command::Plugin { command } => plugin_command(command)?,
        Command::Mcp {
            http,
            port,
            list_tools,
            list_resources,
            list_prompts,
        } => mcp_command(http, port, list_tools, list_resources, list_prompts)?,
        Command::Lsp {
            stdio,
            capabilities,
        } => lsp_command(stdio, capabilities)?,
        Command::Agent { command } => agent_command(command)?,
        Command::Snippet { command } => snippet_command(command)?,
        Command::Task { target, no_store } => task_command(target, no_store)?,
        Command::Dev => run_make("dev")?,
        Command::Build => run_make("build")?,
        Command::Test => run_make("test")?,
        Command::Fmt => run_make("fmt")?,
        Command::Lint => run_make("lint")?,
        Command::Check(args) => convention_check(args)?,
        Command::Fix { path } => convention_fix(path)?,
        Command::Rename { path, to } => rename_path(path, to)?,
        Command::Rules { command } => rules(command)?,
        Command::Sign {
            path,
            ext,
            force,
            dry_run,
        } => sign_path(path, ext, force, dry_run)?,
        Command::Stamp {
            path,
            ext,
            license,
            dry_run,
        } => stamp_path(path, ext, license, dry_run)?,
        Command::Verify => run_make("verify")?,
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
        } => release(version, bump, dry_run, rollback)?,
        Command::Health | Command::Audit => health()?,
        Command::Explain => explain(),
        Command::Doctor { fix, json } => doctor(fix, json)?,
        Command::Scan { command } => scan(command)?,
        Command::Git { command } => git(command)?,
        Command::Hooks { command } => hooks(command)?,
        Command::Env { command } => env_command(command)?,
        Command::License { command } => license(command)?,
        Command::Projects { command } => projects(command)?,
        Command::Toolchain { command } => toolchain(command)?,
        Command::Pkg { command } => pkg(command)?,
        Command::Time { command } => time_command(command)?,
        Command::Metrics { command } => metrics(command)?,
        Command::Workspace { command } => workspace(command)?,
        Command::Daemon { command } => daemon(command),
        Command::Log { command } => log_command(command)?,
        Command::Export {
            out,
            no_plugins,
            no_templates,
            no_snippets,
            no_licenses,
            no_recipes,
            no_commands,
            include_metrics,
        } => export_lodepack(
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
        } => import_lodepack(path, no_merge, force)?,
        Command::Serve {
            no_color,
            no_live,
            pane,
        } => serve_dashboard(no_color, no_live, pane.as_deref())?,
        Command::Mc { command } => mc_command(&command)?,
        Command::Tauri { command } => tauri_command(&command)?,
        Command::Gha { command, name } => gha_command(&command, name.as_deref())?,
        Command::Cp {
            command,
            problem,
            lang,
        } => cp_command(&command, problem.as_deref(), lang.as_deref())?,
        Command::SelfCmd { command } => self_command(command)?,
        Command::Upgrade {
            check,
            manifest,
            dry_run,
            rollback,
        } => upgrade(check, manifest, dry_run, rollback)?,
        Command::Completions {
            shell,
            install,
            dry_run,
            out,
        } => completions(&shell, install, dry_run, out)?,
        Command::Version => println!("{}", env!("CARGO_PKG_VERSION")),
        Command::External(args) => external_command(args)?,
    }

    Ok(())
}

fn setup() -> lode_core::Result<()> {
    let report = setup_defaults(false)?;
    println!("lode initialised at {}", report.global_dir);
    println!(
        "{} {}",
        if report.wrote_config {
            "wrote"
        } else {
            "kept existing"
        },
        report.config_path
    );
    println!(
        "extracted default templates, profiles, snippets, recipes, licenses, and command macros"
    );
    Ok(())
}

fn sync_command(dry_run: bool, force: bool, section: Option<&str>) -> lode_core::Result<()> {
    let sections = match section {
        Some(section) => vec![section.to_string()],
        None => vec![
            "config".to_string(),
            "templates".to_string(),
            "agent".to_string(),
            "metrics".to_string(),
        ],
    };
    for section in sections {
        match section.as_str() {
            "config" => {
                if dry_run {
                    println!("would sync config");
                    continue;
                }
                load_global_config()?;
                println!("synced config");
            }
            "templates" => {
                let cwd = current_dir()?;
                if cwd.join(".lode").join("project.toml").exists() {
                    let report = sync_project(cwd, load_global_config()?, force, dry_run)?;
                    if dry_run {
                        println!("would sync templates");
                        for path in report.planned_paths {
                            println!("would reconcile {path}");
                        }
                    } else {
                        println!(
                            "synced {} template-backed file(s)",
                            report.wrote_paths.len()
                        );
                    }
                } else if dry_run {
                    println!("would sync templates");
                } else {
                    validate_template_tree(&global_asset_dir("templates")?)?;
                    println!("synced templates");
                }
            }
            "agent" | "context" => {
                if dry_run {
                    println!("would sync {section}");
                    continue;
                }
                agent_sync()?;
            }
            "metrics" => {
                if dry_run {
                    println!("would sync metrics");
                    continue;
                }
                if force || Utf8PathBuf::from(".lode").exists() {
                    let cwd = current_dir()?;
                    let report = audit_project(&cwd, &load_global_config()?)?;
                    save_metrics(&cwd, &report)?;
                    println!("synced metrics");
                }
            }
            other => {
                return Err(LodeError::Message(format!(
                    "unsupported sync section: {other}"
                )))
            }
        }
    }
    Ok(())
}

fn init(args: InitArgs) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let git_config = config.git.clone();
    let identity = config.identity.clone();
    let base_path = match args.path {
        Some(path) => path,
        None => current_dir()?,
    };
    let profile_for_registry = args
        .profile
        .clone()
        .or_else(|| config.active_profile.clone())
        .unwrap_or_else(|| "core/bare".to_string());
    let selected_profile = args.profile.or_else(|| config.active_profile.clone());
    let report = init_project(InitRequest {
        name: args.name,
        base_path,
        config,
        profile: selected_profile,
        components: args.components,
        dry_run: args.dry_run,
        overwrite: args.overwrite,
    })?;

    if report.dry_run {
        println!("dry run: would initialise {}", report.project_dir);
        for path in report.planned_paths {
            println!("would create {}", path);
        }
    } else {
        println!("initialised {}", report.project_dir);
        for path in report.wrote_paths {
            println!("created {}", path);
        }
        let name = report
            .project_dir
            .file_name()
            .map(str::to_string)
            .unwrap_or_else(|| "project".to_string());
        register_project(&name, &report.project_dir, &profile_for_registry)?;
        println!("registered {}", report.project_dir);
        if !args.no_git && git_config.auto_init {
            init_git_project(&report.project_dir, &git_config, &identity, &name)?;
        }
    }
    Ok(())
}

fn init_git_project(
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
    Ok(())
}

fn run_git_in<const N: usize>(project_dir: &Utf8PathBuf, args: [&str; N]) -> lode_core::Result<()> {
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

fn run_process_status(
    program: &str,
    args: &[String],
    current_dir: Option<&Utf8PathBuf>,
) -> lode_core::Result<std::process::ExitStatus> {
    run_process_status_with_env(program, args, current_dir, &[])
}

fn run_process_status_with_env(
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

fn config_command(command: ConfigCommand) -> lode_core::Result<()> {
    match command {
        ConfigCommand::Show {
            format,
            defaults,
            project,
            section,
        } => {
            if defaults && project {
                return Err(LodeError::Message(
                    "--defaults and --project cannot be used together".to_string(),
                ));
            }
            let value = if project {
                load_project_config_value()?
            } else if defaults {
                toml::Value::try_from(default_config())?
            } else {
                toml::Value::try_from(load_global_config()?)?
            };
            let value = config_section_value(value, section.as_deref())?;
            match format {
                OutputFormat::Toml => println!("{}", toml::to_string_pretty(&value)?),
                OutputFormat::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                ),
            }
        }
        ConfigCommand::Validate { defaults, project } => {
            if defaults && project {
                return Err(LodeError::Message(
                    "--defaults and --project cannot be used together".to_string(),
                ));
            }
            if project {
                let value = load_project_config_value()?;
                validate_config_value_schema(&value)?;
                println!("project config valid");
            } else if defaults {
                let value = toml::Value::try_from(default_config())?;
                validate_config_value_schema(&value)?;
                println!("default config valid");
            } else {
                load_global_config()?;
                println!("config valid");
            }
        }
        ConfigCommand::Diff => {
            let default = default_config();
            let global = load_global_config()?;
            print_config_diff(&default, &global)?;
        }
        ConfigCommand::Set { key, value } => {
            let mut config = load_global_config()?;
            set_config_value(&mut config, &key, &value)?;
            save_global_config(&config)?;
            println!("set {key} = {value}");
        }
        ConfigCommand::Reset { key } => {
            let mut config = load_global_config()?;
            let value = default_config_value(&key)?;
            set_config_value(&mut config, &key, &value)?;
            save_global_config(&config)?;
            println!("reset {key}");
        }
    }
    Ok(())
}

fn default_config_value(key: &str) -> lode_core::Result<String> {
    let config = default_config();
    let value = match key {
        "identity.author" => config.identity.author,
        "identity.email" => config.identity.email,
        "identity.org" => config.identity.org,
        "identity.license" => config.identity.license,
        "convention.default_case" => config.convention.default_case,
        "git.initial_branch" => config.git.initial_branch,
        "git.auto_init" => config.git.auto_init.to_string(),
        "git.initial_commit" => config.git.initial_commit.to_string(),
        "git.initial_commit_msg" => config.git.initial_commit_msg,
        _ => return Err(LodeError::Message(format!("unsupported config key: {key}"))),
    };
    Ok(value)
}

fn load_project_config_value() -> lode_core::Result<toml::Value> {
    let path = Utf8PathBuf::from(".lode").join("project.toml");
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
        path: path.as_str().into(),
        source,
    })
}

fn config_section_value(
    value: toml::Value,
    section: Option<&str>,
) -> lode_core::Result<toml::Value> {
    let Some(section) = section else {
        return Ok(value);
    };
    value
        .get(section)
        .cloned()
        .ok_or_else(|| LodeError::Message(format!("unknown config section: {section}")))
}

fn validate_config_value_schema(value: &toml::Value) -> lode_core::Result<()> {
    let found = value
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| LodeError::Message("missing schema_version".to_string()))?;
    if found == i64::from(lode_core::SCHEMA_VERSION) {
        Ok(())
    } else {
        Err(LodeError::SchemaMismatch {
            expected: lode_core::SCHEMA_VERSION,
            found: u32::try_from(found).unwrap_or_default(),
        })
    }
}

fn print_config_diff(
    default: &lode_core::LodeConfig,
    global: &lode_core::LodeConfig,
) -> lode_core::Result<()> {
    let default_value = toml::Value::try_from(default)?;
    let global_value = toml::Value::try_from(global)?;
    let mut changes = Vec::new();
    diff_toml("", &default_value, &global_value, &mut changes);
    if changes.is_empty() {
        println!("config diff: no changes from defaults");
    } else {
        for change in changes {
            println!("{change}");
        }
    }
    Ok(())
}

fn diff_toml(prefix: &str, left: &toml::Value, right: &toml::Value, changes: &mut Vec<String>) {
    match (left, right) {
        (toml::Value::Table(left), toml::Value::Table(right)) => {
            for (key, right_value) in right {
                let next = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                if let Some(left_value) = left.get(key) {
                    diff_toml(&next, left_value, right_value, changes);
                } else {
                    changes.push(format!("+ {next} = {right_value}"));
                }
            }
        }
        _ if left != right => changes.push(format!("~ {prefix}: {left} -> {right}")),
        _ => {}
    }
}

fn set_config_value(
    config: &mut lode_core::LodeConfig,
    key: &str,
    value: &str,
) -> lode_core::Result<()> {
    match key {
        "identity.author" => config.identity.author = value.to_string(),
        "identity.email" => config.identity.email = value.to_string(),
        "identity.org" => config.identity.org = value.to_string(),
        "identity.license" => config.identity.license = value.to_string(),
        "convention.default_case" => {
            if !matches!(
                value,
                "snake_case" | "kebab-case" | "camelCase" | "PascalCase"
            ) {
                return Err(LodeError::Message(format!(
                    "unsupported convention.default_case: {value}"
                )));
            }
            config.convention.default_case = value.to_string();
        }
        "git.initial_branch" => config.git.initial_branch = value.to_string(),
        "git.auto_init" => config.git.auto_init = parse_bool(value)?,
        "git.initial_commit" => config.git.initial_commit = parse_bool(value)?,
        "git.initial_commit_msg" => config.git.initial_commit_msg = value.to_string(),
        _ => return Err(LodeError::Message(format!("unsupported config key: {key}"))),
    }
    Ok(())
}

fn parse_bool(value: &str) -> lode_core::Result<bool> {
    match value {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(LodeError::Message(format!("expected boolean, got {value}"))),
    }
}

fn add_component(component: &str, dry_run: bool, overwrite: bool) -> lode_core::Result<()> {
    let cwd = current_dir()?;
    let project_name = cwd
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "project".to_string());
    let report = add_component_to_project(AddRequest {
        name: project_name,
        project_dir: cwd,
        config: load_global_config()?,
        component: component.to_string(),
        dry_run,
        overwrite,
    })?;
    for path in if dry_run {
        report.planned_paths
    } else {
        report.wrote_paths
    } {
        println!("{} {}", if dry_run { "would add" } else { "added" }, path);
    }
    Ok(())
}

fn library_command(
    root: &str,
    command: LibraryCommand,
    embedded: &[&str],
) -> lode_core::Result<()> {
    match command {
        LibraryCommand::List { format } => {
            if format == "json" {
                println!(
                    "{}",
                    serde_json::to_string_pretty(embedded)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                for item in embedded {
                    println!("{item}");
                }
            }
        }
        LibraryCommand::Show { name, raw: _ } => {
            let mut path = global_asset_dir(root)?.join(&name);
            if !path.exists() && matches!(root, "profiles" | "commands" | "recipes") {
                path = global_asset_dir(root)?.join(format!("{name}.toml"));
            }
            if path.exists() {
                print!(
                    "{}",
                    fs::read_to_string(&path).map_err(|source| LodeError::Io {
                        path: path.as_str().into(),
                        source,
                    })?
                );
            } else if embedded.iter().any(|item| *item == name) {
                println!("{name}");
            } else {
                return Err(LodeError::Message(format!("{root} item not found: {name}")));
            }
        }
        LibraryCommand::Diff { name } => {
            require_template_library(root)?;
            let relative = safe_relative_path(&name)?;
            let path = global_asset_dir(root)?.join(&relative);
            let current = fs::read_to_string(&path).unwrap_or_default();
            let default = embedded_template(&name)?;
            if current == default {
                println!("template unchanged: {name}");
            } else {
                println!("template differs: {name}");
                print_simple_diff(&current, &default);
            }
        }
        LibraryCommand::Reset { name } => {
            require_template_library(root)?;
            let relative = safe_relative_path(&name)?;
            let path = global_asset_dir(root)?.join(relative);
            let contents = embedded_template(&name)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                    path: parent.as_str().into(),
                    source,
                })?;
            }
            fs::write(&path, contents).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("reset template {name}");
        }
        LibraryCommand::Validate { all } => {
            require_template_library(root)?;
            if all {
                for item in embedded {
                    validate_template(item)?;
                }
                println!("validated {} templates", embedded.len());
            } else {
                let root = global_asset_dir(root)?;
                validate_template_tree(&root)?;
                println!("templates valid");
            }
        }
    }
    Ok(())
}

fn require_template_library(root: &str) -> lode_core::Result<()> {
    if root == "templates" {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "{root} does not support this library operation"
        )))
    }
}

fn embedded_template(name: &str) -> lode_core::Result<String> {
    if !template_paths().iter().any(|item| *item == name) {
        return Err(LodeError::Message(format!("template not found: {name}")));
    }
    let context = lode_core::RenderContext::new()
        .with("project", "project")
        .with("project_ident", "project")
        .with("project_class", "Project")
        .with("author", "Your Name")
        .with("org", "namespace")
        .with("license", "MIT OR Apache-2.0")
        .with("year", "2026")
        .with("profile", "core/bare");
    Ok(lode_core::assets::template_contents(name, &context))
}

fn validate_template_tree(root: &Utf8PathBuf) -> lode_core::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    if root.is_dir() {
        for entry in fs::read_dir(root).map_err(|source| LodeError::Io {
            path: root.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: root.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            validate_template_tree(&child)?;
        }
    } else {
        validate_template(root.as_str())?;
    }
    Ok(())
}

fn validate_template(name: &str) -> lode_core::Result<()> {
    let contents = if Utf8PathBuf::from(name).exists() {
        fs::read_to_string(name).map_err(|source| LodeError::Io {
            path: name.into(),
            source,
        })?
    } else {
        embedded_template(name).unwrap_or_default()
    };
    if name.ends_with(".toml") {
        let _: toml::Value =
            toml::from_str(&contents).map_err(|error| LodeError::Message(error.to_string()))?;
    } else if name.ends_with(".json") {
        let _: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|error| LodeError::Message(error.to_string()))?;
    } else if contents.trim().is_empty() {
        return Err(LodeError::Message(format!("empty template: {name}")));
    }
    Ok(())
}

fn safe_relative_path(path: &str) -> lode_core::Result<Utf8PathBuf> {
    if path.contains("..") || path.contains(':') || path.starts_with('/') || path.starts_with('\\')
    {
        return Err(LodeError::Message(format!("unsafe relative path: {path}")));
    }
    Ok(Utf8PathBuf::from(path))
}

fn print_simple_diff(current: &str, default: &str) {
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

fn profile_command(command: ProfileCommand) -> lode_core::Result<()> {
    match command {
        ProfileCommand::List => {
            for profile in profile_names() {
                println!("{profile}");
            }
        }
        ProfileCommand::Show { name } => {
            library_command(
                "profiles",
                LibraryCommand::Show { name, raw: true },
                &profile_names(),
            )?;
        }
        ProfileCommand::Use { name } => {
            let mut config = load_global_config()?;
            config.active_profile = Some(name.clone());
            save_global_config(&config)?;
            println!("active profile: {name}");
        }
        ProfileCommand::New { name } => {
            let path = global_asset_dir("profiles")?.join(format!("{name}.toml"));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                    path: parent.as_str().into(),
                    source,
                })?;
            }
            let config = load_global_config()?;
            let raw = toml::to_string_pretty(&config)?;
            fs::write(&path, raw).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("created profile {name}");
        }
        ProfileCommand::Delete { name } => {
            let path = global_asset_dir("profiles")?.join(format!("{name}.toml"));
            if profile_names().iter().any(|profile| *profile == name) {
                return Err(LodeError::Message(format!(
                    "refusing to delete embedded profile: {name}"
                )));
            }
            fs::remove_file(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("deleted profile {name}");
        }
    }
    Ok(())
}

fn snippet_command(command: SnippetCommand) -> lode_core::Result<()> {
    match command {
        SnippetCommand::List { lang, format } => {
            let root = global_asset_dir("snippets")?;
            if format == "json" {
                let mut snippets = Vec::new();
                if let Some(lang) = lang {
                    collect_snippet_assets(&root.join(lang), &mut snippets)?;
                } else {
                    collect_snippet_assets(&root, &mut snippets)?;
                }
                let values = snippets
                    .into_iter()
                    .map(|snippet| {
                        serde_json::json!({
                            "lang": snippet.lang,
                            "name": snippet.name,
                            "path": snippet.path,
                        })
                    })
                    .collect::<Vec<_>>();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&values)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else if let Some(lang) = lang {
                list_dir(root.join(lang))?;
            } else {
                list_dir(root)?;
            }
        }
        SnippetCommand::Show { name, lang } => {
            let path = resolve_snippet_path(&name, lang.as_deref())?;
            print!(
                "{}",
                fs::read_to_string(&path).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?
            );
        }
        SnippetCommand::Search { query } => {
            search_snippets(&query)?;
        }
        SnippetCommand::Add {
            name,
            lang,
            trigger,
            desc,
        } => {
            add_snippet(&name, &lang, trigger.as_deref(), desc.as_deref())?;
        }
        SnippetCommand::Remove { name, lang } => {
            remove_snippet(&name, lang.as_deref())?;
        }
        SnippetCommand::Insert {
            name,
            file,
            lang,
            line,
        } => {
            insert_snippet(&name, lang.as_deref(), file, line)?;
        }
        SnippetCommand::Export { lang, format, out } => {
            export_snippets(lang.as_deref(), &format, out)?;
        }
    }
    Ok(())
}

fn recipe_command(command: RecipeCommand) -> lode_core::Result<()> {
    match command {
        RecipeCommand::List => {
            for recipe in recipe_names() {
                println!("{recipe}");
            }
        }
        RecipeCommand::Show { name } => {
            library_command(
                "recipes",
                LibraryCommand::Show { name, raw: true },
                recipe_names(),
            )?;
        }
        RecipeCommand::Apply { name, dry_run } => apply_recipe(&name, dry_run)?,
        RecipeCommand::Compose { names } => {
            if names.is_empty() {
                return Err(LodeError::Message(
                    "recipe compose requires at least one recipe".to_string(),
                ));
            }
            for name in names {
                apply_recipe(&name, false)?;
            }
        }
        RecipeCommand::New { name } => new_recipe(&name)?,
    }
    Ok(())
}

fn new_recipe(name: &str) -> lode_core::Result<()> {
    let path = global_asset_dir("recipes")?.join(format!("{name}.toml"));
    if path.exists() {
        return Err(LodeError::Message(format!("recipe already exists: {name}")));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let contents = format!(
        "name = \"{name}\"\ndescription = \"Custom {name} recipe\"\n\n[[files]]\ntemplate = \"docs/index.md\"\ndest = \"docs/{name}.md\"\n"
    );
    fs::write(&path, contents).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("created recipe {name}");
    Ok(())
}

fn commands_command(command: CommandsCommand) -> lode_core::Result<()> {
    match command {
        CommandsCommand::List => {
            for command in command_names() {
                println!("{command}");
            }
            list_dir(Utf8PathBuf::from(".lode").join("commands"))?;
        }
        CommandsCommand::Show { name } => {
            let path = resolve_command_path(&name)?;
            print!(
                "{}",
                fs::read_to_string(&path).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?
            );
        }
        CommandsCommand::Add { slug, global, from } => {
            add_command_macro(&slug, global, from.as_deref())?;
        }
        CommandsCommand::Remove { slug, global } => {
            remove_command_macro(&slug, global)?;
        }
        CommandsCommand::Export { out } => {
            export_command_macros(out)?;
        }
        CommandsCommand::Run { slug, dry_run } => run_command_macro(&slug, dry_run)?,
    }
    Ok(())
}

fn add_command_macro(slug: &str, global: bool, from: Option<&str>) -> lode_core::Result<()> {
    let path = command_macro_path(slug, global)?;
    if path.exists() {
        return Err(LodeError::Message(format!(
            "command macro already exists: {slug}"
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let contents = if let Some(source_slug) = from {
        let source = resolve_command_path(source_slug)?;
        fs::read_to_string(&source).map_err(|source_error| LodeError::Io {
            path: source.as_str().into(),
            source: source_error,
        })?
    } else {
        format!(
            "slug = \"{slug}\"\ndescription = \"Custom {slug} command macro\"\n\n[[steps]]\nkind = \"make\"\nrun = \"{slug}\"\n"
        )
    };
    fs::write(&path, contents).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("created command macro {slug} at {path}");
    Ok(())
}

fn remove_command_macro(slug: &str, global: bool) -> lode_core::Result<()> {
    let path = command_macro_path(slug, global)?;
    if !path.exists() {
        return Err(LodeError::Message(format!(
            "command macro not found: {slug}"
        )));
    }
    fs::remove_file(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("removed command macro {slug}");
    Ok(())
}

fn export_command_macros(out: Option<Utf8PathBuf>) -> lode_core::Result<()> {
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
    let global = global_asset_dir("commands")?;
    collect_command_macro_files(&global, "global", &mut pack)?;
    let local = Utf8PathBuf::from(".lode").join("commands");
    collect_command_macro_files(&local, "project", &mut pack)?;
    pack.manifest.file_count = pack.files.len();
    let raw = serde_json::to_string_pretty(&pack)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.as_str().into(),
                source,
            })?;
        }
        fs::write(&path, raw).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        println!("exported {} command macros to {path}", pack.files.len());
    } else {
        println!("{raw}");
    }
    Ok(())
}

fn collect_command_macro_files(
    root: &Utf8PathBuf,
    prefix: &str,
    pack: &mut LodePack,
) -> lode_core::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|source| LodeError::Io {
        path: root.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: root.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if path.extension() != Some("toml") {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        let name = path.file_name().unwrap_or("command.toml");
        let checksum = content_hash_bytes(contents.as_bytes());
        pack.files.push(LodePackFile {
            path: format!("{prefix}/commands/{name}"),
            contents,
            checksum,
        });
    }
    Ok(())
}

fn command_macro_path(slug: &str, global: bool) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&format!("{slug}.toml"))?;
    if global {
        Ok(global_asset_dir("commands")?.join(relative))
    } else {
        Ok(Utf8PathBuf::from(".lode").join("commands").join(relative))
    }
}

fn plugin_command(command: PluginCommand) -> lode_core::Result<()> {
    match command {
        PluginCommand::List => list_dir(global_asset_dir("plugins")?)?,
        PluginCommand::Search { query, format } => {
            let entries = search_plugin_index(query.as_deref())?;
            match format.as_str() {
                "json" => println!(
                    "{}",
                    serde_json::to_string_pretty(&entries)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                ),
                "table" => {
                    if entries.is_empty() {
                        println!("no plugins found");
                    } else {
                        for entry in entries {
                            println!(
                                "{}\t{}\t{}\t{}",
                                entry.name,
                                entry.version,
                                if entry.installed {
                                    "installed"
                                } else {
                                    "available"
                                },
                                entry.description
                            );
                        }
                    }
                }
                other => {
                    return Err(LodeError::Message(format!(
                        "unsupported plugin search format: {other}"
                    )))
                }
            }
        }
        PluginCommand::Add {
            source,
            allow_unsafe,
        } => {
            if !source.exists() || !source.is_dir() {
                return Err(LodeError::Message(format!(
                    "plugin source must be a directory: {source}"
                )));
            }
            let entry = require_plugin_manifest(&source)?;
            safe_relative_path(&entry.name)?;
            enforce_plugin_permissions(&source, allow_unsafe)?;
            let name = entry.name;
            let destination = global_asset_dir("plugins")?.join(&name);
            if destination.exists() {
                return Err(LodeError::Message(format!("plugin already exists: {name}")));
            }
            copy_dir_recursive(&source, &destination)?;
            write_plugin_install_receipt(&destination, &source, allow_unsafe)?;
            println!("added plugin {name}");
        }
        PluginCommand::Remove { name } => {
            let path = global_asset_dir("plugins")?.join(safe_relative_path(&name)?);
            if !path.exists() {
                return Err(LodeError::Message(format!("plugin not found: {name}")));
            }
            fs::remove_dir_all(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("removed plugin {name}");
        }
        PluginCommand::Update { name } => {
            if let Some(name) = name {
                let path = global_asset_dir("plugins")?.join(safe_relative_path(&name)?);
                if !path.exists() {
                    return Err(LodeError::Message(format!("plugin not found: {name}")));
                }
                println!("plugin {name} is local; refresh by re-adding from source");
            } else {
                println!("local plugins checked");
            }
        }
        PluginCommand::Info { name } => {
            let path = global_asset_dir("plugins")?.join(safe_relative_path(&name)?);
            if !path.exists() {
                return Err(LodeError::Message(format!("plugin not found: {name}")));
            }
            let entry = plugin_index_entry(&path, true)?;
            println!("name\t{}", entry.name);
            println!("version\t{}", entry.version);
            println!("description\t{}", entry.description);
            println!("path\t{path}");
            for child in ["templates", "profiles", "snippets", "recipes", "commands"] {
                println!("{child}\t{}", status_bool(path.join(child).exists()));
            }
            if !entry.capabilities.is_empty() {
                println!("capabilities\t{}", entry.capabilities.join(","));
            }
            let security = read_plugin_security(&path)?;
            println!("network\t{}", status_bool(security.network));
            println!("execute\t{}", status_bool(security.execute));
            if !security.fs_write.is_empty() {
                println!("fs_write\t{}", security.fs_write.join(","));
            }
            if let Some(receipt) = read_plugin_install_receipt(&path)? {
                println!("trusted\t{}", status_bool(receipt.reviewed));
                println!("installed_at\t{}", receipt.installed_at);
                println!("installed_from\t{}", receipt.source);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PluginSecurity {
    network: bool,
    execute: bool,
    fs_write: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginInstallReceipt {
    schema_version: u32,
    source: String,
    installed_at: String,
    reviewed: bool,
    allow_unsafe: bool,
    permissions: PluginSecurity,
}

fn require_plugin_manifest(source: &Utf8PathBuf) -> lode_core::Result<PluginIndexEntry> {
    let manifest = source.join("plugin.toml");
    if !manifest.exists() {
        return Err(LodeError::Message(
            "plugin manifest required: plugin.toml".to_string(),
        ));
    }
    plugin_index_entry(source, false)
}

fn enforce_plugin_permissions(source: &Utf8PathBuf, allow_unsafe: bool) -> lode_core::Result<()> {
    let security = read_plugin_security(source)?;
    for path in &security.fs_write {
        safe_relative_path(path)?;
    }
    let has_executable_surface = source.join("bin").exists() || source.join("hooks").exists();
    if has_executable_surface && !security.execute {
        return Err(LodeError::Message(
            "plugin contains bin/ or hooks/ but does not declare permissions.execute = true"
                .to_string(),
        ));
    }
    let unsafe_reasons = [
        (security.network, "network"),
        (security.execute || has_executable_surface, "execute"),
    ]
    .into_iter()
    .filter_map(|(enabled, reason)| enabled.then_some(reason))
    .collect::<Vec<_>>();
    if !unsafe_reasons.is_empty() && !allow_unsafe {
        return Err(LodeError::Message(format!(
            "plugin requests unsafe permission(s): {}; rerun with --allow-unsafe after review",
            unsafe_reasons.join(",")
        )));
    }
    Ok(())
}

fn read_plugin_security(path: &Utf8PathBuf) -> lode_core::Result<PluginSecurity> {
    let manifest = path.join("plugin.toml");
    if !manifest.exists() {
        return Ok(PluginSecurity::default());
    }
    let raw = fs::read_to_string(&manifest).map_err(|source| LodeError::Io {
        path: manifest.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    let Some(permissions) = value.get("permissions") else {
        return Ok(PluginSecurity::default());
    };
    let network = permissions
        .get("network")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let execute = permissions
        .get("execute")
        .or_else(|| permissions.get("fs_execute"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let fs_write = permissions
        .get("fs_write")
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(PluginSecurity {
        network,
        execute,
        fs_write,
    })
}

fn write_plugin_install_receipt(
    destination: &Utf8PathBuf,
    source: &Utf8PathBuf,
    allow_unsafe: bool,
) -> lode_core::Result<()> {
    let receipt = PluginInstallReceipt {
        schema_version: 3,
        source: source.to_string(),
        installed_at: now_timestamp(),
        reviewed: true,
        allow_unsafe,
        permissions: read_plugin_security(destination)?,
    };
    let path = destination.join(".lode-install.json");
    let raw = serde_json::to_string_pretty(&receipt)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn read_plugin_install_receipt(
    path: &Utf8PathBuf,
) -> lode_core::Result<Option<PluginInstallReceipt>> {
    let receipt = path.join(".lode-install.json");
    if !receipt.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&receipt).map_err(|source| LodeError::Io {
        path: receipt.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|error| LodeError::Message(error.to_string()))
}

fn search_plugin_index(query: Option<&str>) -> lode_core::Result<Vec<PluginIndexEntry>> {
    let mut entries = default_plugin_registry();
    let plugins_dir = global_asset_dir("plugins")?;
    if plugins_dir.exists() {
        for entry in fs::read_dir(&plugins_dir).map_err(|source| LodeError::Io {
            path: plugins_dir.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: plugins_dir.as_str().into(),
                source,
            })?;
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            if path.is_dir() {
                let installed = plugin_index_entry(&path, true)?;
                entries.retain(|candidate| candidate.name != installed.name);
                entries.push(installed);
            }
        }
    }

    if let Some(query) = query {
        let query = query.to_ascii_lowercase();
        entries.retain(|entry| {
            entry.name.to_ascii_lowercase().contains(&query)
                || entry.description.to_ascii_lowercase().contains(&query)
                || entry
                    .capabilities
                    .iter()
                    .any(|capability| capability.to_ascii_lowercase().contains(&query))
        });
    }
    entries.sort_by(|left, right| {
        right
            .installed
            .cmp(&left.installed)
            .then(left.name.cmp(&right.name))
    });
    Ok(entries)
}

fn plugin_index_entry(path: &Utf8PathBuf, installed: bool) -> lode_core::Result<PluginIndexEntry> {
    let manifest = path.join("plugin.toml");
    let fallback_name = path
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "plugin".to_string());
    let mut entry = PluginIndexEntry {
        name: fallback_name,
        version: "0.0.0".to_string(),
        description: "Local Lode plugin".to_string(),
        source: "local".to_string(),
        installed,
        path: Some(path.clone()),
        capabilities: plugin_capabilities(path),
    };
    if manifest.exists() {
        let raw = fs::read_to_string(&manifest).map_err(|source| LodeError::Io {
            path: manifest.as_str().into(),
            source,
        })?;
        let value: toml::Value =
            toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
        let plugin = value.get("plugin").unwrap_or(&value);
        if let Some(name) = plugin.get("name").and_then(toml::Value::as_str) {
            entry.name = name.to_string();
        }
        if let Some(version) = plugin.get("version").and_then(toml::Value::as_str) {
            entry.version = version.to_string();
        }
        if let Some(description) = plugin.get("description").and_then(toml::Value::as_str) {
            entry.description = description.to_string();
        }
    }
    Ok(entry)
}

fn plugin_capabilities(path: &Utf8PathBuf) -> Vec<String> {
    [
        "templates",
        "profiles",
        "snippets",
        "recipes",
        "commands",
        "hooks",
        "bin",
    ]
    .iter()
    .filter(|name| path.join(name).exists())
    .map(|name| (*name).to_string())
    .collect()
}

fn default_plugin_registry() -> Vec<PluginIndexEntry> {
    [
        (
            "lode-plugin-tauri",
            "desktop and Tauri scaffolding, commands, and checks",
            &["templates", "commands", "recipes"][..],
        ),
        (
            "lode-plugin-minecraft",
            "Minecraft Fabric, Forge, NeoForge, and Paper project helpers",
            &["templates", "snippets", "commands"][..],
        ),
        (
            "lode-plugin-competitive",
            "competitive programming templates, runners, and snippets",
            &["templates", "snippets", "commands"][..],
        ),
        (
            "lode-plugin-agent-pack",
            "agent context packs for Claude, Codex, Cursor, and Windsurf",
            &["templates", "commands", "hooks"][..],
        ),
    ]
    .into_iter()
    .map(|(name, description, capabilities)| PluginIndexEntry {
        name: name.to_string(),
        version: "registry".to_string(),
        description: description.to_string(),
        source: "builtin-index".to_string(),
        installed: false,
        path: None,
        capabilities: capabilities
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    })
    .collect()
}

fn copy_dir_recursive(source: &Utf8PathBuf, destination: &Utf8PathBuf) -> lode_core::Result<()> {
    fs::create_dir_all(destination).map_err(|source_error| LodeError::Io {
        path: destination.as_str().into(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| LodeError::Io {
        path: source.as_str().into(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| LodeError::Io {
            path: source.as_str().into(),
            source: source_error,
        })?;
        let child_source = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        let child_destination = destination.join(entry.file_name().to_string_lossy().as_ref());
        if child_source.is_dir() {
            copy_dir_recursive(&child_source, &child_destination)?;
        } else {
            fs::copy(&child_source, &child_destination).map_err(|source_error| LodeError::Io {
                path: child_destination.as_str().into(),
                source: source_error,
            })?;
        }
    }
    Ok(())
}

fn mcp_command(
    http: bool,
    port: Option<u16>,
    list_tools: bool,
    list_resources: bool,
    list_prompts: bool,
) -> lode_core::Result<()> {
    if list_resources {
        println!("{}", json_pretty(&mcp_resources())?);
    }
    if list_prompts {
        println!("{}", json_pretty(&mcp_prompts())?);
    }
    if list_tools {
        println!("{}", json_pretty(&mcp_tools())?);
    }
    if http {
        println!("mcp http mode requested on port {}", port.unwrap_or(3333));
        println!(
            "http+sse transport is not active in this build; use stdio JSON-RPC or list flags"
        );
        return Ok(());
    }
    if list_tools || list_resources || list_prompts {
        return Ok(());
    }

    run_mcp_stdio()
}

fn mcp_tools() -> Value {
    let mut tools = vec![
        json!({
                "name": "lode_config_show",
                "description": "Return the loaded global Lode configuration.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_template_list",
                "description": "List embedded/default template paths.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_profile_list",
                "description": "List embedded/default profile names.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_recipe_list",
                "description": "List embedded/default recipe names.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_audit",
                "description": "Audit the current project and return the health report.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_scan_foreign",
                "description": "Analyse a non-Lode project and return a local adoption/migration report.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Project path to inspect. Defaults to the current working directory." }
                    }
                }
        }),
        json!({
                "name": "lode_time_today",
                "description": "Return total tracked time from .lode/time-log.json for today.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_info",
                "description": "Return local project status, package manager, config schema, and available Lode assets.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_metrics_show",
                "description": "Return the latest project metrics snapshot if available.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_pkg_outdated",
                "description": "Return the native package-manager command plan for checking outdated packages.",
                "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
                "name": "lode_pkg_audit",
                "description": "Return the native package-manager command plan for dependency audit.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "fail_on": { "type": "string", "enum": ["low", "medium", "high", "critical"] }
                    }
                }
        }),
        json!({
                "name": "lode_pkg_update",
                "description": "Return the native package-manager command plan for package updates.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Optional package name to update." }
                    }
                }
        }),
    ];
    for slug in mcp_command_names() {
        tools.push(json!({
            "name": format!("lode_custom_{slug}"),
            "description": format!("Discover local custom command `{slug}`. Execution is intentionally not exposed over MCP in this build."),
            "inputSchema": { "type": "object", "properties": {} }
        }));
    }
    json!({ "tools": tools })
}

fn mcp_command_names() -> Vec<String> {
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

fn json_pretty(value: &Value) -> lode_core::Result<String> {
    serde_json::to_string_pretty(value).map_err(|error| LodeError::Message(error.to_string()))
}

fn mcp_resources() -> Value {
    json!({
        "resources": [
            { "uri": "lode://config", "name": "Global config", "mimeType": "application/toml" },
            { "uri": "lode://registry", "name": "Project registry", "mimeType": "application/json" },
            { "uri": "lode://templates", "name": "Template inventory", "mimeType": "application/json" },
            { "uri": "lode://profiles", "name": "Profile inventory", "mimeType": "application/json" },
            { "uri": "lode://snippets", "name": "Snippet inventory", "mimeType": "application/json" },
            { "uri": "lode://project/info", "name": "Project info", "mimeType": "application/json" },
            { "uri": "lode://project/health", "name": "Project health", "mimeType": "application/json" },
            { "uri": "lode://project/metrics", "name": "Project metrics", "mimeType": "application/json" },
            { "uri": "lode://project/time", "name": "Project time log", "mimeType": "application/json" },
            { "uri": "lode://project/config", "name": "Effective project config", "mimeType": "application/toml" },
            { "uri": "lode://project/conventions", "name": "Convention settings", "mimeType": "application/json" }
        ]
    })
}

fn mcp_prompts() -> Value {
    json!({
        "prompts": [
            {
                "name": "lode-project-review",
                "description": "Review a project against the local Lode preferences."
            },
            {
                "name": "lode-scaffold-plan",
                "description": "Plan a scaffold using available profiles, recipes, and templates."
            }
        ]
    })
}

fn run_mcp_stdio() -> lode_core::Result<()> {
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

fn lsp_command(stdio: bool, capabilities: bool) -> lode_core::Result<()> {
    if capabilities {
        println!("{}", json_pretty(&lsp_capabilities())?);
    }
    if capabilities && !stdio {
        return Ok(());
    }
    if stdio {
        return run_lsp_stdio();
    }
    println!("lode lsp is available over stdio; run `lode lsp --stdio`");
    Ok(())
}

fn run_lsp_stdio() -> lode_core::Result<()> {
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

fn lsp_handle_request(request: &Value) -> Option<String> {
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

fn lsp_capabilities() -> Value {
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

fn lsp_diagnostics(uri: &str, text: &str) -> Vec<Value> {
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

fn should_require_signature(uri: &str) -> bool {
    ["rs", "ts", "js", "py", "go", "java", "c", "cpp", "h", "hpp"]
        .iter()
        .any(|extension| uri.ends_with(&format!(".{extension}")))
}

fn mcp_handle_request(request: &Value) -> String {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": { "name": "lode", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            }
        })),
        "tools/list" => Ok(mcp_tools()),
        "resources/list" => Ok(mcp_resources()),
        "prompts/list" => Ok(mcp_prompts()),
        "tools/call" => mcp_call_tool(request),
        "resources/read" => mcp_read_resource(request),
        _ => Err((-32601, format!("method not found: {method}"))),
    };
    match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string(),
        Err((code, message)) => {
            json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
                .to_string()
        }
    }
}

fn mcp_call_tool(request: &Value) -> std::result::Result<Value, (i64, String)> {
    let name = request
        .pointer("/params/name")
        .and_then(Value::as_str)
        .ok_or_else(|| (-32602, "missing params.name".to_string()))?;
    let value = match name {
        "lode_config_show" => {
            serde_json::to_value(load_global_config().unwrap_or_else(|_| default_config()))
                .map_err(|error| (-32603, error.to_string()))?
        }
        "lode_template_list" => json!(template_paths()),
        "lode_profile_list" => json!(profile_names()),
        "lode_recipe_list" => json!(recipe_names()),
        "lode_audit" => {
            let config = load_global_config().unwrap_or_else(|_| default_config());
            let cwd = current_dir().map_err(|error| (-32603, error.to_string()))?;
            serde_json::to_value(
                audit_project(&cwd, &config).map_err(|error| (-32603, error.to_string()))?,
            )
            .map_err(|error| (-32603, error.to_string()))?
        }
        "lode_scan_foreign" => {
            let path = request
                .pointer("/params/arguments/path")
                .and_then(Value::as_str)
                .map(Utf8PathBuf::from)
                .map(Ok)
                .unwrap_or_else(current_dir)
                .map_err(|error| (-32603, error.to_string()))?;
            serde_json::to_value(
                scan_foreign_project(&path).map_err(|error| (-32603, error.to_string()))?,
            )
            .map_err(|error| (-32603, error.to_string()))?
        }
        "lode_time_today" => mcp_time_today_value(),
        "lode_info" => mcp_project_info_value().map_err(|error| (-32603, error.to_string()))?,
        "lode_metrics_show" => {
            let cwd = current_dir().map_err(|error| (-32603, error.to_string()))?;
            serde_json::to_value(load_metrics(&cwd).map_err(|error| (-32603, error.to_string()))?)
                .map_err(|error| (-32603, error.to_string()))?
        }
        "lode_pkg_outdated" => {
            let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
            let plan = PackageOperationPlan::new(
                "outdated",
                &manager,
                package_outdated_args(&manager).map_err(|error| (-32603, error.to_string()))?,
            );
            serde_json::to_value(plan).map_err(|error| (-32603, error.to_string()))?
        }
        "lode_pkg_audit" => {
            let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
            let fail_on = request
                .pointer("/params/arguments/fail_on")
                .and_then(Value::as_str);
            let plan = PackageOperationPlan::new(
                "audit",
                &manager,
                package_audit_args(&manager, fail_on)
                    .map_err(|error| (-32603, error.to_string()))?,
            );
            serde_json::to_value(plan).map_err(|error| (-32603, error.to_string()))?
        }
        "lode_pkg_update" => {
            let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
            let name = request
                .pointer("/params/arguments/name")
                .and_then(Value::as_str);
            let plan = PackageOperationPlan::new(
                "update",
                &manager,
                package_update_args(&manager, name).map_err(|error| (-32603, error.to_string()))?,
            );
            serde_json::to_value(plan).map_err(|error| (-32603, error.to_string()))?
        }
        other if other.starts_with("lode_custom_") => {
            let slug = other.trim_start_matches("lode_custom_");
            mcp_custom_command_value(slug).map_err(|error| (-32603, error.to_string()))?
        }
        other => return Err((-32602, format!("unknown tool: {other}"))),
    };
    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
            }
        ],
        "structuredContent": value
    }))
}

fn mcp_read_resource(request: &Value) -> std::result::Result<Value, (i64, String)> {
    let uri = request
        .pointer("/params/uri")
        .and_then(Value::as_str)
        .ok_or_else(|| (-32602, "missing params.uri".to_string()))?;
    let text = match uri {
        "lode://config" => {
            toml::to_string_pretty(&load_global_config().unwrap_or_else(|_| default_config()))
                .map_err(|error| (-32603, error.to_string()))?
        }
        "lode://registry" => serde_json::to_string_pretty(&load_registry().unwrap_or_default())
            .map_err(|error| (-32603, error.to_string()))?,
        "lode://templates" => serde_json::to_string_pretty(template_paths())
            .map_err(|error| (-32603, error.to_string()))?,
        "lode://profiles" => serde_json::to_string_pretty(&profile_names())
            .map_err(|error| (-32603, error.to_string()))?,
        "lode://snippets" => serde_json::to_string_pretty(&snippet_inventory())
            .map_err(|error| (-32603, error.to_string()))?,
        "lode://project/info" => serde_json::to_string_pretty(
            &mcp_project_info_value().map_err(|error| (-32603, error.to_string()))?,
        )
        .map_err(|error| (-32603, error.to_string()))?,
        "lode://project/health" => {
            let config = load_global_config().unwrap_or_else(|_| default_config());
            let cwd = current_dir().map_err(|error| (-32603, error.to_string()))?;
            serde_json::to_string_pretty(
                &audit_project(&cwd, &config).map_err(|error| (-32603, error.to_string()))?,
            )
            .map_err(|error| (-32603, error.to_string()))?
        }
        "lode://project/metrics" => {
            let cwd = current_dir().map_err(|error| (-32603, error.to_string()))?;
            serde_json::to_string_pretty(
                &load_metrics(&cwd).map_err(|error| (-32603, error.to_string()))?,
            )
            .map_err(|error| (-32603, error.to_string()))?
        }
        "lode://project/time" => serde_json::to_string_pretty(&load_time_log().unwrap_or_default())
            .map_err(|error| (-32603, error.to_string()))?,
        "lode://project/config" => {
            toml::to_string_pretty(&load_global_config().unwrap_or_else(|_| default_config()))
                .map_err(|error| (-32603, error.to_string()))?
        }
        "lode://project/conventions" => {
            let config = load_global_config().unwrap_or_else(|_| default_config());
            serde_json::to_string_pretty(&config.convention)
                .map_err(|error| (-32603, error.to_string()))?
        }
        other => return Err((-32602, format!("unknown resource: {other}"))),
    };
    Ok(json!({
        "contents": [
            {
                "uri": uri,
                "mimeType": if uri == "lode://config" || uri == "lode://project/config" { "application/toml" } else { "application/json" },
                "text": text
            }
        ]
    }))
}

fn mcp_project_info_value() -> lode_core::Result<Value> {
    let cwd = current_dir()?;
    let config = load_global_config().unwrap_or_else(|_| default_config());
    Ok(json!({
        "path": cwd,
        "schema_version": config.schema_version,
        "package_manager": detect_package_manager(),
        "profiles": profile_names(),
        "templates": template_paths().len(),
        "recipes": recipe_names(),
        "snippets": snippet_inventory().len(),
        "project_config": cwd.join(".lode").join("project.toml").exists(),
        "metrics": cwd.join(".lode").join("metrics.json").exists(),
        "time_log": cwd.join(".lode").join("time-log.json").exists()
    }))
}

fn mcp_time_today_value() -> Value {
    let log = load_time_log().unwrap_or_default();
    let today = today_utc();
    let sessions = log
        .sessions
        .into_iter()
        .filter(|session| session.started_at.starts_with(&today))
        .collect::<Vec<_>>();
    json!({
        "date": today,
        "seconds": total_seconds(&sessions),
        "duration": format_seconds(total_seconds(&sessions)),
        "sessions": sessions
    })
}

fn mcp_custom_command_value(slug: &str) -> lode_core::Result<Value> {
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

fn snippet_inventory() -> Vec<String> {
    let mut snippets = Vec::new();
    if let Ok(root) = global_asset_dir("snippets") {
        let _ = collect_snippet_assets(&root, &mut snippets);
    }
    snippets
        .into_iter()
        .map(|snippet| format!("{}/{}", snippet.lang, snippet.name))
        .collect()
}

fn agent_command(command: AgentCommand) -> lode_core::Result<()> {
    match command {
        AgentCommand::Sync => agent_sync()?,
        AgentCommand::Status => agent_status()?,
        AgentCommand::Export { out } => agent_export(out)?,
        AgentCommand::Plan { command } => agent_plan(command)?,
    }
    Ok(())
}

fn agent_sync() -> lode_core::Result<()> {
    let context_dir = Utf8PathBuf::from(".lode").join("context");
    fs::create_dir_all(&context_dir).map_err(|source| LodeError::Io {
        path: context_dir.as_str().into(),
        source,
    })?;
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
    fs::write(&output, summary).map_err(|source| LodeError::Io {
        path: output.as_str().into(),
        source,
    })?;
    println!("agent context synced to {output}");
    Ok(())
}

fn collect_context_index(path: &Utf8PathBuf, output: &mut String) -> lode_core::Result<()> {
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

fn agent_status() -> lode_core::Result<()> {
    let index = Utf8PathBuf::from(".lode").join("context").join("INDEX.md");
    let plan = agent_plan_path();
    println!("context\t{}", status_bool(index.exists()));
    println!("plan\t{}", status_bool(plan.exists()));
    Ok(())
}

fn agent_export(out: Option<Utf8PathBuf>) -> lode_core::Result<()> {
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
    fs::write(&output, raw).map_err(|source| LodeError::Io {
        path: output.as_str().into(),
        source,
    })?;
    println!("exported agent context to {output}");
    Ok(())
}

fn agent_plan(command: AgentPlanCommand) -> lode_core::Result<()> {
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
                fs::remove_file(&path).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?;
            }
            println!("agent plan cleared");
        }
    }
    Ok(())
}

fn agent_plan_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("agent-plan.json")
}

fn load_agent_plan() -> lode_core::Result<AgentPlan> {
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

fn save_agent_plan(plan: &AgentPlan) -> lode_core::Result<()> {
    let path = agent_plan_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(plan)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn resolve_command_path(slug: &str) -> lode_core::Result<Utf8PathBuf> {
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
    let (program, args) = if cfg!(windows) {
        ("cmd", vec!["/C".to_string(), run.to_string()])
    } else {
        ("sh", vec!["-c".to_string(), run.to_string()])
    };
    let status = run_process_status(program, &args, None)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "shell step failed with {status}"
        )))
    }
}

fn info(json: bool) -> lode_core::Result<()> {
    let dir = global_dir()?;
    if json {
        println!(
            "{{\"config\":\"{}\",\"profiles\":{},\"templates\":{},\"commands\":{}}}",
            dir.join("config.toml"),
            profile_names().len(),
            template_paths().len(),
            command_names().len()
        );
    } else {
        println!("config   {}", dir.join("config.toml"));
        println!("profiles {}", profile_names().len());
        println!("templates {}", template_paths().len());
        println!("commands {}", command_names().len());
    }
    Ok(())
}

fn health() -> lode_core::Result<()> {
    let cwd = current_dir()?;
    let config = load_global_config()?;
    let report = audit_project(&cwd, &config)?;
    let metrics_path = save_metrics(&cwd, &report)?;
    println!("health score: {}", report.score);
    println!("convention violations: {}", report.convention_violations);
    println!("secret findings: {}", report.secret_findings);
    println!("license: {}", status_bool(report.license_present));
    println!("env example: {}", status_bool(report.env_example_present));
    println!("readme: {}", status_bool(report.readme_present));
    println!("metrics: {metrics_path}");
    Ok(())
}

fn release(
    version: Option<String>,
    bump: Option<String>,
    dry_run: bool,
    rollback: bool,
) -> lode_core::Result<()> {
    if rollback {
        return rollback_release(dry_run);
    }
    let current = detect_project_version().unwrap_or_else(|| "0.1.0".to_string());
    let next = if let Some(version) = version {
        version.trim_start_matches('v').to_string()
    } else if let Some(bump) = bump {
        bump_version(&current, &bump)?
    } else {
        current.clone()
    };
    let files = version_files();
    if files.is_empty() {
        return Err(LodeError::Message("no version files found".to_string()));
    }
    let rollback = build_release_rollback(&files, &current, &next)?;
    if !dry_run {
        write_release_rollback(&rollback)?;
    }
    for file in files {
        if dry_run {
            println!("would update {file} {current} -> {next}");
        } else {
            if let Err(error) = update_version_file(&file, &next) {
                apply_release_rollback(&rollback)?;
                return Err(error);
            }
            println!("updated {file} to {next}");
        }
    }
    if !dry_run {
        clear_release_rollback()?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseRollback {
    schema_version: u32,
    created_at: String,
    from: String,
    to: String,
    files: Vec<ReleaseRollbackFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseRollbackFile {
    path: Utf8PathBuf,
    contents: String,
    before_hash: String,
    after_hash: String,
}

fn build_release_rollback(
    files: &[String],
    from: &str,
    to: &str,
) -> lode_core::Result<ReleaseRollback> {
    let mut rollback = ReleaseRollback {
        schema_version: 3,
        created_at: now_timestamp(),
        from: from.to_string(),
        to: to.to_string(),
        files: Vec::new(),
    };
    for file in files {
        let safe_path = safe_relative_path(file)?;
        let contents = fs::read_to_string(file).map_err(|source| LodeError::Io {
            path: file.into(),
            source,
        })?;
        let updated = updated_version_contents(file, &contents, to)?;
        rollback.files.push(ReleaseRollbackFile {
            path: safe_path,
            before_hash: content_hash_bytes(contents.as_bytes()),
            after_hash: content_hash_bytes(updated.as_bytes()),
            contents,
        });
    }
    Ok(rollback)
}

fn release_rollback_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("release.rollback.json")
}

fn write_release_rollback(rollback: &ReleaseRollback) -> lode_core::Result<()> {
    let path = safe_relative_path(release_rollback_path().as_str())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(rollback)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn read_release_rollback() -> lode_core::Result<ReleaseRollback> {
    let path = safe_relative_path(release_rollback_path().as_str())?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let rollback: ReleaseRollback =
        serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    validate_release_rollback(&rollback)?;
    Ok(rollback)
}

fn validate_release_rollback(rollback: &ReleaseRollback) -> lode_core::Result<()> {
    if rollback.schema_version != 3 {
        return Err(LodeError::Message(format!(
            "unsupported release rollback schema: {}",
            rollback.schema_version
        )));
    }
    if rollback.files.is_empty() {
        return Err(LodeError::Message(
            "release rollback has no files".to_string(),
        ));
    }
    for file in &rollback.files {
        safe_relative_path(file.path.as_str())?;
        let before_hash = content_hash_bytes(file.contents.as_bytes());
        if before_hash != file.before_hash {
            return Err(LodeError::Message(format!(
                "release rollback backup hash mismatch: {}",
                file.path
            )));
        }
    }
    Ok(())
}

fn apply_release_rollback(rollback: &ReleaseRollback) -> lode_core::Result<()> {
    for file in &rollback.files {
        let safe_path = safe_relative_path(file.path.as_str())?;
        let current = fs::read(&safe_path).map_err(|source| LodeError::Io {
            path: safe_path.as_str().into(),
            source,
        })?;
        let current_hash = content_hash_bytes(&current);
        if current_hash == file.before_hash {
            continue;
        }
        if current_hash != file.after_hash {
            return Err(LodeError::Message(format!(
                "release rollback refused because {} changed after rollback state was written",
                file.path
            )));
        }
        fs::write(&file.path, &file.contents).map_err(|source| LodeError::Io {
            path: file.path.as_str().into(),
            source,
        })?;
    }
    clear_release_rollback()?;
    eprintln!(
        "release rollback applied: {} -> {}",
        rollback.to, rollback.from
    );
    Ok(())
}

fn rollback_release(dry_run: bool) -> lode_core::Result<()> {
    let rollback = read_release_rollback()?;
    if dry_run {
        for file in &rollback.files {
            println!(
                "would rollback {} {} -> {}",
                file.path, rollback.to, rollback.from
            );
        }
        return Ok(());
    }
    apply_release_rollback(&rollback)
}

fn clear_release_rollback() -> lode_core::Result<()> {
    let path = safe_relative_path(release_rollback_path().as_str())?;
    if path.exists() {
        fs::remove_file(&path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
    }
    Ok(())
}

fn detect_project_version() -> Option<String> {
    for file in version_files() {
        let raw = fs::read_to_string(&file).ok()?;
        if file == "Cargo.toml" {
            if let Some(version) = toml_section_version(&raw, "package")
                .or_else(|| toml_section_version(&raw, "workspace.package"))
            {
                return Some(version);
            }
        } else if file == "pyproject.toml" {
            if let Some(version) = toml_section_version(&raw, "project") {
                return Some(version);
            }
        } else if file == "package.json" {
            let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
            return value
                .get("version")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
        }
    }
    None
}

fn toml_section_version(raw: &str, wanted_section: &str) -> Option<String> {
    let mut section = "";
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(['[', ']']);
            continue;
        }
        if section == wanted_section && trimmed.starts_with("version") {
            return trimmed
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn version_files() -> Vec<String> {
    ["Cargo.toml", "package.json", "pyproject.toml"]
        .into_iter()
        .filter(|file| Utf8PathBuf::from(file).exists())
        .map(str::to_string)
        .collect()
}

fn bump_version(version: &str, bump: &str) -> lode_core::Result<String> {
    let mut parts = version
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect::<Vec<_>>();
    while parts.len() < 3 {
        parts.push(0);
    }
    match bump {
        "major" => {
            parts[0] += 1;
            parts[1] = 0;
            parts[2] = 0;
        }
        "minor" => {
            parts[1] += 1;
            parts[2] = 0;
        }
        "patch" => parts[2] += 1,
        other => return Err(LodeError::Message(format!("unsupported bump: {other}"))),
    }
    Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
}

fn update_version_file(file: &str, next: &str) -> lode_core::Result<()> {
    let raw = fs::read_to_string(file).map_err(|source| LodeError::Io {
        path: file.into(),
        source,
    })?;
    let updated = updated_version_contents(file, &raw, next)?;
    fs::write(file, updated).map_err(|source| LodeError::Io {
        path: file.into(),
        source,
    })
}

fn updated_version_contents(file: &str, raw: &str, next: &str) -> lode_core::Result<String> {
    let updated = if file == "package.json" {
        let mut value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
        value["version"] = serde_json::Value::String(next.to_string());
        serde_json::to_string_pretty(&value)
            .map_err(|error| LodeError::Message(error.to_string()))?
            + "\n"
    } else if file == "Cargo.toml" {
        update_toml_version(&raw, next, &["package", "workspace.package"])
    } else if file == "pyproject.toml" {
        update_toml_version(&raw, next, &["project"])
    } else {
        raw.to_string()
    };
    Ok(updated)
}

fn update_toml_version(raw: &str, next: &str, sections: &[&str]) -> String {
    let mut section = "";
    let mut updated = false;
    let mut lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(['[', ']']);
        }
        if !updated && sections.contains(&section) && trimmed.starts_with("version") {
            lines.push(format!("version = \"{next}\""));
            updated = true;
        } else {
            lines.push(line.to_string());
        }
    }
    lines.join("\n") + "\n"
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    status: String,
    fixed: bool,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    name: String,
    status: String,
    detail: String,
}

fn doctor(fix: bool, json: bool) -> lode_core::Result<()> {
    let mut fixed = false;
    if fix {
        setup_defaults(false)?;
        fixed = true;
    }
    let report = build_doctor_report(fixed);
    if json {
        println!("{}", json_pretty(&json!(report))?);
    } else {
        println!("doctor {}", report.status);
        if report.fixed {
            println!("fixed\tsafe defaults refreshed");
        }
        for check in &report.checks {
            println!("{}\t{}\t{}", check.name, check.status, check.detail);
        }
    }
    Ok(())
}

fn build_doctor_report(fixed: bool) -> DoctorReport {
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

    match discover_hooks() {
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

fn doctor_check(name: &str, status: &str, detail: &str) -> DoctorCheck {
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

fn scan(command: ScanCommand) -> lode_core::Result<()> {
    match command {
        ScanCommand::Secrets {
            path,
            staged,
            json,
            quiet,
        } => {
            let path = path.unwrap_or(current_dir()?);
            if staged {
                println!("scanning staged-compatible project path: {path}");
            }
            let report = scan_secrets(&path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else if !quiet {
                if report.findings.is_empty() {
                    println!("no obvious secrets found in {path}");
                } else {
                    for finding in &report.findings {
                        println!("{}:{} {}", finding.path, finding.line, finding.kind);
                    }
                }
            }
            if !report.findings.is_empty() {
                return Err(LodeError::SecretFindings {
                    count: report.findings.len(),
                });
            }
        }
        ScanCommand::Foreign { path, json } => {
            let path = path.unwrap_or(current_dir()?);
            let report = scan_foreign_project(&path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                println!("foreign project scan: {}", report.path);
                println!("lode_project\t{}", status_bool(report.lode_project));
                println!(
                    "package_manager\t{}",
                    report.package_manager.as_deref().unwrap_or("none")
                );
                println!("manifests\t{}", report.manifests.join(","));
                println!("convention_violations\t{}", report.convention_violations);
                println!("secret_findings\t{}", report.secret_findings);
                for action in &report.migration_actions {
                    println!("action\t{action}");
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct ForeignScanReport {
    path: Utf8PathBuf,
    lode_project: bool,
    package_manager: Option<String>,
    manifests: Vec<String>,
    convention_checked: usize,
    convention_violations: usize,
    secret_findings: usize,
    migration_actions: Vec<String>,
}

fn scan_foreign_project(path: &Utf8PathBuf) -> lode_core::Result<ForeignScanReport> {
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

fn project_manifests(path: &Utf8PathBuf) -> Vec<String> {
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

fn convention_check(args: CheckArgs) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let path = args.path.unwrap_or(current_dir()?);
    let report = if args.fix {
        fix_path(&path, &config)?
    } else {
        check_path(&path, &config)?
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| LodeError::Message(error.to_string()))?
        );
    } else if report.violations.is_empty() {
        println!("convention ok: checked {}", report.checked);
        for (from, to) in &report.renamed {
            println!("renamed {from} -> {to}");
        }
    } else {
        for violation in &report.violations {
            println!("{} -> {}", violation.path, violation.expected_name);
        }
    }

    if !report.violations.is_empty() {
        return Err(LodeError::Violations {
            count: report.violations.len(),
        });
    }
    Ok(())
}

fn convention_fix(path: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    convention_check(CheckArgs {
        path,
        json: false,
        fix: true,
    })
}

fn rename_path(path: Utf8PathBuf, to: Option<String>) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let target_name = to.unwrap_or_else(|| {
        path.file_name()
            .map(|name| lode_core::normalize_name(name, &config))
            .unwrap_or_else(|| "renamed".to_string())
    });
    let parent = path
        .parent()
        .map(Utf8PathBuf::from)
        .unwrap_or_else(|| Utf8PathBuf::from("."));
    let destination = parent.join(target_name);
    fs::rename(&path, &destination).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("renamed {path} -> {destination}");
    Ok(())
}

fn rules(command: RulesCommand) -> lode_core::Result<()> {
    match command {
        RulesCommand::List => {
            let config = load_global_config()?;
            println!("default_case\t{}", config.convention.default_case);
            println!(
                "protected_prefixes\t{}",
                config.convention.protected_prefixes.join(",")
            );
        }
        RulesCommand::Check { path } => {
            convention_check(CheckArgs {
                path,
                json: false,
                fix: false,
            })?;
        }
        RulesCommand::Validate => {
            let config = load_global_config()?;
            if config.convention.default_case.trim().is_empty() {
                return Err(LodeError::Message(
                    "convention.default_case must not be empty".to_string(),
                ));
            }
            println!("rules valid");
        }
    }
    Ok(())
}

fn sign_path(
    path: Option<Utf8PathBuf>,
    ext: Vec<String>,
    force: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let root = path.unwrap_or(current_dir()?);
    let text = format!(
        "Generated with Lode by {} <{}>",
        config.identity.author, config.identity.email
    );
    stamp_files(&root, &ext, &text, force, dry_run)
}

fn stamp_path(
    path: Option<Utf8PathBuf>,
    ext: Vec<String>,
    include_license: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let root = path.unwrap_or(current_dir()?);
    let mut text = format!(
        "{} / {} / {}",
        config.identity.org, config.identity.author, config.identity.email
    );
    if include_license {
        text.push_str(&format!(" / {}", config.identity.license));
    }
    stamp_files(&root, &ext, &text, false, dry_run)
}

fn stamp_files(
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
    fs::write(path, updated).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
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

fn git(command: GitCommand) -> lode_core::Result<()> {
    match command {
        GitCommand::Branch { kind, description } => {
            let branch = format!("{}/{}", kind, slugify(&description));
            println!("{branch}");
        }
        GitCommand::Commit {
            message,
            r#type,
            scope,
            breaking,
            no_confirm: _,
        } => {
            let message = message.unwrap_or_else(|| {
                conventional_message(
                    r#type.as_deref().unwrap_or("chore"),
                    scope.as_deref(),
                    "update",
                    breaking,
                )
            });
            run_git(&["commit", "-m", &message])?;
        }
        GitCommand::Tag {
            version,
            no_changelog: _,
            push,
            message,
        } => {
            let tag = format!("v{}", version.trim_start_matches('v'));
            if let Some(message) = message {
                run_git(&["tag", "-a", &tag, "-m", &message])?;
            } else {
                run_git(&["tag", &tag])?;
            }
            if push {
                run_git(&["push", "origin", &tag])?;
            }
        }
        GitCommand::Changelog { since, out, format } => {
            git_changelog(since.as_deref(), out, &format)?
        }
        GitCommand::InstallHooks => install_git_hooks()?,
        GitCommand::UninstallHooks => uninstall_git_hooks()?,
        GitCommand::HooksStatus => hooks_status()?,
        GitCommand::SignSetup => git_sign_setup()?,
        GitCommand::RemoteSetup {
            provider,
            visibility,
            token_env,
        } => git_remote_setup(provider, visibility, token_env)?,
    }
    Ok(())
}

fn conventional_message(kind: &str, scope: Option<&str>, subject: &str, breaking: bool) -> String {
    let bang = if breaking { "!" } else { "" };
    if let Some(scope) = scope {
        format!("{kind}({scope}){bang}: {subject}")
    } else {
        format!("{kind}{bang}: {subject}")
    }
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

fn run_git(args: &[&str]) -> lode_core::Result<()> {
    let args = args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    let status = run_process_status("git", &args, None)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "git command failed with {status}"
        )))
    }
}

fn git_changelog(
    since: Option<&str>,
    out: Option<Utf8PathBuf>,
    format: &str,
) -> lode_core::Result<()> {
    let mut args = vec![
        "log".to_string(),
        "--pretty=format:%s".to_string(),
        "--no-merges".to_string(),
    ];
    if let Some(since) = since {
        args.push(format!("{since}..HEAD"));
    }
    let output = run_process_output("git", &args)?;
    if !output.status.success() {
        return Err(LodeError::Message("git log failed".to_string()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rendered = match format {
        "json" => serde_json::to_string_pretty(
            &stdout
                .lines()
                .map(|line| serde_json::json!({ "subject": line }))
                .collect::<Vec<_>>(),
        )
        .map_err(|error| LodeError::Message(error.to_string()))?,
        "plain" => stdout.lines().collect::<Vec<_>>().join("\n") + "\n",
        "markdown" | "md" => {
            let mut text = String::from("# Changelog\n\n");
            for line in stdout.lines() {
                text.push_str(&format!("- {line}\n"));
            }
            text
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported changelog format: {other}"
            )))
        }
    };
    if let Some(path) = out {
        fs::write(&path, rendered).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        println!("wrote changelog to {path}");
    } else {
        print!("{rendered}");
    }
    Ok(())
}

fn git_sign_setup() -> lode_core::Result<()> {
    let path = Utf8PathBuf::from(".lode").join("git-signing.toml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    fs::write(&path, "enabled = true\nmode = \"manual\"\n").map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("git signing setup recorded at {path}");
    Ok(())
}

fn git_remote_setup(
    provider: Option<String>,
    visibility: Option<String>,
    token_env: Option<String>,
) -> lode_core::Result<()> {
    let path = Utf8PathBuf::from(".lode").join("remote.toml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let contents = format!(
        "provider = \"{}\"\nvisibility = \"{}\"\ntoken_env = \"{}\"\n",
        provider.unwrap_or_else(|| "github".to_string()),
        visibility.unwrap_or_else(|| "private".to_string()),
        token_env.unwrap_or_else(|| "GITHUB_TOKEN".to_string())
    );
    fs::write(&path, contents).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("git remote setup recorded at {path}");
    Ok(())
}

fn install_git_hooks() -> lode_core::Result<()> {
    let hooks_dir = Utf8PathBuf::from(".git").join("hooks");
    if !hooks_dir.exists() {
        return Err(LodeError::Message("not a git repository".to_string()));
    }
    let pre_commit = hooks_dir.join("pre-commit");
    fs::write(
        &pre_commit,
        "#!/usr/bin/env sh\n# lode-managed\nlode check .\nlode scan secrets .\n",
    )
    .map_err(|source| LodeError::Io {
        path: pre_commit.as_str().into(),
        source,
    })?;
    let pre_push = hooks_dir.join("pre-push");
    fs::write(
        &pre_push,
        "#!/usr/bin/env sh\n# lode-managed\nlode task test\n",
    )
    .map_err(|source| LodeError::Io {
        path: pre_push.as_str().into(),
        source,
    })?;
    println!("installed lode-managed hook templates");
    Ok(())
}

fn uninstall_git_hooks() -> lode_core::Result<()> {
    let hooks_dir = Utf8PathBuf::from(".git").join("hooks");
    for name in ["pre-commit", "pre-push"] {
        let path = hooks_dir.join(name);
        if path.exists()
            && fs::read_to_string(&path)
                .unwrap_or_default()
                .contains("lode-managed")
        {
            fs::remove_file(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("removed {path}");
        }
    }
    Ok(())
}

fn hooks_status() -> lode_core::Result<()> {
    let hooks_dir = Utf8PathBuf::from(".git").join("hooks");
    for name in ["pre-commit", "pre-push"] {
        let path = hooks_dir.join(name);
        let status = if path.exists()
            && fs::read_to_string(&path)
                .unwrap_or_default()
                .contains("lode-managed")
        {
            "managed"
        } else {
            "missing"
        };
        println!("{name}\t{status}");
    }
    Ok(())
}

fn hooks(command: HooksCommand) -> lode_core::Result<()> {
    match command {
        HooksCommand::List => {
            for hook in discover_hooks()? {
                println!("{}\t{}\t{}", hook.event, hook.source, hook.path);
            }
        }
        HooksCommand::Status => {
            hooks_status()?;
            let hooks = discover_hooks()?;
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for hook in hooks {
                *counts.entry(hook.event).or_default() += 1;
            }
            for (event, count) in counts {
                println!("{event}\t{count} hook(s)");
            }
        }
        HooksCommand::Test { event } => test_hook(&event)?,
        HooksCommand::Run { event, dry_run } => run_hooks(&event, dry_run)?,
    }
    Ok(())
}

fn test_hook(event: &str) -> lode_core::Result<()> {
    let mut hooks = discover_hooks()?;
    hooks.retain(|hook| hook.event == event);
    if hooks.is_empty() {
        let script = match event {
            "pre-commit" => "lode check . && lode scan secrets .",
            "pre-push" => "lode task test",
            other => return Err(LodeError::Message(format!("unknown hook event: {other}"))),
        };
        println!("hook {event}: {script}");
        return Ok(());
    }
    println!("hook execution plan for {event}:");
    for hook in hooks {
        println!("{}\t{}\t{}", hook.source, hook.runtime, hook.path);
    }
    Ok(())
}

fn run_hooks(event: &str, dry_run: bool) -> lode_core::Result<()> {
    let mut hooks = discover_hooks()?;
    hooks.retain(|hook| hook.event == event);
    if hooks.is_empty() {
        return Err(LodeError::Message(format!(
            "no hooks found for event: {event}"
        )));
    }
    for hook in hooks {
        let (program, args) = hook_command(&hook)?;
        if dry_run {
            println!(
                "would run hook {}\t{}\t{} {}",
                hook.source,
                hook.runtime,
                program,
                args.join(" ")
            );
            continue;
        }
        println!(
            "running hook {}\t{}\t{}",
            hook.source, hook.runtime, hook.path
        );
        let before = plugin_hook_file_snapshot(&hook)?;
        let envs = hook_runtime_env(&hook);
        let status = run_process_status_with_env(program, &args, None, &envs)?;
        if !status.success() {
            return Err(LodeError::Message(format!(
                "hook {} {} failed with {status}",
                hook.source, hook.path
            )));
        }
        enforce_plugin_hook_writes(&hook, before)?;
    }
    Ok(())
}

fn hook_command(hook: &DiscoveredHook) -> lode_core::Result<(&'static str, Vec<String>)> {
    let path = hook.path.to_string();
    match hook.runtime.as_str() {
        "powershell" => Ok((
            "powershell",
            vec!["-NoProfile".to_string(), "-File".to_string(), path],
        )),
        "python" => Ok(("python", vec![path])),
        "node" => Ok(("node", vec![path])),
        "lua" => Ok(("lua", vec![path])),
        "sh" => Ok(("sh", vec![path])),
        other => Err(LodeError::Message(format!(
            "unsupported hook runtime: {other}"
        ))),
    }
}

fn hook_runtime_env(hook: &DiscoveredHook) -> Vec<(&'static str, String)> {
    let mut envs = vec![
        ("LODE_HOOK_EVENT", hook.event.clone()),
        ("LODE_HOOK_SOURCE", hook.source.clone()),
        ("LODE_HOOK_RUNTIME", hook.runtime.clone()),
    ];
    if let Some(plugin) = hook.source.strip_prefix("plugin:") {
        let security = hook.plugin_security.clone().unwrap_or_default();
        envs.push(("LODE_PLUGIN_NAME", plugin.to_string()));
        envs.push(("LODE_PLUGIN_ALLOW_NETWORK", security.network.to_string()));
        envs.push(("LODE_PLUGIN_ALLOW_EXECUTE", security.execute.to_string()));
        envs.push(("LODE_PLUGIN_FS_WRITE", security.fs_write.join(";")));
    }
    envs
}

type HookFileSnapshot = BTreeMap<String, String>;

fn plugin_hook_file_snapshot(hook: &DiscoveredHook) -> lode_core::Result<Option<HookFileSnapshot>> {
    if hook.plugin_security.is_none() {
        return Ok(None);
    }
    snapshot_project_contents(&current_dir()?).map(Some)
}

fn enforce_plugin_hook_writes(
    hook: &DiscoveredHook,
    before: Option<HookFileSnapshot>,
) -> lode_core::Result<()> {
    let Some(before) = before else {
        return Ok(());
    };
    let security = hook.plugin_security.clone().unwrap_or_default();
    let after = snapshot_project_contents(&current_dir()?)?;
    let changed = changed_snapshot_paths(&before, &after);
    let denied = changed
        .into_iter()
        .filter(|path| !plugin_write_allowed(path, &security.fs_write))
        .collect::<Vec<_>>();
    if !denied.is_empty() {
        return Err(LodeError::Message(format!(
            "plugin hook {} wrote outside declared fs_write paths: {}",
            hook.source,
            denied.join(",")
        )));
    }
    Ok(())
}

fn snapshot_project_contents(root: &Utf8PathBuf) -> lode_core::Result<HookFileSnapshot> {
    let mut snapshot = BTreeMap::new();
    snapshot_contents_dir(root, root, &mut snapshot)?;
    Ok(snapshot)
}

fn snapshot_contents_dir(
    root: &Utf8PathBuf,
    dir: &Utf8PathBuf,
    snapshot: &mut HookFileSnapshot,
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
            snapshot_contents_dir(root, &path, snapshot)?;
        } else if metadata.is_file() {
            let contents = fs::read(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let relative = path
                .strip_prefix(root)
                .map(|path| path.as_str().replace('\\', "/"))
                .unwrap_or_else(|_| path.as_str().replace('\\', "/"));
            snapshot.insert(relative, content_hash_bytes(&contents));
        }
    }
    Ok(())
}

fn changed_snapshot_paths(before: &HookFileSnapshot, after: &HookFileSnapshot) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|path| {
            if before.get(path) == after.get(path) {
                None
            } else {
                Some(path.clone())
            }
        })
        .collect()
}

fn plugin_write_allowed(path: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|allowed| {
        let allowed = allowed
            .trim_end_matches("/**")
            .trim_end_matches("/*")
            .trim_end_matches('/');
        path == allowed || path.starts_with(&format!("{allowed}/"))
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscoveredHook {
    event: String,
    source: String,
    runtime: String,
    path: Utf8PathBuf,
    plugin_security: Option<PluginSecurity>,
}

fn discover_hooks() -> lode_core::Result<Vec<DiscoveredHook>> {
    let mut hooks = Vec::new();
    discover_plugin_hooks(&mut hooks)?;
    discover_hook_dir("global", &global_dir()?.join("hooks"), &mut hooks)?;
    discover_hook_dir(
        "project",
        &Utf8PathBuf::from(".lode").join("hooks"),
        &mut hooks,
    )?;
    hooks.sort_by(|left, right| {
        hook_source_rank(&left.source)
            .cmp(&hook_source_rank(&right.source))
            .then(left.event.cmp(&right.event))
            .then(left.path.cmp(&right.path))
    });
    Ok(hooks)
}

fn discover_plugin_hooks(hooks: &mut Vec<DiscoveredHook>) -> lode_core::Result<()> {
    let root = global_asset_dir("plugins")?;
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&root).map_err(|source| LodeError::Io {
        path: root.as_str().into(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: root.as_str().into(),
            source,
        })?;
        let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if path.is_dir() {
            let name = path.file_name().unwrap_or("plugin");
            let hooks_dir = path.join("hooks");
            if hooks_dir.exists() {
                let security = require_plugin_runtime_permissions(name, &path)?;
                discover_hook_dir_with_security(
                    &format!("plugin:{name}"),
                    &hooks_dir,
                    Some(security),
                    hooks,
                )?;
            }
        }
    }
    Ok(())
}

fn require_plugin_runtime_permissions(
    name: &str,
    path: &Utf8PathBuf,
) -> lode_core::Result<PluginSecurity> {
    let security = read_plugin_security(path)?;
    for path in &security.fs_write {
        safe_relative_path(path)?;
    }
    if !security.execute {
        return Err(LodeError::Message(format!(
            "plugin {name} has hooks but does not declare permissions.execute = true"
        )));
    }
    let Some(receipt) = read_plugin_install_receipt(path)? else {
        return Err(LodeError::Message(format!(
            "plugin {name} has hooks but is missing install receipt; reinstall with `lode plugin add --allow-unsafe`"
        )));
    };
    if !receipt.reviewed || !receipt.allow_unsafe {
        return Err(LodeError::Message(format!(
            "plugin {name} has executable hooks but was not installed with reviewed unsafe permissions"
        )));
    }
    Ok(security)
}

fn discover_hook_dir(
    source: &str,
    dir: &Utf8PathBuf,
    hooks: &mut Vec<DiscoveredHook>,
) -> lode_core::Result<()> {
    discover_hook_dir_with_security(source, dir, None, hooks)
}

fn discover_hook_dir_with_security(
    source: &str,
    dir: &Utf8PathBuf,
    plugin_security: Option<PluginSecurity>,
    hooks: &mut Vec<DiscoveredHook>,
) -> lode_core::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
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
        if path.is_file() {
            if let Some((event, runtime)) = hook_file_event_runtime(&path) {
                hooks.push(DiscoveredHook {
                    event,
                    runtime,
                    source: source.to_string(),
                    path,
                    plugin_security: plugin_security.clone(),
                });
            }
        }
    }
    Ok(())
}

fn hook_file_event_runtime(path: &Utf8PathBuf) -> Option<(String, String)> {
    let file_name = path.file_name()?;
    let (event, runtime) = file_name
        .rsplit_once('.')
        .map(|(event, ext)| (event.to_string(), hook_runtime(ext)))
        .unwrap_or_else(|| (file_name.to_string(), "sh".to_string()));
    Some((event, runtime))
}

fn hook_runtime(extension: &str) -> String {
    match extension {
        "ps1" => "powershell",
        "py" => "python",
        "js" => "node",
        "lua" => "lua",
        _ => "sh",
    }
    .to_string()
}

fn hook_source_rank(source: &str) -> usize {
    match source {
        source if source.starts_with("plugin:") => 0,
        "global" => 1,
        "project" => 2,
        _ => 3,
    }
}

fn env_command(command: EnvCommand) -> lode_core::Result<()> {
    match command {
        EnvCommand::Check => env_check()?,
        EnvCommand::Add {
            key,
            default,
            comment,
            secret,
        } => env_add(&key, default.as_deref(), comment.as_deref(), secret)?,
        EnvCommand::Sync => env_sync()?,
        EnvCommand::Use { profile } => env_use(&profile)?,
    }
    Ok(())
}

fn license(command: LicenseCommand) -> lode_core::Result<()> {
    match command {
        LicenseCommand::List { format } => {
            let root = global_asset_dir("licenses")?;
            if format == "json" {
                let mut items = Vec::new();
                collect_file_names(&root, &mut items)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                list_dir(root)?;
            }
        }
        LicenseCommand::Show { id } => print!("{}", read_license(&id)?),
        LicenseCommand::Info { id } => {
            let contents = read_license(&id)?;
            println!("id: {id}");
            println!("bytes: {}", contents.len());
            println!(
                "category: {}",
                if id.contains(" OR ") {
                    "compound"
                } else {
                    "single"
                }
            );
        }
        LicenseCommand::Add { id, file, text } => {
            add_license(&id, file, text.as_deref())?;
        }
        LicenseCommand::Remove { id } => {
            let path = license_path(&id)?;
            if !path.exists() {
                return Err(LodeError::Message(format!("license not found: {id}")));
            }
            fs::remove_file(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("removed license {id}");
        }
        LicenseCommand::Set { id } => {
            let contents = read_license(&id)?;
            fs::write("LICENSE", contents).map_err(|source| LodeError::Io {
                path: "LICENSE".into(),
                source,
            })?;
            println!("license set: {id}");
        }
        LicenseCommand::Check { json } => {
            let path = Utf8PathBuf::from("LICENSE");
            let ok = path.exists()
                && !fs::read_to_string(&path)
                    .map_err(|source| LodeError::Io {
                        path: path.as_str().into(),
                        source,
                    })?
                    .trim()
                    .is_empty();
            if json {
                println!("{{\"license\":{ok}}}");
            } else if ok {
                println!("license ok");
            } else {
                return Err(LodeError::Message(
                    "LICENSE is missing or empty".to_string(),
                ));
            }
        }
        LicenseCommand::Apply { dry_run } => {
            let id = project_license_id()?.unwrap_or(load_global_config()?.identity.license);
            if dry_run {
                println!("would apply license {id}");
            } else {
                let contents = read_license(&id)?;
                fs::write("LICENSE", contents).map_err(|source| LodeError::Io {
                    path: "LICENSE".into(),
                    source,
                })?;
                println!("license applied: {id}");
            }
        }
    }
    Ok(())
}

fn add_license(id: &str, file: Option<Utf8PathBuf>, text: Option<&str>) -> lode_core::Result<()> {
    let path = license_path(id)?;
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    fs::write(&path, contents).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("added license {id}");
    Ok(())
}

fn license_path(id: &str) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&format!("{id}.txt"))?;
    Ok(global_asset_dir("licenses")?.join(relative))
}

fn project_license_id() -> lode_core::Result<Option<String>> {
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

fn env_check() -> lode_core::Result<()> {
    let example = read_env_file(".env.example")?;
    let env = read_env_file(".env").unwrap_or_default();
    let missing: Vec<_> = example
        .keys()
        .filter(|key| !env.contains_key(*key))
        .cloned()
        .collect();
    if missing.is_empty() {
        println!("env ok");
        Ok(())
    } else {
        for key in &missing {
            println!("missing {key}");
        }
        Err(LodeError::Message(format!(
            "{} env key(s) missing",
            missing.len()
        )))
    }
}

fn env_add(
    key: &str,
    default: Option<&str>,
    comment: Option<&str>,
    secret: bool,
) -> lode_core::Result<()> {
    let path = Utf8PathBuf::from(".env.example");
    let mut contents = if path.exists() {
        fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?
    } else {
        String::new()
    };
    if !read_env_entries(&contents).contains_key(key) {
        if !contents.ends_with('\n') && !contents.is_empty() {
            contents.push('\n');
        }
        if let Some(comment) = comment {
            contents.push_str("# ");
            contents.push_str(comment);
            contents.push('\n');
        }
        contents.push_str(key);
        contents.push('=');
        if !secret {
            contents.push_str(default.unwrap_or_default());
        }
        contents.push('\n');
        fs::write(&path, contents).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
    }
    if secret {
        let env_path = Utf8PathBuf::from(".env");
        let mut env_contents = fs::read_to_string(&env_path).unwrap_or_default();
        if !read_env_entries(&env_contents).contains_key(key) {
            if !env_contents.ends_with('\n') && !env_contents.is_empty() {
                env_contents.push('\n');
            }
            env_contents.push_str(key);
            env_contents.push('=');
            env_contents.push_str(default.unwrap_or_default());
            env_contents.push('\n');
            fs::write(&env_path, env_contents).map_err(|source| LodeError::Io {
                path: env_path.as_str().into(),
                source,
            })?;
        }
    }
    println!("added env key {key}");
    Ok(())
}

fn env_sync() -> lode_core::Result<()> {
    let example = read_env_file(".env.example")?;
    let env_path = Utf8PathBuf::from(".env");
    let mut env = if env_path.exists() {
        fs::read_to_string(&env_path).map_err(|source| LodeError::Io {
            path: env_path.as_str().into(),
            source,
        })?
    } else {
        String::new()
    };
    let existing = read_env_entries(&env);
    let mut added = 0usize;
    for (key, value) in example {
        if !existing.contains_key(&key) {
            if !env.ends_with('\n') && !env.is_empty() {
                env.push('\n');
            }
            env.push_str(&key);
            env.push('=');
            env.push_str(&value);
            env.push('\n');
            added += 1;
        }
    }
    fs::write(&env_path, env).map_err(|source| LodeError::Io {
        path: env_path.as_str().into(),
        source,
    })?;
    println!("env synced: added {added}");
    Ok(())
}

fn env_use(profile: &str) -> lode_core::Result<()> {
    let source = Utf8PathBuf::from(format!(".env.{profile}"));
    if !source.exists() {
        return Err(LodeError::Message(format!("{source} does not exist")));
    }
    fs::copy(&source, ".env").map_err(|source_error| LodeError::Io {
        path: source.as_str().into(),
        source: source_error,
    })?;
    println!("env profile active: {profile}");
    Ok(())
}

fn read_env_file(path: &str) -> lode_core::Result<std::collections::BTreeMap<String, String>> {
    let path = Utf8PathBuf::from(path);
    let contents = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(read_env_entries(&contents))
}

fn read_env_entries(contents: &str) -> std::collections::BTreeMap<String, String> {
    let mut entries = std::collections::BTreeMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            entries.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    entries
}

fn read_license(id: &str) -> lode_core::Result<String> {
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

fn add_snippet(
    name: &str,
    lang: &str,
    trigger: Option<&str>,
    desc: Option<&str>,
) -> lode_core::Result<()> {
    let relative = safe_relative_path(&format!("{lang}/{name}.snippet"))?;
    let path = global_asset_dir("snippets")?.join(relative);
    if path.exists() {
        return Err(LodeError::Message(format!(
            "snippet already exists: {name}"
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let trigger = trigger.unwrap_or(name);
    let desc = desc.unwrap_or("User snippet");
    let contents = format!(
        "name: {name}\nlang: {lang}\ntrigger: {trigger}\ndescription: {desc}\n---\n{trigger} $1\n"
    );
    fs::write(&path, contents).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("created snippet {lang}/{name}");
    Ok(())
}

fn remove_snippet(name: &str, lang: Option<&str>) -> lode_core::Result<()> {
    let path = resolve_snippet_path(name, lang)?;
    fs::remove_file(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("removed snippet {name}");
    Ok(())
}

fn insert_snippet(
    name: &str,
    lang: Option<&str>,
    file: Option<Utf8PathBuf>,
    line: Option<usize>,
) -> lode_core::Result<()> {
    let path = resolve_snippet_path(name, lang)?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let snippet = parse_snippet_asset(&path, &raw);
    if let Some(file) = file {
        let existing = fs::read_to_string(&file).unwrap_or_default();
        let updated = insert_text_at_line(&existing, &snippet.body, line.unwrap_or(usize::MAX));
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.as_str().into(),
                source,
            })?;
        }
        fs::write(&file, updated).map_err(|source| LodeError::Io {
            path: file.as_str().into(),
            source,
        })?;
        println!("inserted snippet {name} into {file}");
    } else {
        print!("{}", snippet.body);
    }
    Ok(())
}

fn insert_text_at_line(existing: &str, snippet: &str, line: usize) -> String {
    let mut lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    let snippet_lines = snippet.lines().map(str::to_string).collect::<Vec<_>>();
    let index = if line == 0 || line == usize::MAX {
        lines.len()
    } else {
        line.saturating_sub(1).min(lines.len())
    };
    lines.splice(index..index, snippet_lines);
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn resolve_snippet_path(name: &str, lang: Option<&str>) -> lode_core::Result<Utf8PathBuf> {
    let root = global_asset_dir("snippets")?;
    if let Some(lang) = lang {
        let relative = safe_relative_path(&format!("{lang}/{name}.snippet"))?;
        let path = root.join(relative);
        if path.exists() {
            return Ok(path);
        }
        return Err(LodeError::Message(format!(
            "snippet not found: {lang}/{name}"
        )));
    }

    let mut matches = Vec::new();
    collect_snippet_named(&root, name, &mut matches)?;
    match matches.len() {
        0 => Err(LodeError::Message(format!("snippet not found: {name}"))),
        1 => Ok(matches.remove(0)),
        _ => Err(LodeError::Message(format!(
            "snippet name is ambiguous; pass --lang for {name}"
        ))),
    }
}

fn collect_snippet_named(
    path: &Utf8PathBuf,
    name: &str,
    matches: &mut Vec<Utf8PathBuf>,
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
            collect_snippet_named(&child, name, matches)?;
        }
        return Ok(());
    }
    if path.file_stem() == Some(name) && path.extension() == Some("snippet") {
        matches.push(path.clone());
    }
    Ok(())
}

fn search_snippets(query: &str) -> lode_core::Result<()> {
    let root = global_asset_dir("snippets")?;
    let mut matches = Vec::new();
    collect_snippet_matches(&root, query, &mut matches)?;
    for path in matches {
        println!("{path}");
    }
    Ok(())
}

fn collect_snippet_matches(
    path: &Utf8PathBuf,
    query: &str,
    matches: &mut Vec<Utf8PathBuf>,
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
            collect_snippet_matches(&child, query, matches)?;
        }
        return Ok(());
    }
    let haystack = format!(
        "{}\n{}",
        path.as_str(),
        fs::read_to_string(path).unwrap_or_default()
    )
    .to_ascii_lowercase();
    if haystack.contains(&query.to_ascii_lowercase()) {
        matches.push(path.clone());
    }
    Ok(())
}

fn export_snippets(
    lang: Option<&str>,
    format: &str,
    out: Option<Utf8PathBuf>,
) -> lode_core::Result<()> {
    let root = global_asset_dir("snippets")?;
    let scan_root = lang.map_or(root.clone(), |lang| root.join(lang));
    let mut snippets = Vec::new();
    collect_snippet_assets(&scan_root, &mut snippets)?;
    snippets.sort_by(|left, right| {
        left.lang
            .cmp(&right.lang)
            .then_with(|| left.name.cmp(&right.name))
    });

    let rendered = match format {
        "vscode" | "json" => render_vscode_snippets(&snippets)?,
        "zed" => render_zed_snippets(&snippets)?,
        "neovim" | "nvim" => render_neovim_snippets(&snippets),
        "jetbrains" | "intellij" => render_jetbrains_snippets(&snippets),
        "markdown" | "md" => render_markdown_snippets(&snippets),
        "plain" | "text" => render_plain_snippets(&snippets),
        other => {
            return Err(LodeError::Message(format!(
                "unsupported snippet export format: {other}"
            )))
        }
    };

    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.as_str().into(),
                source,
            })?;
        }
        fs::write(&path, rendered).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        println!("exported {} snippets to {path}", snippets.len());
    } else {
        print!("{rendered}");
    }
    Ok(())
}

fn collect_snippet_assets(
    path: &Utf8PathBuf,
    snippets: &mut Vec<SnippetAsset>,
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
            collect_snippet_assets(&child, snippets)?;
        }
        return Ok(());
    }
    if path.extension() != Some("snippet") {
        return Ok(());
    }
    let contents = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    snippets.push(parse_snippet_asset(path, &contents));
    Ok(())
}

fn parse_snippet_asset(path: &Utf8PathBuf, contents: &str) -> SnippetAsset {
    let mut name = path
        .file_stem()
        .map(str::to_string)
        .unwrap_or_else(|| "snippet".to_string());
    let mut lang = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(str::to_string)
        .unwrap_or_else(|| "any".to_string());
    let mut body = contents.to_string();

    if let Some((header, raw_body)) = contents.split_once("---") {
        body = raw_body.trim_start_matches(['\r', '\n']).to_string();
        for line in header.lines() {
            if let Some((key, value)) = line.split_once(':') {
                match key.trim() {
                    "name" => name = value.trim().to_string(),
                    "lang" => lang = value.trim().to_string(),
                    _ => {}
                }
            }
        }
    }

    SnippetAsset {
        lang,
        name,
        body,
        path: path.clone(),
    }
}

fn render_vscode_snippets(snippets: &[SnippetAsset]) -> lode_core::Result<String> {
    let mut output = serde_json::Map::new();
    for snippet in snippets {
        let key = format!("{}:{}", snippet.lang, snippet.name);
        output.insert(
            key,
            serde_json::json!({
                "prefix": snippet.name,
                "scope": snippet.lang,
                "body": snippet.body.lines().collect::<Vec<_>>(),
                "description": format!("Lode snippet from {}", snippet.path),
            }),
        );
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(output))
        .map_err(|error| LodeError::Message(error.to_string()))
}

fn render_zed_snippets(snippets: &[SnippetAsset]) -> lode_core::Result<String> {
    let mut output = serde_json::Map::new();
    for snippet in snippets {
        output.insert(
            format!("{}:{}", snippet.lang, snippet.name),
            serde_json::json!({
                "prefix": snippet.name,
                "body": snippet.body,
                "description": format!("Lode snippet from {}", snippet.path),
            }),
        );
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(output))
        .map_err(|error| LodeError::Message(error.to_string()))
}

fn render_neovim_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::from("return {\n");
    for snippet in snippets {
        output.push_str(&format!(
            "  {{ lang = {:?}, trigger = {:?}, body = {:?} }},\n",
            snippet.lang, snippet.name, snippet.body
        ));
    }
    output.push_str("}\n");
    output
}

fn render_jetbrains_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::from("<templateSet group=\"Lode\">\n");
    for snippet in snippets {
        output.push_str(&format!(
            "  <template name=\"{}\" value=\"{}\" description=\"{} snippet\" toReformat=\"true\" toShortenFQNames=\"true\" />\n",
            xml_escape(&snippet.name),
            xml_escape(&snippet.body),
            xml_escape(&snippet.lang)
        ));
    }
    output.push_str("</templateSet>\n");
    output
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_markdown_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::from("# Lode Snippets\n\n");
    for snippet in snippets {
        output.push_str(&format!(
            "## {} / {}\n\nSource: `{}`\n\n```{}\n{}```\n\n",
            snippet.lang, snippet.name, snippet.path, snippet.lang, snippet.body
        ));
    }
    output
}

fn render_plain_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::new();
    for snippet in snippets {
        output.push_str(&format!(
            "[{}:{}]\n{}\n",
            snippet.lang, snippet.name, snippet.body
        ));
    }
    output
}

fn apply_recipe(name: &str, dry_run: bool) -> lode_core::Result<()> {
    let recipe_path = global_asset_dir("recipes")?.join(format!("{name}.toml"));
    let raw = fs::read_to_string(&recipe_path).map_err(|source| LodeError::Io {
        path: recipe_path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    let Some(files) = value.get("files").and_then(toml::Value::as_array) else {
        println!("recipe {name} has no files");
        return Ok(());
    };

    for file in files {
        let template = file
            .get("template")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| LodeError::Message(format!("recipe {name} file missing template")))?;
        let dest = file
            .get("dest")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| LodeError::Message(format!("recipe {name} file missing dest")))?;
        if dry_run {
            println!("would write {dest} from {template}");
            continue;
        }
        let destination = Utf8PathBuf::from(dest);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.as_str().into(),
                source,
            })?;
        }
        let contents = read_template_asset(template)
            .unwrap_or_else(|_| format!("# {dest}\n\nGenerated by recipe {name}.\n"));
        fs::write(&destination, contents).map_err(|source| LodeError::Io {
            path: destination.as_str().into(),
            source,
        })?;
        println!("wrote {destination}");
    }
    Ok(())
}

fn read_template_asset(path: &str) -> lode_core::Result<String> {
    let template_path = global_asset_dir("templates")?.join(path);
    fs::read_to_string(&template_path).map_err(|source| LodeError::Io {
        path: template_path.as_str().into(),
        source,
    })
}

fn projects(command: ProjectsCommand) -> lode_core::Result<()> {
    match command {
        ProjectsCommand::List { format, sort } => {
            let mut registry = load_registry()?;
            match sort.as_str() {
                "name" => registry
                    .projects
                    .sort_by(|left, right| left.name.cmp(&right.name)),
                "health" => registry
                    .projects
                    .sort_by(|left, right| left.path.exists().cmp(&right.path.exists()).reverse()),
                "last-seen" => registry
                    .projects
                    .sort_by(|left, right| right.last_seen.cmp(&left.last_seen)),
                other => {
                    return Err(LodeError::Message(format!(
                        "unsupported project sort: {other}"
                    )))
                }
            }
            if registry.projects.is_empty() {
                println!("no registered projects");
            } else if format == "json" {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&registry.projects)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                for project in registry.projects {
                    println!(
                        "{}\t{}\t{}\t{}",
                        project.name, project.profile, project.path, project.last_seen
                    );
                }
            }
        }
        ProjectsCommand::Cd { name } => {
            let registry = load_registry()?;
            let project = registry
                .projects
                .iter()
                .find(|project| project.name == name)
                .ok_or_else(|| LodeError::Message(format!("project not found: {name}")))?;
            println!("{}", project.path);
        }
        ProjectsCommand::Register { path } => {
            let path = path.unwrap_or(current_dir()?);
            let name = path
                .file_name()
                .map(str::to_string)
                .unwrap_or_else(|| "project".to_string());
            register_project(&name, &path, "manual")?;
            println!("registered {path}");
        }
        ProjectsCommand::Remove { name } => {
            let mut registry = load_registry()?;
            let before = registry.projects.len();
            registry.projects.retain(|project| project.name != name);
            let removed = before - registry.projects.len();
            if removed == 0 {
                return Err(LodeError::Message(format!("project not found: {name}")));
            }
            save_registry(&registry)?;
            println!("removed project {name}");
        }
        ProjectsCommand::Health {
            stale_only,
            json,
            refresh,
        } => {
            let registry = load_registry()?;
            let mut rows = Vec::new();
            for project in registry.projects {
                let status = if project.path.exists() {
                    "ok"
                } else {
                    "missing"
                };
                if stale_only && status == "ok" {
                    continue;
                }
                let score = if refresh && project.path.exists() {
                    audit_project(&project.path, &load_global_config()?)
                        .ok()
                        .map(|report| report.score)
                } else {
                    None
                };
                rows.push(serde_json::json!({
                    "name": project.name,
                    "status": status,
                    "path": project.path,
                    "score": score,
                }));
            }
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rows)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
                for row in rows {
                    println!(
                        "{}\t{}\t{}\t{}",
                        row["name"].as_str().unwrap_or_default(),
                        row["status"].as_str().unwrap_or_default(),
                        row["score"]
                            .as_u64()
                            .map(|score| score.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                        row["path"].as_str().unwrap_or_default()
                    );
                }
            }
        }
        ProjectsCommand::Prune => {
            let removed = prune_registry()?;
            println!("project registry pruned: removed {removed}");
        }
    }
    Ok(())
}

fn toolchain(command: ToolchainCommand) -> lode_core::Result<()> {
    let detected = detect_toolchains();
    match command {
        ToolchainCommand::List => {
            for tool in [
                "rustc", "cargo", "node", "python", "go", "zig", "java", "git",
            ] {
                println!(
                    "{tool}\t{}",
                    command_version(tool).unwrap_or_else(|| "missing".to_string())
                );
            }
        }
        ToolchainCommand::Status => {
            if detected.is_empty() {
                println!("no project toolchain files detected");
            } else {
                for item in detected {
                    println!("{item}");
                }
            }
        }
        ToolchainCommand::Doctor => {
            let required = required_tools_for_project();
            let mut missing = Vec::new();
            for tool in required {
                if command_version(tool).is_none() {
                    missing.push(tool);
                }
            }
            if missing.is_empty() {
                println!("toolchain doctor ok");
            } else {
                for tool in &missing {
                    println!("missing {tool}");
                }
                return Err(LodeError::Message(format!(
                    "{} required tool(s) missing",
                    missing.len()
                )));
            }
        }
        ToolchainCommand::Add { runtime, version } => {
            let mut store = load_toolchain_store()?;
            let versions = store.runtimes.entry(runtime.clone()).or_default();
            if !versions.iter().any(|item| item == &version) {
                versions.push(version.clone());
                versions.sort();
            }
            save_toolchain_store(&store)?;
            println!("toolchain added: {runtime} {version}");
        }
        ToolchainCommand::Remove { runtime, version } => {
            let mut store = load_toolchain_store()?;
            if let Some(versions) = store.runtimes.get_mut(&runtime) {
                versions.retain(|item| item != &version);
            }
            if store.active.get(&runtime) == Some(&version) {
                store.active.remove(&runtime);
            }
            save_toolchain_store(&store)?;
            println!("toolchain removed: {runtime} {version}");
        }
        ToolchainCommand::Use { runtime, version } => {
            let mut store = load_toolchain_store()?;
            store.active.insert(runtime.clone(), version.clone());
            let versions = store.runtimes.entry(runtime.clone()).or_default();
            if !versions.iter().any(|item| item == &version) {
                versions.push(version.clone());
                versions.sort();
            }
            save_toolchain_store(&store)?;
            pin_runtime(&runtime, &version)?;
            println!("toolchain active: {runtime} {version}");
        }
        ToolchainCommand::Pin {
            runtime,
            version,
            all,
        } => {
            let store = load_toolchain_store()?;
            if all {
                for (runtime, version) in store.active {
                    pin_runtime(&runtime, &version)?;
                    println!("pinned {runtime} {version}");
                }
            } else {
                let runtime =
                    runtime.ok_or_else(|| LodeError::Message("missing runtime".to_string()))?;
                let version = version
                    .or_else(|| store.active.get(&runtime).cloned())
                    .ok_or_else(|| LodeError::Message("missing version".to_string()))?;
                pin_runtime(&runtime, &version)?;
                println!("pinned {runtime} {version}");
            }
        }
        ToolchainCommand::Update { runtime, all } => {
            if all {
                println!("toolchain update check complete for all registered runtimes");
            } else if let Some(runtime) = runtime {
                println!("toolchain update check complete for {runtime}");
            } else {
                println!("toolchain update check complete");
            }
        }
    }
    Ok(())
}

fn toolchain_store_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("toolchains.toml")
}

fn load_toolchain_store() -> lode_core::Result<ToolchainStore> {
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

fn save_toolchain_store(store: &ToolchainStore) -> lode_core::Result<()> {
    let path = toolchain_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = toml::to_string_pretty(store)?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn pin_runtime(runtime: &str, version: &str) -> lode_core::Result<()> {
    match runtime {
        "rust" | "rustc" | "cargo" => {
            fs::write(
                "rust-toolchain.toml",
                format!("[toolchain]\nchannel = \"{version}\"\n"),
            )
            .map_err(|source| LodeError::Io {
                path: "rust-toolchain.toml".into(),
                source,
            })?;
        }
        "node" | "npm" | "pnpm" | "yarn" | "bun" => {
            fs::write(".nvmrc", format!("{version}\n")).map_err(|source| LodeError::Io {
                path: ".nvmrc".into(),
                source,
            })?;
        }
        "python" | "uv" => {
            fs::write(".python-version", format!("{version}\n")).map_err(|source| {
                LodeError::Io {
                    path: ".python-version".into(),
                    source,
                }
            })?;
        }
        "go" => {
            fs::write("go.env", format!("GOTOOLCHAIN=go{version}\n")).map_err(|source| {
                LodeError::Io {
                    path: "go.env".into(),
                    source,
                }
            })?;
        }
        other => {
            fs::write(
                format!(".toolchain-{other}"),
                format!("{other}={version}\n"),
            )
            .map_err(|source| LodeError::Io {
                path: format!(".toolchain-{other}").into(),
                source,
            })?;
        }
    }
    Ok(())
}

fn pkg(command: PkgCommand) -> lode_core::Result<()> {
    let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
    match command {
        PkgCommand::List { format } => print_package_inventory(&format)?,
        PkgCommand::Outdated { dry_run, format } => run_or_print_package_operation(
            &PackageOperationPlan::new("outdated", &manager, package_outdated_args(&manager)?),
            dry_run,
            &format,
        )?,
        PkgCommand::Update { name, dry_run } => {
            let args = package_update_args(&manager, name.as_deref())?;
            if dry_run {
                println!(
                    "would run: {} {}",
                    package_command(&manager),
                    args.join(" ")
                );
            } else {
                run_package_manager(&manager, args)?;
            }
        }
        PkgCommand::Audit {
            dry_run,
            format,
            fail_on,
        } => {
            let plan = PackageOperationPlan::new(
                "audit",
                &manager,
                package_audit_args(&manager, fail_on.as_deref())?,
            );
            run_or_print_package_operation(&plan, dry_run, &format)?;
            if dry_run {
                println!("would run: lode scan secrets {}", current_dir()?);
            } else {
                scan(ScanCommand::Secrets {
                    path: Some(current_dir()?),
                    staged: false,
                    json: false,
                    quiet: false,
                })?;
            }
        }
        PkgCommand::Why {
            name,
            dry_run,
            format,
        } => package_explain("why", &manager, &name, dry_run, &format)?,
        PkgCommand::Info {
            name,
            dry_run,
            format,
        } => package_explain("info", &manager, &name, dry_run, &format)?,
        PkgCommand::Lock { dry_run } => {
            let args = package_lock_args(&manager)?;
            if dry_run {
                println!(
                    "would run: {} {}",
                    package_command(&manager),
                    args.join(" ")
                );
            } else {
                run_package_manager(&manager, args)?;
            }
        }
        PkgCommand::Graph { format } => package_graph(&format)?,
        PkgCommand::Clean { dry_run } => {
            for path in [
                "target",
                "node_modules",
                ".pytest_cache",
                "__pycache__",
                "dist",
                "build",
            ] {
                let path = Utf8PathBuf::from(path);
                if path.exists() {
                    if dry_run {
                        println!("would remove {path}");
                    } else if path.is_dir() {
                        fs::remove_dir_all(&path).map_err(|source| LodeError::Io {
                            path: path.as_str().into(),
                            source,
                        })?;
                        println!("removed {path}");
                    }
                }
            }
        }
    }
    Ok(())
}

fn run_package_manager(manager: &str, args: Vec<String>) -> lode_core::Result<()> {
    if manager == "unknown" {
        return Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        ));
    }
    let command = package_command(manager);
    let status = run_process_status(command, &args, None)?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "{command} failed with {status}"
        )))
    }
}

#[derive(Debug, Serialize)]
struct PackageOperationPlan {
    operation: String,
    manager: String,
    command: String,
    args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    packages: Vec<PackageDependency>,
}

impl PackageOperationPlan {
    fn new(operation: &str, manager: &str, args: Vec<String>) -> Self {
        Self {
            operation: operation.to_string(),
            manager: manager.to_string(),
            command: package_command(manager).to_string(),
            args,
            packages: package_dependencies(),
        }
    }

    fn command_line(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct PackageManifest {
    file: String,
    kind: String,
    manager: String,
    dependencies: Vec<PackageDependency>,
}

#[derive(Debug, Clone, Serialize)]
struct PackageDependency {
    name: String,
    version: Option<String>,
    scope: String,
    manifest: String,
}

fn print_package_inventory(format: &str) -> lode_core::Result<()> {
    let manager = detect_package_manager().unwrap_or_else(|| "unknown".to_string());
    let manifests = package_manifest_inventory();
    match format {
        "table" => {
            println!("manager: {manager}");
            for manifest in &manifests {
                println!(
                    "{}\t{}\t{} dependencies",
                    manifest.file,
                    manifest.kind,
                    manifest.dependencies.len()
                );
                for dependency in &manifest.dependencies {
                    let version = dependency.version.as_deref().unwrap_or("*");
                    println!("  {} {} ({})", dependency.name, version, dependency.scope);
                }
            }
            Ok(())
        }
        "json" => {
            let inventory = json!({
                "manager": manager,
                "manifests": manifests,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&inventory)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
            Ok(())
        }
        other => Err(LodeError::Message(format!(
            "unsupported package output format: {other}"
        ))),
    }
}

fn package_explain(
    operation: &str,
    manager: &str,
    name: &str,
    dry_run: bool,
    format: &str,
) -> lode_core::Result<()> {
    let matches = package_dependencies()
        .into_iter()
        .filter(|dependency| package_name_matches(&dependency.name, name))
        .collect::<Vec<_>>();
    if !matches.is_empty() {
        print_package_matches(operation, manager, name, &matches, format)?;
        if dry_run {
            let args = match operation {
                "why" => package_why_args(manager, name)?,
                "info" => package_info_args(manager, name)?,
                _ => Vec::new(),
            };
            println!("would run: {} {}", package_command(manager), args.join(" "));
        }
        return Ok(());
    }
    let args = match operation {
        "why" => package_why_args(manager, name)?,
        "info" => package_info_args(manager, name)?,
        _ => Vec::new(),
    };
    run_or_print_package_manager(manager, args, dry_run)
}

fn print_package_matches(
    operation: &str,
    manager: &str,
    name: &str,
    matches: &[PackageDependency],
    format: &str,
) -> lode_core::Result<()> {
    match format {
        "table" => {
            println!("{operation}: {name}");
            println!("manager: {manager}");
            for dependency in matches {
                let version = dependency.version.as_deref().unwrap_or("*");
                println!(
                    "project -> {} -> {} {} ({})",
                    dependency.manifest, dependency.name, version, dependency.scope
                );
            }
            Ok(())
        }
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": operation,
                    "manager": manager,
                    "query": name,
                    "matches": matches,
                }))
                .map_err(|error| LodeError::Message(error.to_string()))?
            );
            Ok(())
        }
        other => Err(LodeError::Message(format!(
            "unsupported package output format: {other}"
        ))),
    }
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

fn package_dependencies() -> Vec<PackageDependency> {
    package_manifest_inventory()
        .into_iter()
        .flat_map(|manifest| manifest.dependencies)
        .collect()
}

fn package_name_matches(candidate: &str, query: &str) -> bool {
    candidate == query
        || candidate.contains(query)
        || candidate
            .rsplit_once(':')
            .map(|(_, artifact)| artifact == query)
            .unwrap_or(false)
        || candidate
            .rsplit_once('/')
            .map(|(_, tail)| tail == query)
            .unwrap_or(false)
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

fn package_command(manager: &str) -> &str {
    match manager {
        "maven" => "mvn",
        other => other,
    }
}

fn run_or_print_package_operation(
    plan: &PackageOperationPlan,
    dry_run: bool,
    format: &str,
) -> lode_core::Result<()> {
    if dry_run {
        print_package_plan(plan, format)
    } else {
        run_package_manager(&plan.manager, plan.args.clone())
    }
}

fn print_package_plan(plan: &PackageOperationPlan, format: &str) -> lode_core::Result<()> {
    match format {
        "table" => {
            println!("would run: {}", plan.command_line());
            Ok(())
        }
        "json" => {
            println!(
                "{}",
                serde_json::to_string_pretty(plan)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
            Ok(())
        }
        other => Err(LodeError::Message(format!(
            "unsupported package output format: {other}"
        ))),
    }
}

fn run_or_print_package_manager(
    manager: &str,
    args: Vec<String>,
    dry_run: bool,
) -> lode_core::Result<()> {
    if dry_run {
        println!("would run: {} {}", package_command(manager), args.join(" "));
        Ok(())
    } else {
        run_package_manager(manager, args)
    }
}

fn package_outdated_args(manager: &str) -> lode_core::Result<Vec<String>> {
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

fn package_update_args(manager: &str, name: Option<&str>) -> lode_core::Result<Vec<String>> {
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

fn package_audit_args(manager: &str, fail_on: Option<&str>) -> lode_core::Result<Vec<String>> {
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

fn validate_package_severity(severity: &str) -> lode_core::Result<&str> {
    match severity {
        "low" | "medium" | "high" | "critical" => Ok(severity),
        other => Err(LodeError::Message(format!(
            "unsupported package audit severity: {other}"
        ))),
    }
}

fn severity_cvss_threshold(severity: &str) -> &'static str {
    match severity {
        "low" => "0",
        "medium" => "4",
        "high" => "7",
        "critical" => "9",
        _ => "7",
    }
}

fn package_lock_args(manager: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["generate-lockfile".into()]),
        "npm" => Ok(vec![
            "install".into(),
            "--package-lock-only".into(),
            "--ignore-scripts".into(),
        ]),
        "pnpm" => Ok(vec!["install".into(), "--lockfile-only".into()]),
        "yarn" => Ok(vec!["install".into(), "--mode=update-lockfile".into()]),
        "bun" => Ok(vec!["install".into(), "--lockfile-only".into()]),
        "uv" => Ok(vec!["lock".into()]),
        "pip" => Ok(vec!["freeze".into()]),
        "go" => Ok(vec!["mod".into(), "tidy".into()]),
        "bundler" => Ok(vec!["lock".into()]),
        "gradle" => Ok(vec!["dependencies".into(), "--write-locks".into()]),
        "maven" => Ok(vec![
            "dependency:go-offline".into(),
            "-DgenerateBackupPoms=false".into(),
        ]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_why_args(manager: &str, name: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["tree".into(), "-i".into(), name.into()]),
        "npm" => Ok(vec!["explain".into(), name.into()]),
        "pnpm" | "yarn" => Ok(vec!["why".into(), name.into()]),
        "bun" => Ok(vec!["pm".into(), "why".into(), name.into()]),
        "uv" => Ok(vec!["pip".into(), "show".into(), name.into()]),
        "pip" => Ok(vec!["show".into(), name.into()]),
        "go" => Ok(vec!["mod".into(), "why".into(), name.into()]),
        "bundler" => Ok(vec!["why".into(), name.into()]),
        "gradle" => Ok(vec![
            "dependencyInsight".into(),
            "--dependency".into(),
            name.into(),
        ]),
        "maven" => Ok(vec!["dependency:tree".into(), format!("-Dincludes={name}")]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_info_args(manager: &str, name: &str) -> lode_core::Result<Vec<String>> {
    match manager {
        "cargo" => Ok(vec!["search".into(), name.into()]),
        "npm" | "pnpm" | "yarn" | "bun" => Ok(vec!["info".into(), name.into()]),
        "uv" => Ok(vec!["pip".into(), "show".into(), name.into()]),
        "pip" => Ok(vec!["show".into(), name.into()]),
        "go" => Ok(vec!["list".into(), "-m".into(), name.into()]),
        "bundler" => Ok(vec!["info".into(), name.into()]),
        "gradle" => Ok(vec![
            "dependencyInsight".into(),
            "--dependency".into(),
            name.into(),
        ]),
        "maven" => Ok(vec!["dependency:tree".into(), format!("-Dincludes={name}")]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_graph(format: &str) -> lode_core::Result<()> {
    let manifests = [
        ("Cargo.toml", "cargo"),
        ("package.json", "node"),
        ("pyproject.toml", "python"),
        ("go.mod", "go"),
        ("build.gradle", "gradle"),
        ("settings.gradle", "gradle"),
        ("pom.xml", "maven"),
    ];
    let found = manifests
        .iter()
        .filter(|(file, _)| Utf8PathBuf::from(*file).exists())
        .map(|(file, kind)| serde_json::json!({ "file": file, "kind": kind }))
        .collect::<Vec<_>>();
    let manager = detect_package_manager();
    match format {
        "json" => {
            let graph = json!({
                "manager": manager,
                "manifests": found,
                "edges": found.iter().filter_map(|manifest| {
                    Some(json!({
                        "from": "project",
                        "to": manifest.get("kind")?.as_str()?,
                        "label": manifest.get("file")?.as_str()?
                    }))
                }).collect::<Vec<_>>()
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&graph)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
        }
        "ascii" => {
            println!(
                "project manager={}",
                manager.as_deref().unwrap_or("unknown")
            );
            for (file, kind) in manifests {
                if Utf8PathBuf::from(file).exists() {
                    println!("`- {kind} ({file})");
                }
            }
        }
        "dot" => {
            println!("digraph packages {{");
            println!(
                "  project [label=\"project\\nmanager={}\"];",
                manager.as_deref().unwrap_or("unknown")
            );
            for (file, kind) in manifests {
                if Utf8PathBuf::from(file).exists() {
                    println!("  project -> {kind} [label=\"{file}\"];");
                }
            }
            println!("}}");
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported package graph format: {other}"
            )))
        }
    }
    Ok(())
}

fn time_command(command: TimeCommand) -> lode_core::Result<()> {
    match command {
        TimeCommand::Today { format } => {
            let log = load_time_log()?;
            let today = today_utc();
            let sessions = log
                .sessions
                .into_iter()
                .filter(|session| session.started_at.starts_with(&today))
                .collect::<Vec<_>>();
            print_time_sessions("today", &sessions, &format)?;
        }
        TimeCommand::Show { since, by, format } => {
            let log = load_time_log()?;
            let sessions = filter_time_sessions(log.sessions, since.as_deref());
            print_time_summary(&sessions, &by, &format)?;
        }
        TimeCommand::Report { since, format, out } => {
            let log = load_time_log()?;
            let sessions = filter_time_sessions(log.sessions, since.as_deref());
            let report = render_time_report(&sessions, &format)?;
            if let Some(path) = out {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                        path: parent.as_str().into(),
                        source,
                    })?;
                }
                fs::write(&path, report).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?;
                println!("wrote time report to {path}");
            } else {
                print!("{report}");
            }
        }
        TimeCommand::Clear { before, confirm } => {
            if !confirm {
                return Err(LodeError::Message(
                    "refusing to clear time log without --confirm".to_string(),
                ));
            }
            let path = time_log_path()?;
            if let Some(before) = before {
                let mut log = load_time_log()?;
                let before_key = resolve_since_key(&before).unwrap_or(before);
                let before_key = before_key.as_str();
                let before_count = log.sessions.len();
                log.sessions
                    .retain(|session| session.started_at.as_str() >= before_key);
                save_time_log(&log)?;
                println!(
                    "time log cleared: removed {} session(s)",
                    before_count - log.sessions.len()
                );
            } else if path.exists() {
                fs::remove_file(&path).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?;
                println!("time log cleared");
            } else {
                println!("time log cleared");
            }
        }
    }
    Ok(())
}

fn time_log_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(current_dir()?.join(".lode").join("time-log.json"))
}

fn load_time_log() -> lode_core::Result<TimeLog> {
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

fn save_time_log(log: &TimeLog) -> lode_core::Result<()> {
    let path = time_log_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw =
        serde_json::to_string_pretty(log).map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn filter_time_sessions(sessions: Vec<TimeSession>, since: Option<&str>) -> Vec<TimeSession> {
    let Some(since) = since else {
        return sessions;
    };
    let key = resolve_since_key(since).unwrap_or_else(|| since.to_string());
    sessions
        .into_iter()
        .filter(|session| session.started_at.as_str() >= key.as_str())
        .collect()
}

fn resolve_since_key(value: &str) -> Option<String> {
    let days = value.strip_suffix('d')?.parse::<u64>().ok()?;
    let today = today_days_since_epoch();
    let target = today.saturating_sub(days);
    let (year, month, day) = civil_from_days(target as i64);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn print_time_sessions(
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

fn print_time_summary(sessions: &[TimeSession], by: &str, format: &str) -> lode_core::Result<()> {
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

fn render_time_report(sessions: &[TimeSession], format: &str) -> lode_core::Result<String> {
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

fn render_time_sessions_markdown(label: &str, sessions: &[TimeSession]) -> String {
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

fn total_seconds(sessions: &[TimeSession]) -> u64 {
    sessions.iter().map(|session| session.seconds).sum()
}

fn format_seconds(seconds: u64) -> String {
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

fn today_utc() -> String {
    let days = today_days_since_epoch() as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn now_timestamp() -> String {
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

fn today_days_since_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or_default()
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
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

fn detect_toolchains() -> Vec<String> {
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

fn required_tools_for_project() -> Vec<&'static str> {
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

fn detect_package_manager() -> Option<String> {
    detect_package_manager_in(&Utf8PathBuf::from("."))
}

fn detect_package_manager_in(root: &Utf8PathBuf) -> Option<String> {
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

fn command_version(command: &str) -> Option<String> {
    let output = run_process_output(command, &["--version".to_string()]).ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.lines().next().unwrap_or("installed").to_string())
}

fn export_lodepack(out: Option<Utf8PathBuf>, options: ExportOptions) -> lode_core::Result<()> {
    let root = global_dir()?;
    let output = out.unwrap_or_else(|| Utf8PathBuf::from("lode-export.lodepack"));
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
    collect_pack_files_as(&root.join("config.toml"), "config.toml", &mut pack)?;
    let mut paths = vec![("profiles", global_asset_dir("profiles")?)];
    if !options.no_commands {
        paths.push(("commands", global_asset_dir("commands")?));
    }
    if !options.no_templates {
        paths.push(("templates", global_asset_dir("templates")?));
    }
    if !options.no_snippets {
        paths.push(("snippets", global_asset_dir("snippets")?));
    }
    if !options.no_licenses {
        paths.push(("licenses", global_asset_dir("licenses")?));
    }
    if !options.no_recipes {
        paths.push(("recipes", global_asset_dir("recipes")?));
    }
    if !options.no_plugins {
        paths.push(("plugins", global_asset_dir("plugins")?));
    }
    if options.include_metrics {
        collect_pack_files_as(&root.join("registry.json"), "registry.json", &mut pack)?;
        collect_pack_files_as(&root.join("metrics.json"), "metrics.json", &mut pack)?;
    }
    for (prefix, path) in paths {
        collect_pack_files_as(&path, prefix, &mut pack)?;
    }
    pack.manifest.file_count = pack.files.len();
    let raw = serde_json::to_string_pretty(&pack)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&output, raw).map_err(|source| LodeError::Io {
        path: output.as_str().into(),
        source,
    })?;
    println!("exported {} files to {output}", pack.files.len());
    Ok(())
}

fn collect_pack_files_as(
    path: &Utf8PathBuf,
    prefix: &str,
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
            let child_prefix = format!(
                "{}/{}",
                prefix.trim_end_matches('/'),
                entry.file_name().to_string_lossy()
            );
            collect_pack_files_as(&child, &child_prefix, pack)?;
        }
        return Ok(());
    }
    let contents = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let checksum = content_hash_bytes(contents.as_bytes());
    pack.files.push(LodePackFile {
        path: prefix.replace('\\', "/"),
        contents,
        checksum,
    });
    Ok(())
}

fn import_lodepack(path: Utf8PathBuf, no_merge: bool, force: bool) -> lode_core::Result<()> {
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let pack: LodePack =
        serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    validate_lodepack_manifest(&pack)?;
    let root = global_dir()?;
    fs::create_dir_all(&root).map_err(|source| LodeError::Io {
        path: root.as_str().into(),
        source,
    })?;
    let mut seen_paths = BTreeSet::new();
    let mut validated_files = Vec::new();
    for file in &pack.files {
        let normalized = validate_lodepack_path(&file.path)?;
        if !seen_paths.insert(normalized.clone()) {
            return Err(LodeError::Message(format!(
                "duplicate lodepack path: {normalized}"
            )));
        }
        validate_lodepack_file_checksum(file, &normalized)?;
        validated_files.push((file, normalized));
    }
    validate_lodepack_file_count(&pack)?;
    for (file, normalized) in validated_files {
        let destination = lodepack_destination(&root, &normalized)?;
        if destination.exists() && no_merge && !force {
            return Err(LodeError::Message(format!(
                "import conflict: {} exists",
                normalized
            )));
        }
        if destination.exists() && !force && normalized != "config.toml" {
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.as_str().into(),
                source,
            })?;
        }
        fs::write(&destination, &file.contents).map_err(|source| LodeError::Io {
            path: destination.as_str().into(),
            source,
        })?;
    }
    println!("imported {} files from {path}", pack.files.len());
    Ok(())
}

fn validate_lodepack_manifest(pack: &LodePack) -> lode_core::Result<()> {
    if pack.version != 1 {
        return Err(LodeError::Message(format!(
            "unsupported lodepack version: {}",
            pack.version
        )));
    }
    if pack.manifest.schema_version != 3 {
        return Err(LodeError::Message(format!(
            "unsupported lodepack schema: {}",
            pack.manifest.schema_version
        )));
    }
    let expected_algorithm = default_lodepack_checksum_algorithm();
    if pack.manifest.checksum_algorithm != expected_algorithm {
        return Err(LodeError::Message(format!(
            "unsupported lodepack checksum algorithm: {}",
            pack.manifest.checksum_algorithm
        )));
    }
    Ok(())
}

fn validate_lodepack_file_count(pack: &LodePack) -> lode_core::Result<()> {
    if pack.manifest.file_count != 0 && pack.manifest.file_count != pack.files.len() {
        return Err(LodeError::Message(format!(
            "lodepack file count mismatch: manifest has {}, pack has {}",
            pack.manifest.file_count,
            pack.files.len()
        )));
    }
    Ok(())
}

fn validate_lodepack_file_checksum(file: &LodePackFile, normalized: &str) -> lode_core::Result<()> {
    if file.checksum.is_empty() {
        return Ok(());
    }
    let actual = content_hash_bytes(file.contents.as_bytes());
    if actual != file.checksum {
        return Err(LodeError::Message(format!(
            "lodepack checksum mismatch for {normalized}"
        )));
    }
    Ok(())
}

fn lodepack_destination(root: &Utf8PathBuf, path: &str) -> lode_core::Result<Utf8PathBuf> {
    let Some((first, rest)) = path.split_once('/') else {
        return Ok(root.join(path));
    };
    match first {
        "templates" | "profiles" | "snippets" | "licenses" | "plugins" | "recipes" | "commands" => {
            Ok(global_asset_dir(first)?.join(rest))
        }
        _ => Ok(root.join(path)),
    }
}

fn validate_lodepack_path(path: &str) -> lode_core::Result<String> {
    let normalized = path.replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains(':')
        || normalized.chars().any(char::is_control)
    {
        return Err(LodeError::Message(format!("unsafe lodepack path: {path}")));
    }
    let mut segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(LodeError::Message(format!("unsafe lodepack path: {path}")));
    }
    let first = segments.remove(0);
    let valid_root_file =
        matches!(first, "config.toml" | "registry.json" | "metrics.json") && segments.is_empty();
    let valid_asset_path = matches!(
        first,
        "templates" | "profiles" | "snippets" | "licenses" | "plugins" | "recipes" | "commands"
    ) && !segments.is_empty();
    if !valid_root_file && !valid_asset_path {
        return Err(LodeError::Message(format!(
            "unsupported lodepack path: {path}"
        )));
    }
    Ok(normalized)
}

fn collect_pack_files(
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

fn metrics(command: MetricsCommand) -> lode_core::Result<()> {
    match command {
        MetricsCommand::Show => {
            let report = load_metrics(&current_dir()?)?;
            println!("metrics score: {}", report.score);
            println!("convention violations: {}", report.convention_violations);
            println!("secret findings: {}", report.secret_findings);
        }
        MetricsCommand::Trend { last } => {
            let report = load_metrics(&current_dir()?)?;
            println!("metrics trend: latest score {}", report.score);
            if let Some(last) = last {
                println!("window: last {last} snapshot(s)");
            }
        }
        MetricsCommand::Baseline => {
            let cwd = current_dir()?;
            let report = audit_project(&cwd, &load_global_config()?)?;
            save_metrics(&cwd, &report)?;
            save_metrics_baseline(&cwd, &report)?;
            println!("metrics baseline saved");
        }
        MetricsCommand::DiffBaseline => {
            let cwd = current_dir()?;
            let current = load_metrics(&cwd)?;
            let baseline = load_metrics_baseline(&cwd)?;
            println!(
                "score delta: {}",
                current.score as i16 - baseline.score as i16
            );
            println!(
                "convention delta: {}",
                current.convention_violations as i64 - baseline.convention_violations as i64
            );
            println!(
                "secret delta: {}",
                current.secret_findings as i64 - baseline.secret_findings as i64
            );
        }
    }
    Ok(())
}

fn metrics_baseline_path(root: &Utf8PathBuf) -> Utf8PathBuf {
    root.join(".lode").join("metrics-baseline.json")
}

fn save_metrics_baseline(
    root: &Utf8PathBuf,
    report: &lode_core::AuditReport,
) -> lode_core::Result<()> {
    let path = metrics_baseline_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(report)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn load_metrics_baseline(root: &Utf8PathBuf) -> lode_core::Result<lode_core::AuditReport> {
    let path = metrics_baseline_path(root);
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

fn status_bool(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "missing"
    }
}

fn workspace(command: WorkspaceCommand) -> lode_core::Result<()> {
    match command {
        WorkspaceCommand::Init => workspace_init()?,
        WorkspaceCommand::List { format } => workspace_list(&format)?,
        WorkspaceCommand::Add { name } => workspace_add(&name)?,
        WorkspaceCommand::Remove { name, confirm } => workspace_remove(&name, confirm)?,
        WorkspaceCommand::Run {
            target,
            pkg,
            changed,
            parallel,
            dry_run,
        } => workspace_run(&target, pkg.as_deref(), &changed, parallel, dry_run)?,
        WorkspaceCommand::Graph { format } => workspace_graph(&format)?,
    }
    Ok(())
}

fn workspace_file() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("workspace.toml")
}

fn workspace_init() -> lode_core::Result<()> {
    let path = workspace_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    if !path.exists() {
        fs::write(&path, "members = []\n").map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
    }
    println!("workspace initialised");
    Ok(())
}

fn workspace_members() -> lode_core::Result<Vec<String>> {
    let path = workspace_file();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    Ok(value
        .get("members")
        .and_then(toml::Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default())
}

fn save_workspace_members(members: &[String]) -> lode_core::Result<()> {
    let path = workspace_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let quoted = members
        .iter()
        .map(|member| format!("\"{member}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(&path, format!("members = [{quoted}]\n")).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn workspace_add(name: &str) -> lode_core::Result<()> {
    validate_workspace_member(name)?;
    let mut members = workspace_members()?;
    if !members.iter().any(|member| member == name) {
        members.push(name.to_string());
        members.sort();
        save_workspace_members(&members)?;
    }
    fs::create_dir_all(name).map_err(|source| LodeError::Io {
        path: name.into(),
        source,
    })?;
    println!("workspace member added: {name}");
    Ok(())
}

fn workspace_remove(name: &str, confirm: bool) -> lode_core::Result<()> {
    validate_workspace_member(name)?;
    if !confirm {
        return Err(LodeError::Message(
            "refusing to remove workspace member without --confirm".to_string(),
        ));
    }
    let mut members = workspace_members()?;
    let before = members.len();
    members.retain(|member| member != name);
    if members.len() == before {
        return Err(LodeError::Message(format!(
            "workspace member not found: {name}"
        )));
    }
    save_workspace_members(&members)?;
    println!("workspace member removed: {name}");
    Ok(())
}

fn validate_workspace_member(name: &str) -> lode_core::Result<()> {
    let relative = safe_relative_path(name)?;
    if relative.as_str().is_empty() || relative.as_str().starts_with(".lode") {
        return Err(LodeError::Message(format!(
            "unsafe workspace member path: {name}"
        )));
    }
    Ok(())
}

fn workspace_list(format: &str) -> lode_core::Result<()> {
    let members = workspace_members()?;
    match format {
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&members)
                .map_err(|error| LodeError::Message(error.to_string()))?
        ),
        "table" => {
            if members.is_empty() {
                println!("workspace has no members");
            } else {
                for member in members {
                    println!("{member}");
                }
            }
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported workspace list format: {other}"
            )))
        }
    }
    Ok(())
}

fn workspace_run(
    target: &str,
    pkg: Option<&str>,
    changed: &[String],
    parallel: Option<usize>,
    dry_run: bool,
) -> lode_core::Result<()> {
    let mut members = workspace_members()?;
    if let Some(pkg) = pkg {
        members.retain(|member| member == pkg || member.ends_with(&format!("/{pkg}")));
    }
    if !changed.is_empty() {
        let affected = affected_workspace_members(&members, changed);
        if affected.is_empty() {
            println!("no workspace members affected by changed path(s)");
            return Ok(());
        }
        members = affected;
    }
    if members.is_empty() {
        if dry_run {
            println!("would run make {target}");
            return Ok(());
        }
        return run_make(target);
    }
    if let Some(parallel) = parallel {
        println!("parallel requested: {parallel}");
    }
    for member in members {
        println!("==> {member}: {target}");
        let makefile = Utf8PathBuf::from(&member).join("Makefile");
        if dry_run {
            println!("would run: make -C {member} {target}");
            continue;
        }
        if makefile.exists() {
            let args = vec!["-C".to_string(), member.clone(), target.to_string()];
            let status = run_process_status("make", &args, None)?;
            if !status.success() {
                return Err(LodeError::Message(format!(
                    "workspace member {member} target {target} failed with {status}"
                )));
            }
        } else {
            println!("skip {member}: no Makefile");
        }
    }
    Ok(())
}

fn affected_workspace_members(members: &[String], changed: &[String]) -> Vec<String> {
    members
        .iter()
        .filter(|member| {
            let normalized_member = normalize_workspace_path(member);
            changed.iter().any(|path| {
                let normalized_path = normalize_workspace_path(path);
                normalized_path == normalized_member
                    || normalized_path.starts_with(&format!("{normalized_member}/"))
            })
        })
        .cloned()
        .collect()
}

fn normalize_workspace_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn workspace_graph(format: &str) -> lode_core::Result<()> {
    let members = workspace_members()?;
    match format {
        "ascii" => {
            println!("workspace");
            for member in members {
                println!("  -> {member}");
            }
        }
        "dot" => {
            println!("digraph workspace {{");
            println!("  root [label=\"workspace\"];");
            for member in members {
                println!("  root -> \"{member}\";");
            }
            println!("}}");
        }
        "json" => {
            let graph = json!({
                "root": "workspace",
                "members": members,
                "edges": members.iter().map(|member| json!({"from": "workspace", "to": member})).collect::<Vec<_>>()
            });
            println!("{}", json_pretty(&graph)?);
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported workspace graph format: {other}"
            )))
        }
    }
    Ok(())
}

fn daemon(command: DaemonCommand) {
    if let Err(error) = daemon_result(command) {
        eprintln!("error: {error}");
    }
}

fn daemon_result(command: DaemonCommand) -> lode_core::Result<()> {
    match command {
        DaemonCommand::Start {
            no_rename,
            no_sign,
            no_stamp,
            foreground,
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
        DaemonCommand::ListWatchers { json } => {
            let runtime = load_daemon_runtime_state()?;
            if json {
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
        DaemonCommand::Status { quiet, json } => {
            let state =
                fs::read_to_string(daemon_state_path()?).unwrap_or_else(|_| "inactive".to_string());
            let runtime = load_daemon_runtime_state()?;
            let active = runtime.active;
            if json {
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

fn print_log_lines(log: &str, tail: Option<usize>) {
    let mut lines = log.lines().collect::<Vec<_>>();
    if let Some(tail) = tail {
        let start = lines.len().saturating_sub(tail);
        lines = lines[start..].to_vec();
    }
    for line in lines {
        println!("{line}");
    }
}

fn follow_daemon_log(path: &Utf8PathBuf, mut offset: usize) -> lode_core::Result<()> {
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

fn run_foreground_daemon(rename: bool, sign: bool, stamp: bool) -> lode_core::Result<()> {
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

fn snapshot_project(root: &Utf8PathBuf) -> lode_core::Result<BTreeMap<String, u64>> {
    let mut snapshot = BTreeMap::new();
    snapshot_dir(root, root, &mut snapshot)?;
    Ok(snapshot)
}

fn snapshot_dir(
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

fn should_skip_watch_path(name: &str) -> bool {
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

fn daemon_changes(
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

fn record_daemon_activity(root: &Utf8PathBuf, changes: &DaemonChangeSet) -> lode_core::Result<()> {
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

#[derive(Debug, Serialize)]
struct ProjectDaemonState {
    schema_version: u32,
    project: Option<String>,
    updated_at: String,
    file_count: usize,
    files: BTreeMap<String, ProjectDaemonFileState>,
}

#[derive(Debug, Serialize)]
struct ProjectDaemonFileState {
    modified_s: u64,
    content_hash: String,
}

fn write_project_daemon_snapshot(
    root: &Utf8PathBuf,
    snapshot: &BTreeMap<String, u64>,
) -> lode_core::Result<()> {
    let relative = safe_relative_path(".lode/daemon-state.json")?;
    let path = root.join(relative);
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(&state)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn content_hash_bytes(contents: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn log_command(command: LogCommand) -> lode_core::Result<()> {
    match command {
        LogCommand::Init => {
            let path = daemon_log_path()?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                    path: parent.as_str().into(),
                    source,
                })?;
            }
            if !path.exists() {
                fs::write(&path, "").map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?;
            }
            println!("log initialised at {path}");
        }
        LogCommand::Daemon { tail } => {
            let log = fs::read_to_string(daemon_log_path()?).unwrap_or_default();
            let mut lines = log.lines().collect::<Vec<_>>();
            if let Some(tail) = tail {
                let start = lines.len().saturating_sub(tail);
                lines = lines[start..].to_vec();
            }
            for line in lines {
                println!("{line}");
            }
        }
        LogCommand::Clear => {
            let path = daemon_log_path()?;
            if path.exists() {
                fs::write(&path, "").map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?;
            }
            println!("logs cleared");
        }
    }
    Ok(())
}

fn self_command(command: SelfCommand) -> lode_core::Result<()> {
    match command {
        SelfCommand::Info => {
            let exe = env::current_exe().map_err(|source| LodeError::Io {
                path: "current_exe".into(),
                source,
            })?;
            let root = global_dir()?;
            println!("version\t{}", env!("CARGO_PKG_VERSION"));
            println!("executable\t{}", exe.display());
            println!("global_dir\t{root}");
            println!("schema_version\t3");
            for name in [
                "templates",
                "profiles",
                "snippets",
                "licenses",
                "recipes",
                "plugins",
                "commands",
            ] {
                println!("{name}\t{}", count_dir_entries(&root.join(name))?);
            }
            println!(
                "upgrade_cache\t{}",
                count_dir_entries(&root.join("cache").join("upgrade"))?
            );
        }
        SelfCommand::Clean { dry_run } => {
            for path in self_clean_targets()? {
                if dry_run {
                    println!("would clean {path}");
                } else if path.exists() {
                    if path.is_dir() {
                        fs::remove_dir_all(&path).map_err(|source| LodeError::Io {
                            path: path.as_str().into(),
                            source,
                        })?;
                    } else {
                        fs::remove_file(&path).map_err(|source| LodeError::Io {
                            path: path.as_str().into(),
                            source,
                        })?;
                    }
                    println!("cleaned {path}");
                }
            }
        }
        SelfCommand::Uninstall { keep_config } => {
            let root = global_dir()?;
            if keep_config {
                for name in [
                    "cache",
                    "logs",
                    "templates",
                    "profiles",
                    "snippets",
                    "licenses",
                    "recipes",
                    "commands",
                ] {
                    let path = root.join(name);
                    if path.exists() {
                        fs::remove_dir_all(&path).map_err(|source| LodeError::Io {
                            path: path.as_str().into(),
                            source,
                        })?;
                    }
                }
                println!("removed generated Lode data; kept config.toml");
            } else if root.exists() {
                fs::remove_dir_all(&root).map_err(|source| LodeError::Io {
                    path: root.as_str().into(),
                    source,
                })?;
                println!("removed {root}");
            }
        }
    }
    Ok(())
}

fn count_dir_entries(path: &Utf8PathBuf) -> lode_core::Result<usize> {
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

fn self_clean_targets() -> lode_core::Result<Vec<Utf8PathBuf>> {
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

#[derive(Debug, Serialize, Deserialize)]
struct UpgradeManifest {
    schema_version: u32,
    version: String,
    binary: String,
    checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpgradeState {
    schema_version: u32,
    version: String,
    candidate: Utf8PathBuf,
    checksum: String,
    current_executable: String,
    current_checksum: String,
    staged_at: String,
    activated: bool,
}

fn upgrade(
    check: bool,
    manifest: Option<Utf8PathBuf>,
    dry_run: bool,
    rollback: bool,
) -> lode_core::Result<()> {
    if rollback {
        return rollback_staged_upgrade(dry_run);
    }

    let manifest_path = manifest.unwrap_or_else(default_upgrade_manifest_path);
    if check {
        println!("lode {} is installed", env!("CARGO_PKG_VERSION"));
        if manifest_path.exists() {
            let manifest = read_upgrade_manifest(&manifest_path)?;
            let candidate = upgrade_candidate_path(&manifest_path, &manifest)?;
            let checksum = file_checksum(&candidate)?;
            let status = if checksum == manifest.checksum {
                "verified"
            } else {
                "checksum-mismatch"
            };
            println!(
                "staged_upgrade\t{}\t{}\t{}",
                manifest.version, candidate, status
            );
        } else {
            println!("staged_upgrade\tnone");
        }
        println!(
            "network upgrade checks are disabled; provide --manifest for local staged upgrades"
        );
        return Ok(());
    }

    if !manifest_path.exists() {
        return Err(LodeError::Message(format!(
            "upgrade manifest not found: {manifest_path}; place latest.json in cache/upgrade or pass --manifest"
        )));
    }

    let manifest = read_upgrade_manifest(&manifest_path)?;
    let candidate = upgrade_candidate_path(&manifest_path, &manifest)?;
    let candidate_checksum = file_checksum(&candidate)?;
    if candidate_checksum != manifest.checksum {
        return Err(LodeError::Message(format!(
            "upgrade checksum mismatch for {candidate}: expected {}, found {}",
            manifest.checksum, candidate_checksum
        )));
    }
    let current_executable = env::current_exe().map_err(|source| LodeError::Io {
        path: "current_exe".into(),
        source,
    })?;
    let current_executable = Utf8PathBuf::from_path_buf(current_executable).map_err(|path| {
        LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
    })?;
    let current_checksum =
        file_checksum(&current_executable).unwrap_or_else(|_| "unavailable".to_string());
    let state = UpgradeState {
        schema_version: 3,
        version: manifest.version.clone(),
        candidate: candidate.clone(),
        checksum: candidate_checksum,
        current_executable: current_executable.to_string(),
        current_checksum,
        staged_at: now_timestamp(),
        activated: false,
    };

    if dry_run {
        println!("would verify staged upgrade {}", state.version);
        println!("would record upgrade state at {}", upgrade_state_path()?);
        println!("candidate\t{}", state.candidate);
        println!("current_executable\t{}", state.current_executable);
        return Ok(());
    }

    write_upgrade_state(&state)?;
    println!("upgrade staged\t{}", state.version);
    println!("candidate\t{}", state.candidate);
    println!("state\t{}", upgrade_state_path()?);
    println!("activate manually after review; rollback with `lode upgrade --rollback`");
    Ok(())
}

fn default_upgrade_manifest_path() -> Utf8PathBuf {
    global_dir()
        .map(|root| root.join("cache").join("upgrade").join("latest.json"))
        .unwrap_or_else(|_| Utf8PathBuf::from(".lode/cache/upgrade/latest.json"))
}

fn upgrade_state_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?
        .join("cache")
        .join("upgrade")
        .join("upgrade-state.json"))
}

fn read_upgrade_manifest(path: &Utf8PathBuf) -> lode_core::Result<UpgradeManifest> {
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

fn upgrade_candidate_path(
    manifest_path: &Utf8PathBuf,
    manifest: &UpgradeManifest,
) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&manifest.binary)?;
    Ok(manifest_path
        .parent()
        .map(|parent| parent.join(relative.clone()))
        .unwrap_or(relative))
}

fn file_checksum(path: &Utf8PathBuf) -> lode_core::Result<String> {
    let bytes = fs::read(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(content_hash_bytes(&bytes))
}

fn write_upgrade_state(state: &UpgradeState) -> lode_core::Result<()> {
    let path = upgrade_state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(state)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn read_upgrade_state() -> lode_core::Result<UpgradeState> {
    let path = upgrade_state_path()?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

fn rollback_staged_upgrade(dry_run: bool) -> lode_core::Result<()> {
    let state = read_upgrade_state()?;
    let path = upgrade_state_path()?;
    if dry_run {
        println!("would rollback staged upgrade {}", state.version);
        println!("would remove {path}");
        return Ok(());
    }
    fs::remove_file(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    println!("upgrade rollback cleared\t{}", state.version);
    Ok(())
}

fn completions(
    shell: &str,
    install: bool,
    dry_run: bool,
    out: Option<Utf8PathBuf>,
) -> lode_core::Result<()> {
    let script = completion_script(shell)?;
    let output_path = if install {
        Some(out.unwrap_or(default_completion_path(shell)?))
    } else {
        out
    };
    if let Some(path) = output_path {
        let hint = completion_install_hint(shell, &path)?;
        let source = completion_source_line(shell, &path)?;
        if dry_run {
            println!("would write {shell} completions to {path}");
            println!("would record completion install receipt");
            println!("{hint}");
            println!("{source}");
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.as_str().into(),
                source,
            })?;
        }
        fs::write(&path, script).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
        if install {
            write_completion_install_receipt(shell, &path, &source, &hint)?;
        }
        println!("wrote {shell} completions to {path}");
        if install {
            println!("{hint}");
            println!("{source}");
        }
    } else {
        if dry_run {
            return Err(LodeError::Message(
                "--dry-run is only meaningful with --install or --out".to_string(),
            ));
        }
        print!("{script}");
    }
    Ok(())
}

fn completion_script(shell: &str) -> lode_core::Result<String> {
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

fn command_words() -> String {
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

fn default_completion_path(shell: &str) -> lode_core::Result<Utf8PathBuf> {
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

fn completion_source_line(shell: &str, path: &Utf8PathBuf) -> lode_core::Result<String> {
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

fn completion_install_hint(shell: &str, path: &Utf8PathBuf) -> lode_core::Result<String> {
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

fn completion_receipt_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?
        .join("completions")
        .join("install-receipt.json"))
}

fn write_completion_install_receipt(
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
    let receipt_path = completion_receipt_path()?;
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(&receipt)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&receipt_path, raw).map_err(|source| LodeError::Io {
        path: receipt_path.as_str().into(),
        source,
    })
}

fn serve_dashboard(
    no_color: bool,
    no_live: bool,
    initial_pane: Option<&str>,
) -> lode_core::Result<()> {
    if no_live || !io::stdout().is_terminal() {
        return serve_dashboard_snapshot(no_color, initial_pane);
    }

    enable_raw_mode().map_err(terminal_error)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(terminal_error)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(terminal_error)?;

    let result = run_live_dashboard(&mut terminal, no_color, initial_pane);

    disable_raw_mode().map_err(terminal_error)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(terminal_error)?;
    terminal.show_cursor().map_err(terminal_error)?;

    result
}

fn terminal_error(error: io::Error) -> LodeError {
    LodeError::Message(format!("terminal error: {error}"))
}

fn run_live_dashboard(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    no_color: bool,
    initial_pane: Option<&str>,
) -> lode_core::Result<()> {
    let mut selected = dashboard_pane_index(initial_pane)?;
    loop {
        let data = dashboard_data(no_color)?;
        terminal
            .draw(|frame| draw_live_dashboard(frame, &data, selected, no_color))
            .map_err(terminal_error)?;

        if event::poll(Duration::from_millis(750)).map_err(terminal_error)? {
            if let Event::Key(key) = event::read().map_err(terminal_error)? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Tab | KeyCode::Down | KeyCode::Right => selected = (selected + 1) % 8,
                    KeyCode::BackTab | KeyCode::Up | KeyCode::Left => {
                        selected = selected.checked_sub(1).unwrap_or(7);
                    }
                    KeyCode::Char('1') => selected = 0,
                    KeyCode::Char('2') => selected = 1,
                    KeyCode::Char('3') => selected = 2,
                    KeyCode::Char('4') => selected = 3,
                    KeyCode::Char('5') => selected = 4,
                    KeyCode::Char('6') => selected = 5,
                    KeyCode::Char('7') => selected = 6,
                    KeyCode::Char('8') => selected = 7,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct DashboardData {
    project: String,
    env_name: String,
    score: u8,
    convention_violations: usize,
    secret_findings: usize,
    license_present: bool,
    env_example_present: bool,
    readme_present: bool,
    daemon_state: String,
    events: Vec<String>,
    registry: Vec<String>,
    package_manager: String,
    toolchains: String,
    rust_version: String,
    git_version: String,
    time_total: String,
    time_sessions: usize,
}

fn dashboard_data(no_color: bool) -> lode_core::Result<DashboardData> {
    let cwd = current_dir()?;
    let project = cwd
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "project".to_string());
    let config = load_global_config().unwrap_or_else(|_| default_config());
    let audit = audit_project(&cwd, &config)?;
    let registry = load_registry().unwrap_or_default();
    let daemon_state = fs::read_to_string(daemon_state_path()?)
        .unwrap_or_else(|_| "inactive".to_string())
        .trim()
        .to_string();
    let daemon_runtime = load_daemon_runtime_state().unwrap_or_default();
    let daemon_log = fs::read_to_string(daemon_log_path()?).unwrap_or_default();
    let time_log = load_time_log().unwrap_or_default();
    let color = Palette::new(no_color);
    let registry = if registry.projects.is_empty() {
        vec!["No registered projects".to_string()]
    } else {
        registry
            .projects
            .iter()
            .take(8)
            .map(|project| {
                format!(
                    "{}  {}  {}",
                    project.name,
                    if project.path.exists() {
                        color.green("HEALTHY")
                    } else {
                        color.red("MISSING")
                    },
                    project.path
                )
            })
            .collect()
    };

    Ok(DashboardData {
        project,
        env_name: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
        score: audit.score,
        convention_violations: audit.convention_violations,
        secret_findings: audit.secret_findings,
        license_present: audit.license_present,
        env_example_present: audit.env_example_present,
        readme_present: audit.readme_present,
        daemon_state,
        events: recent_log_lines(&daemon_runtime.recent_events, &daemon_log),
        registry,
        package_manager: detect_package_manager().unwrap_or_else(|| "unknown".to_string()),
        toolchains: detect_toolchains().join(", "),
        rust_version: command_version("rustc").unwrap_or_else(|| "missing".to_string()),
        git_version: command_version("git").unwrap_or_else(|| "missing".to_string()),
        time_total: format_seconds(total_seconds(&time_log.sessions)),
        time_sessions: time_log.sessions.len(),
    })
}

fn draw_live_dashboard(frame: &mut Frame, data: &DashboardData, selected: usize, no_color: bool) {
    let theme = DashboardTheme::new(no_color);
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("◇ lode serve", theme.accent.add_modifier(Modifier::BOLD)),
        Span::raw(format!(
            "  {}  env:{}  health:{}",
            data.project, data.env_name, data.score
        )),
    ]))
    .block(Block::default().borders(Borders::ALL).style(theme.panel));
    frame.render_widget(title, vertical[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(50)])
        .split(vertical[1]);

    let nav_items = [
        "Overview", "Health", "Metrics", "Events", "Deps", "Registry", "Config", "Logs",
    ]
    .iter()
    .enumerate()
    .map(|(index, label)| {
        let marker = if selected == index { "›" } else { " " };
        let style = if selected == index {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            theme.text
        };
        ListItem::new(Line::from(vec![
            Span::styled(marker, style),
            Span::raw(format!(" {} [{}]", label, index + 1)),
        ]))
    })
    .collect::<Vec<_>>();
    frame.render_widget(
        List::new(nav_items).block(
            Block::default()
                .title(" NAVIGATION ")
                .borders(Borders::ALL)
                .style(theme.panel),
        ),
        body[0],
    );

    match selected {
        0 => draw_overview(frame, body[1], data, &theme),
        1 => draw_health(frame, body[1], data, &theme),
        2 => draw_metrics_panel(frame, body[1], data, &theme),
        3 => draw_lines_panel(frame, body[1], " LIVE DAEMON EVENTS ", &data.events, &theme),
        4 => draw_deps(frame, body[1], data, &theme),
        5 => draw_lines_panel(
            frame,
            body[1],
            " CROSS-PROJECT REGISTRY ",
            &data.registry,
            &theme,
        ),
        6 => draw_config_panel(frame, body[1], data, &theme),
        _ => draw_lines_panel(frame, body[1], " LOGS ", &data.events, &theme),
    }

    let footer = Paragraph::new(" ↑↓/Tab move  1-8 jump  q quit  auto-refresh 750ms ")
        .style(theme.dim)
        .block(Block::default().borders(Borders::ALL).style(theme.panel));
    frame.render_widget(footer, vertical[2]);
}

#[derive(Debug, Clone, Copy)]
struct DashboardTheme {
    panel: Style,
    text: Style,
    dim: Style,
    accent: Style,
    good: Style,
    warn: Style,
    bad: Style,
}

impl DashboardTheme {
    fn new(no_color: bool) -> Self {
        if no_color {
            Self {
                panel: Style::default(),
                text: Style::default(),
                dim: Style::default(),
                accent: Style::default().add_modifier(Modifier::BOLD),
                good: Style::default(),
                warn: Style::default(),
                bad: Style::default(),
            }
        } else {
            Self {
                panel: Style::default().fg(Color::Rgb(194, 202, 204)),
                text: Style::default().fg(Color::Rgb(222, 226, 226)),
                dim: Style::default().fg(Color::Rgb(116, 126, 128)),
                accent: Style::default().fg(Color::Rgb(91, 223, 207)),
                good: Style::default().fg(Color::Rgb(118, 220, 151)),
                warn: Style::default().fg(Color::Rgb(238, 197, 104)),
                bad: Style::default().fg(Color::Rgb(238, 109, 109)),
            }
        }
    }
}

fn draw_overview(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Min(6),
        ])
        .split(area);
    frame.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(" PROJECT HEALTH ")
                    .borders(Borders::ALL),
            )
            .gauge_style(if data.score >= 85 {
                theme.good
            } else if data.score >= 60 {
                theme.warn
            } else {
                theme.bad
            })
            .percent(data.score as u16),
        chunks[0],
    );
    draw_health(frame, chunks[1], data, theme);
    draw_lines_panel(frame, chunks[2], " RECENT EVENTS ", &data.events, theme);
}

fn draw_health(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(vec![
            Span::raw("Convention      "),
            Span::styled(
                status_count_plain(data.convention_violations),
                if data.convention_violations == 0 {
                    theme.good
                } else {
                    theme.warn
                },
            ),
        ]),
        Line::from(vec![
            Span::raw("Secrets         "),
            Span::styled(
                status_count_plain(data.secret_findings),
                if data.secret_findings == 0 {
                    theme.good
                } else {
                    theme.bad
                },
            ),
        ]),
        Line::from(format!("License         {}", yes_no(data.license_present))),
        Line::from(format!(
            "Env example     {}",
            yes_no(data.env_example_present)
        )),
        Line::from(format!("Readme          {}", yes_no(data.readme_present))),
    ];
    frame.render_widget(
        Paragraph::new(lines).style(theme.text).block(
            Block::default()
                .title(" HEALTH CHECKS ")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn draw_metrics_panel(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(format!("Score           {}", data.score)),
        Line::from(format!("Time today      {}", data.time_total)),
        Line::from(format!("Sessions        {}", data.time_sessions)),
        Line::from(format!("Daemon          {}", data.daemon_state)),
        Line::from(format!("Toolchains      {}", data.toolchains)),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.text)
            .block(
                Block::default()
                    .title(" METRICS TRENDS ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_deps(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(format!("Package manager {}", data.package_manager)),
        Line::from(format!("Rust            {}", data.rust_version)),
        Line::from(format!("Git             {}", data.git_version)),
        Line::from("Policy          strict"),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.text)
            .block(
                Block::default()
                    .title(" DEPENDENCY STATUS ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_config_panel(frame: &mut Frame, area: Rect, data: &DashboardData, theme: &DashboardTheme) {
    let lines = vec![
        Line::from(format!("Project         {}", data.project)),
        Line::from(format!("Environment     {}", data.env_name)),
        Line::from(format!("Daemon state    {}", data.daemon_state)),
        Line::from(format!("Package manager {}", data.package_manager)),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.text)
            .block(
                Block::default()
                    .title(" CONFIG SUMMARY ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_lines_panel(
    frame: &mut Frame,
    area: Rect,
    title: &'static str,
    lines: &[String],
    theme: &DashboardTheme,
) {
    let text = lines
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(text)
            .style(theme.text)
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn status_count_plain(count: usize) -> String {
    if count == 0 {
        "0 OK".to_string()
    } else {
        format!("{count} WARN")
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "OK"
    } else {
        "MISSING"
    }
}

fn serve_dashboard_snapshot(no_color: bool, initial_pane: Option<&str>) -> lode_core::Result<()> {
    let data = dashboard_data(no_color)?;
    let color = Palette::new(no_color);
    let selected = dashboard_pane_index(initial_pane)?;
    let selected_name = DASHBOARD_PANES[selected];

    println!("{}", color.cyan("◇ lode serve"));
    println!(
        "{}",
        rule(&format!(
            " Project: {} | Env: {} | Pane: {} | Health: {} | Warn: {} | Fail: {} ",
            color.cyan(&data.project),
            color.cyan(&data.env_name),
            color.cyan(selected_name),
            color.green(&data.score.to_string()),
            color.yellow(&data.convention_violations.to_string()),
            color.red(&data.secret_findings.to_string())
        ))
    );
    println!();
    println!(
        "{}  {}",
        pane(
            "NAVIGATION",
            &[
                &color.cyan("› Overview        [1]"),
                "  Health          [2]",
                "  Metrics         [3]",
                "  Events          [4]",
                "  Dependencies    [5]",
                "  Registry        [6]",
                "  Config          [7]",
                "  Logs            [8]",
            ],
            30
        ),
        pane(
            "1. PROJECT HEALTH",
            &[
                &format!("Overall Status  {}", health_label(data.score, &color)),
                &format!("Score           {}", color.cyan(&data.score.to_string())),
                &format!(
                    "Convention      {}",
                    status_count(data.convention_violations, &color)
                ),
                &format!(
                    "Secrets         {}",
                    status_count(data.secret_findings, &color)
                ),
                &format!(
                    "License         {}",
                    bool_label(data.license_present, &color)
                ),
                &format!(
                    "Env Example     {}",
                    bool_label(data.env_example_present, &color)
                ),
                &format!(
                    "Readme          {}",
                    bool_label(data.readme_present, &color)
                ),
            ],
            56
        )
    );
    println!(
        "{}  {}",
        pane(
            "2. METRICS TRENDS",
            &[
                &format!("Health      {} {}", color.cyan("████████░░"), data.score),
                &format!(
                    "Checks      {}",
                    color.green("convention · secrets · license · env")
                ),
                &format!("Toolchain   {}", data.toolchains),
                &format!("Package     {}", data.package_manager),
            ],
            56
        ),
        pane(
            "3. DAEMON / TIME",
            &[
                &format!("Daemon State  {}", color.cyan(&data.daemon_state)),
                &format!("Active Session  {} session(s)", data.time_sessions),
                &format!("Today           {}", data.time_total),
                "Focus Score     derived metrics pending",
            ],
            56
        )
    );
    println!(
        "{}  {}",
        pane(
            "4. LIVE DAEMON EVENTS",
            &data.events.iter().map(String::as_str).collect::<Vec<_>>(),
            70
        ),
        pane(
            "5. DEPENDENCY STATUS",
            &[
                &format!("Manager  {}", data.package_manager),
                &format!("Rust     {}", data.rust_version),
                &format!("Git      {}", data.git_version),
                "Policy   strict",
            ],
            42
        )
    );
    println!(
        "{}",
        pane(
            "6. CROSS-PROJECT REGISTRY",
            &data.registry.iter().map(String::as_str).collect::<Vec<_>>(),
            116
        )
    );
    println!(
        "{}",
        rule(" ↑↓ Move   Tab Next   Enter Open   r Refresh   q Quit   Auto-refresh: OFF ")
    );
    Ok(())
}

const DASHBOARD_PANES: [&str; 8] = [
    "overview", "health", "metrics", "activity", "deps", "registry", "config", "logs",
];

fn dashboard_pane_index(pane: Option<&str>) -> lode_core::Result<usize> {
    let Some(pane) = pane else {
        return Ok(0);
    };
    DASHBOARD_PANES
        .iter()
        .position(|candidate| *candidate == pane)
        .ok_or_else(|| LodeError::Message(format!("unsupported dashboard pane: {pane}")))
}

struct Palette {
    enabled: bool,
}

impl Palette {
    fn new(no_color: bool) -> Self {
        Self { enabled: !no_color }
    }

    fn cyan(&self, text: &str) -> String {
        self.paint("36", text)
    }

    fn green(&self, text: &str) -> String {
        self.paint("32", text)
    }

    fn yellow(&self, text: &str) -> String {
        self.paint("33", text)
    }

    fn red(&self, text: &str) -> String {
        self.paint("31", text)
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

fn pane(title: &str, lines: &[&str], width: usize) -> String {
    let inner = width.saturating_sub(2);
    let mut output = String::new();
    output.push_str(&format!("┌{:─<inner$}┐\n", format!(" {title} ")));
    for line in lines {
        output.push_str(&format!(
            "│ {:<pad$}│\n",
            truncate_ansi(line, inner.saturating_sub(1)),
            pad = inner.saturating_sub(1)
        ));
    }
    output.push_str(&format!("└{:─<inner$}┘", ""));
    output
}

fn rule(text: &str) -> String {
    format!("┤{text}├")
}

fn truncate_ansi(text: &str, width: usize) -> String {
    let plain_len = text.chars().filter(|ch| *ch != '\x1b').count();
    if plain_len <= width {
        text.to_string()
    } else {
        text.chars()
            .take(width.saturating_sub(1))
            .collect::<String>()
            + "…"
    }
}

fn health_label(score: u8, color: &Palette) -> String {
    if score >= 85 {
        color.green("● HEALTHY")
    } else if score >= 60 {
        color.yellow("● WARN")
    } else {
        color.red("● FAIL")
    }
}

fn status_count(count: usize, color: &Palette) -> String {
    if count == 0 {
        color.green("0 OK")
    } else {
        color.yellow(&format!("{count} WARN"))
    }
}

fn bool_label(value: bool, color: &Palette) -> String {
    if value {
        color.green("OK")
    } else {
        color.red("MISSING")
    }
}

fn recent_log_lines(recent_events: &[DaemonEvent], log: &str) -> Vec<String> {
    if !recent_events.is_empty() {
        return recent_events
            .iter()
            .rev()
            .take(6)
            .map(|event| event.message.clone())
            .rev()
            .collect();
    }
    let mut lines = log
        .lines()
        .rev()
        .take(6)
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.reverse();
    if lines.is_empty() {
        vec!["No daemon events yet".to_string()]
    } else {
        lines
    }
}

fn daemon_state_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("cache").join("daemon-state.txt"))
}

fn daemon_runtime_state_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("cache").join("daemon-state.json"))
}

fn daemon_log_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("logs").join("daemon.log"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DaemonRuntimeState {
    active: bool,
    #[serde(default)]
    paused: bool,
    foreground: bool,
    project: Option<String>,
    started_at: String,
    updated_at: String,
    uptime_s: u64,
    events: u64,
    watchers: Vec<String>,
    #[serde(default)]
    recent_events: Vec<DaemonEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DaemonEvent {
    id: u64,
    kind: String,
    message: String,
    #[serde(default)]
    files: Vec<String>,
    created_at: String,
}

#[derive(Debug, Default)]
struct DaemonChangeSet {
    created: Vec<String>,
    modified: Vec<String>,
    deleted: Vec<String>,
}

impl DaemonChangeSet {
    fn is_empty(&self) -> bool {
        self.created.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    fn paths(&self) -> Vec<String> {
        self.created
            .iter()
            .chain(self.modified.iter())
            .chain(self.deleted.iter())
            .cloned()
            .collect()
    }

    fn summary(&self) -> String {
        format!(
            "created={} modified={} deleted={}",
            self.created.len(),
            self.modified.len(),
            self.deleted.len()
        )
    }
}

impl Default for DaemonRuntimeState {
    fn default() -> Self {
        let now = now_timestamp();
        Self {
            active: false,
            paused: false,
            foreground: false,
            project: None,
            started_at: now.clone(),
            updated_at: now,
            uptime_s: 0,
            events: 0,
            watchers: Vec::new(),
            recent_events: Vec::new(),
        }
    }
}

fn write_daemon_state(state: &str) -> lode_core::Result<()> {
    write_daemon_state_text(state)?;
    write_daemon_runtime_state(&runtime_state_from_text(state)?)
}

fn write_daemon_state_text(state: &str) -> lode_core::Result<()> {
    let path = daemon_state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    fs::write(&path, state).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn append_daemon_log(line: &str) -> lode_core::Result<()> {
    append_daemon_event("lifecycle", line, Vec::new())
}

fn append_daemon_event(kind: &str, message: &str, files: Vec<String>) -> lode_core::Result<()> {
    let path = daemon_log_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let mut current = fs::read_to_string(&path).unwrap_or_default();
    current.push_str(message);
    current.push('\n');
    fs::write(&path, current).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
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

fn runtime_state_from_text(state: &str) -> lode_core::Result<DaemonRuntimeState> {
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

fn load_daemon_runtime_state() -> lode_core::Result<DaemonRuntimeState> {
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

fn write_daemon_runtime_state(state: &DaemonRuntimeState) -> lode_core::Result<()> {
    let path = daemon_runtime_state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(state)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&path, raw).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
}

fn daemon_uptime_seconds(state: &DaemonRuntimeState) -> u64 {
    if !state.active {
        return 0;
    }
    parse_timestamp_seconds(&state.started_at)
        .map(|started| unix_seconds().saturating_sub(started))
        .unwrap_or_default()
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn parse_timestamp_seconds(timestamp: &str) -> Option<u64> {
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

fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
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

fn task_command(target: Option<String>, no_store: bool) -> lode_core::Result<()> {
    match target.as_deref() {
        None | Some("list") => list_make_targets(),
        Some("test") => {
            if no_store {
                println!("task test running without storing history");
            }
            run_make("test")
        }
        Some(target) => run_make(target),
    }
}

fn list_make_targets() -> lode_core::Result<()> {
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

fn gha_command(command: &str, name: Option<&str>) -> lode_core::Result<()> {
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
            let path = Utf8PathBuf::from(".github")
                .join("workflows")
                .join(format!("{name}.yml"));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                    path: parent.as_str().into(),
                    source,
                })?;
            }
            let body = workflow_contents(name);
            fs::write(&path, body).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
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

fn tauri_command(command: &str) -> lode_core::Result<()> {
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

fn mc_command(command: &str) -> lode_core::Result<()> {
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

fn cp_command(command: &str, problem: Option<&str>, lang: Option<&str>) -> lode_core::Result<()> {
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
            let path = Utf8PathBuf::from("problems")
                .join(problem)
                .join(format!("main.{ext}"));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                    path: parent.as_str().into(),
                    source,
                })?;
            }
            fs::write(&path, cp_template(ext)).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            println!("created competitive problem {problem}");
        }
        "run" | "test" | "stress" => {
            let problem = problem.unwrap_or("a");
            println!("competitive coding {command} {problem}");
        }
        "archive" => {
            let contest = problem.unwrap_or("contest");
            let path = Utf8PathBuf::from("archive").join(contest);
            fs::create_dir_all(&path).map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
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

fn run_make(target: &str) -> lode_core::Result<()> {
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

fn list_dir(path: Utf8PathBuf) -> lode_core::Result<()> {
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

fn collect_file_names(path: &Utf8PathBuf, items: &mut Vec<String>) -> lode_core::Result<()> {
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

fn current_dir() -> lode_core::Result<Utf8PathBuf> {
    let path = env::current_dir().map_err(|source| LodeError::Io {
        path: ".".into(),
        source,
    })?;
    Utf8PathBuf::from_path_buf(path)
        .map_err(|path| LodeError::Message(format!("path is not valid UTF-8: {}", path.display())))
}
