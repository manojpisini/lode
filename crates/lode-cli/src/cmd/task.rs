pub fn task_command(target: Option<String>, no_store: bool) -> lode_core::Result<()> {
    crate::task_impl(target, no_store)
}
