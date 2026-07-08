use crate::ExportOptions;
use camino::Utf8PathBuf;
pub(crate) fn export_lodepack(
    out: Option<Utf8PathBuf>,
    options: ExportOptions,
) -> lode_core::Result<()> {
    crate::export_impl(out, options)
}
pub(crate) fn import_lodepack(
    path: Utf8PathBuf,
    no_merge: bool,
    force: bool,
) -> lode_core::Result<()> {
    crate::import_impl(path, no_merge, force)
}
