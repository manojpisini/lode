use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

/// Supported template kinds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateKind {
    File,
    Bundle,
    Feature,
    Project,
    Overlay,
    Organization,
}

impl std::str::FromStr for TemplateKind {
    type Err = LodeError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "file" => Ok(Self::File),
            "bundle" => Ok(Self::Bundle),
            "feature" => Ok(Self::Feature),
            "project" => Ok(Self::Project),
            "overlay" => Ok(Self::Overlay),
            "organization" => Ok(Self::Organization),
            _ => Err(LodeError::Message(format!("unknown template kind: {s}"))),
        }
    }
}

/// Ownership classification for generated files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Ownership {
    Seed,
    Managed,
    Merged,
    Derived,
    Protected,
    Ephemeral,
    Vendored,
}

/// Overwrite policy when destination exists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OverwritePolicy {
    Error,
    Skip,
    Prompt,
    Replace,
    Merge,
    #[serde(rename = "three_way")]
    ThreeWay,
    Backup,
    Version,
}

/// Variable definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
    #[serde(default)]
    pub values: Vec<String>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
}

/// Directory declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub path: String,
    pub mode: Option<String>,
    #[serde(default)]
    pub keep: bool,
    pub owner: Option<String>,
    #[serde(default)]
    pub gitignore_contents: bool,
}

