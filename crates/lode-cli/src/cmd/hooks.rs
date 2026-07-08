use crate::HooksCommand;
pub(crate) fn hooks(command: HooksCommand) -> lode_core::Result<()> {
    crate::hooks_impl(command)
}
