pub fn add_component(component: &str, dry_run: bool, overwrite: bool) -> lode_core::Result<()> {
    crate::add_component_impl(component, dry_run, overwrite)
}
