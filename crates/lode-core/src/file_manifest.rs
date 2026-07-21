use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LodeError, Result, ValidatedRoot};

const FILE_MANIFEST_SCHEMA_VERSION: u32 = 1;
const FILE_MANIFEST_RELATIVE: &str = ".lode/state/file-manifest.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedBy {
    Scaffold,
    Adopt,
    Sync,
    Agent,
    Init,
    Context,
    Handoff,
    Verify,
    Manifest,
    DepGraph,
}

impl std::fmt::Display for ManagedBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scaffold => write!(f, "scaffold"),
            Self::Adopt => write!(f, "adopt"),
            Self::Sync => write!(f, "sync"),
            Self::Agent => write!(f, "agent"),
            Self::Init => write!(f, "init"),
            Self::Context => write!(f, "context"),
            Self::Handoff => write!(f, "handoff"),
            Self::Verify => write!(f, "verify"),
            Self::Manifest => write!(f, "manifest"),
            Self::DepGraph => write!(f, "depgraph"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: Utf8PathBuf,
    #[serde(default)]
    pub managed_by: Vec<ManagedBy>,
    pub content_hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileCheckResult {
    pub path: Utf8PathBuf,
    pub status: String,
    pub expected_hash: String,
    pub actual_hash: String,
}

impl FileManifest {
    pub fn new() -> Self {
        Self {
            schema_version: FILE_MANIFEST_SCHEMA_VERSION,
            files: Vec::new(),
        }
    }
}

impl Default for FileManifest {
    fn default() -> Self {
        Self::new()
    }
}

pub fn file_manifest_path(root: &Utf8Path) -> Utf8PathBuf {
    root.join(FILE_MANIFEST_RELATIVE)
}

pub fn load_file_manifest(root: &Utf8Path) -> Result<FileManifest> {
    let path = file_manifest_path(root);
    if !path.exists() {
        return Ok(FileManifest::new());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))
}

pub fn save_file_manifest(root: &Utf8Path, manifest: &FileManifest) -> Result<()> {
    let rel = Utf8Path::new(FILE_MANIFEST_RELATIVE);
    let parent = rel
        .parent()
        .ok_or_else(|| LodeError::Message("file manifest path has no parent".to_string()))?;
    let root = ValidatedRoot::new(root)?;
    root.create_dir_all(parent)?;
    let raw = serde_json::to_string_pretty(manifest)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    root.write_atomic(rel, raw)?;
    Ok(())
}

pub fn add_managed_file(
    root: &Utf8Path,
    path: &Utf8Path,
    managed_by: ManagedBy,
    description: &str,
) -> Result<FileEntry> {
    let mut manifest = load_file_manifest(root)?;

    let resolved = if path.is_relative() {
        root.join(path)
    } else {
        path.to_path_buf()
    };

    let content_hash = if resolved.exists() {
        compute_file_hash(&resolved)?
    } else {
        "not_tracked".to_string()
    };

    let now = timestamp();
    let relative_path = normalize_path(&relativize_path(root, path));

    if let Some(existing) = manifest.files.iter_mut().find(|e| e.path == relative_path) {
        if !existing.managed_by.contains(&managed_by) {
            existing.managed_by.push(managed_by);
        }
        existing.content_hash = content_hash;
        existing.updated_at = now.clone();
        if description != existing.description && !description.is_empty() {
            existing.description = description.to_string();
        }
        let entry = existing.clone();
        save_file_manifest(root, &manifest)?;
        return Ok(entry);
    }

    let entry = FileEntry {
        path: relative_path,
        managed_by: vec![managed_by],
        content_hash,
        created_at: now.clone(),
        updated_at: now,
        description: description.to_string(),
    };
    manifest.files.push(entry.clone());
    save_file_manifest(root, &manifest)?;
    Ok(entry)
}

pub fn remove_managed_file(root: &Utf8Path, path: &Utf8Path) -> Result<bool> {
    let mut manifest = load_file_manifest(root)?;
    let relative = relativize_path(root, path);
    let before = manifest.files.len();
    manifest.files.retain(|e| e.path != relative);
    let removed = manifest.files.len() < before;
    if removed {
        save_file_manifest(root, &manifest)?;
    }
    Ok(removed)
}

pub fn list_managed_files(root: &Utf8Path) -> Result<Vec<FileEntry>> {
    let manifest = load_file_manifest(root)?;
    Ok(manifest.files)
}

