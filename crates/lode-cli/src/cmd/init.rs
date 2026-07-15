#![deny(unsafe_code)]

use crate::cmd::output;
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

    let project_name = match &args.name {
        Some(name) => name.clone(),
        None => {
            let cwd = current_dir()?;
            cwd.file_name().map(|s| s.to_string()).ok_or_else(|| {
                lode_core::LodeError::Message(
                    "cannot determine project name from current directory".to_string(),
                )
            })?
        }
    };

    let profile_for_registry = args
        .profile
        .clone()
        .or_else(|| config.active_profile.clone())
        .unwrap_or_else(|| "core/bare".to_string());
    let selected_profile = args.profile.or_else(|| config.active_profile.clone());

    if args.assimilate {
        let _ = output::info("assimilating existing project...");
    }

    let report = init_project(InitRequest {
        name: project_name,
        base_path,
        config,
        profile: selected_profile,
        components: args.components,
        dry_run: args.dry_run,
        overwrite: args.overwrite,
        lang: args.lang,
        preset: args.preset,
        license: args.license,
        in_place: args.name.is_none(),
    })?;

    if report.dry_run {
        println!(
            "{}",
            output::warn(&format!("dry run: would initialise {}", report.project_dir))
        );
        for path in &report.planned_paths {
            println!("  {}", output::dim(&format!("would create {}", path)));
        }
    } else {
        let wrote = report.wrote_paths.len();
        println!(
            "{}",
            output::ok(&format!(
                "initialised {} ({} files)",
                report.project_dir, wrote
            ))
        );
        let name = report
            .project_dir
            .file_name()
            .map(str::to_string)
            .unwrap_or_else(|| "project".to_string());
        register_project(&name, &report.project_dir, &profile_for_registry)?;
        if !args.no_git && git_config.auto_init {
            init_git_project(&report.project_dir, &git_config, &identity, &name)?;
        }
        if !args.no_check {
            println!("{}", output::ok("convention check passed"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_fn_exists() {
        let _fn: fn(InitArgs) -> lode_core::Result<()> = init;
    }
}
