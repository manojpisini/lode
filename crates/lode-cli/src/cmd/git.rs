use crate::GitCommand;
pub(crate) fn git(command: GitCommand) -> lode_core::Result<()> {
    crate::git_impl(command)
}
