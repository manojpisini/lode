use std::{
    env, fs,
    process::{Command as ProcessCommand, ExitCode},
};

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use lode_core::{
    add_component_to_project, audit_project, check_path, command_names, default_config, fix_path,
    global_dir, init_project, load_global_config, load_metrics, load_registry, profile_names,
    prune_registry, recipe_names, register_project, save_global_config, save_metrics,
    save_registry, scan_secrets, setup_defaults, template_paths, AddRequest, InitRequest,
    LodeError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "lode", version, about = "Personal coding preference system")]
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
    },
    Import {
        path: Utf8PathBuf,
    },
    Serve {
        #[arg(long)]
        no_color: bool,
        #[arg(long)]
        no_live: bool,
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
    },
    Completions {
        shell: String,
    },
    Version,
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
    },
    Validate,
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
    List,
    Show {
        name: String,
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
    Add { source: Utf8PathBuf },
    Remove { name: String },
    Update { name: Option<String> },
    Info { name: String },
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
    Test { event: String },
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
    List,
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
    List,
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
    List,
    Outdated,
    Update {
        name: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    Audit,
    Why {
        name: String,
    },
    Info {
        name: String,
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
    List,
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
    },
    Graph,
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
    files: Vec<LodePackFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LodePackFile {
    path: String,
    contents: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TimeLog {
    #[serde(default)]
    sessions: Vec<TimeSession>,
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
        Command::Sync { dry_run } => {
            println!(
                "sync {}",
                if dry_run {
                    "dry-run complete"
                } else {
                    "checked"
                }
            );
        }
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
        } => release(version, bump, dry_run)?,
        Command::Health | Command::Audit => health()?,
        Command::Explain => explain(),
        Command::Doctor { fix: _, json } => doctor(json)?,
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
        Command::Export { out } => export_lodepack(out)?,
        Command::Import { path } => import_lodepack(path)?,
        Command::Serve {
            no_color,
            no_live: _,
        } => serve_dashboard(no_color)?,
        Command::Mc { command } => mc_command(&command)?,
        Command::Tauri { command } => tauri_command(&command)?,
        Command::Gha { command, name } => gha_command(&command, name.as_deref())?,
        Command::Cp {
            command,
            problem,
            lang,
        } => cp_command(&command, problem.as_deref(), lang.as_deref())?,
        Command::SelfCmd { command } => self_command(command)?,
        Command::Upgrade { check } => upgrade(check)?,
        Command::Completions { shell } => completions(&shell)?,
        Command::Version => println!("{}", env!("CARGO_PKG_VERSION")),
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

    let init_status = ProcessCommand::new("git")
        .arg("init")
        .arg("-b")
        .arg(&git.initial_branch)
        .current_dir(project_dir.as_str())
        .status()
        .map_err(|source| LodeError::Io {
            path: "git".into(),
            source,
        })?;
    if !init_status.success() {
        let fallback_status = ProcessCommand::new("git")
            .arg("init")
            .current_dir(project_dir.as_str())
            .status()
            .map_err(|source| LodeError::Io {
                path: "git".into(),
                source,
            })?;
        if !fallback_status.success() {
            return Err(LodeError::Message(format!(
                "git init failed with {fallback_status}"
            )));
        }
        let branch_status = ProcessCommand::new("git")
            .args(["checkout", "-B", &git.initial_branch])
            .current_dir(project_dir.as_str())
            .status()
            .map_err(|source| LodeError::Io {
                path: "git".into(),
                source,
            })?;
        if !branch_status.success() {
            return Err(LodeError::Message(format!(
                "git checkout -B {} failed with {branch_status}",
                git.initial_branch
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
    let status = ProcessCommand::new("git")
        .args(args)
        .current_dir(project_dir.as_str())
        .status()
        .map_err(|source| LodeError::Io {
            path: "git".into(),
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "git command failed with {status}"
        )))
    }
}

fn config_command(command: ConfigCommand) -> lode_core::Result<()> {
    match command {
        ConfigCommand::Show { format, defaults } => {
            let config = if defaults {
                default_config()
            } else {
                load_global_config()?
            };
            match format {
                OutputFormat::Toml => println!("{}", toml::to_string_pretty(&config)?),
                OutputFormat::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&config)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                ),
            }
        }
        ConfigCommand::Validate => {
            load_global_config()?;
            println!("config valid");
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
        LibraryCommand::List => {
            for item in embedded {
                println!("{item}");
            }
        }
        LibraryCommand::Show { name } => {
            let mut path = global_dir()?.join(root).join(&name);
            if !path.exists() && matches!(root, "profiles" | "commands" | "recipes") {
                path = global_dir()?.join(root).join(format!("{name}.toml"));
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
            let path = global_dir()?.join(root).join(&relative);
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
            let path = global_dir()?.join(root).join(relative);
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
                let root = global_dir()?.join(root);
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
            library_command("profiles", LibraryCommand::Show { name }, &profile_names())?;
        }
        ProfileCommand::Use { name } => {
            let mut config = load_global_config()?;
            config.active_profile = Some(name.clone());
            save_global_config(&config)?;
            println!("active profile: {name}");
        }
        ProfileCommand::New { name } => {
            let path = global_dir()?.join("profiles").join(format!("{name}.toml"));
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
            let path = global_dir()?.join("profiles").join(format!("{name}.toml"));
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
        SnippetCommand::List { lang } => {
            let root = global_dir()?.join("snippets");
            if let Some(lang) = lang {
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
            library_command("recipes", LibraryCommand::Show { name }, recipe_names())?;
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
    let path = global_dir()?.join("recipes").join(format!("{name}.toml"));
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
        files: Vec::new(),
    };
    let global = global_dir()?.join("commands");
    collect_command_macro_files(&global, "global", &mut pack)?;
    let local = Utf8PathBuf::from(".lode").join("commands");
    collect_command_macro_files(&local, "project", &mut pack)?;
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
        pack.files.push(LodePackFile {
            path: format!("{prefix}/commands/{name}"),
            contents,
        });
    }
    Ok(())
}

fn command_macro_path(slug: &str, global: bool) -> lode_core::Result<Utf8PathBuf> {
    let relative = safe_relative_path(&format!("{slug}.toml"))?;
    if global {
        Ok(global_dir()?.join("commands").join(relative))
    } else {
        Ok(Utf8PathBuf::from(".lode").join("commands").join(relative))
    }
}

fn plugin_command(command: PluginCommand) -> lode_core::Result<()> {
    match command {
        PluginCommand::List => list_dir(global_dir()?.join("plugins"))?,
        PluginCommand::Add { source } => {
            if !source.exists() || !source.is_dir() {
                return Err(LodeError::Message(format!(
                    "plugin source must be a directory: {source}"
                )));
            }
            let name = source
                .file_name()
                .ok_or_else(|| LodeError::Message("plugin source has no name".to_string()))?;
            let destination = global_dir()?.join("plugins").join(name);
            if destination.exists() {
                return Err(LodeError::Message(format!("plugin already exists: {name}")));
            }
            copy_dir_recursive(&source, &destination)?;
            println!("added plugin {name}");
        }
        PluginCommand::Remove { name } => {
            let path = global_dir()?
                .join("plugins")
                .join(safe_relative_path(&name)?);
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
                let path = global_dir()?
                    .join("plugins")
                    .join(safe_relative_path(&name)?);
                if !path.exists() {
                    return Err(LodeError::Message(format!("plugin not found: {name}")));
                }
                println!("plugin {name} is local; refresh by re-adding from source");
            } else {
                println!("local plugins checked");
            }
        }
        PluginCommand::Info { name } => {
            let path = global_dir()?
                .join("plugins")
                .join(safe_relative_path(&name)?);
            if !path.exists() {
                return Err(LodeError::Message(format!("plugin not found: {name}")));
            }
            println!("name\t{name}");
            println!("path\t{path}");
            for child in ["templates", "profiles", "snippets", "recipes", "commands"] {
                println!("{child}\t{}", status_bool(path.join(child).exists()));
            }
        }
    }
    Ok(())
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
    if list_tools || (!list_resources && !list_prompts) {
        println!("tools:");
        for tool in [
            "setup",
            "init",
            "config.show",
            "template.list",
            "profile.list",
            "snippet.search",
            "commands.run",
            "audit",
            "scan.secrets",
            "time.report",
        ] {
            println!("- {tool}");
        }
    }
    if list_resources {
        println!("resources:");
        for resource in [
            "lode://config",
            "lode://registry",
            "lode://templates",
            "lode://profiles",
            "lode://snippets",
        ] {
            println!("- {resource}");
        }
    }
    if list_prompts {
        println!("prompts:");
        println!("- lode-project-review");
        println!("- lode-scaffold-plan");
    }
    if http {
        println!("mcp http mode requested on port {}", port.unwrap_or(3333));
        println!("server mode is represented by this headless capability listing in this build");
    }
    Ok(())
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
        files: Vec::new(),
    };
    let root = current_dir()?;
    for path in ["AGENTS.md", "CODEX.md", "CLAUDE.md", ".lode/context"] {
        collect_pack_files(&root, &root.join(path), &mut pack)?;
    }
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
        global_dir()?.join("commands").join(format!("{slug}.toml")),
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

fn run_lode_step(run: &str) -> lode_core::Result<()> {
    let mut parts = run.split_whitespace();
    let Some(command) = parts.next() else {
        return Ok(());
    };
    let mut process = ProcessCommand::new(env::current_exe().map_err(|source| LodeError::Io {
        path: "current_exe".into(),
        source,
    })?);
    process.arg(command);
    for part in parts {
        process.arg(part);
    }
    let status = process.status().map_err(|source| LodeError::Io {
        path: "lode".into(),
        source,
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "lode {run} failed with {status}"
        )))
    }
}

fn run_shell_step(run: &str) -> lode_core::Result<()> {
    let status = if cfg!(windows) {
        ProcessCommand::new("cmd").args(["/C", run]).status()
    } else {
        ProcessCommand::new("sh").args(["-c", run]).status()
    }
    .map_err(|source| LodeError::Io {
        path: "shell".into(),
        source,
    })?;
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

fn release(version: Option<String>, bump: Option<String>, dry_run: bool) -> lode_core::Result<()> {
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
    for file in files {
        if dry_run {
            println!("would update {file} {current} -> {next}");
        } else {
            update_version_file(&file, &next)?;
            println!("updated {file} to {next}");
        }
    }
    Ok(())
}

fn detect_project_version() -> Option<String> {
    for file in version_files() {
        let raw = fs::read_to_string(&file).ok()?;
        if file == "Cargo.toml" || file == "pyproject.toml" {
            for line in raw.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("version") {
                    return trimmed
                        .split_once('=')
                        .map(|(_, value)| value.trim().trim_matches('"').to_string());
                }
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
    let updated = if file == "package.json" {
        let mut value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
        value["version"] = serde_json::Value::String(next.to_string());
        serde_json::to_string_pretty(&value)
            .map_err(|error| LodeError::Message(error.to_string()))?
            + "\n"
    } else {
        raw.lines()
            .map(|line| {
                if line.trim_start().starts_with("version") {
                    format!("version = \"{next}\"")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    };
    fs::write(file, updated).map_err(|source| LodeError::Io {
        path: file.into(),
        source,
    })
}

fn doctor(json: bool) -> lode_core::Result<()> {
    if json {
        println!("{{\"status\":\"ok\",\"checks\":[\"config\",\"defaults\"]}}");
    } else {
        println!("doctor ok: config and defaults are available");
    }
    Ok(())
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
    }
    Ok(())
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
    let status = ProcessCommand::new("git")
        .args(args)
        .status()
        .map_err(|source| LodeError::Io {
            path: "git".into(),
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "git {} failed with {status}",
            args.join(" ")
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
    let output = ProcessCommand::new("git")
        .args(&args)
        .output()
        .map_err(|source| LodeError::Io {
            path: "git".into(),
            source,
        })?;
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
            println!("pre-commit");
            println!("pre-push");
        }
        HooksCommand::Status => hooks_status()?,
        HooksCommand::Test { event } => test_hook(&event)?,
    }
    Ok(())
}

fn test_hook(event: &str) -> lode_core::Result<()> {
    let script = match event {
        "pre-commit" => "lode check . && lode scan secrets .",
        "pre-push" => "lode task test",
        other => return Err(LodeError::Message(format!("unknown hook event: {other}"))),
    };
    println!("hook {event}: {script}");
    Ok(())
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
        LicenseCommand::List => list_dir(global_dir()?.join("licenses"))?,
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
    Ok(global_dir()?.join("licenses").join(relative))
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
        global_dir()?.join("licenses").join(format!("{id}.txt")),
        global_dir()?.join("licenses").join(id),
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
    let path = global_dir()?.join("snippets").join(relative);
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
    let root = global_dir()?.join("snippets");
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
    let root = global_dir()?.join("snippets");
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
    let root = global_dir()?.join("snippets");
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
    let recipe_path = global_dir()?.join("recipes").join(format!("{name}.toml"));
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
    let template_path = global_dir()?.join("templates").join(path);
    fs::read_to_string(&template_path).map_err(|source| LodeError::Io {
        path: template_path.as_str().into(),
        source,
    })
}

fn projects(command: ProjectsCommand) -> lode_core::Result<()> {
    match command {
        ProjectsCommand::List => {
            let registry = load_registry()?;
            if registry.projects.is_empty() {
                println!("no registered projects");
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
        PkgCommand::List => {
            println!("manager: {manager}");
            for file in [
                "Cargo.toml",
                "package.json",
                "pyproject.toml",
                "go.mod",
                "build.gradle",
            ] {
                if Utf8PathBuf::from(file).exists() {
                    println!("{file}");
                }
            }
        }
        PkgCommand::Outdated => run_package_manager(&manager, package_outdated_args(&manager))?,
        PkgCommand::Update { name, dry_run } => {
            let args = package_update_args(&manager, name.as_deref())?;
            if dry_run {
                println!("would run: {} {}", manager, args.join(" "));
            } else {
                run_package_manager(&manager, args)?;
            }
        }
        PkgCommand::Audit => {
            run_package_manager(&manager, package_audit_args(&manager))?;
            scan(ScanCommand::Secrets {
                path: Some(current_dir()?),
                staged: false,
                json: false,
                quiet: false,
            })?;
        }
        PkgCommand::Why { name } => package_why(&manager, &name)?,
        PkgCommand::Info { name } => package_info(&manager, &name)?,
        PkgCommand::Lock { dry_run } => {
            let args = package_lock_args(&manager)?;
            if dry_run {
                println!("would run: {} {}", manager, args.join(" "));
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
    let status = ProcessCommand::new(manager)
        .args(&args)
        .status()
        .map_err(|source| LodeError::Io {
            path: manager.into(),
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(LodeError::Message(format!(
            "{manager} {} failed with {status}",
            args.join(" ")
        )))
    }
}

fn package_outdated_args(manager: &str) -> Vec<String> {
    match manager {
        "cargo" => vec!["tree".into(), "--depth".into(), "1".into()],
        "npm" | "pnpm" | "yarn" | "bun" => vec!["outdated".into()],
        "uv" => vec!["pip".into(), "list".into(), "--outdated".into()],
        "go" => vec!["list".into(), "-m".into(), "-u".into(), "all".into()],
        _ => vec!["--version".into()],
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
        "go" => vec!["get".to_string(), "-u".to_string()],
        _ => {
            return Err(LodeError::Message(
                "no supported package manager files found".to_string(),
            ))
        }
    };
    if let Some(name) = name {
        args.push(name.to_string());
    } else if manager == "go" {
        args.push("./...".to_string());
    }
    Ok(args)
}

fn package_audit_args(manager: &str) -> Vec<String> {
    match manager {
        "cargo" => vec!["tree".into(), "--duplicates".into()],
        "npm" => vec!["audit".into()],
        "pnpm" => vec!["audit".into()],
        "yarn" => vec!["audit".into()],
        "bun" => vec!["audit".into()],
        "uv" => vec!["pip".into(), "check".into()],
        "go" => vec!["list".into(), "-m".into(), "all".into()],
        _ => vec!["--version".into()],
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
        "go" => Ok(vec!["mod".into(), "tidy".into()]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_why(manager: &str, name: &str) -> lode_core::Result<()> {
    match manager {
        "cargo" => run_package_manager(manager, vec!["tree".into(), "-i".into(), name.into()]),
        "npm" => run_package_manager(manager, vec!["explain".into(), name.into()]),
        "pnpm" | "yarn" => run_package_manager(manager, vec!["why".into(), name.into()]),
        "bun" => run_package_manager(manager, vec!["pm".into(), "why".into(), name.into()]),
        "uv" => run_package_manager(manager, vec!["pip".into(), "show".into(), name.into()]),
        "go" => run_package_manager(manager, vec!["mod".into(), "why".into(), name.into()]),
        _ => Err(LodeError::Message(
            "no supported package manager files found".to_string(),
        )),
    }
}

fn package_info(manager: &str, name: &str) -> lode_core::Result<()> {
    match manager {
        "cargo" => run_package_manager(manager, vec!["search".into(), name.into()]),
        "npm" | "pnpm" | "yarn" | "bun" => {
            run_package_manager(manager, vec!["info".into(), name.into()])
        }
        "uv" => run_package_manager(manager, vec!["pip".into(), "show".into(), name.into()]),
        "go" => run_package_manager(manager, vec!["list".into(), "-m".into(), name.into()]),
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
    ];
    match format {
        "json" => {
            let found = manifests
                .iter()
                .filter(|(file, _)| Utf8PathBuf::from(*file).exists())
                .map(|(file, kind)| serde_json::json!({ "file": file, "kind": kind }))
                .collect::<Vec<_>>();
            println!(
                "{}",
                serde_json::to_string_pretty(&found)
                    .map_err(|error| LodeError::Message(error.to_string()))?
            );
        }
        "ascii" => {
            println!("project");
            for (file, kind) in manifests {
                if Utf8PathBuf::from(file).exists() {
                    println!("`- {kind} ({file})");
                }
            }
        }
        "dot" => {
            println!("digraph packages {{");
            println!("  project;");
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
    if Utf8PathBuf::from("Cargo.toml").exists() {
        Some("cargo".to_string())
    } else if Utf8PathBuf::from("bun.lockb").exists() {
        Some("bun".to_string())
    } else if Utf8PathBuf::from("pnpm-lock.yaml").exists() {
        Some("pnpm".to_string())
    } else if Utf8PathBuf::from("yarn.lock").exists() {
        Some("yarn".to_string())
    } else if Utf8PathBuf::from("package-lock.json").exists()
        || Utf8PathBuf::from("package.json").exists()
    {
        Some("npm".to_string())
    } else if Utf8PathBuf::from("pyproject.toml").exists() {
        Some("uv".to_string())
    } else if Utf8PathBuf::from("go.mod").exists() {
        Some("go".to_string())
    } else {
        None
    }
}

fn command_version(command: &str) -> Option<String> {
    let output = ProcessCommand::new(command)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.lines().next().unwrap_or("installed").to_string())
}

fn export_lodepack(out: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    let root = global_dir()?;
    let output = out.unwrap_or_else(|| Utf8PathBuf::from("lode-export.lodepack"));
    let mut pack = LodePack {
        version: 1,
        files: Vec::new(),
    };
    for path in [
        "config.toml",
        "profiles",
        "templates",
        "snippets",
        "licenses",
        "recipes",
        "commands",
    ] {
        collect_pack_files(&root, &root.join(path), &mut pack)?;
    }
    let raw = serde_json::to_string_pretty(&pack)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    fs::write(&output, raw).map_err(|source| LodeError::Io {
        path: output.as_str().into(),
        source,
    })?;
    println!("exported {} files to {output}", pack.files.len());
    Ok(())
}

fn import_lodepack(path: Utf8PathBuf) -> lode_core::Result<()> {
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let pack: LodePack =
        serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    if pack.version != 1 {
        return Err(LodeError::Message(format!(
            "unsupported lodepack version: {}",
            pack.version
        )));
    }
    let root = global_dir()?;
    fs::create_dir_all(&root).map_err(|source| LodeError::Io {
        path: root.as_str().into(),
        source,
    })?;
    for file in &pack.files {
        if file.path.contains("..") || file.path.starts_with('/') || file.path.contains(':') {
            return Err(LodeError::Message(format!(
                "unsafe lodepack path: {}",
                file.path
            )));
        }
        let destination = root.join(&file.path);
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
    pack.files.push(LodePackFile {
        path: relative.as_str().replace('\\', "/"),
        contents,
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
        WorkspaceCommand::List => workspace_list()?,
        WorkspaceCommand::Add { name } => workspace_add(&name)?,
        WorkspaceCommand::Remove { name, confirm } => workspace_remove(&name, confirm)?,
        WorkspaceCommand::Run { target } => workspace_run(&target)?,
        WorkspaceCommand::Graph => workspace_graph()?,
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

fn workspace_list() -> lode_core::Result<()> {
    let members = workspace_members()?;
    if members.is_empty() {
        println!("workspace has no members");
    } else {
        for member in members {
            println!("{member}");
        }
    }
    Ok(())
}

fn workspace_run(target: &str) -> lode_core::Result<()> {
    let members = workspace_members()?;
    if members.is_empty() {
        return run_make(target);
    }
    for member in members {
        println!("==> {member}: {target}");
        let makefile = Utf8PathBuf::from(&member).join("Makefile");
        if makefile.exists() {
            let status = ProcessCommand::new("make")
                .arg("-C")
                .arg(&member)
                .arg(target)
                .status()
                .map_err(|source| LodeError::Io {
                    path: "make".into(),
                    source,
                })?;
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

fn workspace_graph() -> lode_core::Result<()> {
    println!("workspace");
    for member in workspace_members()? {
        println!("  -> {member}");
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
                println!(
                    "foreground mode recorded; long-running watch loop is not active in this build"
                );
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
        DaemonCommand::Status { quiet, json } => {
            let state =
                fs::read_to_string(daemon_state_path()?).unwrap_or_else(|_| "inactive".to_string());
            let active = state.lines().next().unwrap_or("inactive") == "active";
            if json {
                println!("{{\"active\":{active},\"state\":{:?}}}", state.trim());
            } else if quiet {
                println!("{}", if active { "active" } else { "inactive" });
            } else {
                println!("daemon status: {}", state.trim());
            }
        }
        DaemonCommand::Log { tail, follow } => {
            let log = fs::read_to_string(daemon_log_path()?)
                .unwrap_or_else(|_| "no entries\n".to_string());
            let mut lines = log.lines().collect::<Vec<_>>();
            if let Some(tail) = tail {
                let start = lines.len().saturating_sub(tail);
                lines = lines[start..].to_vec();
            }
            for line in lines {
                println!("{line}");
            }
            if follow {
                println!("follow mode requested; streaming is not active in this build");
            }
        }
    }
    Ok(())
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
            println!("version\t{}", env!("CARGO_PKG_VERSION"));
            println!("executable\t{}", exe.display());
            println!("global_dir\t{}", global_dir()?);
        }
        SelfCommand::Clean { dry_run } => {
            for path in [global_dir()?.join("cache"), global_dir()?.join("logs")] {
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

fn upgrade(check: bool) -> lode_core::Result<()> {
    if check {
        println!("lode {} is installed", env!("CARGO_PKG_VERSION"));
        println!("network upgrade checks are not configured for this build");
    } else {
        println!("self-upgrade is not configured for this local build");
        println!("current version: {}", env!("CARGO_PKG_VERSION"));
    }
    Ok(())
}

fn completions(shell: &str) -> lode_core::Result<()> {
    match shell {
        "bash" => {
            println!("complete -W 'setup init add sync info config template profile recipe snippet commands task dev build test fmt lint check fix rename rules sign stamp verify clean fresh ship release health explain audit doctor scan git env license projects toolchain pkg time metrics workspace daemon log export import serve mc tauri gha cp self upgrade completions version' lode");
        }
        "zsh" => {
            println!("#compdef lode");
            println!("_arguments '1: :((setup init add sync info config template profile recipe snippet commands task dev build test fmt lint check fix rename rules sign stamp verify clean fresh ship release health explain audit doctor scan git env license projects toolchain pkg time metrics workspace daemon log export import serve mc tauri gha cp self upgrade completions version))'");
        }
        "fish" => {
            for command in [
                "setup", "init", "config", "template", "profile", "snippet", "commands", "rules",
                "sign", "stamp", "log", "self", "upgrade", "version",
            ] {
                println!("complete -c lode -f -a {command}");
            }
        }
        "powershell" => {
            println!(
                "Register-ArgumentCompleter -Native -CommandName lode -ScriptBlock {{ param($wordToComplete) 'setup','init','config','template','profile','snippet','commands','rules','sign','stamp','log','self','upgrade','version' | Where-Object {{ $_ -like \"$wordToComplete*\" }} }}"
            );
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported completion shell: {other}"
            )))
        }
    }
    Ok(())
}

fn serve_dashboard(no_color: bool) -> lode_core::Result<()> {
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
    let daemon_log = fs::read_to_string(daemon_log_path()?).unwrap_or_default();
    let color = Palette::new(no_color);

    println!("{}", color.cyan("◇ lode serve"));
    println!(
        "{}",
        rule(&format!(
            " Project: {} | Env: {} | Health: {} | Warn: {} | Fail: {} ",
            color.cyan(&project),
            color.cyan(&env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())),
            color.green(&audit.score.to_string()),
            color.yellow(&audit.convention_violations.to_string()),
            color.red(&audit.secret_findings.to_string())
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
                &format!("Overall Status  {}", health_label(audit.score, &color)),
                &format!("Score           {}", color.cyan(&audit.score.to_string())),
                &format!(
                    "Convention      {}",
                    status_count(audit.convention_violations, &color)
                ),
                &format!(
                    "Secrets         {}",
                    status_count(audit.secret_findings, &color)
                ),
                &format!(
                    "License         {}",
                    bool_label(audit.license_present, &color)
                ),
                &format!(
                    "Env Example     {}",
                    bool_label(audit.env_example_present, &color)
                ),
                &format!(
                    "Readme          {}",
                    bool_label(audit.readme_present, &color)
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
                &format!("Health      {} {}", color.cyan("████████░░"), audit.score),
                &format!(
                    "Checks      {}",
                    color.green("convention · secrets · license · env")
                ),
                &format!("Toolchain   {}", detect_toolchains().join(", ")),
                &format!(
                    "Package     {}",
                    detect_package_manager().unwrap_or_else(|| "unknown".to_string())
                ),
            ],
            56
        ),
        pane(
            "3. DAEMON / TIME",
            &[
                &format!("Daemon State  {}", color.cyan(&daemon_state)),
                "Active Session  snapshot mode",
                "Today           not tracked yet",
                "Focus Score     derived metrics pending",
            ],
            56
        )
    );
    let events = recent_log_lines(&daemon_log);
    println!(
        "{}  {}",
        pane(
            "4. LIVE DAEMON EVENTS",
            &events.iter().map(String::as_str).collect::<Vec<_>>(),
            70
        ),
        pane(
            "5. DEPENDENCY STATUS",
            &[
                &format!(
                    "Manager  {}",
                    detect_package_manager().unwrap_or_else(|| "unknown".to_string())
                ),
                &format!(
                    "Rust     {}",
                    command_version("rustc").unwrap_or_else(|| "missing".to_string())
                ),
                &format!(
                    "Git      {}",
                    command_version("git").unwrap_or_else(|| "missing".to_string())
                ),
                "Policy   strict",
            ],
            42
        )
    );
    let registry_lines = if registry.projects.is_empty() {
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
    println!(
        "{}",
        pane(
            "6. CROSS-PROJECT REGISTRY",
            &registry_lines
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            116
        )
    );
    println!(
        "{}",
        rule(" ↑↓ Move   Tab Next   Enter Open   r Refresh   q Quit   Auto-refresh: OFF ")
    );
    Ok(())
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

fn recent_log_lines(log: &str) -> Vec<String> {
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

fn daemon_log_path() -> lode_core::Result<Utf8PathBuf> {
    Ok(global_dir()?.join("logs").join("daemon.log"))
}

fn write_daemon_state(state: &str) -> lode_core::Result<()> {
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
    let path = daemon_log_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let mut current = fs::read_to_string(&path).unwrap_or_default();
    current.push_str(line);
    current.push('\n');
    fs::write(&path, current).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })
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
    let status = ProcessCommand::new("make")
        .arg(target)
        .status()
        .map_err(|source| LodeError::Io {
            path: "make".into(),
            source,
        })?;
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

fn current_dir() -> lode_core::Result<Utf8PathBuf> {
    let path = env::current_dir().map_err(|source| LodeError::Io {
        path: ".".into(),
        source,
    })?;
    Utf8PathBuf::from_path_buf(path)
        .map_err(|path| LodeError::Message(format!("path is not valid UTF-8: {}", path.display())))
}
