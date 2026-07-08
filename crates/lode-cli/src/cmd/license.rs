use crate::LicenseCommand;
pub(crate) fn license(command: LicenseCommand) -> lode_core::Result<()> {
    crate::license_impl(command)
}
