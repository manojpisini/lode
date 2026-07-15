pub mod agent;
pub mod assets;
pub mod build;
pub mod convention;
pub mod daemon;
pub mod env;
pub mod git;
pub mod identity;
pub mod license;
pub mod mcp;
pub mod metrics;
pub mod migrations;
pub mod pkg;
pub mod preferences;
pub mod prereq;
pub mod recipe;
pub mod scaffold;
pub mod serve;
pub mod signature;
pub mod snippets;
pub mod stack;
pub mod time;
pub mod toolchain;
pub mod workspace;

pub use agent::AgentConfig;
pub use assets::AssetsConfig;
pub use build::BuildConfig;
pub use convention::ConventionConfig;
pub use daemon::DaemonConfig;
pub use env::EnvConfig;
pub use git::GitConfig;
pub use identity::IdentityConfig;
pub use license::LicenseConfig;
pub use mcp::McpConfig;
pub use metrics::MetricsConfig;
pub use pkg::PkgConfig;
pub use preferences::{AgentPrefs, ArchitecturePrefs, GitPrefs, PreferencesConfig, TestingPrefs};
pub use prereq::PrereqConfig;
pub use recipe::RecipeConfig;
pub use scaffold::ScaffoldConfig;
pub use serve::ServeConfig;
pub use signature::SignatureConfig;
pub use snippets::SnippetsConfig;
pub use stack::StackConfig;
pub use time::TimeConfig;
pub use toolchain::ToolchainConfig;
pub use workspace::WorkspaceConfig;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LodeConfig {
    pub schema_version: u32,
    pub active_profile: Option<String>,
    pub identity: IdentityConfig,
    pub assets: AssetsConfig,
    pub convention: ConventionConfig,
    pub signature: SignatureConfig,
    pub scaffold: ScaffoldConfig,
    pub git: GitConfig,
    pub env: EnvConfig,
    pub build: BuildConfig,
    pub daemon: DaemonConfig,
    pub stack: StackConfig,
    pub mcp: McpConfig,
    pub agent: AgentConfig,
    pub metrics: MetricsConfig,
    pub serve: ServeConfig,
    pub toolchain: ToolchainConfig,
    pub pkg: PkgConfig,
    pub license: LicenseConfig,
    pub snippets: SnippetsConfig,
    pub workspace: WorkspaceConfig,
    pub recipe: RecipeConfig,
    pub time: TimeConfig,
    pub prereq: PrereqConfig,
    pub preferences: PreferencesConfig,
}

impl Default for LodeConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            active_profile: None,
            identity: IdentityConfig::default(),
            assets: AssetsConfig::default(),
            convention: ConventionConfig::default(),
            signature: SignatureConfig::default(),
            scaffold: ScaffoldConfig::default(),
            git: GitConfig::default(),
            env: EnvConfig::default(),
            build: BuildConfig::default(),
            daemon: DaemonConfig::default(),
            stack: StackConfig::default(),
            mcp: McpConfig::default(),
            agent: AgentConfig::default(),
            metrics: MetricsConfig::default(),
            serve: ServeConfig::default(),
            toolchain: ToolchainConfig::default(),
            pkg: PkgConfig::default(),
            license: LicenseConfig::default(),
            snippets: SnippetsConfig::default(),
            workspace: WorkspaceConfig::default(),
            recipe: RecipeConfig::default(),
            time: TimeConfig::default(),
            prereq: PrereqConfig::default(),
            preferences: PreferencesConfig::default(),
        }
    }
}

pub const SCHEMA_VERSION: u32 = 3;
pub fn validate_schema(config: &LodeConfig) -> crate::Result<()> {
    if config.schema_version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(crate::LodeError::SchemaMismatch {
            expected: SCHEMA_VERSION,
            found: config.schema_version,
        })
    }
}

pub fn default_config() -> LodeConfig {
    LodeConfig::default()
}

pub fn merge_chain(
    defaults: LodeConfig,
    global: Option<LodeConfig>,
    project: Option<LodeConfig>,
) -> LodeConfig {
    let mut merged = defaults;

    if let Some(global_cfg) = global {
        merged = merge_configs(merged, global_cfg);
    }

    if let Some(project_cfg) = project {
        merged = merge_configs(merged, project_cfg);
    }

    merged
}

