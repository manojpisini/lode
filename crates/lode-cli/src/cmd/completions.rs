use camino::Utf8PathBuf;
pub fn completions(
    shell: &str,
    install: bool,
    dry_run: bool,
    out: Option<Utf8PathBuf>,
) -> lode_core::Result<()> {
    crate::completions(shell, install, dry_run, out)
}
