pub fn mcp_command(
    http: bool,
    port: Option<u16>,
    list_tools: bool,
    list_resources: bool,
    list_prompts: bool,
) -> lode_core::Result<()> {
    crate::mcp_command(http, port, list_tools, list_resources, list_prompts)
}
