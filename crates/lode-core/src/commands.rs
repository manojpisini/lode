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

/// Walk a directory collecting text files (skipping .git, target, node_modules, .venv).
fn collect_files(dir: &std::path::Path, base: &camino::Utf8Path) -> Result<Vec<LodePackFile>> {
    use std::fs;
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name_str.as_ref(),
                ".git" | "target" | "node_modules" | ".venv"
            ) {
                continue;
            }
            files.extend(collect_files(&path, base)?);
        } else if path.is_file() {
            let relative = camino::Utf8PathBuf::from_path_buf(
                path.strip_prefix(base.as_std_path())
                    .unwrap_or_else(|_| &path)
                    .to_path_buf(),
            )
            .map_err(|p| LodeError::Message(format!("non-UTF-8 path: {}", p.to_string_lossy())))?;
            let contents = fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("warning: could not read {}: {}", path.display(), e);
                String::new()
            });
            let checksum = crate::signature::compute_content_hash(&contents);
            files.push(LodePackFile {
                path: relative.to_string(),
                contents,
                checksum,
            });
        }
    }
    Ok(files)
}

/// Export a LodePack from project files.
pub fn export_lodepack(
    project_dir: &camino::Utf8Path,
    out: Option<&camino::Utf8Path>,
) -> Result<()> {
    let files = collect_files(project_dir.as_std_path(), project_dir)?;
    let mut manifest = default_lodepack_manifest();
    manifest.file_count = files.len();
    let pack = LodePack {
        version: 1,
        manifest,
        files,
    };

    let raw = serde_json::to_string_pretty(&pack).map_err(|e| LodeError::Message(e.to_string()))?;
    let output_dir = out.unwrap_or(project_dir);
    let root = ValidatedRoot::new(output_dir.as_std_path())?;
    root.write_atomic("lodepack.json", raw)?;

    Ok(())
}

/// Import a LodePack and write all files to the destination directory.
pub fn import_lodepack(path: &camino::Utf8Path, dest: &camino::Utf8Path) -> Result<()> {
    let raw = std::fs::read_to_string(path.as_std_path()).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let pack: LodePack =
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))?;

    let root = ValidatedRoot::new(dest.as_std_path())?;
    for file in &pack.files {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(&file.path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = root.create_dir_all(parent) {
                    eprintln!(
                        "warning: could not create directory {}: {}",
                        parent.display(),
                        e
                    );
                }
            }
        }
        root.write_atomic(&file.path, &file.contents)?;
    }

    Ok(())
}

/// List custom command names from `.lode/commands/` directory
pub fn command_names(project_dir: &std::path::Path) -> Vec<String> {
    let commands_dir = project_dir.join(".lode").join("commands");
    let mut names = Vec::new();
    let entries = match std::fs::read_dir(&commands_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("lode: warning: cannot read commands directory: {}", e);
            return vec![];
        }
    };
    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".toml") {
                names.push(name.trim_end_matches(".toml").to_string());
            }
        }
    }
    names
}