/// Inline text file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub content: String,
    pub encoding: Option<String>,
    #[serde(rename = "line_ending")]
    pub line_ending: Option<String>,
    pub mode: Option<String>,
    pub owner: Option<String>,
    pub overwrite: Option<String>,
    #[serde(default = "default_true")]
    pub render: bool,
    pub condition: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Linked asset entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub source: String,
    pub destination: String,
    pub owner: Option<String>,
    pub overwrite: Option<String>,
    pub condition: Option<String>,
    pub mode: Option<String>,
    #[serde(default)]
    pub executable: bool,
    pub checksum: Option<String>,
    #[serde(default)]
    pub verify_checksum: bool,
    #[serde(default)]
    pub render_content: bool,
    #[serde(default)]
    pub render_path: bool,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub decompress: bool,
    #[serde(default)]
    pub strip_components: u32,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub architectures: Vec<String>,
    #[serde(default)]
    pub provenance: Option<Provenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: Option<String>,
    pub original_path: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
    pub source_revision: Option<String>,
    pub source_url: Option<String>,
    pub retrieved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLicense {
    pub path: String,
    pub license: String,
    pub license_file: Option<String>,
    pub attribution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: Option<String>,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub kind: Option<String>,
    pub run: String,
    pub working_dir: Option<String>,
    pub permission: Option<String>,
    #[serde(default)]
    pub confirmation: bool,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Compatibility {
    pub lode: Option<String>,
    #[serde(rename = "asset_api")]
    pub asset_api: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub architectures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub last_verified: Option<String>,
    #[serde(default)]
    pub deprecated: bool,
    pub replacement: Option<String>,
    pub remove_after: Option<String>,
    pub migration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub engine: Option<String>,
    #[serde(default)]
    pub strict: bool,
    pub undefined_variables: Option<String>,
    pub default_encoding: Option<String>,
    pub default_line_ending: Option<String>,
    #[serde(default)]
    pub create_parent_directories: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMeta {
    #[serde(default)]
    pub captured: bool,
    pub source_kind: Option<String>,
    pub source_revision: Option<String>,
    pub generator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub allow_symlinks: bool,
    #[serde(default)]
    pub allow_external_assets: bool,
    #[serde(default)]
    pub allow_sensitive_content: bool,
    #[serde(default = "default_max_asset_mb")]
    pub maximum_asset_size_mb: u64,
}

fn default_max_asset_mb() -> u64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictConfig {
    #[serde(default = "default_file_policy")]
    pub default_file_policy: String,
    #[serde(default = "default_asset_policy")]
    pub default_asset_policy: String,
    #[serde(default)]
    pub backup_before_replace: bool,
}

fn default_file_policy() -> String {
    "prompt".into()
}
fn default_asset_policy() -> String {
    "error".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependencies {
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub recommends: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
}

/// Top-level template manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub schema_version: u32,
    pub template: TemplateMeta,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub compatibility: Compatibility,
    #[serde(default)]
    pub lifecycle: Option<Lifecycle>,
    #[serde(default)]
    pub render: Option<RenderConfig>,
    #[serde(default)]
    pub capture: Option<CaptureMeta>,
    #[serde(default)]
    pub security: Option<SecurityConfig>,
    #[serde(default)]
    pub conflicts: Option<ConflictConfig>,
    #[serde(default)]
    pub variables: Vec<Variable>,
    #[serde(default)]
    pub directories: Vec<Directory>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub assets: Vec<AssetEntry>,
    #[serde(default)]
    pub hooks: Vec<Hook>,
    #[serde(default)]
    pub dependencies: Option<Dependencies>,
    #[serde(default)]
    pub changelog: Vec<ChangelogEntry>,
    #[serde(default)]
    pub asset_licenses: Vec<AssetLicense>,
    #[serde(default)]
    pub extends: Vec<String>,
    #[serde(default)]
    pub includes: Vec<String>,
}

impl TemplateManifest {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path).map_err(|source| LodeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&raw).map_err(|e| LodeError::TomlDeserialize {
            path: path.to_path_buf(),
            source: Box::new(e),
        })
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| LodeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let raw =
            toml::to_string_pretty(self).map_err(|e| LodeError::TomlSerialize(Box::new(e)))?;
        std::fs::write(path, &raw).map_err(|source| LodeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    }

    pub fn validate(&self, base_dir: &std::path::Path) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.schema_version != 1 {
            warnings.push(format!(
                "expected schema_version 1, got {}",
                self.schema_version
            ));
        }
        if self.template.id.is_empty() {
            warnings.push("template.id is empty".into());
        }
        if self.template.name.is_empty() {
            warnings.push("template.name is empty".into());
        }
        for (i, f) in self.files.iter().enumerate() {
            if f.path.is_empty() {
                warnings.push(format!("files[{i}] has empty path"));
            }
            if Self::is_unsafe_path(&f.path) {
                warnings.push(format!("files[{i}] unsafe path: {}", f.path));
            }
        }
        for (i, a) in self.assets.iter().enumerate() {
            if a.source.is_empty() {
                warnings.push(format!("assets[{i}] has empty source"));
            }
            if a.destination.is_empty() {
                warnings.push(format!("assets[{i}] has empty destination"));
            }
            if Self::is_unsafe_path(&a.source) {
                warnings.push(format!("assets[{i}] unsafe source: {}", a.source));
            }
            if Self::is_unsafe_path(&a.destination) {
                warnings.push(format!("assets[{i}] unsafe dest: {}", a.destination));
            }
            let asset_path = base_dir.join(&a.source);
            if !asset_path.exists() && !a.optional {
                warnings.push(format!(
                    "asset not found: {} (at {})",
                    a.source,
                    asset_path.display()
                ));
            }
        }
        let mut dests: HashMap<&str, Vec<&str>> = HashMap::new();
        for f in &self.files {
            dests.entry(f.path.as_str()).or_default().push("files");
        }
        for a in &self.assets {
            dests
                .entry(a.destination.as_str())
                .or_default()
                .push("assets");
        }
        for (dest, sources) in &dests {
            if sources.len() > 1 {
                warnings.push(format!(
                    "duplicate destination {dest} from {}",
                    sources.join(" and ")
                ));
            }
        }
        warnings
    }

    fn is_unsafe_path(p: &str) -> bool {
        p.contains("..")
            || p.starts_with('/')
            || p.starts_with('\\')
            || p.chars().any(|c| c == '\0')
            || p.contains(':')
    }
}

