use camino::Utf8PathBuf;
pub fn sign_path(
    path: Option<Utf8PathBuf>,
    ext: Vec<String>,
    force: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    crate::sign_path(path, ext, force, dry_run)
}