pub fn check_file_integrity(root: &Utf8Path) -> Result<Vec<FileCheckResult>> {
    let manifest = load_file_manifest(root)?;
    let mut results = Vec::new();

    for entry in &manifest.files {
        let full_path = root.join(&entry.path);
        let status: String;
        let actual_hash: String;

        if !full_path.exists() {
            status = "missing".to_string();
            actual_hash = String::new();
        } else if entry.content_hash == "not_tracked" {
            status = "not_tracked".to_string();
            actual_hash = "not_tracked".to_string();
        } else {
            match compute_file_hash(&full_path) {
                Ok(hash) => {
                    if hash == entry.content_hash {
                        status = "ok".to_string();
                    } else {
                        status = "modified".to_string();
                    }
                    actual_hash = hash;
                }
                Err(e) => {
                    status = format!("error: {e}");
                    actual_hash = String::new();
                }
            }
        }

        results.push(FileCheckResult {
            path: entry.path.clone(),
            status,
            expected_hash: entry.content_hash.clone(),
            actual_hash,
        });
    }

    Ok(results)
}

pub fn format_file_manifest_table(entries: &[FileEntry]) -> String {
    if entries.is_empty() {
        return "No managed files.".to_string();
    }

    let header = format!(
        " {:<4} {:<40} {:<20} {:<15} {}",
        "#", "Path", "Managed By", "Hash", "Description"
    );
    let sep = format!("{:-<100}", "");
    let mut lines = vec![header, sep];

    for (i, entry) in entries.iter().enumerate() {
        let managed = entry
            .managed_by
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let hash_short = if entry.content_hash.len() > 12 {
            &entry.content_hash[..12]
        } else {
            &entry.content_hash
        };
        let desc_short = if entry.description.len() > 30 {
            format!("{}...", &entry.description[..27])
        } else {
            entry.description.clone()
        };
        lines.push(format!(
            " {:<4} {:<40} {:<20} {:<15} {}",
            i + 1,
            entry.path,
            managed,
            hash_short,
            desc_short,
        ));
    }

    lines.join("\n")
}

