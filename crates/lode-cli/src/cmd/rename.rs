#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::{load_global_config, ValidatedRoot, LodeError};

pub fn rename_path(path: Utf8PathBuf, to: Option<String>) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let target_name = to.unwrap_or_else(|| {
        path.file_name()
            .map(|name| lode_core::normalize_name(name, &config))
            .unwrap_or_else(|| "renamed".to_string())
    });
    let parent = path
        .parent()
        .map(Utf8PathBuf::from)
        .unwrap_or_else(|| Utf8PathBuf::from("."));
    let destination = parent.join(&target_name);
    ValidatedRoot::new(parent)?.rename_entry(
        path.file_name()
            .ok_or_else(|| LodeError::Message(format!("cannot rename path: {path}")))?,
        &target_name,
    )?;
    println!("renamed {path} -> {destination}");
    Ok(())
}
