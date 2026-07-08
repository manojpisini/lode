use camino::Utf8PathBuf;
pub fn rename_path(path: Utf8PathBuf, to: Option<String>) -> lode_core::Result<()> {
    crate::rename_impl(path, to)
}
