use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgConfig {
    pub auto_detect: bool,
    pub warn_outdated_days: u32,
    pub fail_on_high_vuln: bool,
    pub audit_on_update: bool,
}

impl Default for PkgConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            warn_outdated_days: 90,
            fail_on_high_vuln: true,
            audit_on_update: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageOperationPlan {
    pub operation: String,
    pub manager: String,
    pub command: String,
    pub args: Vec<String>,
}

impl PackageOperationPlan {
    pub fn new(operation: &str, manager: &str, args: Vec<String>) -> Self {
        let command = detect_package_command(manager, operation);
        Self {
            operation: operation.to_string(),
            manager: manager.to_string(),
            command,
            args,
        }
    }
}

pub fn detect_package_manager(project_dir: &Path) -> Option<String> {
    if project_dir.join("Cargo.lock").exists() {
        Some("cargo".to_string())
    } else if project_dir.join("bun.lockb").exists() || project_dir.join("bun.lock").exists() {
        Some("bun".to_string())
    } else if project_dir.join("pnpm-lock.yaml").exists() {
        Some("pnpm".to_string())
    } else if project_dir.join("yarn.lock").exists() {
        Some("yarn".to_string())
    } else if project_dir.join("package-lock.json").exists() {
        Some("npm".to_string())
    } else if project_dir.join("uv.lock").exists() {
        Some("uv".to_string())
    } else if project_dir.join("requirements.txt").exists() {
        Some("pip".to_string())
    } else if project_dir.join("go.sum").exists() {
        Some("go".to_string())
    } else if project_dir.join("Gemfile.lock").exists() {
        Some("bundler".to_string())
    } else {
        None
    }
}

fn detect_package_command(manager: &str, operation: &str) -> String {
    match (manager, operation) {
        ("cargo", "list") => "cargo metadata --format-version 1".to_string(),
        ("cargo", "outdated") => "cargo outdated".to_string(),
        ("cargo", "update") => "cargo update".to_string(),
        ("cargo", "audit") => "cargo audit".to_string(),
        ("cargo", "clean") => "cargo clean".to_string(),
        ("npm", "list") => "npm list".to_string(),
        ("npm", "outdated") => "npm outdated".to_string(),
        ("npm", "update") => "npm update".to_string(),
        ("npm", "audit") => "npm audit".to_string(),
        ("npm", "clean") => "rm -rf node_modules".to_string(),
        ("bun", "list") => "bun pm ls".to_string(),
        ("bun", "outdated") => "bun outdated".to_string(),
        ("bun", "update") => "bun update".to_string(),
        ("bun", "audit") => "bun audit".to_string(),
        ("pnpm", "list") => "pnpm list".to_string(),
        ("pnpm", "outdated") => "pnpm outdated".to_string(),
        ("pnpm", "update") => "pnpm update".to_string(),
        ("yarn", "list") => "yarn list".to_string(),
        ("yarn", "outdated") => "yarn outdated".to_string(),
        ("yarn", "update") => "yarn upgrade".to_string(),
        ("uv", "list") => "uv pip list".to_string(),
        ("uv", "outdated") => "uv pip list --outdated".to_string(),
        ("uv", "update") => "uv pip install -U".to_string(),
        ("go", "list") => "go list -m all".to_string(),
        ("go", "outdated") => "go list -m -u all".to_string(),
        ("go", "update") => "go get -u".to_string(),
        ("go", "clean") => "go clean -modcache".to_string(),
        _ => format!("{manager} {operation}"),
    }
}

pub fn package_outdated_args(manager: &str) -> Result<Vec<String>> {
    let plan = PackageOperationPlan::new("outdated", manager, Vec::new());
    Ok(plan.args)
}

pub fn package_audit_args(_manager: &str, fail_on: Option<&str>) -> Result<Vec<String>> {
    let mut args = Vec::new();
    if let Some(level) = fail_on {
        args.push(format!("--fail-on={level}"));
    }
    Ok(args)
}

pub fn package_update_args(_manager: &str, name: Option<&str>) -> Result<Vec<String>> {
    let mut args = Vec::new();
    if let Some(name) = name {
        args.push(name.to_string());
    }
    Ok(args)
}

pub fn detect_package_manager_in(path: &Path) -> Option<String> {
    detect_package_manager(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_cargo() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.lock"), "").unwrap();
        assert_eq!(detect_package_manager(dir.path()).unwrap(), "cargo");
    }

    #[test]
    fn detects_npm() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package-lock.json"), "").unwrap();
        assert_eq!(detect_package_manager(dir.path()).unwrap(), "npm");
    }
}
