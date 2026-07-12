#![deny(unsafe_code)]

pub fn lsp_command(stdio: bool, capabilities: bool) -> lode_core::Result<()> {
    if capabilities {
        println!("{}", crate::json_pretty(&crate::lsp_capabilities())?);
    }
    if capabilities && !stdio {
        return Ok(());
    }
    if stdio {
        return crate::run_lsp_stdio();
    }
    println!("lode lsp is available over stdio; run `lode lsp --stdio`");
    Ok(())
}
