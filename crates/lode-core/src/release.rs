use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};
use crate::ValidatedRoot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub tag_prefix: String,
    pub push_after_tag: bool,
    pub update_changelog: bool,
    pub conventional_bump: bool,
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            tag_prefix: "v".to_string(),
            push_after_tag: false,
            update_changelog: true,
            conventional_bump: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReleaseReport {
    pub old_version: String,
    pub new_version: String,
    pub tag: String,
    pub files_updated: Vec<PathBuf>,
    pub dry_run: bool,
}

fn read_version_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let version = content.trim().to_string();
    if version.is_empty() {
        return None;
    }
    Some(version)
}

pub fn detect_version(project_dir: &Path) -> Option<String> {
    if let Some(v) = read_version_file(&project_dir.join("Cargo.toml")) {
        let parsed: toml::Value = toml::from_str(&v).ok()?;
        let pkg = parsed.get("package")?;
        let ver = pkg.get("version")?.as_str()?;
        return Some(ver.to_string());
    }

    if let Some(v) = read_version_file(&project_dir.join("package.json")) {
        let parsed: serde_json::Value = serde_json::from_str(&v).ok()?;
        let ver = parsed.get("version")?.as_str()?;
        return Some(ver.to_string());
    }

    if let Some(v) = read_version_file(&project_dir.join("pyproject.toml")) {
        let parsed: toml::Value = toml::from_str(&v).ok()?;
        let project = parsed.get("project")?;
        let ver = project.get("version")?.as_str()?;
        return Some(ver.to_string());
    }

    if let Some(v) = read_version_file(&project_dir.join("version.txt")) {
        return Some(v);
    }

    if let Some(v) = read_version_file(&project_dir.join("VERSION")) {
        return Some(v);
    }

    None
}

fn parse_semver(version: &str) -> Result<(u32, u32, u32)> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(LodeError::Message(format!(
            "invalid semver format: {version}"
        )));
    }
    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| LodeError::Message(format!("invalid major version: {}", parts[0])))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| LodeError::Message(format!("invalid minor version: {}", parts[1])))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| LodeError::Message(format!("invalid patch version: {}", parts[2])))?;
    Ok((major, minor, patch))
}

pub fn bump_version(version: &str, bump_type: &BumpType) -> Result<String> {
    let (major, minor, patch) = parse_semver(version)?;
    let new_version = match bump_type {
        BumpType::Major => format!("{}.0.0", major + 1),
        BumpType::Minor => format!("{}.{}.0", major, minor + 1),
        BumpType::Patch => format!("{}.{}.{}", major, minor, patch + 1),
    };
    Ok(new_version)
}

fn replace_version_in_content(content: &str, old_version: &str, new_version: &str) -> String {
    content.replace(
        &format!("version = \"{old_version}\""),
        &format!("version = \"{new_version}\""),
    )
}

pub fn update_version_files(
    project_dir: &Path,
    old_version: &str,
    new_version: &str,
) -> Result<Vec<PathBuf>> {
    let mut updated = Vec::new();
    let root = ValidatedRoot::new(project_dir)?;

    let cargo_path = project_dir.join("Cargo.toml");
    if cargo_path.exists() {
        let content = std::fs::read_to_string(&cargo_path).map_err(|source| LodeError::Io {
            path: cargo_path.clone(),
            source,
        })?;
        let new_content = replace_version_in_content(&content, old_version, new_version);
        if new_content != content {
            root.write_atomic("Cargo.toml", new_content)?;
            updated.push(cargo_path);
        }
    }

    let pkg_path = project_dir.join("package.json");
    if pkg_path.exists() {
        let content = std::fs::read_to_string(&pkg_path).map_err(|source| LodeError::Io {
            path: pkg_path.clone(),
            source,
        })?;
        let new_content = content.replace(
            &format!("\"version\": \"{old_version}\""),
            &format!("\"version\": \"{new_version}\""),
        );
        if new_content != content {
            root.write_atomic("package.json", new_content)?;
            updated.push(pkg_path);
        }
    }

    let pyproject_path = project_dir.join("pyproject.toml");
    if pyproject_path.exists() {
        let content = std::fs::read_to_string(&pyproject_path).map_err(|source| LodeError::Io {
            path: pyproject_path.clone(),
            source,
        })?;
        let new_content = replace_version_in_content(&content, old_version, new_version);
        if new_content != content {
            root.write_atomic("pyproject.toml", new_content)?;
            updated.push(pyproject_path);
        }
    }

    for name in ["version.txt", "VERSION"] {
        let path = project_dir.join(name);
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|source| LodeError::Io {
                path: path.clone(),
                source,
            })?;
            if content.trim() == old_version {
                root.write_atomic(name, format!("{new_version}\n"))?;
                updated.push(path);
            }
        }
    }

    Ok(updated)
}

pub fn create_release(
    project_dir: &Path,
    config: &ReleaseConfig,
    dry_run: bool,
) -> Result<ReleaseReport> {
    let version = detect_version(project_dir)
        .ok_or_else(|| LodeError::Message("no version detected in project".to_string()))?;

    let new_version = bump_version(&version, &BumpType::Patch)?;
    let tag = format!("{}{}", config.tag_prefix, new_version);

    let files_updated = if dry_run {
        Vec::new()
    } else {
        update_version_files(project_dir, &version, &new_version)?
    };

    Ok(ReleaseReport {
        old_version: version,
        new_version,
        tag,
        files_updated,
        dry_run,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_version_from_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        assert_eq!(detect_version(dir.path()).unwrap(), "1.2.3");
    }

    #[test]
    fn detect_version_from_package_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"name":"test","version":"0.5.0"}"#,
        )
        .unwrap();
        assert_eq!(detect_version(dir.path()).unwrap(), "0.5.0");
    }

    #[test]
    fn detect_version_from_version_txt() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("version.txt"), "3.1.0\n").unwrap();
        assert_eq!(detect_version(dir.path()).unwrap(), "3.1.0");
    }

    #[test]
    fn bump_major() {
        assert_eq!(bump_version("1.2.3", &BumpType::Major).unwrap(), "2.0.0");
    }

    #[test]
    fn bump_minor() {
        assert_eq!(bump_version("1.2.3", &BumpType::Minor).unwrap(), "1.3.0");
    }

    #[test]
    fn bump_patch() {
        assert_eq!(bump_version("1.2.3", &BumpType::Patch).unwrap(), "1.2.4");
    }

    #[test]
    fn bump_with_v_prefix() {
        assert_eq!(bump_version("v1.2.3", &BumpType::Patch).unwrap(), "1.2.4");
    }

    #[test]
    fn update_version_files_replaces_cargo() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let updated = update_version_files(dir.path(), "1.0.0", "2.0.0").unwrap();
        assert_eq!(updated.len(), 1);
        let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(content.contains("version = \"2.0.0\""));
    }

    #[test]
    fn create_release_dry_run_does_not_modify() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let config = ReleaseConfig::default();
        let report = create_release(dir.path(), &config, true).unwrap();
        assert!(report.dry_run);
        assert!(report.files_updated.is_empty());
        assert_eq!(report.tag, "v0.1.1");
    }
}
