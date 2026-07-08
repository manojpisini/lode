use crate::LibraryCommand;
pub(crate) fn library_command(
    root: &str,
    command: LibraryCommand,
    embedded: &[&str],
) -> lode_core::Result<()> {
    crate::library_command(root, command, embedded)
}
