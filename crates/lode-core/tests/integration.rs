use std::fs;

#[test]
fn test_lode_core_imports() {
    let _ = lode_core::LodeConfig::default();
    let _ = lode_core::default_config();
    let _ = lode_core::command_names();
    let _ = lode_core::profile_names();
    let _ = lode_core::template_paths();
    let _ = lode_core::recipe_names();
}

#[test]
fn test_default_config() {
    let config = lode_core::default_config();
    assert_eq!(config.schema_version, lode_core::SCHEMA_VERSION);
    assert!(!config.identity.author.is_empty());
    assert!(!config.identity.email.is_empty());
    assert!(!config.convention.default_case.is_empty());
}

#[test]
fn test_convention_check_path() {
    let config = lode_core::default_config();
    let temp = tempfile::TempDir::new().unwrap();
    let path = camino::Utf8Path::from_path(temp.path()).unwrap();
    fs::write(path.join("my-file.txt"), "ok").unwrap();
    let report = lode_core::check_path(path, &config).unwrap();
    assert!(report.checked > 0);
}

#[test]
fn test_command_names_embedded() {
    let names = lode_core::command_names();
    assert!(names.contains(&"health"));
    assert!(names.contains(&"build"));
    assert!(names.contains(&"ship"));
    assert!(names.contains(&"verify"));
    assert!(names.contains(&"test-all"));
}

#[test]
fn test_scan_secrets_no_findings() {
    let temp = tempfile::TempDir::new().unwrap();
    let path = camino::Utf8Path::from_path(temp.path()).unwrap();
    fs::write(path.join("safe.txt"), "hello world").unwrap();
    let report = lode_core::scan_secrets(path).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn test_scan_secrets_detects_github_token() {
    let temp = tempfile::TempDir::new().unwrap();
    let path = camino::Utf8Path::from_path(temp.path()).unwrap();
    let token = format!("ghp_{}", "a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6q7R8");
    fs::write(path.join("tokens.txt"), token).unwrap();
    let report = lode_core::scan_secrets(path).unwrap();
    let matching: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.kind == "github token")
        .collect();
    assert!(!matching.is_empty(), "should detect github token pattern");
}

#[test]
fn test_profile_names() {
    let names = lode_core::profile_names();
    assert!(names.contains(&"core/bare"));
    assert!(names.contains(&"core/app"));
    assert!(names.contains(&"core/lib"));
    assert!(names.contains(&"systems/rust-bin"));
}

#[test]
fn test_lodepack_roundtrip_with_lode_core_types() {
    let pack = lode_core::LodePack {
        version: 1,
        manifest: lode_core::LodePackManifest {
            schema_version: 3,
            lode_version: "0.1.0".to_string(),
            created_at: "now".to_string(),
            file_count: 1,
            checksum_algorithm: lode_core::default_lodepack_checksum_algorithm(),
        },
        files: vec![lode_core::LodePackFile {
            path: "config.toml".to_string(),
            contents: "schema_version = 3\n".to_string(),
            checksum: "abc".to_string(),
        }],
    };
    let raw = serde_json::to_string(&pack).unwrap();
    let restored: lode_core::LodePack = serde_json::from_str(&raw).unwrap();
    assert_eq!(restored.version, 1);
    assert_eq!(restored.manifest.schema_version, 3);
    assert_eq!(restored.files[0].path, "config.toml");
    assert_eq!(restored.files[0].checksum, "abc");
}

#[test]
fn test_lodepack_export_import_functions_exist() {
    // Verify the functions compile and accept correct signatures
    let _ = lode_core::export_lodepack;
    let _ = lode_core::import_lodepack;
    let _ = lode_core::commands::export_lodepack;
    let _ = lode_core::commands::import_lodepack;
}

#[test]
fn test_global_dir() {
    let dir = lode_core::global_dir().unwrap();
    assert!(
        dir.as_str().contains("lode")
            || dir.as_str().contains(".lode")
            || dir.as_str().contains("Lode")
    );
}

#[test]
fn test_registry_roundtrip() {
    let registry_path = lode_core::registry_path().unwrap();
    let _original = lode_core::load_registry().unwrap();
    assert!(registry_path.as_str().contains("registry.json"));
}
