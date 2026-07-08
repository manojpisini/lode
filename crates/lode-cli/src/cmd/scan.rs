use crate::ScanCommand;
pub(crate) fn scan(command: ScanCommand) -> lode_core::Result<()> {
    crate::scan_impl(command)
}
