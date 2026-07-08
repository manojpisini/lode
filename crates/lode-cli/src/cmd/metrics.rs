use crate::MetricsCommand;
pub(crate) fn metrics(command: MetricsCommand) -> lode_core::Result<()> {
    crate::metrics_impl(command)
}
