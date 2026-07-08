use camino::Utf8PathBuf;
pub fn upgrade(
    check: bool,
    manifest: Option<Utf8PathBuf>,
    dry_run: bool,
    rollback: bool,
) -> lode_core::Result<()> {
    crate::upgrade(check, manifest, dry_run, rollback)
}
