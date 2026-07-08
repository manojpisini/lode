pub mod agent;
pub mod assets;
pub mod audit;
pub mod commands;
pub mod config;
pub mod convention;
pub mod env;
pub mod error;
pub mod fs_safety;
pub mod git;
pub mod hooks;
pub mod install;
pub mod license;
pub mod pkg;
pub mod prereq;
pub mod process;
pub mod recipe;
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

pub use agent::*;
pub use assets::{command_names, default_assets, profile_names, recipe_names, template_paths};
pub use audit::{audit_project, load_metrics, save_metrics, AuditReport};
pub use commands::{
    default_lodepack_checksum_algorithm, export_lodepack, import_lodepack, LodePack, LodePackFile,
    LodePackManifest,
};
pub use config::{default_config, LodeConfig, SCHEMA_VERSION};
pub use convention::{check_path, fix_path, normalize_name, ConventionReport, ConventionViolation};
pub use env::*;
pub use error::{ExitCode, LodeError, Result};
pub use fs_safety::ValidatedRoot;
pub use git::*;
pub use hooks::*;
pub use install::{
    ensure_global_workspace, global_asset_dir, global_config_path, global_dir, load_global_config,
    save_global_config, setup_defaults, SetupReport,
};
pub use license::*;
pub use pkg::*;
pub use prereq::*;
pub use process::Process;
pub use recipe::*;
pub use registry::{
    load_registry, prune_registry, register_project, registry_path, save_registry, ProjectRecord,
    Registry,
};
pub use release::*;
pub use rules::*;
pub use scaffold::{
    add_component_to_project, init_project, load_scaffold_lock, scaffold_lock_path, sync_project,
    AddRequest, InitRequest, ProjectConfig, ScaffoldLock, ScaffoldLockEntry, ScaffoldReport,
};
pub use secrets::{scan_secrets, SecretFinding, SecretScanReport};
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
