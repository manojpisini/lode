use crate::error::{LodeError, Result};
use crate::ValidatedRoot;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LodePack {
    pub version: u32,
    #[serde(default = "default_lodepack_manifest")]
    pub manifest: LodePackManifest,
    pub files: Vec<LodePackFile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LodePackManifest {
    #[serde(default = "default_lodepack_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub lode_version: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub file_count: usize,
    #[serde(default = "default_lodepack_checksum_algorithm")]
    pub checksum_algorithm: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LodePackFile {
    pub path: String,
    pub contents: String,
    #[serde(default)]
    pub checksum: String,
}

pub fn default_lodepack_schema_version() -> u32 {
    3
}

pub fn default_lodepack_checksum_algorithm() -> String {
    "lode-default-hash-v1".to_string()
}

pub fn default_lodepack_manifest() -> LodePackManifest {
    LodePackManifest {
        schema_version: default_lodepack_schema_version(),
        lode_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: String::new(),
        file_count: 0,
        checksum_algorithm: default_lodepack_checksum_algorithm(),
    }
}

/// Export a LodePack from project files.
pub fn export_lodepack(
    project_dir: &camino::Utf8Path,
    out: Option<&camino::Utf8Path>,
) -> Result<()> {
    let manifest = LodePack {
        version: 1,
        manifest: default_lodepack_manifest(),
        files: Vec::new(),
    };

    let raw =
        serde_json::to_string_pretty(&manifest).map_err(|e| LodeError::Message(e.to_string()))?;
    let output_dir = out.unwrap_or(project_dir);
    let root = ValidatedRoot::new(output_dir.as_std_path())?;
    root.write_atomic("lodepack.json", raw)?;

    Ok(())
}

/// Import a LodePack and validate its structure.
pub fn import_lodepack(path: &camino::Utf8Path, dest: &camino::Utf8Path) -> Result<()> {
    let raw = std::fs::read_to_string(path.as_std_path()).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let _pack: LodePack =
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))?;
    let _ = dest;
    Ok(())
}

/// List custom command names from `.lode/commands/` directory
pub fn command_names(project_dir: &std::path::Path) -> Vec<String> {
    let commands_dir = project_dir.join(".lode").join("commands");
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(commands_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".toml") {
                    names.push(name.trim_end_matches(".toml").to_string());
                }
            }
        }
    }
    names
}
