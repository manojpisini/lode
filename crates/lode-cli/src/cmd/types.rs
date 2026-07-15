#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::{now_timestamp, package_command, package_dependencies};

#[derive(Debug, Parser)]
#[command(
    name = "lode",
    version,
    about = "LODE — Local developer tool: projects, secrets, daemon, MCP, LSP, plugins, git, and more",
    long_about = "LODE is an all-in-one local developer tool for Rust projects.\n\n\
Manage projects, track time, scan secrets, enforce conventions, run\ndaemons, serve MCP/LSP protocols, manage plugins and packages,\nautomate git workflows, and export/import portable LodePacks.\n\nUse lode <command> --help for detailed options.",
    allow_external_subcommands = true
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Initialize lode configuration, directories, and defaults
    Setup {
        #[arg(long, help = "Apply default settings")]
        defaults: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Create a new project with scaffolding
    #[command(alias = "new")]
    Init(InitArgs),
    /// Analyze an existing project and generate an adoption plan
    Adopt {
        #[arg(help = "Project directory to analyze (default: current dir)")]
        path: Option<camino::Utf8PathBuf>,
        #[arg(long, help = "Apply the adoption plan (create .lode/project.toml)")]
        apply: bool,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Add a component, toolchain, or integration to the project
    Add {
        component: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, help = "Overwrite existing files")]
        overwrite: bool,
    },
    /// Sync configuration, templates, agent files, and metrics
    Sync {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, help = "Overwrite existing files")]
        force: bool,
        #[arg(
            long,
            help = "Only sync a specific section (config, templates, agent, metrics)"
        )]
        section: Option<String>,
    },
    /// Show project information and metadata
    Info {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// View or change configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Manage the template library (list, show, diff, reset, edit)
    Template {
        #[command(subcommand)]
        command: LibraryCommand,
    },
    /// Manage template bundles (apply, capture, preview, list, show, validate, verify)
    #[command(name = "template-bundle")]
    TemplateBundle {
        #[command(subcommand)]
        command: TemplateBundleCommand,
    },
    /// Manage profiles (list, show, use, new, delete)
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    /// Manage task recipes (list, show, apply, compose, new)
    Recipe {
        #[command(subcommand)]
        command: RecipeCommand,
    },
    /// Manage code snippets (list, show, search, add, remove, export, edit)
    Snippet {
        #[command(subcommand)]
        command: SnippetCommand,
    },
    /// Manage custom command macros (list, show, add, remove, run, edit)
    Commands {
        #[command(subcommand)]
        command: CommandsCommand,
    },
    /// Manage plugins (list, search, add, remove, update, info)
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },
    /// Manage policy-as-code checks and waivers
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    /// Start the MCP protocol server (stdio or HTTP)
    Mcp {
        #[arg(long, help = "Use HTTP transport instead of stdio")]
        http: bool,
        #[arg(long, help = "Port for HTTP transport (default: 8080)")]
        port: Option<u16>,
        #[arg(long, help = "List available MCP tools and exit")]
        list_tools: bool,
        #[arg(long, help = "List available MCP resources and exit")]
        list_resources: bool,
        #[arg(long, help = "List available MCP prompts and exit")]
        list_prompts: bool,
    },
    /// Start the LSP protocol server (stdio or capabilities)
    Lsp {
        #[arg(long, help = "Use stdin/stdout transport")]
        stdio: bool,
        #[arg(long, help = "Print server capabilities and exit")]
        capabilities: bool,
    },
    /// Manage AI agent files and plans (sync, status, export, plan)
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Simulate agent intent resolution
    AgentSim {
        #[command(subcommand)]
        command: AgentSimCommand,
    },
    /// List and apply project archetypes
    Archetype {
        #[command(subcommand)]
        command: ArchetypeCommand,
    },
    /// Manage the result cache (stats, clear)
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
    /// Track the current work task
    Task {
        target: Option<String>,
        #[arg(long, help = "Skip storing the task in logs")]
        no_store: bool,
    },
    /// Run the development workflow
    Dev,
    /// Build the project
    Build,
    /// Run tests
    Test,
    /// Format code with rustfmt
    Fmt,
    /// Lint with clippy
    Lint,
    /// Check project convention compliance
    Check(CheckArgs),
    /// Auto-fix convention violations
    Fix { path: Option<Utf8PathBuf> },
    /// Rename a file or directory
    Rename {
        path: Utf8PathBuf,
        #[arg(long, help = "New name (if omitted, computed from conventions)")]
        to: Option<String>,
    },
    /// Manage convention rules (list, check, validate)
    Rules {
        #[command(subcommand)]
        command: RulesCommand,
    },
    /// Insert or update file signature blocks
    Sign {
        path: Option<Utf8PathBuf>,
        #[arg(long, value_delimiter = ',', help = "File extensions to target")]
        ext: Vec<String>,
        #[arg(long, help = "Overwrite existing signatures")]
        force: bool,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Insert or update license header stamps
    Stamp {
        path: Option<Utf8PathBuf>,
        #[arg(long, value_delimiter = ',', help = "File extensions to target")]
        ext: Vec<String>,
        #[arg(long, help = "Also insert license text")]
        license: bool,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Execute commands in a hermetic sandbox
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommand,
    },
    /// Run project verification checks
    Verify {
        #[arg(long, help = "Check file manifest for externally modified files")]
        changed: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Clean build artifacts
    Clean,
    /// Clean and perform a full rebuild
    Fresh,
    /// Verify project and create a release
    Ship,
    /// Manage version releases (bump, dry-run, rollback)
    Release {
        version: Option<String>,
        #[arg(long, help = "Semver bump level (major, minor, patch)")]
        bump: Option<String>,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, help = "Roll back a failed release")]
        rollback: bool,
    },
    /// Run a project health audit
    Health {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Explain a concept or lode feature
    Explain,
    /// Run a project health audit (alias for health)
    Audit {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Run system diagnostics and optionally fix issues
    Doctor {
        #[arg(long, help = "Attempt to auto-fix detected issues")]
        fix: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Scan for secrets or foreign (non-lode) projects
    Scan {
        #[command(subcommand)]
        command: ScanCommand,
    },
    /// Manage the secret vault (set, get, list, remove, grant, revoke)
    SecretVault {
        #[command(subcommand)]
        command: SecretVaultCommand,
    },
    /// Git workflow automation (branch, commit, tag, changelog, hooks)
    Git {
        #[command(subcommand)]
        command: GitCommand,
    },
    /// Manage and search LODE assets (search, show, catalog, list)
    Assets {
        #[command(subcommand)]
        command: AssetsCommand,
    },
    /// Manage organization packs (list, use, layer, export)
    Pack {
        #[command(subcommand)]
        command: PackCommand,
    },
    /// Manage execution plans (create, show, validate, apply, rollback, list)
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
    /// Declarative project manifest management (plan, apply, diff, reconcile, explain)
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// Manage the asset lockfile (show, verify, update, diff)
    Lock {
        #[command(subcommand)]
        command: LockCommand,
    },
    /// Manage command receipts (list, show, resume)
    Receipts {
        #[command(subcommand)]
        command: ReceiptCommand,
    },
    /// Manage project context packs (build, show, diff, verify)
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
    /// Create and manage agent handoffs (create, show, verify, resume, list)
    Handoff {
        #[command(subcommand)]
        command: HandoffCommand,
    },
    /// Manage git hooks (list, status, test, run)
    Hooks {
        #[command(subcommand)]
        command: HooksCommand,
    },
    /// Manage environment variables (check, add, sync, use)
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    /// Manage environment snapshots (create, list, compare, restore)
    EnvSnapshot {
        #[command(subcommand)]
        command: EnvSnapshotCommand,
    },
    /// Manage licenses (list, show, add, remove, set, apply)
    License {
        #[command(subcommand)]
        command: LicenseCommand,
    },
    /// Manage tracked files in the file manifest (list, check, add, remove)
    File {
        #[command(subcommand)]
        command: FileCommand,
    },
    /// Manage the project registry (list, cd, register, remove, health, prune)
    Projects {
        #[command(subcommand)]
        command: ProjectsCommand,
    },
    /// Manage runtime toolchains (list, status, add, remove, use, pin, update)
    Toolchain {
        #[command(subcommand)]
        command: ToolchainCommand,
    },
    /// Manage packages (list, outdated, update, audit, graph, clean)
    Pkg {
        #[command(subcommand)]
        command: PkgCommand,
    },
    /// Generate or check self-documenting asset docs from catalog metadata
    Docs {
        #[command(subcommand)]
        command: DocsCommand,
    },
    /// Track time spent on projects
    Time {
        #[command(subcommand)]
        command: TimeCommand,
    },
    /// Diagnose build/test failures from known patterns
    Diagnose {
        #[command(subcommand)]
        command: DiagnoseCommand,
    },
    /// View project metrics (show, trend, baseline, diff-baseline)
    Metrics {
        #[command(subcommand)]
        command: MetricsCommand,
    },
    /// Manage schema and data migrations (plan, apply, rollback, list)
    Migration {
        #[command(subcommand)]
        command: MigrationCommand,
    },
    /// Manage multi-crate workspaces (init, list, add, remove, run, graph)
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    /// Analyze and visualize the asset dependency graph
    DepGraph {
        #[command(subcommand)]
        command: DepGraphCommand,
    },
    /// Manage the background file watcher daemon
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Manage logs (init, daemon, clear)
    Log {
        #[command(subcommand)]
        command: LogCommand,
    },
    /// Export the project as a portable LodePack archive
    Export {
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
        #[arg(long, help = "Exclude plugins from the pack")]
        no_plugins: bool,
        #[arg(long, help = "Exclude templates from the pack")]
        no_templates: bool,
        #[arg(long, help = "Exclude snippets from the pack")]
        no_snippets: bool,
        #[arg(long, help = "Exclude licenses from the pack")]
        no_licenses: bool,
        #[arg(long, help = "Exclude recipes from the pack")]
        no_recipes: bool,
        #[arg(long, help = "Exclude command macros from the pack")]
        no_commands: bool,
        #[arg(long, help = "Include metrics snapshot in the pack")]
        include_metrics: bool,
    },
    /// Import a LodePack archive into the project
    Import {
        path: Utf8PathBuf,
        #[arg(long, help = "Merge instead of overwriting")]
        no_merge: bool,
        #[arg(long, help = "Force import even if conflicts exist")]
        force: bool,
    },
    /// Start the terminal UI dashboard
    Serve {
        #[arg(long, help = "Disable color output")]
        no_color: bool,
        #[arg(long, help = "Disable live file watching")]
        no_live: bool,
        #[arg(long, help = "Initial pane to show")]
        pane: Option<String>,
        #[arg(long, default_value = "5000", help = "Refresh interval in ms")]
        refresh: u64,
        #[arg(long, default_value = "dark", help = "UI theme (dark or light)")]
        theme: String,
    },
    /// Run Minecraft-related commands
    Mc { command: String },
    /// Run Tauri-related commands
    Tauri { command: String },
    /// Manage GitHub Actions workflows
    Gha {
        command: String,
        name: Option<String>,
    },
    /// Competitive programming helper (create problem files from templates)
    Cp {
        command: String,
        problem: Option<String>,
        #[arg(long, help = "Programming language")]
        lang: Option<String>,
    },
    /// Self-management commands (info, clean, uninstall)
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        command: SelfCommand,
    },
    /// Check for and apply self-upgrades
    Upgrade {
        #[arg(long, help = "Check for updates without upgrading")]
        check: bool,
        #[arg(long, help = "Path to upgrade manifest")]
        manifest: Option<Utf8PathBuf>,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, help = "Roll back the last upgrade")]
        rollback: bool,
    },
    /// Generate shell completion scripts
    Completions {
        shell: String,
        #[arg(long, help = "Install completions for your shell")]
        install: bool,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
    },
    /// Print version information
    Version,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Debug, Subcommand)]