pub fn find_template(id: &str, global_dir: &std::path::Path) -> Option<PathBuf> {
    let path = global_dir.join("templates").join(id);
    let manifest = path.join(format!("{}.toml", id.rsplit('/').next().unwrap_or(id)));
    if manifest.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn load_template_bundle(dir: &std::path::Path) -> Result<TemplateManifest> {
    let dirname = dir
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let candidate1 = dir.join(format!("{dirname}.toml"));
    let candidate2 = dir.join(format!(
        "{}.toml",
        dir.file_name().unwrap_or_default().to_string_lossy()
    ));
    let manifest_path = if candidate1.exists() {
        candidate1
    } else {
        candidate2
    };
    TemplateManifest::load(&manifest_path)
}

#[cfg(test)]
pub(crate) fn sample_manifest() -> TemplateManifest {
    TemplateManifest {
        schema_version: 1,
        template: TemplateMeta {
            id: "python/fastapi-service".into(),
            name: "FastAPI Service".into(),
            version: "1.0.0".into(),
            kind: Some("project".into()),
            description: Some("A FastAPI service".into()),
            status: Some("stable".into()),
            license: Some("MIT".into()),
            authors: vec!["Manoj".into()],
            homepage: None,
            documentation: None,
        },
        tags: vec!["python".into(), "fastapi".into()],
        compatibility: Compatibility::default(),
        lifecycle: None,
        render: None,
        capture: None,
        security: None,
        conflicts: None,
        variables: vec![
            Variable {
                name: "project".into(),
                var_type: "slug".into(),
                required: true,
                default: None,
                description: Some("Project slug".into()),
                values: vec![],
                minimum: None,
                maximum: None,
            },
            Variable {
                name: "port".into(),
                var_type: "integer".into(),
                required: false,
                default: Some(serde_json::Value::Number(8000.into())),
                description: Some("Port number".into()),
                values: vec![],
                minimum: Some(1),
                maximum: Some(65535),
            },
        ],
        directories: vec![],
        files: vec![
            FileEntry {
                path: "./README.md".into(),
                content: "# {{ project }}\n\nA FastAPI service.\n".into(),
                encoding: None,
                line_ending: None,
                mode: None,
                owner: Some("seed".into()),
                overwrite: None,
                render: true,
                condition: None,
            },
            FileEntry {
                path: "./src/{{ ident }}/__init__.py".into(),
                content: "".into(),
                encoding: None,
                line_ending: None,
                mode: None,
                owner: Some("seed".into()),
                overwrite: None,
                render: true,
                condition: None,
            },
        ],
        assets: vec![AssetEntry {
            source: "./assets/images/logo.svg".into(),
            destination: "./docs/assets/logo.svg".into(),
            owner: Some("protected".into()),
            overwrite: None,
            condition: None,
            mode: None,
            executable: false,
            checksum: None,
            verify_checksum: false,
            render_content: false,
            render_path: false,
            sensitive: false,
            optional: false,
            decompress: false,
            strip_components: 0,
            platforms: vec![],
            architectures: vec![],
            provenance: None,
        }],
        hooks: vec![],
        dependencies: None,
        changelog: vec![],
        asset_licenses: vec![],
        extends: vec![],
        includes: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serialize_deserialize() {
        let m = sample_manifest();
        let toml_str = toml::to_string_pretty(&m).unwrap();
        let parsed: TemplateManifest = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.template.id, "python/fastapi-service");
        assert_eq!(parsed.variables.len(), 2);
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.assets.len(), 1);
    }

    #[test]
    fn test_validate_missing_asset() {
        let dir = std::env::temp_dir();
        let warnings = sample_manifest().validate(&dir);
        assert!(warnings.iter().any(|w| w.contains("asset not found")));
    }

    #[test]
    fn test_validate_unsafe_paths() {
        let mut m = sample_manifest();
        m.files.push(FileEntry {
            path: "../../escape.txt".into(),
            content: "bad".into(),
            encoding: None,
            line_ending: None,
            mode: None,
            owner: None,
            overwrite: None,
            render: true,
            condition: None,
        });
        assert!(m
            .validate(&std::env::temp_dir())
            .iter()
            .any(|w| w.contains("unsafe")));
    }

    #[test]
    fn test_validate_duplicate_destinations() {
        let mut m = sample_manifest();
        m.files.push(FileEntry {
            path: "./README.md".into(),
            content: "dup".into(),
            encoding: None,
            line_ending: None,
            mode: None,
            owner: None,
            overwrite: None,
            render: true,
            condition: None,
        });
        assert!(m
            .validate(&std::env::temp_dir())
            .iter()
            .any(|w| w.contains("duplicate destination")));
    }

    #[test]
    fn test_ownership_seed() {
        assert_eq!(
            serde_json::from_str::<Ownership>("\"seed\"").unwrap(),
            Ownership::Seed
        );
    }

    #[test]
    fn test_overwrite_three_way() {
        assert_eq!(
            serde_json::from_str::<OverwritePolicy>("\"three_way\"").unwrap(),
            OverwritePolicy::ThreeWay
        );
    }

    #[test]
    fn test_template_kind_from_str() {
        assert_eq!(
            "project".parse::<TemplateKind>().unwrap(),
            TemplateKind::Project
        );
        assert!("unknown".parse::<TemplateKind>().is_err());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("lode-test-{:x}", now_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        sample_manifest().save(&path).unwrap();
        let loaded = TemplateManifest::load(&path).unwrap();
        assert_eq!(loaded.template.id, "python/fastapi-service");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn now_nanos() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}
