pub fn sync(dry_run: bool, force: bool, section: Option<&str>) -> lode_core::Result<()> {
    crate::sync_impl(dry_run, force, section)
}
