use camino::Utf8PathBuf;
pub fn stamp_path(
    path: Option<Utf8PathBuf>,
    ext: Vec<String>,
    license: bool,
    dry_run: bool,
) -> lode_core::Result<()> {
    crate::stamp_path(path, ext, license, dry_run)
}
