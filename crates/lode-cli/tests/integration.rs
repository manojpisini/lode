//! Cross-crate integration tests for the LODE workspace.
//! Tests exercise daemon, MCP, and LSP functionality together.

use std::path::Path;

// ---------------------------------------------------------------------------
// Daemon integration: state serialization via lode-daemon types
// ---------------------------------------------------------------------------
#[test]
fn test_daemon_state_roundtrip() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let state_path = dir.path().join("daemon-state.json");

    let mut state = lode_daemon::DaemonState::new();
    state.add_watcher("src".to_string());
    state.add_watcher("tests".to_string());
    state.increment_events();

    lode_daemon::save_state(&state_path, &state).expect("save_state");
    assert!(state_path.exists(), "state file must exist");

    let loaded = lode_daemon::load_state(&state_path).expect("load_state");
    assert!(loaded.active);
    assert_eq!(loaded.events_count, 1);
    assert_eq!(loaded.watchers.len(), 2);
    assert!(loaded.watchers.contains(&"src".to_string()));
}

#[test]
fn test_daemon_state_load_default_for_missing() {
    let state = lode_daemon::load_state(Path::new("nonexistent-state.json")).expect("load missing");
    assert!(!state.active);
    assert_eq!(state.events_count, 0);
}

#[test]
fn test_daemon_state_save_creates_parent_dirs() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let nested = dir.path().join("sub").join("daemon-state.json");

    let state = lode_daemon::DaemonState::new();
    lode_daemon::save_state(&nested, &state).expect("save nested");
    assert!(nested.exists());
}

// ---------------------------------------------------------------------------
// MCP / Security: path validation and process safety via lode-core
// ---------------------------------------------------------------------------
#[test]
fn test_mcp_validated_root_accepts_valid_paths() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let root = lode_core::ValidatedRoot::new(dir.path()).expect("ValidatedRoot");
    assert!(root.path().exists());
}

#[test]
fn test_mcp_validated_root_rejects_missing_path() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let missing = dir.path().join("does-not-exist");
    assert!(lode_core::ValidatedRoot::new(&missing).is_err());
}

#[test]
fn test_mcp_validated_root_resolve_rejects_parent_traversal() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let root = lode_core::ValidatedRoot::new(dir.path()).expect("ValidatedRoot");

    assert!(root.resolve("subdir/file.txt").is_ok());
    assert!(root.resolve("../escape").is_err());
    assert!(root.resolve("a/../../escape").is_err());
}

#[test]
fn test_mcp_process_validates_program_names() {
    assert!(lode_core::Process::new("git").is_ok());
    assert!(lode_core::Process::new("cargo").is_ok());

    assert!(lode_core::Process::new("").is_err());
    assert!(lode_core::Process::new("../sh").is_err());
    assert!(lode_core::Process::new("cmd|echo").is_err());
    assert!(lode_core::Process::new("cmd;ls").is_err());
    assert!(lode_core::Process::new("cmd$PATH").is_err());
    assert!(lode_core::Process::new("cmd`ls`").is_err());
}

// ---------------------------------------------------------------------------
// LSP diagnostics: convention checking and secret scanning via lode-core
// ---------------------------------------------------------------------------
#[test]
fn test_lsp_check_filename_convention_lowercase() {
    let config = lode_core::default_config();
    let good = lode_core::normalize_name("main.rs", &config);
    assert_eq!(good, "main.rs");

    let fixed = lode_core::normalize_name("BadName.rs", &config);
    assert_eq!(fixed, "bad_name.rs");
}

#[test]
fn test_lsp_check_filename_convention_known_good_files() {
    let config = lode_core::default_config();
    for name in &["README.md", "LICENSE", ".gitignore"] {
        let normalized = lode_core::normalize_name(name, &config);
        assert_eq!(normalized, *name, "{name} should be unchanged");
    }
    let normalized = lode_core::normalize_name("Cargo.toml", &config);
    assert_eq!(normalized, "cargo.toml");
}

#[test]
fn test_lsp_scan_secrets_detects_tokens() {
    let findings = lode_core::scan_content("GITHUB_TOKEN=ghp_abc123def456token");
    assert!(!findings.is_empty(), "should detect GitHub token");
}

#[test]
fn test_lsp_scan_secrets_clean_text() {
    let findings = lode_core::scan_content("fn main() { println!(\"hello\"); }");
    assert!(findings.is_empty(), "no secrets in clean code");
}

#[test]
fn test_lsp_scan_secrets_empty_content() {
    let findings = lode_core::scan_content("");
    assert!(findings.is_empty());
}

// ---------------------------------------------------------------------------
// Cross-crate: demonstrate lode-core + lode-daemon working together
// ---------------------------------------------------------------------------
#[test]
fn test_cross_crate_validated_root_with_daemon_state() {
    let dir = tempfile::TempDir::new().expect("temp dir");

    let root = lode_core::ValidatedRoot::new(dir.path()).expect("ValidatedRoot");
    let state_path = root.path().join("daemon-state.json");

    let mut daemon = lode_daemon::DaemonState::new();
    daemon.add_watcher("cross-crate".to_string());
    daemon.increment_events();

    lode_daemon::save_state(&state_path, &daemon).expect("save_state");
    let loaded = lode_daemon::load_state(&state_path).expect("load_state");

    assert!(loaded.active);
    assert_eq!(loaded.events_count, 1);
    assert!(loaded.watchers.contains(&"cross-crate".to_string()));
}

#[test]
fn test_cross_crate_process_and_convention() {
    let config = lode_core::default_config();

    let cargo = lode_core::Process::new("cargo").expect("Process::new");
    let normalized = lode_core::normalize_name("MyProcess.rs", &config);
    assert_eq!(normalized, "my_process.rs");

    let _ = cargo;
}
