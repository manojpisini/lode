#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{LodeError, ValidatedRoot};
use serde::{Deserialize, Serialize};

pub fn release(
    version: Option<String>,
    bump: Option<String>,
    dry_run: bool,
    rollback: bool,
) -> lode_core::Result<()> {
    if rollback {
        return rollback_release(dry_run);
    }
    let current = detect_project_version().unwrap_or_else(|| "0.1.0".to_string());
    let next = if let Some(version) = version {
        version.trim_start_matches('v').to_string()
    } else if let Some(bump) = bump {
        bump_version(&current, &bump)?
    } else {
        current.clone()
    };
    let files = version_files();
    if files.is_empty() {
        return Err(LodeError::Message("no version files found".to_string()));
    }
    let rollback = build_release_rollback(&files, &current, &next)?;
    if !dry_run {
        write_release_rollback(&rollback)?;
    }
    for file in files {
        if dry_run {
            println!("would update {file} {current} -> {next}");
        } else {
            if let Err(error) = update_version_file(&file, &next) {
                apply_release_rollback(&rollback)?;
                return Err(error);
            }
            println!("updated {file} to {next}");
        }
    }
    if !dry_run {
        clear_release_rollback()?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseRollback {
    schema_version: u32,
    created_at: String,
    from: String,
    to: String,
    files: Vec<ReleaseRollbackFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseRollbackFile {
    path: Utf8PathBuf,
    contents: String,
    before_hash: String,
    after_hash: String,
}

fn build_release_rollback(
    files: &[String],
    from: &str,
    to: &str,
) -> lode_core::Result<ReleaseRollback> {
    let mut rollback = ReleaseRollback {
        schema_version: 3,
        created_at: crate::now_timestamp(),
        from: from.to_string(),
        to: to.to_string(),
        files: Vec::new(),
    };
    for file in files {
        let safe_path = crate::safe_relative_path(file)?;
        let contents = fs::read_to_string(file).map_err(|source| LodeError::Io {
            path: file.into(),
            source,
        })?;
        let updated = updated_version_contents(file, &contents, to)?;
        rollback.files.push(ReleaseRollbackFile {
            path: safe_path,
            before_hash: crate::content_hash_bytes(contents.as_bytes()),
            after_hash: crate::content_hash_bytes(updated.as_bytes()),
            contents,
        });
    }
    Ok(rollback)
}

fn release_rollback_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".lode").join("release.rollback.json")
}

fn write_release_rollback(rollback: &ReleaseRollback) -> lode_core::Result<()> {
    let path = crate::safe_relative_path(release_rollback_path().as_str())?;
    let project_root = ValidatedRoot::new(crate::current_dir()?)?;
    if let Some(parent) = path.parent() {
        project_root.create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(rollback)
        .map_err(|error| LodeError::Message(error.to_string()))?;
    project_root.write_atomic(&path, raw).map(|_| ())
}

fn read_release_rollback() -> lode_core::Result<ReleaseRollback> {
    let path = crate::safe_relative_path(release_rollback_path().as_str())?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let rollback: ReleaseRollback =
        serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    validate_release_rollback(&rollback)?;
    Ok(rollback)
}

fn validate_release_rollback(rollback: &ReleaseRollback) -> lode_core::Result<()> {
    if rollback.schema_version != 3 {
        return Err(LodeError::Message(format!(
            "unsupported release rollback schema: {}",
            rollback.schema_version
        )));
    }
    if rollback.files.is_empty() {
        return Err(LodeError::Message(
            "release rollback has no files".to_string(),
        ));
    }
    for file in &rollback.files {
        crate::safe_relative_path(file.path.as_str())?;
        let before_hash = crate::content_hash_bytes(file.contents.as_bytes());
        if before_hash != file.before_hash {
            return Err(LodeError::Message(format!(
                "release rollback backup hash mismatch: {}",
                file.path
            )));
        }
    }
    Ok(())
}

fn apply_release_rollback(rollback: &ReleaseRollback) -> lode_core::Result<()> {
    for file in &rollback.files {
        let safe_path = crate::safe_relative_path(file.path.as_str())?;
        let current = fs::read(&safe_path).map_err(|source| LodeError::Io {
            path: safe_path.as_str().into(),
            source,
        })?;
        let current_hash = crate::content_hash_bytes(&current);
        if current_hash == file.before_hash {
            continue;
        }
        if current_hash != file.after_hash {
            return Err(LodeError::Message(format!(
                "release rollback refused because {} changed after rollback state was written",
                file.path
            )));
        }
        ValidatedRoot::new(crate::current_dir()?)?.write_atomic(&file.path, &file.contents)?;
    }
    clear_release_rollback()?;
    eprintln!(
        "release rollback applied: {} -> {}",
        rollback.to, rollback.from
    );
    Ok(())
}

fn rollback_release(dry_run: bool) -> lode_core::Result<()> {
    let rollback = read_release_rollback()?;
    if dry_run {
        for file in &rollback.files {
            println!(
                "would rollback {} {} -> {}",
                file.path, rollback.to, rollback.from
            );
        }
        return Ok(());
    }
    apply_release_rollback(&rollback)
}

fn clear_release_rollback() -> lode_core::Result<()> {
    let path = crate::safe_relative_path(release_rollback_path().as_str())?;
    if path.exists() {
        ValidatedRoot::new(crate::current_dir()?)?.remove_file(&path)?;
    }
    Ok(())
}

fn detect_project_version() -> Option<String> {
    for file in version_files() {
        let raw = fs::read_to_string(&file).ok()?;
        if file == "Cargo.toml" {
            if let Some(version) = toml_section_version(&raw, "package")
                .or_else(|| toml_section_version(&raw, "workspace.package"))
            {
                return Some(version);
            }
        } else if file == "pyproject.toml" {
            if let Some(version) = toml_section_version(&raw, "project") {
                return Some(version);
            }
        } else if file == "package.json" {
            let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
            return value
                .get("version")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
        }
    }
    None
}

fn toml_section_version(raw: &str, wanted_section: &str) -> Option<String> {
    let mut section = "";
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(['[', ']']);
            continue;
        }
        if section == wanted_section && trimmed.starts_with("version") {
            return trimmed
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn version_files() -> Vec<String> {
    ["Cargo.toml", "package.json", "pyproject.toml"]
        .into_iter()
        .filter(|file| Utf8PathBuf::from(file).exists())
        .map(str::to_string)
        .collect()
}

fn bump_version(version: &str, bump: &str) -> lode_core::Result<String> {
    let mut parts: Vec<u64> = Vec::new();
    for part in version.split('.') {
        parts.push(part.parse::<u64>().map_err(|_| {
            LodeError::Message(format!(
                "non-numeric version segment in '{version}': '{part}' is not a valid number"
            ))
        })?);
    }
    while parts.len() < 3 {
        parts.push(0);
    }
    match bump {
        "major" => {
            parts[0] += 1;
            parts[1] = 0;
            parts[2] = 0;
        }
        "minor" => {
            parts[1] += 1;
            parts[2] = 0;
        }
        "patch" => parts[2] += 1,
        other => return Err(LodeError::Message(format!("unsupported bump: {other}"))),
    }
    Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
}

fn update_version_file(file: &str, next: &str) -> lode_core::Result<()> {
    let raw = fs::read_to_string(file).map_err(|source| LodeError::Io {
        path: file.into(),
        source,
    })?;
    let updated = updated_version_contents(file, &raw, next)?;
    ValidatedRoot::new(crate::current_dir()?)?
        .write_atomic(file, updated)
        .map(|_| ())
}

fn updated_version_contents(file: &str, raw: &str, next: &str) -> lode_core::Result<String> {
    let updated = if file == "package.json" {
        let mut value: serde_json::Value =
            serde_json::from_str(raw).map_err(|error| LodeError::Message(error.to_string()))?;
        value["version"] = serde_json::Value::String(next.to_string());
        serde_json::to_string_pretty(&value)
            .map_err(|error| LodeError::Message(error.to_string()))?
            + "\n"
    } else if file == "Cargo.toml" {
        update_toml_version(raw, next, &["package", "workspace.package"])
    } else if file == "pyproject.toml" {
        update_toml_version(raw, next, &["project"])
    } else {
        raw.to_string()
    };
    Ok(updated)
}

fn update_toml_version(raw: &str, next: &str, sections: &[&str]) -> String {
    let mut section = "";
    let mut updated = false;
    let mut lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(['[', ']']);
        }
        if !updated && sections.contains(&section) && trimmed.starts_with("version") {
            lines.push(format!("version = \"{next}\""));
            updated = true;
        } else {
            lines.push(line.to_string());
        }
    }
    lines.join("\n") + "\n"
}
