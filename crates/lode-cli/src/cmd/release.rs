pub fn release(
    version: Option<String>,
    bump: Option<String>,
    dry_run: bool,
    rollback: bool,
) -> lode_core::Result<()> {
    crate::release(version, bump, dry_run, rollback)
}