fn merge_configs(base: LodeConfig, override_cfg: LodeConfig) -> LodeConfig {
    LodeConfig {
        schema_version: base.schema_version,
        active_profile: override_cfg.active_profile.or(base.active_profile),
        identity: merge_identity(base.identity, override_cfg.identity),
        assets: assets::merge_assets(&base.assets, &override_cfg.assets),
        convention: merge_convention(base.convention, override_cfg.convention),
        signature: merge_signature(base.signature, override_cfg.signature),
        scaffold: merge_scaffold(base.scaffold, override_cfg.scaffold),
        git: merge_git(base.git, override_cfg.git),
        env: merge_env(base.env, override_cfg.env),
        build: merge_build(base.build, override_cfg.build),
        daemon: merge_daemon(base.daemon, override_cfg.daemon),
        stack: merge_stack(base.stack, override_cfg.stack),
        mcp: merge_mcp(base.mcp, override_cfg.mcp),
        agent: merge_agent(base.agent, override_cfg.agent),
        metrics: merge_metrics(base.metrics, override_cfg.metrics),
        serve: merge_serve(base.serve, override_cfg.serve),
        toolchain: merge_toolchain(base.toolchain, override_cfg.toolchain),
        pkg: merge_pkg(base.pkg, override_cfg.pkg),
        license: merge_license(base.license, override_cfg.license),
        snippets: merge_snippets(base.snippets, override_cfg.snippets),
        workspace: merge_workspace(base.workspace, override_cfg.workspace),
        recipe: merge_recipe(base.recipe, override_cfg.recipe),
        time: merge_time(base.time, override_cfg.time),
        prereq: merge_prereq(base.prereq, override_cfg.prereq),
        preferences: merge_preferences(base.preferences, override_cfg.preferences),
    }
}

fn merge_preferences(base: PreferencesConfig, o: PreferencesConfig) -> PreferencesConfig {
    PreferencesConfig {
        architecture: ArchitecturePrefs {
            default_style: non_empty(
                o.architecture.default_style,
                base.architecture.default_style,
            ),
            service_style: non_empty(
                o.architecture.service_style,
                base.architecture.service_style,
            ),
            prefer_explicit_boundaries: o.architecture.prefer_explicit_boundaries,
            avoid_premature_microservices: o.architecture.avoid_premature_microservices,
        },
        testing: TestingPrefs {
            require_unit_tests: o.testing.require_unit_tests,
            require_integration_tests_for_io: o.testing.require_integration_tests_for_io,
            minimum_coverage: o.testing.minimum_coverage,
            prefer_property_tests: o.testing.prefer_property_tests,
            framework: non_empty(o.testing.framework, base.testing.framework),
        },
        agents: AgentPrefs {
            reuse_lode_assets_first: o.agents.reuse_lode_assets_first,
            require_plan_before_write: o.agents.require_plan_before_write,
            require_verification_before_completion: o.agents.require_verification_before_completion,
            handoff_format: non_empty(o.agents.handoff_format, base.agents.handoff_format),
            context_budget_tokens: o.agents.context_budget_tokens,
        },
        git: GitPrefs {
            commit_style: non_empty(o.git.commit_style, base.git.commit_style),
            prefer_atomic_commits: o.git.prefer_atomic_commits,
            require_clean_verification: o.git.require_clean_verification,
        },
    }
}

fn merge_identity(base: IdentityConfig, o: IdentityConfig) -> IdentityConfig {
    IdentityConfig {
        author: non_empty(o.author, base.author),
        name: non_empty(o.name, base.name),
        email: non_empty(o.email, base.email),
        org: non_empty(o.org, base.org),
        url: non_empty(o.url, base.url),
        license: non_empty(o.license, base.license),
    }
}

fn merge_convention(base: ConventionConfig, o: ConventionConfig) -> ConventionConfig {
    ConventionConfig {
        folder_case: non_empty(o.folder_case, base.folder_case),
        file_case: non_empty(o.file_case, base.file_case),
        default_case: non_empty(o.default_case, base.default_case),
        enforce: o.enforce || base.enforce,
        exclude: if o.exclude.is_empty() {
            base.exclude
        } else {
            o.exclude
        },
        protected_prefixes: if o.protected_prefixes.is_empty() {
            base.protected_prefixes
        } else {
            o.protected_prefixes
        },
        prefix_map: if o.prefix_map.is_empty() {
            base.prefix_map
        } else {
            o.prefix_map
        },
    }
}

