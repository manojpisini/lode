pub mod assets;
pub mod audit;
pub mod config;
pub mod convention;
pub mod error;
pub mod install;
pub mod registry;
pub mod scaffold;
pub mod secrets;
pub mod template;

pub use assets::{command_names, default_assets, profile_names, recipe_names, template_paths};
pub use audit::{audit_project, load_metrics, save_metrics, AuditReport};
pub use config::{
    default_config, GitConfig, IdentityConfig, LodeConfig, ScaffoldConfig, SCHEMA_VERSION,
};
pub use convention::{check_path, fix_path, normalize_name, ConventionReport, ConventionViolation};
pub use error::{ExitCode, LodeError, Result};
pub use install::{
    ensure_global_workspace, global_config_path, global_dir, load_global_config,
    save_global_config, setup_defaults, SetupReport,
};
pub use registry::{
    load_registry, prune_registry, register_project, registry_path, save_registry, ProjectRecord,
    Registry,
};
pub use scaffold::{
    add_component_to_project, init_project, AddRequest, InitRequest, ProjectConfig, ScaffoldReport,
};
pub use secrets::{scan_secrets, SecretFinding, SecretScanReport};
pub use template::{render_template, slug_to_class, slug_to_ident, RenderContext};

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
            let lock = ENV_LOCK.lock().unwrap();
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
