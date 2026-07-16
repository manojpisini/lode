use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;

use crate::error::{LodeError, Result};
use crate::secrets::{scan_content, SecretFinding};
use crate::template_bundle::*;

/// Capture mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureMode {
    Minimal,
    Source,
    Development,
    Complete,
    Custom,
}

/// Configuration for template capture
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub mode: CaptureMode,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub destination: Option<PathBuf>,
    pub project: bool,
    pub dry_run: bool,
    pub redact_secrets: bool,
    pub git_tracked: bool,
    pub git_diff: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub inline_text_max_kb: u64,
    pub asset_warning_mb: u64,
    pub asset_maximum_mb: u64,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            mode: CaptureMode::Source,
            template_id: None,
            template_name: None,
            destination: None,
            project: false,
            dry_run: false,
            redact_secrets: false,
            git_tracked: false,
            git_diff: None,
            include_patterns: vec![],
            exclude_patterns: vec![],
            inline_text_max_kb: 256,
            asset_warning_mb: 25,
            asset_maximum_mb: 100,
        }
    }
}

/// Preview of a capture operation
#[derive(Debug, Clone)]
pub struct CapturePreview {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub template_id: String,
    pub template_name: String,
    pub inline_count: usize,
    pub asset_count: usize,
    pub directory_count: usize,
    pub file_classifications: Vec<FileClassification>,
    pub variables: Vec<String>,
    pub warnings: Vec<String>,
    pub secrets_found: Vec<SecretFinding>,
    pub estimated_size_kb: u64,
}

