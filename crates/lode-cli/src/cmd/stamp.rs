#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::load_global_config;

pub fn stamp_path(
    path: Option<Utf8PathBuf>,
    ext: Vec<String>,
    license: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let root = path.unwrap_or(crate::current_dir()?);
    let mut text = format!(
        "{} / {} / {}",
        config.identity.org, config.identity.author, config.identity.email
    );
    if license {
        text.push_str(&format!(" / {}", config.identity.license));
    }
    crate::stamp_files(&root, &ext, &text, false, dry_run)
}
