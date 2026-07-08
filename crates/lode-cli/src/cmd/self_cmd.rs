use crate::SelfCommand;
pub(crate) fn self_command(command: SelfCommand) -> lode_core::Result<()> {
    crate::self_impl(command)
}
