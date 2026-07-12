#![deny(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;

use camino::Utf8PathBuf;
use lode_core::{
    default_lodepack_checksum_algorithm, ensure_global_workspace, global_asset_dir, LodeError,
    LodePack, LodePackFile, LodePackManifest, ValidatedRoot,
};

use crate::ExportOptions;

pub(crate) fn export_lodepack(
    out: Option<Utf8PathBuf>,
    options: ExportOptions,
) -> lode_core::Result<()> {
    let root = crate::global_dir()?;
    let output = out.unwrap_or_else(|| Utf8PathBuf::from("lode-export.lodepack"));
    let mut pack = LodePack {
        version: 1,
        manifest: LodePackManifest {
            schema_version: 3,
            lode_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: crate::now_timestamp(),
            file_count: 0,
            checksum_algorithm: default_lodepack_checksum_algorithm(),
        },
        files: Vec::new(),
    };
    collect_pack_files_as(&root.join("config.toml"), "config.toml", &mut pack)?;
    let mut paths = vec![("profiles", global_asset_dir("profiles")?)];
    if !options.no_commands {
        paths.push(("commands", global_asset_dir("commands")?));
    }
    if !options.no_templates {
        paths.push(("templates", global_asset_dir("templates")?));
    }
    if !options.no_snippets {
        paths.push(("snippets", global_asset_dir("snippets")?));
    }
    if !options.no_licenses {
        paths.push(("licenses", global_asset_dir("licenses")?));
    }
    if !options.no_recipes {
        paths.push(("recipes", global_asset_dir("recipes")?));
    }
    if !options.no_plugins {
        paths.push(("plugins", global_asset_dir("plugins")?));
    }
    if options.include_metrics {
        collect_pack_files_as(&root.join("registry.json"), "registry.json", &mut pack)?;
        collect_pack_files_as(&root.join("metrics.json"), "metrics.json", &mut pack)?;
    }
    for (prefix, path) in paths {
        collect_pack_files_as(&path, prefix, &mut pack)?;
    }
    pack.manifest.file_count = pack.files.len();
    let raw =
        serde_json::to_string_pretty(&pack).map_err(|error| LodeError::Message(error.to_string()))?;
    crate::write_validated_output(&output, raw)?;
    println!("exported {} files to {output}", pack.files.len());
    Ok(())
}

fn collect_pack_files_as(
    path: &Utf8PathBuf,
    prefix: &str,
    pack: &mut LodePack,
) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            let child_prefix = format!(
                "{}/{}",
                prefix.trim_end_matches('/'),
                entry.file_name().to_string_lossy()
            );
            collect_pack_files_as(&child, &child_prefix, pack)?;
        }
        return Ok(());
    }
    let contents = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let checksum = crate::content_hash_bytes(contents.as_bytes());
    pack.files.push(LodePackFile {
        path: prefix.replace('\\', "/"),
        contents,
        checksum,
    });
    Ok(())
}

pub(crate) fn import_lodepack(
    path: Utf8PathBuf,
    no_merge: bool,
    force: bool,
) -> lode_core::Result<()> {
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let pack: LodePack =
        serde_json::from_str(&raw).map_err(|error| LodeError::Message(error.to_string()))?;
    validate_lodepack_manifest(&pack)?;
    let root = crate::global_dir()?;
    ensure_global_workspace()?;
    let global_root = ValidatedRoot::new(&root)?;
    let mut seen_paths = BTreeSet::new();
    let mut validated_files = Vec::new();
    for file in &pack.files {
        let normalized = validate_lodepack_path(&file.path)?;
        if !seen_paths.insert(normalized.clone()) {
            return Err(LodeError::Message(format!(
                "duplicate lodepack path: {normalized}"
            )));
        }
        validate_lodepack_file_checksum(file, &normalized)?;
        validated_files.push((file, normalized));
    }
    validate_lodepack_file_count(&pack)?;
    for (file, normalized) in validated_files {
        let destination = lodepack_destination(&root, &normalized)?;
        if destination.exists() && no_merge && !force {
            return Err(LodeError::Message(format!(
                "import conflict: {} exists",
                normalized
            )));
        }
        if destination.exists() && !force && normalized != "config.toml" {
            continue;
        }
        let relative = destination.strip_prefix(root.as_str()).map_err(|_| {
            LodeError::Message(format!("unsafe lodepack destination: {destination}"))
        })?;
        if let Some(parent) = relative.parent() {
            global_root.create_dir_all(parent)?;
        }
        global_root.write_atomic(relative, &file.contents)?;
    }
    println!("imported {} files from {path}", pack.files.len());
    Ok(())
}

fn validate_lodepack_manifest(pack: &LodePack) -> lode_core::Result<()> {
    if pack.version != 1 {
        return Err(LodeError::Message(format!(
            "unsupported lodepack version: {}",
            pack.version
        )));
    }
    if pack.manifest.schema_version != 3 {
        return Err(LodeError::Message(format!(
            "unsupported lodepack schema: {}",
            pack.manifest.schema_version
        )));
    }
    let expected_algorithm = default_lodepack_checksum_algorithm();
    if pack.manifest.checksum_algorithm != expected_algorithm {
        return Err(LodeError::Message(format!(
            "unsupported lodepack checksum algorithm: {}",
            pack.manifest.checksum_algorithm
        )));
    }
    Ok(())
}

fn validate_lodepack_file_count(pack: &LodePack) -> lode_core::Result<()> {
    if pack.manifest.file_count != 0 && pack.manifest.file_count != pack.files.len() {
        return Err(LodeError::Message(format!(
            "lodepack file count mismatch: manifest has {}, pack has {}",
            pack.manifest.file_count,
            pack.files.len()
        )));
    }
    Ok(())
}

fn validate_lodepack_file_checksum(
    file: &LodePackFile,
    normalized: &str,
) -> lode_core::Result<()> {
    if file.checksum.is_empty() {
        return Ok(());
    }
    let actual = crate::content_hash_bytes(file.contents.as_bytes());
    if actual != file.checksum {
        return Err(LodeError::Message(format!(
            "lodepack checksum mismatch for {normalized}"
        )));
    }
    Ok(())
}

fn lodepack_destination(root: &Utf8PathBuf, path: &str) -> lode_core::Result<Utf8PathBuf> {
    let Some((first, rest)) = path.split_once('/') else {
        return Ok(root.join(path));
    };
    match first {
        "templates" | "profiles" | "snippets" | "licenses" | "plugins" | "recipes"
        | "commands" => Ok(global_asset_dir(first)?.join(rest)),
        _ => Ok(root.join(path)),
    }
}

fn validate_lodepack_path(path: &str) -> lode_core::Result<String> {
    let normalized = path.replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains(':')
        || normalized.chars().any(char::is_control)
    {
        return Err(LodeError::Message(format!("unsafe lodepack path: {path}")));
    }
    let mut segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(LodeError::Message(format!("unsafe lodepack path: {path}")));
    }
    let first = segments.remove(0);
    let valid_root_file =
        matches!(first, "config.toml" | "registry.json" | "metrics.json") && segments.is_empty();
    let valid_asset_path = matches!(
        first,
        "templates" | "profiles" | "snippets" | "licenses" | "plugins" | "recipes" | "commands"
    ) && !segments.is_empty();
    if !valid_root_file && !valid_asset_path {
        return Err(LodeError::Message(format!(
            "unsupported lodepack path: {path}"
        )));
    }
    Ok(normalized)
}
