use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};
use crate::process::Process;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRunnerConfig {
    pub runner: String,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskResult {
    pub runner: String,
    pub target: String,
    pub dry_run: bool,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn detect_task_runner(project_dir: &Path) -> Option<TaskRunnerConfig> {
    if project_dir.join("Makefile").exists() || project_dir.join("makefile").exists() {
        return Some(TaskRunnerConfig {
            runner: "make".to_string(),
            targets: vec![
                "build".to_string(),
                "test".to_string(),
                "clean".to_string(),
                "lint".to_string(),
            ],
        });
    }

    if project_dir.join("justfile").exists() || project_dir.join("Justfile").exists() {
        return Some(TaskRunnerConfig {
            runner: "just".to_string(),
            targets: vec![
                "build".to_string(),
                "test".to_string(),
                "clean".to_string(),
                "lint".to_string(),
            ],
        });
    }

    if project_dir.join("package.json").exists() {
        let content = std::fs::read_to_string(project_dir.join("package.json")).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        let scripts = parsed.get("scripts")?;
        let targets: Vec<String> = scripts.as_object()?.keys().map(|k| k.to_string()).collect();
        if targets.is_empty() {
            return None;
        }
        return Some(TaskRunnerConfig {
            runner: "npm".to_string(),
            targets,
        });
    }

    None
}

pub fn run_task(project_dir: &Path, target: &str, dry_run: bool) -> Result<TaskResult> {
    let config = detect_task_runner(project_dir)
        .ok_or_else(|| LodeError::Message("no task runner detected in project".to_string()))?;

    let mut cmd = Process::new(&config.runner)?;
    cmd.current_dir(project_dir);

    match config.runner.as_str() {
        "make" => {
            if dry_run {
                cmd.args(["--dry-run", target]);
            } else {
                cmd.args([target]);
            }
        }
        "just" => {
            if dry_run {
                cmd.args(["--dry-run", target]);
            } else {
                cmd.args([target]);
            }
        }
        "npm" => {
            cmd.args(["run", target]);
            if dry_run {
                cmd.args(["--dry-run"]);
            }
        }
        other => {
            return Err(LodeError::Message(format!(
                "unsupported task runner: {other}"
            )));
        }
    }

    let output = cmd.output()?;
    let success = output.status.success();

    Ok(TaskResult {
        runner: config.runner,
        target: target.to_string(),
        dry_run,
        success,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_make_runner() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Makefile"), "").unwrap();
        let config = detect_task_runner(dir.path()).unwrap();
        assert_eq!(config.runner, "make");
        assert!(config.targets.contains(&"build".to_string()));
    }

    #[test]
    fn detects_just_runner() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("justfile"), "").unwrap();
        let config = detect_task_runner(dir.path()).unwrap();
        assert_eq!(config.runner, "just");
    }

    #[test]
    fn detects_npm_scripts() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"start":"node index.js","test":"jest"}}"#,
        )
        .unwrap();
        let config = detect_task_runner(dir.path()).unwrap();
        assert_eq!(config.runner, "npm");
        assert!(config.targets.contains(&"start".to_string()));
        assert!(config.targets.contains(&"test".to_string()));
    }

    #[test]
    fn returns_none_for_empty_project() {
        let dir = tempfile::tempdir().unwrap();
        assert!(detect_task_runner(dir.path()).is_none());
    }

    #[test]
    fn run_task_fails_without_runner() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_task(dir.path(), "build", false);
        assert!(result.is_err());
    }
}
