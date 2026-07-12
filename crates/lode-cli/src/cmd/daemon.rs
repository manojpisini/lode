#![deny(unsafe_code)]

use crate::DaemonCommand;
pub(crate) fn daemon(command: DaemonCommand) {
    if let Err(error) = crate::daemon_result(command) {
        eprintln!("error: {error}");
    }
}
