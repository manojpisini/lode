use crate::DaemonCommand;
pub(crate) fn daemon(command: DaemonCommand) {
    crate::daemon_impl(command)
}