fn merge_signature(base: SignatureConfig, o: SignatureConfig) -> SignatureConfig {
    SignatureConfig {
        enabled: o.enabled,
        auto_insert: o.auto_insert,
        auto_update_date: o.auto_update_date,
        include_path: o.include_path,
        include_hash: o.include_hash,
        include_license: o.include_license,
        separator_char: o.separator_char,
        section_markers: o.section_markers,
        comment_styles: if o.comment_styles.is_empty() {
            base.comment_styles
        } else {
            o.comment_styles
        },
    }
}

fn merge_scaffold(base: ScaffoldConfig, o: ScaffoldConfig) -> ScaffoldConfig {
    ScaffoldConfig {
        always_dirs: if o.always_dirs.is_empty() {
            base.always_dirs
        } else {
            o.always_dirs
        },
        always_files: if o.always_files.is_empty() {
            base.always_files
        } else {
            o.always_files
        },
        optional: if o.optional.is_empty() {
            base.optional
        } else {
            o.optional
        },
    }
}

fn merge_git(base: GitConfig, o: GitConfig) -> GitConfig {
    GitConfig {
        auto_init: o.auto_init,
        initial_branch: non_empty(o.initial_branch, base.initial_branch),
        initial_commit: o.initial_commit,
        initial_commit_msg: non_empty(o.initial_commit_msg, base.initial_commit_msg),
        branch_strategy: non_empty(o.branch_strategy, base.branch_strategy),
        commit_convention: non_empty(o.commit_convention, base.commit_convention),
        commit_signing: o.commit_signing,
        sign_key: o.sign_key.or(base.sign_key),
        hooks: GitHooksConfig {
            pre_commit: o.hooks.pre_commit,
            pre_push: o.hooks.pre_push,
            commit_msg: o.hooks.commit_msg,
        },
    }
}

fn merge_env(base: EnvConfig, o: EnvConfig) -> EnvConfig {
    EnvConfig {
        auto_create: o.auto_create,
        runtime_lock: o.runtime_lock,
        vars: if o.vars.is_empty() { base.vars } else { o.vars },
        validation: EnvValidation {
            required: if o.validation.required.is_empty() {
                base.validation.required
            } else {
                o.validation.required
            },
            warn_missing: if o.validation.warn_missing.is_empty() {
                base.validation.warn_missing
            } else {
                o.validation.warn_missing
            },
        },
    }
}

fn merge_build(base: BuildConfig, o: BuildConfig) -> BuildConfig {
    BuildConfig {
        generate_makefile: o.generate_makefile,
        task_runner: non_empty(o.task_runner, base.task_runner),
        targets: if o.targets.is_empty() {
            base.targets
        } else {
            o.targets
        },
    }
}

fn merge_daemon(_base: DaemonConfig, o: DaemonConfig) -> DaemonConfig {
    DaemonConfig {
        enabled: o.enabled,
        idle_timeout_s: o.idle_timeout_s,
        debounce_ms: o.debounce_ms,
        watch_rename: o.watch_rename,
        watch_headers: o.watch_headers,
        watch_path_sync: o.watch_path_sync,
        watch_env_drift: o.watch_env_drift,
        watch_license: o.watch_license,
    }
}

fn merge_stack(base: StackConfig, o: StackConfig) -> StackConfig {
    StackConfig {
        languages: if o.languages.is_empty() {
            base.languages
        } else {
            o.languages
        },
        framework: o.framework.or(base.framework),
        indent: non_empty(o.indent, base.indent),
        line_width: o.line_width,
        comment_style: non_empty(o.comment_style, base.comment_style),
        package_manager: o.package_manager.or(base.package_manager),
    }
}

