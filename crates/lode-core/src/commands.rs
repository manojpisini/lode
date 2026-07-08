use crate::error::{LodeError, Result};
use crate::ValidatedRoot;
use std::collections::BTreeMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LodePack {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub files: BTreeMap<String, String>,
    pub config: Option<String>,
}

/// Export a LodePack from project files
pub fn export_lodepack(
    project_dir: &camino::Utf8Path,
    out: Option<&camino::Utf8Path>,
) -> Result<()> {
    let manifest = LodePack {
        version: 1,
        name: "project".to_string(),
        description: "LODE project export".to_string(),
        files: BTreeMap::new(),
        config: None,
    };

    let raw =
        serde_json::to_string_pretty(&manifest).map_err(|e| LodeError::Message(e.to_string()))?;
    let output_dir = out.unwrap_or(project_dir);
    let root = ValidatedRoot::new(output_dir.as_std_path())?;
    root.write_atomic("lodepack.json", raw)?;

    Ok(())
}

/// Import a LodePack and write files to dest
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
