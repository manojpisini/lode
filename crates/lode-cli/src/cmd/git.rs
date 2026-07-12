#![deny(unsafe_code)]

use crate::GitCommand;

pub(crate) fn git(command: GitCommand) -> lode_core::Result<()> {
    match command {
        GitCommand::Branch { kind, description } => {
            let branch = format!("{}/{}", kind, crate::slugify(&description));
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
                crate::conventional_message(
                    r#type.as_deref().unwrap_or("chore"),
                    scope.as_deref(),
                    "update",
                    breaking,
                )
            });
            crate::run_git(&["commit", "-m", &message])?;
        }
        GitCommand::Tag {
            version,
            no_changelog: _,
            push,
            message,
        } => {
            let tag = format!("v{}", version.trim_start_matches('v'));
            if let Some(message) = message {
                crate::run_git(&["tag", "-a", &tag, "-m", &message])?;
            } else {
                crate::run_git(&["tag", &tag])?;
            }
            if push {
                crate::run_git(&["push", "origin", &tag])?;
            }
        }
        GitCommand::Changelog { since, out, format } => {
            crate::git_changelog(since.as_deref(), out, &format)?
        }
        GitCommand::InstallHooks => crate::install_git_hooks()?,
        GitCommand::UninstallHooks => crate::uninstall_git_hooks()?,
        GitCommand::HooksStatus => crate::cmd::hooks::hooks_status()?,
        GitCommand::SignSetup => crate::git_sign_setup()?,
        GitCommand::RemoteSetup {
            provider,
            visibility,
            token_env,
        } => crate::git_remote_setup(provider, visibility, token_env)?,
    }
    Ok(())
}
