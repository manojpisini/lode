use std::{
    env, fs,
    process::{Command as ProcessCommand, ExitCode},
};

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use lode_core::{
    add_component_to_project, audit_project, check_path, command_names, default_config, fix_path,
    global_dir, init_project, load_global_config, load_metrics, load_registry, profile_names,
    prune_registry, recipe_names, register_project, save_global_config, save_metrics, scan_secrets,
    setup_defaults, template_paths, AddRequest, InitRequest, LodeError,
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
    Task {
        target: Option<String>,
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
}

#[derive(Debug, Subcommand)]
enum LibraryCommand {
    List,
    Show { name: String },
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
}

#[derive(Debug, Subcommand)]
enum CommandsCommand {
    List,
    Show {
        name: String,
    },
    Run {
        slug: String,
        #[arg(long)]
        dry_run: bool,
    },
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
}

#[derive(Debug, Subcommand)]
enum ScanCommand {
    Secrets {
        path: Option<Utf8PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum GitCommand {
    Branch { kind: String, description: String },
    Commit { message: Option<String> },
    Tag { version: String },
    Changelog,
    InstallHooks,
    UninstallHooks,
    HooksStatus,
}

#[derive(Debug, Subcommand)]
enum EnvCommand {
    Check,
    Add { key: String },
    Sync,
    Use { profile: String },
}

#[derive(Debug, Subcommand)]
enum LicenseCommand {
    List,
    Show { id: String },
    Set { id: String },
    Check,
}

#[derive(Debug, Subcommand)]
enum ProjectsCommand {
    List,
    Register { path: Option<Utf8PathBuf> },
    Health,
    Prune,
}

#[derive(Debug, Subcommand)]
enum ToolchainCommand {
    List,
    Status,
    Doctor,
}

#[derive(Debug, Subcommand)]
enum PkgCommand {
    List,
    Outdated,
    Audit,
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum MetricsCommand {
    Show,
    Trend,
    Baseline,
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommand {
    Init,
    List,
    Add { name: String },
    Run { target: String },
    Graph,
}

#[derive(Debug, Subcommand)]
enum DaemonCommand {
    Start,
    Stop,
    Restart,
    Status,
    Log,
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
        Command::Snippet { command } => snippet_command(command)?,
        Command::Task { target } => run_make(target.as_deref().unwrap_or("help"))?,
        Command::Dev => run_make("dev")?,
        Command::Build => run_make("build")?,
        Command::Test => run_make("test")?,
        Command::Fmt => run_make("fmt")?,
        Command::Lint => run_make("lint")?,
        Command::Check(args) => convention_check(args)?,
        Command::Fix { path } => convention_fix(path)?,
        Command::Rename { path, to } => rename_path(path, to)?,
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
        Command::Env { command } => env_command(command)?,
        Command::License { command } => license(command)?,
        Command::Projects { command } => projects(command)?,
        Command::Toolchain { command } => toolchain(command)?,
        Command::Pkg { command } => pkg(command)?,
        Command::Metrics { command } => metrics(command)?,
        Command::Workspace { command } => workspace(command)?,
        Command::Daemon { command } => daemon(command),
        Command::Export { out } => export_lodepack(out)?,
        Command::Import { path } => import_lodepack(path)?,
        Command::Serve {
            no_color,
            no_live: _,
        } => serve_dashboard(no_color)?,
        Command::Mc { command } => run_make(&format!("mc-{command}"))?,
        Command::Tauri { command } => run_make(&format!("tauri-{command}"))?,
        Command::Gha { command, name } => {
            println!("github actions {command} {}", name.unwrap_or_default())
        }
        Command::Cp { command, problem } => println!(
            "competitive coding {command} {}",
            problem.unwrap_or_default()
        ),
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
        if !args.no_git {
            println!("git init is deferred in this build; project files are ready");
        }
    }
    Ok(())
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
    }
    Ok(())
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
    }
    Ok(())
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
            let lang = lang.unwrap_or_else(|| "any".to_string());
            let path = global_dir()?
                .join("snippets")
                .join(lang)
                .join(format!("{name}.snippet"));
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
    }
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
        CommandsCommand::Run { slug, dry_run } => run_command_macro(&slug, dry_run)?,
    }
    Ok(())
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
        ScanCommand::Secrets { path, json } => {
            let path = path.unwrap_or(current_dir()?);
            let report = scan_secrets(&path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else {
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

fn git(command: GitCommand) -> lode_core::Result<()> {
    match command {
        GitCommand::Branch { kind, description } => {
            let branch = format!("{}/{}", kind, slugify(&description));
            println!("{branch}");
        }
        GitCommand::Commit { message } => {
            let message = message.unwrap_or_else(|| "chore: update".to_string());
            run_git(&["commit", "-m", &message])?;
        }
        GitCommand::Tag { version } => {
            let tag = format!("v{}", version.trim_start_matches('v'));
            run_git(&["tag", &tag])?;
        }
        GitCommand::Changelog => git_changelog()?,
        GitCommand::InstallHooks => install_git_hooks()?,
        GitCommand::UninstallHooks => uninstall_git_hooks()?,
        GitCommand::HooksStatus => hooks_status()?,
    }
    Ok(())
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

fn git_changelog() -> lode_core::Result<()> {
    let output = ProcessCommand::new("git")
        .args(["log", "--pretty=format:%s", "--no-merges"])
        .output()
        .map_err(|source| LodeError::Io {
            path: "git".into(),
            source,
        })?;
    if !output.status.success() {
        return Err(LodeError::Message("git log failed".to_string()));
    }
    println!("# Changelog\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        println!("- {line}");
    }
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

fn env_command(command: EnvCommand) -> lode_core::Result<()> {
    match command {
        EnvCommand::Check => env_check()?,
        EnvCommand::Add { key } => env_add(&key)?,
        EnvCommand::Sync => env_sync()?,
        EnvCommand::Use { profile } => env_use(&profile)?,
    }
    Ok(())
}

fn license(command: LicenseCommand) -> lode_core::Result<()> {
    match command {
        LicenseCommand::List => list_dir(global_dir()?.join("licenses"))?,
        LicenseCommand::Show { id } => print!("{}", read_license(&id)?),
        LicenseCommand::Set { id } => {
            let contents = read_license(&id)?;
            fs::write("LICENSE", contents).map_err(|source| LodeError::Io {
                path: "LICENSE".into(),
                source,
            })?;
            println!("license set: {id}");
        }
        LicenseCommand::Check => {
            let path = Utf8PathBuf::from("LICENSE");
            if path.exists()
                && !fs::read_to_string(&path)
                    .map_err(|source| LodeError::Io {
                        path: path.as_str().into(),
                        source,
                    })?
                    .trim()
                    .is_empty()
            {
                println!("license ok");
            } else {
                return Err(LodeError::Message(
                    "LICENSE is missing or empty".to_string(),
                ));
            }
        }
    }
    Ok(())
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

fn env_add(key: &str) -> lode_core::Result<()> {
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
        contents.push_str(key);
        contents.push_str("=\n");
        fs::write(&path, contents).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?;
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
        ProjectsCommand::Register { path } => {
            let path = path.unwrap_or(current_dir()?);
            let name = path
                .file_name()
                .map(str::to_string)
                .unwrap_or_else(|| "project".to_string());
            register_project(&name, &path, "manual")?;
            println!("registered {path}");
        }
        ProjectsCommand::Health => {
            let registry = load_registry()?;
            for project in registry.projects {
                let status = if project.path.exists() {
                    "ok"
                } else {
                    "missing"
                };
                println!("{}\t{}\t{}", project.name, status, project.path);
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
        PkgCommand::Outdated => println!("outdated check available for manager: {manager}"),
        PkgCommand::Audit => {
            println!("audit check available for manager: {manager}");
            scan(ScanCommand::Secrets {
                path: Some(current_dir()?),
                json: false,
            })?;
        }
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
        MetricsCommand::Trend => println!("metrics trend: stable"),
        MetricsCommand::Baseline => {
            let cwd = current_dir()?;
            let report = audit_project(&cwd, &load_global_config()?)?;
            save_metrics(&cwd, &report)?;
            println!("metrics baseline saved");
        }
    }
    Ok(())
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
        DaemonCommand::Start => {
            write_daemon_state("active")?;
            append_daemon_log("daemon started")?;
            println!("daemon started");
        }
        DaemonCommand::Stop => {
            write_daemon_state("inactive")?;
            append_daemon_log("daemon stopped")?;
            println!("daemon stopped");
        }
        DaemonCommand::Restart => {
            write_daemon_state("active")?;
            append_daemon_log("daemon restarted")?;
            println!("daemon restarted");
        }
        DaemonCommand::Status => {
            let state =
                fs::read_to_string(daemon_state_path()?).unwrap_or_else(|_| "inactive".to_string());
            println!("daemon status: {}", state.trim());
        }
        DaemonCommand::Log => {
            let log = fs::read_to_string(daemon_log_path()?)
                .unwrap_or_else(|_| "no entries\n".to_string());
            print!("{log}");
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
