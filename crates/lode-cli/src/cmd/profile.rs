use crate::ProfileCommand;
pub(crate) fn profile_command(command: ProfileCommand) -> lode_core::Result<()> {
    crate::profile_impl(command)
}