fn merge_mcp(base: McpConfig, o: McpConfig) -> McpConfig {
    McpConfig {
        enabled: o.enabled,
        default_transport: non_empty(o.default_transport, base.default_transport),
        http_port: o.http_port,
        http_host: non_empty(o.http_host, base.http_host),
        auth_token_env: o.auth_token_env.or(base.auth_token_env),
    }
}

fn merge_agent(base: AgentConfig, o: AgentConfig) -> AgentConfig {
    AgentConfig {
        auto_sync: o.auto_sync,
        generate_claude: o.generate_claude,
        generate_agents: o.generate_agents,
        generate_cursor: o.generate_cursor,
        generate_windsurf: o.generate_windsurf,
        generate_mcp_json: o.generate_mcp_json,
        context_dir: o.context_dir.or(base.context_dir),
    }
}

fn merge_metrics(_base: MetricsConfig, o: MetricsConfig) -> MetricsConfig {
    MetricsConfig {
        enabled: o.enabled,
        auto_snapshot: o.auto_snapshot,
        snapshot_history: o.snapshot_history,
    }
}

fn merge_serve(base: ServeConfig, o: ServeConfig) -> ServeConfig {
    ServeConfig {
        refresh_ms: o.refresh_ms,
        default_pane: non_empty(o.default_pane, base.default_pane),
        theme: non_empty(o.theme, base.theme),
        show_registry: o.show_registry,
        border_style: non_empty(o.border_style, base.border_style),
    }
}

fn merge_toolchain(base: ToolchainConfig, o: ToolchainConfig) -> ToolchainConfig {
    ToolchainConfig {
        rust_version: o.rust_version.or(base.rust_version),
        clippy_lints: o.clippy_lints.or(base.clippy_lints),
        rustfmt_edition: o.rustfmt_edition.or(base.rustfmt_edition),
        target: o.target.or(base.target),
    }
}

fn merge_pkg(base: PkgConfig, o: PkgConfig) -> PkgConfig {
    PkgConfig {
        name: non_empty(o.name, base.name),
        version: non_empty(o.version, base.version),
        edition: non_empty(o.edition, base.edition),
        description: o.description.or(base.description),
        repository: o.repository.or(base.repository),
        publish: o.publish,
    }
}

fn merge_license(base: LicenseConfig, o: LicenseConfig) -> LicenseConfig {
    LicenseConfig {
        kind: non_empty(o.kind, base.kind),
        copyright_holder: o.copyright_holder.or(base.copyright_holder),
        year: o.year.or(base.year),
        auto_insert: o.auto_insert,
        file_header: o.file_header,
    }
}

fn merge_snippets(base: SnippetsConfig, o: SnippetsConfig) -> SnippetsConfig {
    SnippetsConfig {
        enabled: o.enabled,
        dir: o.dir.or(base.dir),
        snippets: if o.snippets.is_empty() {
            base.snippets
        } else {
            o.snippets
        },
    }
}

fn merge_workspace(base: WorkspaceConfig, o: WorkspaceConfig) -> WorkspaceConfig {
    WorkspaceConfig {
        members: if o.members.is_empty() {
            base.members
        } else {
            o.members
        },
        shared_deps: o.shared_deps,
        shared_toolchain: o.shared_toolchain,
    }
}

fn merge_recipe(base: RecipeConfig, o: RecipeConfig) -> RecipeConfig {
    RecipeConfig {
        recipes: if o.recipes.is_empty() {
            base.recipes
        } else {
            o.recipes
        },
    }
}

fn merge_time(base: TimeConfig, o: TimeConfig) -> TimeConfig {
    TimeConfig {
        timezone: o.timezone.or(base.timezone),
        date_format: non_empty(o.date_format, base.date_format),
        time_format: non_empty(o.time_format, base.time_format),
        timestamp_format: non_empty(o.timestamp_format, base.timestamp_format),
    }
}

fn merge_prereq(base: PrereqConfig, o: PrereqConfig) -> PrereqConfig {
    PrereqConfig {
        checks: if o.checks.is_empty() {
            base.checks
        } else {
            o.checks
        },
        auto_install: o.auto_install,
    }
}

fn non_empty(override_val: String, base_val: String) -> String {
    if override_val.is_empty() {
        base_val
    } else {
        override_val
    }
}

use env::EnvValidation;
use git::GitHooksConfig;
