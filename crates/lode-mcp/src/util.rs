pub(crate) fn load_config(
    root: &camino::Utf8Path,
) -> Result<lode_core::config::LodeConfig, String> {
    let project_toml = root.join(".lode").join("project.toml");
    if !project_toml.exists() {
        return Ok(lode_core::config::default_config());
    }
    let raw = std::fs::read_to_string(&project_toml).map_err(|e| e.to_string())?;
    let config: lode_core::config::LodeConfig = toml::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(config)
}