fn compute_file_hash(path: &Utf8Path) -> Result<String> {
    let data = fs::read(path.as_std_path()).map_err(|source| LodeError::Io {
        path: PathBuf::from(path.as_str()),
        source,
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(crate::util::hex_lower(hasher.finalize()))
}

fn relativize_path(root: &Utf8Path, path: &Utf8Path) -> Utf8PathBuf {
    if path.starts_with(root) {
        path.strip_prefix(root)
            .map(Utf8PathBuf::from)
            .unwrap_or_else(|_| path.to_path_buf())
    } else {
        path.to_path_buf()
    }
}

fn normalize_path(path: &Utf8Path) -> Utf8PathBuf {
    let s = path.as_str().replace('\\', "/");
    Utf8PathBuf::from(s)
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loads_default_manifest_when_missing() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let manifest = load_file_manifest(&root).unwrap();
        assert_eq!(manifest.schema_version, FILE_MANIFEST_SCHEMA_VERSION);
        assert!(manifest.files.is_empty());
    }

    #[test]
    fn saves_and_loads_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let mut manifest = FileManifest::new();
        manifest.files.push(FileEntry {
            path: Utf8PathBuf::from("test.txt"),
            managed_by: vec![ManagedBy::Scaffold],
            content_hash: "abc123".to_string(),
            created_at: "unix:1000".to_string(),
            updated_at: "unix:1000".to_string(),
            description: "test file".to_string(),
        });
        save_file_manifest(&root, &manifest).unwrap();

        let loaded = load_file_manifest(&root).unwrap();
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(loaded.files[0].path.as_str(), "test.txt");
        assert!(loaded.files[0].managed_by.contains(&ManagedBy::Scaffold));
    }

    #[test]
    fn adds_managed_file() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("hello.txt");
        fs::write(&file_path, b"hello world").unwrap();

        let entry = add_managed_file(
            &root,
            &Utf8PathBuf::from("hello.txt"),
            ManagedBy::Scaffold,
            "a test file",
        )
        .unwrap();

        assert_eq!(entry.path.as_str(), "hello.txt");
        assert!(entry.managed_by.contains(&ManagedBy::Scaffold));
        assert_ne!(entry.content_hash, "not_tracked");
    }

    #[test]
    fn adds_managed_file_with_relative_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("subdir").join("nested.txt");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, b"nested content").unwrap();

        let entry = add_managed_file(&root, &file_path, ManagedBy::Adopt, "nested file").unwrap();

        assert_eq!(entry.path.as_str(), "subdir/nested.txt");
    }

    #[test]
    fn removes_managed_file() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();

        add_managed_file(
            &root,
            &Utf8PathBuf::from("a.txt"),
            ManagedBy::Init,
            "file a",
        )
        .unwrap();
        add_managed_file(
            &root,
            &Utf8PathBuf::from("b.txt"),
            ManagedBy::Init,
            "file b",
        )
        .unwrap();

        assert_eq!(list_managed_files(&root).unwrap().len(), 2);

        let removed = remove_managed_file(&root, &Utf8PathBuf::from("a.txt")).unwrap();
        assert!(removed);

        let files = list_managed_files(&root).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.as_str(), "b.txt");
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let removed = remove_managed_file(&root, &Utf8PathBuf::from("nonexistent.txt")).unwrap();
        assert!(!removed);
    }

    #[test]
    fn update_existing_entry_adds_managed_by() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("shared.txt");
        fs::write(&file_path, b"shared content").unwrap();

        add_managed_file(
            &root,
            &Utf8PathBuf::from("shared.txt"),
            ManagedBy::Scaffold,
            "first",
        )
        .unwrap();
        add_managed_file(
            &root,
            &Utf8PathBuf::from("shared.txt"),
            ManagedBy::Agent,
            "second",
        )
        .unwrap();

        let files = list_managed_files(&root).unwrap();
        assert_eq!(files.len(), 1);
        let entry = &files[0];
        assert!(entry.managed_by.contains(&ManagedBy::Scaffold));
        assert!(entry.managed_by.contains(&ManagedBy::Agent));
    }

    #[test]
    fn check_integrity_detects_modifications() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("tracked.txt");
        fs::write(&file_path, b"original content").unwrap();

        add_managed_file(
            &root,
            &Utf8PathBuf::from("tracked.txt"),
            ManagedBy::Scaffold,
            "tracked",
        )
        .unwrap();

        // Modify the file
        fs::write(&file_path, b"modified content").unwrap();

        let results = check_file_integrity(&root).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "modified");
    }

    #[test]
    fn check_integrity_ok_for_unchanged() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("stable.txt");
        fs::write(&file_path, b"stable content").unwrap();

        add_managed_file(
            &root,
            &Utf8PathBuf::from("stable.txt"),
            ManagedBy::Sync,
            "stable",
        )
        .unwrap();

        let results = check_file_integrity(&root).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "ok");
    }

    #[test]
    fn check_integrity_reports_missing() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("will vanish.txt");
        fs::write(&file_path, b"temporary").unwrap();

        add_managed_file(
            &root,
            &Utf8PathBuf::from("will vanish.txt"),
            ManagedBy::Verify,
            "will be deleted",
        )
        .unwrap();

        fs::remove_file(&file_path).unwrap();

        let results = check_file_integrity(&root).unwrap();
        assert_eq!(results[0].status, "missing");
    }

    #[test]
    fn format_table_with_entries() {
        let entries = vec![FileEntry {
            path: Utf8PathBuf::from("src/main.rs"),
            managed_by: vec![ManagedBy::Scaffold],
            content_hash: "abcdef1234567890abcdef1234567890".to_string(),
            created_at: "unix:1000".to_string(),
            updated_at: "unix:1000".to_string(),
            description: "main source file".to_string(),
        }];
        let table = format_file_manifest_table(&entries);
        assert!(table.contains("src/main.rs"));
        assert!(table.contains("scaffold"));
        assert!(table.contains("main source file"));
    }

    #[test]
    fn format_table_empty() {
        let table = format_file_manifest_table(&[]);
        assert_eq!(table, "No managed files.");
    }

    #[test]
    fn manifest_path_is_under_lode_state() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let path = file_manifest_path(&root);
        assert!(path.as_str().ends_with(".lode/state/file-manifest.json"));
    }

    #[test]
    fn saves_and_loads_through_cycle() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let file_path = root.join("cycle.txt");
        fs::write(&file_path, b"cycle content").unwrap();

        add_managed_file(
            &root,
            &Utf8PathBuf::from("cycle.txt"),
            ManagedBy::Scaffold,
            "cycle test",
        )
        .unwrap();

        let loaded = load_file_manifest(&root).unwrap();
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(loaded.files[0].path.as_str(), "cycle.txt");

        let results = check_file_integrity(&root).unwrap();
        assert_eq!(results[0].status, "ok");
    }
}