pub(crate) enum PlanCommand {
    /// Create a new plan from an intent
    Create {
        #[arg(long, help = "Natural language intent")]
        intent: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show plan details
    Show {
        plan_id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Validate a plan against the project
    Validate {
        plan_id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Apply a plan to the project
    Apply {
        plan_id: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Rollback a plan's changes
    Rollback {
        plan_id: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// List all plans
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProjectCommand {
    /// Generate a plan to sync the project with the manifest
    Plan {
        #[arg(long, help = "Plan intent description")]
        intent: Option<String>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Apply a project plan
    Apply {
        plan_id: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show differences between current state and the manifest
    Diff {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Resolve discrepancies between the manifest and filesystem
    Reconcile {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Explain the current project configuration
    Explain {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum LockCommand {
    /// Show the lockfile contents
    Show {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Verify all asset hashes match the lockfile
    Verify {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Update the lockfile with current asset versions and hashes
    Update {
        #[arg(long, help = "Asset IDs to update (e.g., recipe://database/postgres)")]
        id: Option<Vec<String>>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show differences between current state and the lockfile
    Diff {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum FileCommand {
    /// List managed files in the manifest
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Check integrity of managed files
    Check {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Add a file to the managed manifest
    Add {
        #[arg(help = "Path to the file to manage")]
        path: camino::Utf8PathBuf,
        #[arg(
            long,
            help = "Subsystem that manages this file (scaffold, adopt, sync, agent, init, context, handoff, verify, depgraph)"
        )]
        managed_by: Option<String>,
        #[arg(long, help = "Description of why this file is managed")]
        desc: Option<String>,
    },
    /// Remove a file from the managed manifest
    Remove {
        #[arg(help = "Path to the file to unmanage")]
        path: camino::Utf8PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ReceiptCommand {
    /// List all receipts
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show receipt details
    Show {
        receipt_id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Resume from a receipt
    Resume { receipt_id: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ContextCommand {
    /// Build context pack for the project
    Build {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show the context pack
    Show {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Diff context against last build
    Diff,
    /// Verify context files exist
    Verify,
    /// Compile context with token budget enforcement
    Compile {
        #[arg(
            long,
            help = "Token budget override (default: from config preferences.agents.context_budget_tokens, 6000)"
        )]
        budget: Option<usize>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DepGraphCommand {
    /// List all assets in the dependency graph
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show dependency details for a specific asset
    Show {
        id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Check the graph for conflicts, cycles, and missing deps
    Check {
        #[arg(long, help = "Root asset IDs to start resolution from")]
        root: Vec<String>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Export the graph in DOT format for visualization
    Dot {
        #[arg(long, help = "Root asset IDs to include (empty = all)")]
        root: Vec<String>,
        #[arg(long, help = "Output file path (default: stdout)")]
        out: Option<camino::Utf8PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum HandoffCommand {
    /// Create a handoff
    Create {
        #[arg(long, help = "Task description")]
        task: String,
        #[arg(
            long,
            default_value = "pidgin",
            help = "Format (pidgin, markdown, json)"
        )]
        format: String,
        #[arg(long, help = "Next action")]
        next: String,
        #[arg(long, help = "Plan ID to reference")]
        plan_id: Option<String>,
    },
    /// Show handoff details
    Show {
        handoff_id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Verify a handoff
    Verify { handoff_id: String },
    /// Resume from a handoff
    Resume { handoff_id: String },
    /// List all handoffs
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AssetsCommand {
    /// Search assets by intent or keyword
    Search {
        query: String,
        #[arg(
            long,
            help = "Filter by asset kind (profile, template, recipe, command, snippet, license)"
        )]
        kind: Option<String>,
        #[arg(
            long,
            help = "Filter by lifecycle status (experimental, preview, stable, deprecated, retired)"
        )]
        status: Option<String>,
        #[arg(long, help = "Minimum quality score (0-100)")]
        min_quality: Option<u32>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show detailed asset info
    Show {
        id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// List all assets of a kind
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Export the asset catalog to a JSON file
    Catalog {
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
    },
    /// Build or show status of the local search index
    Index {
        #[arg(long, help = "Rebuild the search index")]
        rebuild: bool,
        #[arg(long, help = "Show index statistics")]
        stats: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Test assets against contract expectations
    Test {
        #[arg(help = "Asset ID to test (omit to test all)")]
        id: Option<String>,
        #[arg(long, help = "Test only changed assets")]
        changed: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// Project name (omit to init/assimilate the current directory)
    pub(crate) name: Option<String>,
    #[arg(short = 'p', long = "path", help = "Parent directory for the project")]
    pub(crate) path: Option<Utf8PathBuf>,
    #[arg(long, help = "Profile to use (e.g. core/bare)")]
    pub(crate) profile: Option<String>,
    #[arg(
        long = "with",
        value_delimiter = ',',
        help = "Components to include (testing, ci, docs, etc.)"
    )]
    pub(crate) components: Vec<String>,
    #[arg(long, help = "Show what would be created without writing")]
    pub(crate) dry_run: bool,
    #[arg(long, help = "Overwrite existing project files")]
    pub(crate) overwrite: bool,
    #[arg(long, help = "Skip git repository initialization")]
    pub(crate) no_git: bool,
    #[arg(
        long,
        help = "Assimilate existing project (detect git remote, files, etc.)"
    )]
    pub(crate) assimilate: bool,
    #[arg(long, help = "Programming language (rust, python, node, etc.)")]
    pub(crate) lang: Option<String>,
    #[arg(long, help = "Scaffold preset")]
    pub(crate) preset: Option<String>,
    #[arg(long, help = "License to apply")]
    pub(crate) license: Option<String>,
    #[arg(
        long = "extra",
        value_delimiter = '=',
        help = "Extra template variables (key=value)"
    )]
    pub(crate) extra: Vec<String>,
    #[arg(long, help = "Skip post-init convention check")]
    pub(crate) no_check: bool,
    #[arg(short = 'y', long, help = "Skip confirmation prompts")]
    pub(crate) yes: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CheckArgs {
    /// Path to check (defaults to current directory)
    pub(crate) path: Option<Utf8PathBuf>,
    #[arg(long, value_enum, default_value = "table", help = "Output format")]
    pub(crate) output: OutputFormat,
    #[arg(long, help = "Auto-fix violations")]
    pub(crate) fix: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    /// Show current configuration
    Show {
        #[arg(long, value_enum, default_value = "toml", help = "Output format")]
        format: OutputFormat,
        #[arg(long, help = "Show default values instead of current")]
        defaults: bool,
        #[arg(long, help = "Show project-level config only")]
        project: bool,
        #[arg(long, help = "Show a specific config section")]
        section: Option<String>,
    },
    /// Validate configuration against the schema
    Validate {
        #[arg(long, help = "Validate default values")]
        defaults: bool,
        #[arg(long, help = "Validate project-level config")]
        project: bool,
    },
    /// Show differences between current and default config
    Diff,
    /// Set a config key to a value
    Set {
        /// Config key (e.g. identity.author)
        key: String,
        /// Value to set
        value: String,
    },
    /// Reset a config key to its default value
    Reset {
        /// Config key to reset
        key: String,
    },
    /// Open a config key in the default editor
    Edit {
        /// Config key to edit
        key: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum LibraryCommand {
    /// List available templates
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show a template's content
    Show {
        /// Template name
        name: String,
        #[arg(long, help = "Show raw template source")]
        raw: bool,
    },
    /// Show differences from the embedded default
    Diff {
        /// Template name
        name: String,
    },
    /// Reset a template to its embedded default
    Reset {
        /// Template name
        name: String,
    },
    /// Validate template syntax
    Validate {
        #[arg(long, help = "Validate all templates")]
        all: bool,
    },
    /// Edit a template in the default editor
    Edit {
        /// Template name
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum TemplateBundleCommand {
    /// Apply a template bundle to the current directory
    Apply {
        #[arg(help = "Path to the template bundle directory")]
        path: PathBuf,
        #[arg(long, value_delimiter = '=', help = "Template variables (key=value)")]
        variables: Vec<String>,
        #[arg(
            long,
            default_value = "error",
            help = "Overwrite policy: skip, error, replace"
        )]
        overwrite: Option<String>,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Capture a directory as a template bundle
    Capture {
        #[arg(help = "Source directory to capture")]
        source: PathBuf,
        #[arg(help = "Destination directory for the bundle")]
        dest: PathBuf,
        #[arg(
            long,
            help = "Capture mode: minimal, source, development, complete (default: source)"
        )]
        mode: Option<String>,
        #[arg(long, help = "Show what would be captured without writing")]
        dry_run: bool,
        #[arg(long, help = "Skip secret redaction")]
        no_redact: bool,
        #[arg(long, help = "Template name (defaults to directory name)")]
        name: Option<String>,
        #[arg(
            long,
            default_value = "bundle",
            help = "Template kind: file, bundle, feature, project"
        )]
        kind: Option<String>,
    },
    /// Preview what would be captured from a directory
    Preview {
        #[arg(help = "Source directory to preview")]
        source: PathBuf,
        #[arg(
            long,
            help = "Capture mode: minimal, source, development, complete (default: source)"
        )]
        mode: Option<String>,
    },
    /// List available template bundles
    List {
        #[arg(
            long,
            help = "Directory to search for template bundles (default: global template dir)"
        )]
        path: Option<PathBuf>,
    },
    /// Show a template bundle manifest
    Show {
        #[arg(help = "Path to the template bundle")]
        path: PathBuf,
    },
    /// Validate a template bundle manifest
    Validate {
        #[arg(help = "Path to the template bundle")]
        path: PathBuf,
    },
    /// Verify all referenced files and assets exist
    Verify {
        #[arg(help = "Path to the template bundle")]
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProfileCommand {
    /// List all profiles
    List,
    /// Show profile details
    Show {
        name: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Activate a profile
    Use { name: String },
    /// Create a new profile
    New { name: String },
    /// Delete a profile
    Delete { name: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum RecipeCommand {
    /// List all recipes
    List,
    /// Show a recipe's content
    Show {
        name: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Apply a recipe to the project
    Apply {
        name: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Compose multiple recipes into one
    Compose { names: Vec<String> },
    /// Create a new recipe
    New { name: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum CommandsCommand {
    /// List custom command macros
    List,
    /// Show a command macro's definition
    Show {
        name: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Register a new command macro
    Add {
        slug: String,
        #[arg(long, help = "Install globally (not project-local)")]
        global: bool,
        #[arg(long, help = "Import from a file path")]
        from: Option<String>,
    },
    /// Remove a command macro
    Remove {
        slug: String,
        #[arg(long, help = "Remove from global install")]
        global: bool,
    },
    /// Export command macros to a file
    Export {
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
    },
    /// Run a command macro
    Run {
        slug: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Edit a command macro in the default editor
    Edit { name: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PluginCommand {
    /// List installed plugins
    List,
    /// Search the plugin registry
    Search {
        query: Option<String>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Install a plugin from a manifest path
    Add {
        source: Utf8PathBuf,
        #[arg(long, help = "Allow unsafe permissions")]
        allow_unsafe: bool,
    },
    /// Remove an installed plugin
    Remove { name: String },
    /// Update one or all plugins
    Update { name: Option<String> },
    /// Show plugin information
    Info { name: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AgentCommand {
    /// Sync agent context files
    Sync,
    /// Show agent sync status
    Status,
    /// Export agent files
    Export {
        #[arg(long, help = "Output directory path")]
        out: Option<Utf8PathBuf>,
    },
    /// Manage agent task plans
    Plan {
        #[command(subcommand)]
        command: AgentPlanCommand,
    },
    /// Bootstrap agent discovery info
    Bootstrap {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Resolve an intent to LODE capabilities
    Resolve {
        #[arg(long, help = "Natural language intent description")]
        intent: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Generate canonical agent policy files (AGENTS.md, CLAUDE.md, CODEX.md, .cursorrules, .mcp.json)
    Policy {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AgentPlanCommand {
    /// Initialize a new plan
    Init,
    /// Add a task to the plan
    Add {
        task: String,
        #[arg(long, help = "Git branch for this task")]
        branch: Option<String>,
    },
    /// Mark a task as done
    Done { id: u64 },
    /// Show the current plan
    Show,
    /// Clear all tasks
    Clear,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SnippetCommand {
    /// List code snippets
    List {
        #[arg(long, help = "Filter by language")]
        lang: Option<String>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show a snippet's content
    Show {
        name: String,
        #[arg(long, help = "Filter by language")]
        lang: Option<String>,
    },
    /// Search snippets by content
    Search { query: String },
    /// Add a new snippet
    Add {
        name: String,
        #[arg(long, default_value = "any", help = "Programming language")]
        lang: String,
        #[arg(long, help = "Tab trigger / shortcut")]
        trigger: Option<String>,
        #[arg(long, help = "Description")]
        desc: Option<String>,
    },
    /// Remove a snippet
    Remove {
        name: String,
        #[arg(long, help = "Filter by language")]
        lang: Option<String>,
    },
    /// Insert a snippet into a file
    Insert {
        name: String,
        file: Option<Utf8PathBuf>,
        #[arg(long, help = "Filter by language")]
        lang: Option<String>,
        #[arg(long, help = "Line number to insert at")]
        line: Option<usize>,
    },
    /// Export snippets to editor format
    Export {
        #[arg(long, help = "Filter by language")]
        lang: Option<String>,
        #[arg(long, default_value = "vscode", help = "Export format (vscode, zed)")]
        format: String,
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
    },
    /// Edit a snippet in the default editor
    Edit { name: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ScanCommand {
    /// Scan for secrets and credentials in files
    Secrets {
        path: Option<Utf8PathBuf>,
        #[arg(long, help = "Scan staged git changes")]
        staged: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
        #[arg(long, help = "Suppress verbose output")]
        quiet: bool,
    },
    /// Scan a directory for non-lode projects
    Foreign {
        path: Option<Utf8PathBuf>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum RulesCommand {
    /// List all convention rules
    List,
    /// Check files against convention rules
    Check { path: Option<Utf8PathBuf> },
    /// Validate rule definitions
    Validate,
}

#[derive(Debug, Subcommand)]
pub(crate) enum GitCommand {
    /// Create a standardized branch name
    Branch { kind: String, description: String },
    /// Create a conventional commit
    Commit {
        message: Option<String>,
        #[arg(long, help = "Commit type (feat, fix, docs, etc.)")]
        r#type: Option<String>,
        #[arg(long, help = "Commit scope")]
        scope: Option<String>,
        #[arg(long, help = "Mark as breaking change")]
        breaking: bool,
        #[arg(long, help = "Skip confirmation prompt")]
        no_confirm: bool,
    },
    /// Create and optionally push a version tag
    Tag {
        version: String,
        #[arg(long, help = "Skip changelog update")]
        no_changelog: bool,
        #[arg(long, help = "Push tag to remote")]
        push: bool,
        #[arg(long, help = "Tag annotation message")]
        message: Option<String>,
    },
    /// Generate or update the changelog
    Changelog {
        #[arg(long, help = "Generate since a specific tag")]
        since: Option<String>,
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
        #[arg(
            long,
            default_value = "markdown",
            help = "Output format (markdown, json)"
        )]
        format: String,
    },
    /// Install git hooks managed by lode
    InstallHooks,
    /// Uninstall lode-managed git hooks
    UninstallHooks,
    /// Show git hooks status
    HooksStatus,
    /// Configure git commit signing
    SignSetup,
    /// Configure a git remote (GitHub, GitLab, etc.)
    RemoteSetup {
        #[arg(long, help = "Git provider (github, gitlab, etc.)")]
        provider: Option<String>,
        #[arg(long, help = "Repository visibility (public, private)")]
        visibility: Option<String>,
        #[arg(long, help = "Environment variable with auth token")]
        token_env: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum HooksCommand {
    /// List available hooks
    List,
    /// Show hook execution status
    Status,
    /// Test a hook by event name
    Test { event: String },
    /// Run a hook for a specific event
    Run {
        event: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DocsCommand {
    /// Generate markdown documentation from the asset catalog
    Generate {
        #[arg(long, help = "Output directory (default: docs/)")]
        out: Option<Utf8PathBuf>,
        #[arg(long, help = "Regenerate all documentation")]
        force: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Check documentation completeness against the catalog
    Check {
        #[arg(long, help = "Documentation directory to check (default: docs/)")]
        dir: Option<Utf8PathBuf>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum EnvCommand {
    /// Check environment variables against the lockfile
    Check,
    /// Register a new environment variable
    Add {
        key: String,
        #[arg(long, help = "Default value")]
        default: Option<String>,
        #[arg(long, help = "Comment / description")]
        comment: Option<String>,
        #[arg(long, help = "Mark as secret (redacted in output)")]
        secret: bool,
    },
    /// Sync env files with the lockfile
    Sync,
    /// Switch to a different profile's environment
    Use { profile: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum LicenseCommand {
    /// List available licenses
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show a license's full text
    Show { id: String },
    /// Show license metadata
    Info { id: String },
    /// Add a custom license
    Add {
        id: String,
        #[arg(long, help = "License file path")]
        file: Option<Utf8PathBuf>,
        #[arg(long, help = "License text (inline)")]
        text: Option<String>,
    },
    /// Remove a custom license
    Remove { id: String },
    /// Set the project license
    Set { id: String },
    /// Check license compliance
    Check {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Apply license headers to project files
    Apply {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProjectsCommand {
    /// List registered projects
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
        #[arg(long, default_value = "name", help = "Sort field (name, path, added)")]
        sort: String,
    },
    /// Print the path of a registered project
    Cd { name: String },
    /// Register a directory as a lode project
    Register { path: Option<Utf8PathBuf> },
    /// Unregister a project
    Remove { name: String },
    /// Run health checks on registered projects
    Health {
        #[arg(long, help = "Show only stale projects")]
        stale_only: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
        #[arg(long, help = "Force refresh health data")]
        refresh: bool,
    },
    /// Remove missing projects from the registry
    Prune,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ToolchainCommand {
    /// List installed toolchains
    List,
    /// Show toolchain status
    Status,
    /// Run toolchain diagnostics
    Doctor,
    /// Install a toolchain version
    Add { runtime: String, version: String },
    /// Remove a toolchain version
    Remove { runtime: String, version: String },
    /// Activate a toolchain version
    Use { runtime: String, version: String },
    /// Pin a toolchain version for the project
    Pin {
        runtime: Option<String>,
        version: Option<String>,
        #[arg(long, help = "Pin all runtimes")]
        all: bool,
    },
    /// Update toolchains
    Update {
        runtime: Option<String>,
        #[arg(long, help = "Update all toolchains")]
        all: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PkgCommand {
    /// List project dependencies
    List {
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Check for outdated dependencies
    Outdated {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Update dependencies
    Update {
        name: Option<String>,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Run security audit on dependencies
    Audit {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
        #[arg(long, help = "Fail on severity level or higher")]
        fail_on: Option<String>,
    },
    /// Show why a dependency is included
    Why {
        name: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Show dependency information
    Info {
        name: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Generate or update a lockfile
    Lock {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Visualize the dependency graph
    Graph {
        #[arg(
            long,
            default_value = "ascii",
            help = "Output format (ascii, dot, json)"
        )]
        format: String,
    },
    /// Clean dependency caches
    Clean {
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum TimeCommand {
    /// Show today's time log
    Today {
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Show time log for a period
    Show {
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(long, default_value = "day", help = "Group by (day, week, month)")]
        by: String,
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Generate a time report
    Report {
        #[arg(long, help = "Start date (YYYY-MM-DD)")]
        since: Option<String>,
        #[arg(
            long,
            default_value = "markdown",
            help = "Output format (markdown, json)"
        )]
        format: String,
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
    },
    /// Clear time log entries
    Clear {
        #[arg(long, help = "Clear entries before this date (YYYY-MM-DD)")]
        before: Option<String>,
        #[arg(long, help = "Skip confirmation prompt")]
        confirm: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum MetricsCommand {
    /// Show current metrics snapshot
    Show,
    /// Show metrics trend over time
    Trend {
        #[arg(long, help = "Number of recent snapshots to show")]
        last: Option<usize>,
    },
    /// Set the current metrics as baseline
    Baseline,
    /// Show differences from the baseline
    DiffBaseline,
}

#[derive(Debug, Subcommand)]
pub(crate) enum WorkspaceCommand {
    /// Initialize a new workspace layout
    Init,
    /// List workspace members
    List {
        #[arg(long, default_value = "table", help = "Output format (table, json)")]
        format: String,
    },
    /// Add a new member crate
    Add { name: String },
    /// Remove a member crate
    Remove {
        name: String,
        #[arg(long, help = "Skip confirmation prompt")]
        confirm: bool,
    },
    /// Run a command across workspace members
    Run {
        target: String,
        #[arg(long, help = "Run in a specific package only")]
        pkg: Option<String>,
        #[arg(long, help = "Run only in changed packages")]
        changed: Vec<String>,
        #[arg(long, help = "Maximum parallel jobs")]
        parallel: Option<usize>,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Visualize the workspace dependency graph
    Graph {
        #[arg(
            long,
            default_value = "ascii",
            help = "Output format (ascii, dot, json)"
        )]
        format: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DaemonCommand {
    /// Start the file watcher daemon
    Start {
        #[arg(long, help = "Disable rename detection")]
        no_rename: bool,
        #[arg(long, help = "Disable signature insertion")]
        no_sign: bool,
        #[arg(long, help = "Disable license stamping")]
        no_stamp: bool,
        #[arg(long, help = "Run in foreground (no background process)")]
        foreground: bool,
        #[arg(long, help = "Disable env drift detection")]
        no_env_drift: bool,
        #[arg(long, help = "Disable license drift detection")]
        no_license_drift: bool,
    },
    /// Stop the file watcher daemon
    Stop {
        #[arg(long, help = "Project name to stop watching")]
        project: Option<String>,
    },
    /// Restart the daemon
    Restart,
    /// Pause file watching
    Pause,
    /// Resume file watching
    Resume,
    /// List active file watchers
    ListWatchers {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show daemon status
    Status {
        #[arg(long, help = "Suppress output (exit code only)")]
        quiet: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show daemon log
    Log {
        #[arg(long, help = "Show last N lines")]
        tail: Option<usize>,
        #[arg(long, help = "Follow new log entries")]
        follow: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum LogCommand {
    /// Initialize logging system
    Init,
    /// Show daemon log entries
    Daemon {
        #[arg(long, help = "Show last N lines")]
        tail: Option<usize>,
    },
    /// Clear all logs
    Clear,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SelfCommand {
    /// Show lode installation information
    Info,
    /// Clean lode caches and temporary files
    Clean {
        #[arg(long, help = "Show what would be cleaned without doing it")]
        dry_run: bool,
    },
    /// Uninstall lode from the system
    Uninstall {
        #[arg(long, help = "Keep configuration files")]
        keep_config: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ArchetypeCommand {
    /// List available archetypes
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Show archetype details
    Show {
        id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Apply an archetype to the project
    Apply {
        id: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PolicyCommand {
    /// Check project against active policies
    Check {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// List available policies
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Explain a specific policy
    Explain {
        id: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Waive a failing policy
    Waive {
        policy_id: String,
        #[arg(long, help = "Reason for the waiver")]
        reason: String,
        #[arg(long, help = "Expiry date or condition")]
        expires: Option<String>,
        #[arg(long, help = "Owner of the waiver")]
        owner: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PackCommand {
    /// List available packs
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Activate a pack
    Use {
        id: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Layer a pack on top of the current configuration
    Layer {
        id: String,
        #[arg(long, help = "Show what would be done without doing it")]
        dry_run: bool,
    },
    /// Export active pack configuration
    Export {
        #[arg(long, help = "Output file path")]
        out: Option<Utf8PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum CacheCommand {
    /// Show cache statistics
    Stats {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Clear the entire cache
    Clear,
}

#[derive(Debug, Subcommand)]
pub(crate) enum DiagnoseCommand {
    /// Diagnose build/test output against known patterns
    Run {
        #[arg(help = "File path containing build/test output, or - for stdin")]
        input: Option<String>,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// List known diagnosis patterns
    Patterns {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum SandboxCommand {
    /// Run a command in a sandboxed environment
    Run {
        #[arg(help = "Command to execute")]
        command: String,
        #[arg(long, help = "Arguments for the command")]
        args: Vec<String>,
        #[arg(long, default_value = "30", help = "Timeout in seconds")]
        timeout: u64,
        #[arg(long, help = "Inherit environment variables")]
        inherit_env: bool,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum EnvSnapshotCommand {
    /// Create a new environment snapshot
    Create {
        #[arg(help = "Label for the snapshot")]
        label: String,
    },
    /// List environment snapshots
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Compare two environment snapshots
    Compare {
        #[arg(help = "First snapshot ID")]
        id1: String,
        #[arg(help = "Second snapshot ID")]
        id2: String,
    },
    /// Restore an environment snapshot
    Restore {
        #[arg(help = "Snapshot ID to restore")]
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AgentSimCommand {
    /// Simulate agent intent resolution against the catalog
    Simulate {
        #[arg(help = "Natural language intent to simulate")]
        intent: String,
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum MigrationCommand {
    /// Plan a new migration
    Plan {
        #[arg(help = "Description of the migration")]
        description: String,
        #[arg(long, default_value = "schema", help = "Kind of migration")]
        kind: String,
    },
    /// Apply a pending migration
    Apply {
        #[arg(help = "Migration ID to apply")]
        id: String,
    },
    /// Roll back an applied migration
    Rollback {
        #[arg(help = "Migration ID to roll back")]
        id: String,
    },
    /// List all migrations and their state
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum SecretVaultCommand {
    /// Set a secret in the vault
    Set {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(help = "Secret value")]
        value: String,
        #[arg(long, default_value = "local", help = "Scope (local, project, global)")]
        scope: String,
    },
    /// Get a secret value from the vault
    Get {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(long, help = "Show the secret value (redacted by default)")]
        show: bool,
    },
    /// List all secrets in the vault
    List {
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        output: OutputFormat,
    },
    /// Remove a secret from the vault
    Remove {
        #[arg(help = "Secret key to remove")]
        key: String,
    },
    /// Grant access to a secret
    Grant {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(help = "Principal (e.g. tool, user)")]
        principal: String,
        #[arg(long, default_value = "read", help = "Permission (read, write)")]
        permission: String,
    },
    /// Revoke access to a secret
    Revoke {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(help = "Principal to revoke access for")]
        principal: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum OutputFormat {
    Table,
    Toml,
    Json,
}

impl OutputFormat {
    pub(crate) fn should_use_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ResultEnvelope<T: Serialize> {
    pub command: String,
    pub status: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ResultEnvelope<T> {
    pub(crate) fn success(command: impl Into<String>, data: T) -> Self {
        Self {
            command: command.into(),
            status: "success".to_string(),
            data,
            error: None,
        }
    }
}

pub(crate) fn print_output<T: Serialize>(
    command_name: &str,
    data: T,
    format: OutputFormat,
    render_table: impl Fn() -> String,
) {
    match format {
        OutputFormat::Table => print!("{}", render_table()),
        OutputFormat::Json => {
            let envelope = ResultEnvelope::success(command_name, &data);
            println!(
                "{}",
                serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| "{}".to_string())
            );
        }
        OutputFormat::Toml => {
            println!(
                "{}",
                toml::to_string_pretty(&data).unwrap_or_else(|_| String::new())
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExportOptions {
    pub(crate) no_plugins: bool,
    pub(crate) no_templates: bool,
    pub(crate) no_snippets: bool,
    pub(crate) no_licenses: bool,
    pub(crate) no_recipes: bool,
    pub(crate) no_commands: bool,
    pub(crate) include_metrics: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct TimeLog {
    #[serde(default)]
    pub(crate) sessions: Vec<TimeSession>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CompletionInstallReceipt {
    pub(crate) schema_version: u32,
    pub(crate) shell: String,
    pub(crate) path: String,
    pub(crate) installed_at: String,
    pub(crate) source: String,
    pub(crate) hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TimeSession {
    pub(crate) started_at: String,
    #[serde(default)]
    pub(crate) ended_at: Option<String>,
    #[serde(default)]
    pub(crate) seconds: u64,
    #[serde(default)]
    pub(crate) project: Option<String>,
    #[serde(default)]
    pub(crate) file: Option<String>,
    #[serde(default)]
    pub(crate) task: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SnippetAsset {
    pub(crate) lang: String,
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) path: Utf8PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct AgentPlan {
    #[serde(default)]
    pub(crate) next_id: u64,
    #[serde(default)]
    pub(crate) tasks: Vec<AgentTask>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AgentTask {
    pub(crate) id: u64,
    pub(crate) task: String,
    #[serde(default)]
    pub(crate) branch: Option<String>,
    pub(crate) done: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct ToolchainStore {
    #[serde(default)]
    pub(crate) runtimes: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(crate) active: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorReport {
    pub(crate) status: String,
    pub(crate) fixed: bool,
    pub(crate) checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorCheck {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) detail: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ForeignScanReport {
    pub(crate) path: Utf8PathBuf,
    pub(crate) lode_project: bool,
    pub(crate) package_manager: Option<String>,
    pub(crate) manifests: Vec<String>,
    pub(crate) convention_checked: usize,
    pub(crate) convention_violations: usize,
    pub(crate) secret_findings: usize,
    pub(crate) migration_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PackageOperationPlan {
    pub(crate) operation: String,
    pub(crate) manager: String,
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) packages: Vec<PackageDependency>,
}

impl PackageOperationPlan {
    pub(crate) fn new(operation: &str, manager: &str, args: Vec<String>) -> Self {
        Self {
            operation: operation.to_string(),
            manager: manager.to_string(),
            command: package_command(manager).to_string(),
            args,
            packages: package_dependencies(),
        }
    }

    pub(crate) fn command_line(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PackageManifest {
    pub(crate) file: String,
    pub(crate) kind: String,
    pub(crate) manager: String,
    pub(crate) dependencies: Vec<PackageDependency>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PackageDependency {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) scope: String,
    pub(crate) manifest: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectDaemonState {
    pub(crate) schema_version: u32,
    pub(crate) project: Option<String>,
    pub(crate) updated_at: String,
    pub(crate) file_count: usize,
    pub(crate) files: BTreeMap<String, ProjectDaemonFileState>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectDaemonFileState {
    pub(crate) modified_s: u64,
    pub(crate) content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UpgradeManifest {
    pub(crate) schema_version: u32,
    pub(crate) version: String,
    pub(crate) binary: String,
    pub(crate) checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UpgradeState {
    pub(crate) schema_version: u32,
    pub(crate) version: String,
    pub(crate) candidate: Utf8PathBuf,
    pub(crate) checksum: String,
    pub(crate) current_executable: String,
    pub(crate) current_checksum: String,
    pub(crate) staged_at: String,
    pub(crate) activated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DaemonRuntimeState {
    pub(crate) active: bool,
    #[serde(default)]
    pub(crate) paused: bool,
    pub(crate) foreground: bool,
    pub(crate) project: Option<String>,
    pub(crate) started_at: String,
    pub(crate) updated_at: String,
    pub(crate) uptime_s: u64,
    pub(crate) events: u64,
    pub(crate) watchers: Vec<String>,
    #[serde(default)]
    pub(crate) recent_events: Vec<DaemonEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DaemonEvent {
    pub(crate) id: u64,
    pub(crate) kind: String,
    pub(crate) message: String,
    #[serde(default)]
    pub(crate) files: Vec<String>,
    pub(crate) created_at: String,
}

#[derive(Debug, Default)]
pub(crate) struct DaemonChangeSet {
    pub(crate) created: Vec<String>,
    pub(crate) modified: Vec<String>,
    pub(crate) deleted: Vec<String>,
}

impl DaemonChangeSet {
    pub(crate) fn is_empty(&self) -> bool {
        self.created.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    pub(crate) fn paths(&self) -> Vec<String> {
        self.created
            .iter()
            .chain(self.modified.iter())
            .chain(self.deleted.iter())
            .cloned()
            .collect()
    }

    pub(crate) fn summary(&self) -> String {
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
