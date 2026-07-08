use camino::Utf8PathBuf;
pub fn convention_fix(path: Option<Utf8PathBuf>) -> lode_core::Result<()> {
    crate::convention_fix_impl(path)
}
