use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};
use crate::Process;
use crate::ValidatedRoot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_init: bool,
    pub initial_branch: String,
    pub initial_commit: bool,
    pub initial_commit_msg: String,
    pub branch_strategy: String,
    pub commit_convention: String,
    pub commit_signing: bool,
    pub sign_key: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_init: true,
            initial_branch: "main".to_string(),
            initial_commit: true,
            initial_commit_msg: "chore: scaffold [{org}/{project}]".to_string(),
            branch_strategy: "simple".to_string(),
            commit_convention: "conventional".to_string(),
            commit_signing: false,
            sign_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHooksConfig {
    pub pre_commit: Vec<String>,
    pub commit_msg: bool,
    pub pre_push: Vec<String>,
    pub pre_commit_secrets: bool,
}

impl Default for GitHooksConfig {
    fn default() -> Self {
        Self {
            pre_commit: vec!["fmt".to_string(), "lint".to_string()],
            commit_msg: true,
            pre_push: vec!["test".to_string()],
            pre_commit_secrets: true,
        }
    }
}

pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

fn git_status<I, S>(path: &Path, args: I) -> Result<std::process::ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Process::new("git")?.args(args).current_dir(path).status()
}

fn git_output<I, S>(path: &Path, args: I) -> Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Process::new("git")?.args(args).current_dir(path).output()
}
pub fn git_init(path: &Path, branch: &str) -> Result<()> {
    let status = git_status(path, ["init", "-b", branch])?;

    if !status.success() {
        return Err(LodeError::Message(format!("git init failed: {status}")));
    }
    Ok(())
}

pub fn git_add_all(path: &Path) -> Result<()> {
    let status = git_status(path, ["add", "."])?;

    if !status.success() {
        return Err(LodeError::Message(format!("git add failed: {status}")));
    }
    Ok(())
}

pub fn git_commit(path: &Path, message: &str) -> Result<()> {
    let status = git_status(path, ["commit", "-m", message])?;

    if !status.success() {
        return Err(LodeError::Message(format!("git commit failed: {status}")));
    }
    Ok(())
}

pub fn git_config_user(path: &Path, name: &str, email: &str) -> Result<()> {
    git_config_set(path, "user.name", name)?;
    git_config_set(path, "user.email", email)?;
    Ok(())
}

fn git_config_set(path: &Path, key: &str, value: &str) -> Result<()> {
    let status = git_status(path, ["config", key, value])?;

    if !status.success() {
        return Err(LodeError::Message(format!(
            "git config {key} failed: {status}"
        )));
    }
    Ok(())
}

pub fn git_tag(path: &Path, tag: &str, message: Option<&str>) -> Result<()> {
    let mut args = vec!["tag".to_string()];
    if let Some(msg) = message {
        args.extend([
            "-a".to_string(),
            tag.to_string(),
            "-m".to_string(),
            msg.to_string(),
        ]);
    } else {
        args.push(tag.to_string());
    }
    let status = git_status(path, &args)?;

    if !status.success() {
        return Err(LodeError::Message(format!("git tag failed: {status}")));
    }
    Ok(())
}

pub fn git_log(path: &Path, since: Option<&str>, count: usize) -> Result<Vec<String>> {
    let mut args = vec![
        "log".to_string(),
        "--pretty=format:%s".to_string(),
        "--no-merges".to_string(),
        format!("-{count}"),
    ];
    if let Some(since) = since {
        args.push(format!("{since}..HEAD"));
    }
    let output = git_output(path, &args)?;

    if !output.status.success() {
        return Err(LodeError::Message("git log failed".to_string()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(str::to_string).collect())
}

pub fn conventional_commit_message(
    kind: &str,
    scope: Option<&str>,
    subject: &str,
    breaking: bool,
) -> String {
    let bang = if breaking { "!" } else { "" };
    match scope {
        Some(scope) => format!("{kind}({scope}){bang}: {subject}"),
        None => format!("{kind}{bang}: {subject}"),
    }
}

pub fn branch_name(kind: &str, description: &str) -> String {
    let slug = description
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if !kind.ends_with('-') {
                '-'
            } else {
                return '\0';
            }
        })
        .filter(|ch| *ch != '\0')
        .collect::<String>();
    let slug = slug.trim_matches('-');
    format!("{kind}/{slug}")
}

pub fn install_git_hooks(path: &Path) -> Result<()> {
    let hooks_dir = path.join(".git").join("hooks");
    if !hooks_dir.exists() {
        return Err(LodeError::Message("not a git repository".to_string()));
    }
    let root = ValidatedRoot::new(path)?;
    root.write_atomic(
        Path::new(".git").join("hooks").join("pre-commit"),
        "#!/usr/bin/env sh\n# lode-managed\nlode check .\nlode scan secrets .\n",
    )?;
    root.write_atomic(
        Path::new(".git").join("hooks").join("pre-push"),
        "#!/usr/bin/env sh\n# lode-managed\nlode task test\n",
    )?;
    Ok(())
}

pub fn uninstall_git_hooks(path: &Path) -> Result<()> {
    let hooks_dir = path.join(".git").join("hooks");
    let root = ValidatedRoot::new(path)?;
    for name in ["pre-commit", "pre-push"] {
        let hook_path = hooks_dir.join(name);
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path).unwrap_or_default();
            if content.contains("lode-managed") {
                root.remove_file(Path::new(".git").join("hooks").join(name))?;
            }
        }
    }
    Ok(())
}

pub fn git_changelog(path: &Path, since: Option<&str>) -> Result<String> {
    let commits = git_log(path, since, 100)?;
    let mut changelog = String::from("# Changelog\n\n");
    for commit in &commits {
        changelog.push_str(&format!("- {commit}\n"));
    }
    Ok(changelog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conventional_message_formats() {
        assert_eq!(
            conventional_commit_message("feat", Some("auth"), "add login", false),
            "feat(auth): add login"
        );
        assert_eq!(
            conventional_commit_message("fix", None, "typo", true),
            "fix!: typo"
        );
    }

    #[test]
    fn branch_name_formats() {
        assert_eq!(branch_name("feat", "add login"), "feat/add-login");
    }

    #[test]
    fn git_hooks_install_and_uninstall_managed_files() {
        let dir = tempfile::tempdir().unwrap();
        let hooks = dir.path().join(".git").join("hooks");
        std::fs::create_dir_all(&hooks).unwrap();

        install_git_hooks(dir.path()).unwrap();
        assert!(std::fs::read_to_string(hooks.join("pre-commit"))
            .unwrap()
            .contains("lode-managed"));
        assert!(std::fs::read_to_string(hooks.join("pre-push"))
            .unwrap()
            .contains("lode-managed"));

        uninstall_git_hooks(dir.path()).unwrap();
        assert!(!hooks.join("pre-commit").exists());
        assert!(!hooks.join("pre-push").exists());
    }
}
