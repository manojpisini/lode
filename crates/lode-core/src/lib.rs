//! Core library for the LODE developer tool.
//!
//! This crate provides the foundational types and operations used by all LODE
//! components: filesystem safety (ValidatedRoot), process execution (Process),
//! project scaffolding, naming conventions, secret scanning, git integration,
//! configuration management, package manager overlays, templates, recipes,
//! snippets, plugins, hooks, release management, time tracking, and more.

// Note: #![deny(unsafe_code)] is intentionally omitted because
// fs_safety.rs contains unsafe ReplaceFileW for atomic Windows replacements.

pub mod adopt;
pub mod agent;
pub mod assets;
pub mod audit;
pub mod catalog;
pub mod commands;
pub mod config;
pub mod context;
pub mod convention;
pub mod dep_graph;
pub mod env;
pub mod error;
pub mod file_manifest;
pub mod fs_safety;
pub mod git;
pub mod handoff;
pub mod hooks;
pub mod install;
pub mod ipc;
pub mod license;
pub mod lockfile;
pub mod pkg;
pub mod plan;
pub mod prereq;
pub mod process;
pub mod receipt;
pub mod recipe;
pub mod redact;
pub mod registry;
pub mod release;
pub mod rules;
pub mod scaffold;
pub mod secrets;
pub mod signature;
pub mod snippet;
pub mod task;
pub mod template;
pub mod template_sync;
pub mod test_history;
pub mod time_tracker;
pub mod toolchain;
pub mod workspace;

pub use adopt::{analyze_project, format_adoption_report, AdoptionReport, FrameworkInfo, LanguageInfo};
pub use agent::*;
pub use catalog::{
    build_catalog, export_catalog, resolve_intent, AssetCatalog, AssetCatalogEntry, BootstrapInfo,
    IntentResolution, ProjectInfo, RecommendedAction,
};
pub use assets::{command_names, default_assets, profile_names, recipe_names, template_paths};
pub use audit::{audit_project, load_metrics, save_metrics, AuditReport};
pub use commands::{
    default_lodepack_checksum_algorithm, export_lodepack, import_lodepack, LodePack, LodePackFile,
    LodePackManifest,
};
pub use config::{default_config, LodeConfig, SCHEMA_VERSION};
pub use context::{
    compile_context, estimate_token_count, CompileEntry, CompileReport, ContextPack, Decision,
    QualityGate,
};
pub use convention::{check_path, fix_path, normalize_name, ConventionReport, ConventionViolation};
pub use dep_graph::{
    builtin_asset_deps, find_asset_by_provides, format_dep_graph_dot, format_dep_resolution_table,
    AssetDependency, AssetDeps, AssetResolution, DepConflict, DepEdge, DepGraphBuilder,
    DepGraphNode, DepResolution, DependencyGraph,
};
pub use env::*;
pub use error::{ExitCode, LodeError, Result};
pub use file_manifest::{
    add_managed_file, check_file_integrity, file_manifest_path, list_managed_files,
    load_file_manifest, remove_managed_file, save_file_manifest, format_file_manifest_table,
    FileCheckResult, FileEntry, FileManifest, ManagedBy,
};
pub use fs_safety::ValidatedRoot;
pub use git::*;
pub use handoff::{Handoff, HandoffDecision, HandoffFormat};
pub use hooks::{PluginInstallReceipt, PluginSecurity, *};
pub use plan::{ApplyReport, Operation, Plan, PlanMetadata, PlanStatus, PlanValidation};
pub use receipt::{CommandReceipt, NextAction, ReceiptResult, ReceiptStatus, ReceiptStep};
pub use install::{
    auto_register_assets, auto_register_global_assets, ensure_global_workspace, global_asset_dir,
    global_config_path, global_dir, load_global_config, save_global_config, setup_defaults,
    SetupReport,
};
pub use license::*;
pub use lockfile::{
    diff_locks, hash_file, load_lock, lockfile_path, new_lock, save_lock, update_lock,
    verify_lock, LockAssetEntry, LockDiff, LockVerifyReport, LodeLock,
};
pub use pkg::*;
pub use prereq::*;
pub use process::Process;
pub use recipe::*;
pub use redact::{redact, redact_findings};
pub use registry::{
    load_registry, prune_registry, register_project, registry_path, save_registry, ProjectRecord,
    Registry,
};
pub use release::*;
pub use rules::*;
pub use scaffold::{
    add_component_to_project, init_project, load_project_config, load_scaffold_lock,
    scaffold_lock_path, sync_project, AddRequest, InitRequest, ProjectConfig, ProjectDependency,
    ProjectSection, ScaffoldLock, ScaffoldLockEntry, ScaffoldReport,
};
pub use secrets::{scan_content, scan_secrets, SecretFinding, SecretScanReport};
pub use signature::*;
pub use snippet::*;
pub use task::*;
pub use template::{render_template, slug_to_class, slug_to_ident, RenderContext};
pub use template_sync::*;
pub use test_history::*;
pub use time_tracker::*;
pub use toolchain::*;
pub use workspace::*;

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
        _lock: MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        pub(crate) fn set(key: &'static str, value: &str) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self {
                key,
                previous,
                _lock: lock,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}
