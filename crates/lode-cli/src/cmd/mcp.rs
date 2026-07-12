#![deny(unsafe_code)]

pub fn mcp_command(
    http: bool,
    port: Option<u16>,
    list_tools: bool,
    list_resources: bool,
    list_prompts: bool,
) -> lode_core::Result<()> {
    if list_resources {
        println!("{}", crate::json_pretty(&crate::mcp_resources())?);
    }
    if list_prompts {
        println!("{}", crate::json_pretty(&crate::mcp_prompts())?);
    }
    if list_tools {
        println!("{}", crate::json_pretty(&crate::mcp_tools())?);
    }
    if http {
        println!(
            "mcp http mode requested on port {}",
            port.unwrap_or(crate::MCP_HTTP_PORT)
        );
        println!(
            "http+sse transport is not active in this build; use stdio JSON-RPC or list flags"
        );
        return Ok(());
    }
    if list_tools || list_resources || list_prompts {
        return Ok(());
    }

    crate::run_mcp_stdio()
}
