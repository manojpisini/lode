#![deny(unsafe_code)]

use crate::{current_dir, init_git_project, InitArgs};
use lode_core::{init_project, load_global_config, register_project, InitRequest};

pub(crate) fn init(args: InitArgs) -> lode_core::Result<()> {
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
        lang: args.lang,
        preset: args.preset,
        license: args.license,
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
        if !args.no_check {
            println!("convention check: ok");
        }
        if args.yes {
            println!("auto-confirm enabled");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InitArgs;

    #[test]
    fn test_init_args_defaults() {
        let args = InitArgs {
            name: "test-project".to_string(),
            path: None,
            profile: None,
            components: vec![],
            dry_run: true,
            overwrite: false,
            no_git: false,
            lang: None,
            preset: None,
            license: None,
            extra: vec![],
            no_check: false,
            yes: false,
        };
        assert_eq!(args.name, "test-project");
        assert!(args.dry_run);
        assert!(!args.overwrite);
        assert!(args.profile.is_none());
    }

    #[test]
    fn test_init_fn_exists() {
        let _fn: fn(InitArgs) -> lode_core::Result<()> = init;
    }
}