/// Classification for each file during walk
#[derive(Debug, Clone)]
pub struct FileClassification {
    pub path: String,
    pub classification: ClassKind,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassKind {
    Inline,
    Asset,
    Excluded,
    Binary,
    OversizedAsset,
    SecretFound,
}

/// A capture receipt
#[derive(Debug, Clone, Serialize)]
pub struct CaptureReceipt {
    pub operation_id: String,
    pub timestamp: u64,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub template_id: String,
    pub inline_files: Vec<String>,
    pub assets_copied: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub secret_findings: Vec<SecretFinding>,
    pub warnings: Vec<String>,
    pub rendered_preview: bool,
    pub round_trip_passed: bool,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Preview a capture without writing anything
pub fn capture_preview(source: &Path, config: &CaptureConfig) -> Result<CapturePreview> {
    let template_name = config.template_name.clone().unwrap_or_else(|| {
        source
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    let template_id = config
        .template_id
        .clone()
        .unwrap_or_else(|| template_name.clone());

    let mut classifications = Vec::new();
    let mut warnings = Vec::new();
    let mut secrets_found = Vec::new();
    let mut inline_count = 0;
    let mut asset_count = 0;
    let mut total_size = 0u64;

    walk_source(
        source,
        source,
        config,
        &mut classifications,
        &mut warnings,
        &mut secrets_found,
    )?;

    for fc in &classifications {
        match fc.classification {
            ClassKind::Inline => inline_count += 1,
            ClassKind::Asset | ClassKind::OversizedAsset => asset_count += 1,
            _ => {}
        }
        total_size += fc.size_bytes;
    }

    let variables = infer_variables(source, &template_name);

    let destination = match &config.destination {
        Some(d) => d.clone(),
        None => {
            let dest_base = if config.project {
                source.join(".lode").join("templates")
            } else {
                crate::install::global_asset_dir("templates")
                    .map(|p| p.into_std_path_buf())
                    .unwrap_or_else(|_| std::env::temp_dir().join(".lode/templates"))
            };
            dest_base.join(&template_id)
        }
    };

    Ok(CapturePreview {
        source: source.to_path_buf(),
        destination,
        template_id,
        template_name,
        inline_count,
        asset_count,
        directory_count: classifications
            .iter()
            .filter(|f| f.classification == ClassKind::Excluded)
            .count(),
        file_classifications: classifications,
        variables,
        warnings,
        secrets_found,
        estimated_size_kb: total_size / 1024,
    })
}

/// Capture a template from source to destination
pub fn capture_template(source: &Path, config: &CaptureConfig) -> Result<CaptureReceipt> {
    let preview = capture_preview(source, config)?;
    if config.dry_run {
        return Ok(CaptureReceipt {
            operation_id: String::new(),
            timestamp: now_secs(),
            source: source.to_path_buf(),
            destination: preview.destination.clone(),
            template_id: preview.template_id.clone(),
            inline_files: vec![],
            assets_copied: vec![],
            excluded_paths: vec![],
            secret_findings: preview.secrets_found.clone(),
            warnings: preview.warnings.clone(),
            rendered_preview: true,
            round_trip_passed: false,
        });
    }

    let dest_dir = &preview.destination;
    std::fs::create_dir_all(dest_dir).map_err(|e| LodeError::Io {
        path: dest_dir.to_path_buf(),
        source: e,
    })?;
    let assets_dir = dest_dir.join("assets");
    let manifest_name = format!(
        "{}.toml",
        preview
            .template_id
            .rsplit('/')
            .next()
            .unwrap_or(&preview.template_id)
    );

    let mut inline_files = Vec::new();
    let mut assets_copied = Vec::new();
    let mut excluded = Vec::new();
    let mut file_entries = Vec::new();
    let mut asset_entries = Vec::new();
    let mut classifications = Vec::new();
    let mut warnings = preview.warnings.clone();
    let mut secrets = preview.secrets_found.clone();

    walk_source(
        source,
        source,
        config,
        &mut classifications,
        &mut warnings,
        &mut secrets,
    )?;

    for fc in &classifications {
        let rel_path = &fc.path;
        let full_path = source.join(rel_path);
        match fc.classification {
            ClassKind::Inline => {
                let content = std::fs::read_to_string(&full_path).unwrap_or_default();
                let rendered_content = if config.redact_secrets {
                    redact_content(&content)
                } else {
                    content.clone()
                };
                file_entries.push(FileEntry {
                    path: format!("./{rel_path}"),
                    content: rendered_content,
                    encoding: None,
                    line_ending: None,
                    mode: None,
                    owner: classify_owner(rel_path),
                    overwrite: None,
                    render: true,
                    condition: None,
                });
                inline_files.push(rel_path.clone());
            }
            ClassKind::Asset | ClassKind::OversizedAsset => {
                let asset_dir = assets_dir.join(rel_path);
                if let Some(parent) = asset_dir.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::copy(&full_path, &asset_dir) {
                    warnings.push(format!("failed to copy asset {rel_path}: {e}"));
                    continue;
                }
                asset_entries.push(AssetEntry {
                    source: format!("./assets/{rel_path}"),
                    destination: format!("./{rel_path}"),
                    owner: classify_owner(rel_path),
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
                    provenance: Some(Provenance {
                        origin: Some("project".into()),
                        original_path: Some(rel_path.clone()),
                        license: None,
                        author: None,
                        source_revision: None,
                        source_url: None,
                        retrieved_at: None,
                    }),
                });
                assets_copied.push(rel_path.clone());
            }
            ClassKind::Excluded => {
                excluded.push(rel_path.clone());
            }
            ClassKind::Binary | ClassKind::SecretFound => {
                warnings.push(format!("skipped {rel_path}: {:?}", fc.classification));
            }
        }
    }

    let template_id = preview.template_id.clone();
    let template_name = preview.template_name.clone();
    let kind = if file_entries.len() == 1 && asset_entries.is_empty() {
        "file"
    } else if source.join("Cargo.toml").exists() || source.join("pyproject.toml").exists() {
        "project"
    } else {
        "bundle"
    };

    let variables = infer_variables(source, &template_name);
    let mut manifest_vars = Vec::new();
    for v in &variables {
        manifest_vars.push(Variable {
            name: v.clone(),
            var_type: "string".into(),
            required: false,
            default: None,
            description: None,
            values: vec![],
            minimum: None,
            maximum: None,
        });
    }

    let manifest = TemplateManifest {
        schema_version: 1,
        template: TemplateMeta {
            id: template_id.clone(),
            name: template_name,
            version: "1.0.0".into(),
            kind: Some(kind.into()),
            description: None,
            status: Some("experimental".into()),
            license: None,
            authors: vec![],
            homepage: None,
            documentation: None,
        },
        tags: vec![],
        compatibility: Compatibility::default(),
        lifecycle: Some(Lifecycle {
            created_at: Some(iso_now()),
            updated_at: Some(iso_now()),
            last_verified: Some(iso_now()),
            deprecated: false,
            replacement: None,
            remove_after: None,
            migration: None,
        }),
        render: Some(RenderConfig {
            engine: Some("lode".into()),
            strict: true,
            undefined_variables: Some("error".into()),
            default_encoding: Some("utf-8".into()),
            default_line_ending: Some("lf".into()),
            create_parent_directories: true,
        }),
        capture: Some(CaptureMeta {
            captured: true,
            source_kind: Some("directory".into()),
            source_revision: None,
            generator: Some("lode".into()),
        }),
        security: None,
        conflicts: None,
        variables: manifest_vars,
        directories: vec![],
        files: file_entries,
        assets: asset_entries,
        hooks: vec![],
        dependencies: None,
        changelog: vec![],
        asset_licenses: vec![],
        extends: vec![],
        includes: vec![],
    };

    manifest.save(&dest_dir.join(&manifest_name))?;

    Ok(CaptureReceipt {
        operation_id: format!("capture-{:x}", now_secs()),
        timestamp: now_secs(),
        source: source.to_path_buf(),
        destination: dest_dir.to_path_buf(),
        template_id,
        inline_files,
        assets_copied,
        excluded_paths: excluded,
        secret_findings: secrets,
        warnings,
        rendered_preview: false,
        round_trip_passed: false,
    })
}

/// Walk a source directory, classifying files
fn walk_source(
    root: &Path,
    dir: &Path,
    config: &CaptureConfig,
    classifications: &mut Vec<FileClassification>,
    warnings: &mut Vec<String>,
    secrets: &mut Vec<SecretFinding>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if is_excluded(&rel, config) {
            classifications.push(FileClassification {
                path: rel,
                classification: ClassKind::Excluded,
                size_bytes: 0,
            });
            continue;
        }

        if path.is_dir() {
            walk_source(root, &path, config, classifications, warnings, secrets)?;
            continue;
        }

        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();

        let class = classify_file(&path, size, config);
        match class {
            ClassKind::Inline => {
                if !config.redact_secrets {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let findings = scan_content(&content);
                        if !findings.is_empty() {
                            let f = findings.into_iter().map(|mut f| {
                                f.path = rel.clone().into();
                                f
                            });
                            secrets.extend(f);
                            if config.mode != CaptureMode::Complete {
                                classifications.push(FileClassification {
                                    path: rel,
                                    classification: ClassKind::SecretFound,
                                    size_bytes: size,
                                });
                                continue;
                            }
                        }
                    }
                }
            }
            ClassKind::Binary => {
                warnings.push(format!("binary file skipped: {rel}"));
            }
            _ => {}
        }

        classifications.push(FileClassification {
            path: rel,
            classification: class,
            size_bytes: size,
        });
    }
    Ok(())
}

fn is_excluded(rel: &str, config: &CaptureConfig) -> bool {
    let default_excludes = [
        ".git",
        ".lode/state",
        ".lode/cache",
        "target",
        "node_modules",
        ".venv",
        "venv",
        "dist",
        "build",
        "coverage",
        "logs",
        "reports",
        "artifacts",
        "tmp",
        ".cache",
        ".lodecaptureignore",
    ];
    for pat in &default_excludes {
        if rel == *pat
            || rel.starts_with(&format!("{pat}/"))
            || rel.starts_with(&format!("{pat}\\"))
        {
            return true;
        }
    }
    for pat in &config.exclude_patterns {
        if rel.contains(pat) {
            return true;
        }
    }
    if !config.include_patterns.is_empty() {
        return !config.include_patterns.iter().any(|p| rel.contains(p));
    }
    false
}

fn classify_file(path: &Path, size: u64, config: &CaptureConfig) -> ClassKind {
    if size > config.asset_maximum_mb * 1024 * 1024 {
        return ClassKind::OversizedAsset;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let binary_exts = [
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "avif", "woff", "woff2", "ttf", "otf",
        "eot", "mp4", "avi", "mov", "mkv", "webm", "mp3", "wav", "ogg", "flac", "aac", "zip",
        "tar", "gz", "bz2", "xz", "7z", "rar", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "sqlite", "db", "dbf", "exe", "dll", "so", "dylib", "wasm", "class", "jar", "woff2", "eot",
    ];
    if binary_exts.contains(&ext.as_str()) {
        return ClassKind::Asset;
    }

    let max_inline = config.inline_text_max_kb * 1024;
    if size > max_inline {
        return ClassKind::Asset;
    }

    // Try to detect null bytes
    if let Ok(content) = std::fs::read(path) {
        if content.contains(&0u8) {
            return ClassKind::Binary;
        }
        // Check if valid UTF-8
        if std::str::from_utf8(&content).is_err() {
            return ClassKind::Asset;
        }
    }

    ClassKind::Inline
}

fn classify_owner(rel_path: &str) -> Option<String> {
    if rel_path.starts_with(".lode") || rel_path.ends_with(".lode") || rel_path.starts_with(".git")
    {
        Some("managed".into())
    } else if rel_path.ends_with(".lock") || rel_path.ends_with(".gitkeep") {
        Some("derived".into())
    } else if rel_path.contains("fixtures") || rel_path.contains("vendor") {
        Some("vendored".into())
    } else {
        Some("seed".into())
    }
}

fn infer_variables(source: &Path, template_name: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    let dir_name = source
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    if !dir_name.is_empty() && dir_name != template_name {
        candidates.push("project".into());
        candidates.push("name".into());
    }

    // Check for common project files
    let name_hints = [
        ("Cargo.toml", "name"),
        ("package.json", "name"),
        ("pyproject.toml", "name"),
    ];
    for (file, _) in &name_hints {
        let path = source.join(file);
        if path.exists() {
            candidates.push("project".into());
            break;
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

fn redact_content(content: &str) -> String {
    let mut result = String::new();
    for line in content.lines() {
        if !crate::secrets::scan_content(line).is_empty() {
            result.push_str("# REDACTED: potential secret\n");
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn iso_now() -> String {
    // simple ISO date without chrono
    let secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Approximate date from days since epoch
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let year_days = if is_leap(y) { 366 } else { 365 };
        if d < year_days {
            break;
        }
        d -= year_days;
        y += 1;
    }
    let is_leap_year = is_leap(y);
    let month_days = [
        31,
        if is_leap_year { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    for (i, md) in month_days.iter().enumerate() {
        if d < *md {
            m = i;
            break;
        }
        d -= *md;
    }
    let day = d + 1;

    format!(
        "{y:04}-{:02}-{:02}T{hours:02}:{minutes:02}:{seconds:02}Z",
        m + 1,
        day
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lode-capture-test-{name}-{:x}", now_nanos()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir.join("src")).unwrap();
        dir
    }

    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    #[test]
    fn test_capture_preview_empty_dir() {
        let dir = test_dir("empty");
        let config = CaptureConfig::default();
        let preview = capture_preview(&dir, &config).unwrap();
        assert_eq!(preview.inline_count, 0);
        assert!(preview.warnings.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_capture_preview_with_files() {
        let dir = test_dir("files");
        fs::write(dir.join("README.md"), "# Test\n").unwrap();
        fs::write(dir.join("src/main.py"), "print('hello')\n").unwrap();
        let config = CaptureConfig::default();
        let preview = capture_preview(&dir, &config).unwrap();
        assert_eq!(preview.inline_count, 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_capture_template_dry_run() {
        let dir = test_dir("dryrun");
        fs::write(dir.join("main.py"), "print(1)\n").unwrap();
        let config = CaptureConfig {
            dry_run: true,
            ..CaptureConfig::default()
        };
        let receipt = capture_template(&dir, &config).unwrap();
        assert!(receipt.rendered_preview);
        assert!(receipt.inline_files.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_capture_template_creates_bundle() {
        let dir = test_dir("bundle");
        fs::write(dir.join("README.md"), "# My Project\n").unwrap();
        fs::write(dir.join("src/__init__.py"), "").unwrap();
        fs::create_dir_all(dir.join("docs/images")).unwrap();
        // create a small PNG-like file (just a header for binary detection)
        fs::write(
            dir.join("docs/images/icon.png"),
            &[137, 80, 78, 71, 13, 10, 26, 10],
        )
        .unwrap();

        let config = CaptureConfig::default();
        let receipt = capture_template(&dir, &config).unwrap();
        assert!(!receipt.assets_copied.is_empty() || !receipt.inline_files.is_empty());

        // Verify the bundle was written
        let _template_id = dir.file_name().unwrap().to_string_lossy().to_string();
        // The bundle might be in temp dir, just check receipt fields
        assert!(receipt.destination.exists());
        let _ = fs::remove_dir_all(&receipt.destination);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_excluded() {
        let config = CaptureConfig::default();
        assert!(is_excluded(".git", &config));
        assert!(is_excluded("target/debug/main.o", &config));
        assert!(!is_excluded("src/main.rs", &config));
    }

    #[test]
    fn test_classify_file_text() {
        let dir = test_dir("classify");
        let path = dir.join("hello.txt");
        fs::write(&path, "hello").unwrap();
        let kind = classify_file(&path, 5, &CaptureConfig::default());
        assert_eq!(kind, ClassKind::Inline);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_classify_file_binary_ext() {
        let kind = classify_file(Path::new("image.png"), 1000, &CaptureConfig::default());
        assert_eq!(kind, ClassKind::Asset);
    }

    #[test]
    fn test_classify_file_oversized() {
        let kind = classify_file(
            Path::new("data.txt"),
            300_000_000,
            &CaptureConfig::default(),
        );
        assert_eq!(kind, ClassKind::OversizedAsset);
    }

    #[test]
    fn test_classify_owner() {
        assert_eq!(classify_owner("src/main.rs"), Some("seed".into()));
        assert_eq!(classify_owner(".lode/project.toml"), Some("managed".into()));
        assert_eq!(classify_owner("Cargo.lock"), Some("derived".into()));
        assert_eq!(
            classify_owner("tests/fixtures/data.csv"),
            Some("vendored".into())
        );
    }

    #[test]
    fn test_infer_variables() {
        let dir = test_dir("infer");
        let vars = infer_variables(&dir, "test-template");
        assert!(vars.is_empty() || vars.contains(&"project".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_iso_now_format() {
        let s = iso_now();
        assert!(s.len() >= 20);
        assert!(s.ends_with('Z'));
    }

    #[test]
    fn test_capture_excludes_git_dir() {
        let dir = test_dir("git-exclude");
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::write(dir.join(".git/config"), "[core]\n").unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let config = CaptureConfig::default();
        let preview = capture_preview(&dir, &config).unwrap();
        assert!(preview
            .file_classifications
            .iter()
            .any(|f| f.classification == ClassKind::Excluded && f.path.starts_with(".git")));
        assert!(
            preview
                .file_classifications
                .iter()
                .any(|f| f.path.contains("main.rs")),
            "expected main.rs in classifications, got: {:#?}",
            preview.file_classifications
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
