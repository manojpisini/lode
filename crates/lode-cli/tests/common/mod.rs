#![allow(dead_code)]

use assert_cmd::Command;
use tempfile::TempDir;

pub fn lode() -> Command {
    Command::cargo_bin("lode").expect("cargo bin exists")
}

pub fn isolated_config(temp: &TempDir) -> String {
    temp.path()
        .join(".lode")
        .join("config.toml")
        .to_string_lossy()
        .into_owned()
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
pub fn test_content_hash(contents: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(contents.as_bytes());
    hex_lower(hasher.finalize())
}

pub fn write_release_rollback(temp: &TempDir, before: &str, after: &str) {
    let rollback_dir = temp.path().join(".lode");
    std::fs::create_dir_all(&rollback_dir).expect("create directory");
    let state = serde_json::json!({
        "schema_version": 3,
        "created_at": "2026-01-01T00:00:00Z",
        "from": "0.1.0",
        "to": "0.1.1",
        "files": [{
            "path": "Cargo.toml",
            "contents": before,
            "before_hash": test_content_hash(before),
            "after_hash": test_content_hash(after),
        }]
    });
    std::fs::write(
        rollback_dir.join("release.rollback.json"),
        serde_json::to_string_pretty(&state).expect("serialize JSON"),
    )
    .expect("write file");
}
