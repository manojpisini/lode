use std::{fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{LodeError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub enabled: bool,
    pub members_glob: String,
    pub parallel_jobs: u32,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            members_glob: "crates/*".to_string(),
            parallel_jobs: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: Utf8PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub root: Utf8PathBuf,
    pub members: Vec<WorkspaceMember>,
}

impl Workspace {
    pub fn member_names(&self) -> Vec<&str> {
        self.members.iter().map(|m| m.name.as_str()).collect()
    }

    pub fn find_member(&self, name: &str) -> Option<&WorkspaceMember> {
        self.members.iter().find(|m| m.name == name)
    }
}

pub fn discover_members(
    project_dir: &Utf8Path,
    config: &WorkspaceConfig,
) -> Result<Vec<WorkspaceMember>> {
    if !config.enabled {
        return Ok(Vec::new());
    }
    let glob_pattern = project_dir.join(&config.members_glob);
    let pattern_str = glob_pattern.to_string();
    let mut members = Vec::new();

    if pattern_str.contains('*') {
        let base = if let Some(star_pos) = pattern_str.rfind('/') {
            Utf8PathBuf::try_from(&pattern_str[..star_pos])
                .map_err(|e| LodeError::Message(format!("invalid glob base path: {}", e)))?
        } else {
            project_dir.to_path_buf()
        };
        let suffix = if let Some(star_pos) = pattern_str.rfind('/') {
            &pattern_str[star_pos + 1..]
        } else {
            &pattern_str
        };

        if base.exists() && base.is_dir() {
            visit_members(&base, suffix, &mut members)?;
        }
    } else if glob_pattern.exists() && glob_pattern.is_dir() {
        if let Some(name) = glob_pattern.file_name() {
            members.push(WorkspaceMember {
                name: name.to_string(),
                path: glob_pattern,
            });
        }
    }

    members.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(members)
}

fn visit_members(dir: &Utf8Path, suffix: &str, members: &mut Vec<WorkspaceMember>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: PathBuf::from(dir.as_str()),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: PathBuf::from(dir.as_str()),
            source,
        })?;
        let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
            LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
        })?;
        if !child.is_dir() {
            continue;
        }
        let matches = if suffix == "*" || suffix == "**" {
            true
        } else if let Some(inner) = suffix.strip_suffix("/*").or(suffix.strip_suffix("\\*")) {
            child.file_name().is_some_and(|n| n == inner) || suffix.ends_with("/*")
        } else {
            child.file_name().is_some_and(|n| suffix.contains(n))
        };
        if matches {
            if let Some(name) = child.file_name() {
                members.push(WorkspaceMember {
                    name: name.to_string(),
                    path: child,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_config_defaults() {
        let config = WorkspaceConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.parallel_jobs, 4);
    }

    #[test]
    fn discover_members_returns_empty_when_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let config = WorkspaceConfig {
            enabled: false,
            ..Default::default()
        };
        let members = discover_members(&root, &config).unwrap();
        assert!(members.is_empty());
    }

    #[test]
    fn discover_members_finds_directories() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        fs::create_dir_all(root.join("crates").join("alpha")).unwrap();
        fs::create_dir_all(root.join("crates").join("beta")).unwrap();
        fs::write(root.join("crates").join("alpha").join("Cargo.toml"), "").unwrap();
        fs::write(root.join("crates").join("beta").join("Cargo.toml"), "").unwrap();

        let config = WorkspaceConfig {
            enabled: true,
            members_glob: "crates/*".to_string(),
            parallel_jobs: 4,
        };
        let members = discover_members(&root, &config).unwrap();
        assert_eq!(members.len(), 2);
        let names: Vec<&str> = members.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn workspace_find_member() {
        let workspace = Workspace {
            root: Utf8PathBuf::from("/project"),
            members: vec![
                WorkspaceMember {
                    name: "alpha".to_string(),
                    path: Utf8PathBuf::from("/project/crates/alpha"),
                },
                WorkspaceMember {
                    name: "beta".to_string(),
                    path: Utf8PathBuf::from("/project/crates/beta"),
                },
            ],
        };
        assert!(workspace.find_member("alpha").is_some());
        assert!(workspace.find_member("gamma").is_none());
    }

    #[test]
    fn workspace_member_names() {
        let workspace = Workspace {
            root: Utf8PathBuf::from("/project"),
            members: vec![
                WorkspaceMember {
                    name: "a".to_string(),
                    path: Utf8PathBuf::from("/project/a"),
                },
                WorkspaceMember {
                    name: "b".to_string(),
                    path: Utf8PathBuf::from("/project/b"),
                },
            ],
        };
        let names = workspace.member_names();
        assert_eq!(names, vec!["a", "b"]);
    }
}
