#![deny(unsafe_code)]

use camino::Utf8PathBuf;
use lode_core::load_global_config;

pub fn sign_path(
    path: Option<Utf8PathBuf>,
    ext: Vec<String>,
    force: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    let config = load_global_config()?;
    let root = path.unwrap_or(crate::current_dir()?);
    let text = format!(
        "Generated with Lode by {} <{}>",
        config.identity.author, config.identity.email
    );
    crate::stamp_files(&root, &ext, &text, force, dry_run)
}
