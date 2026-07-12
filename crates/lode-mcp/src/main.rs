#![deny(unsafe_code)]

use clap::Parser;
use lode_mcp::server::McpServer;
use lode_mcp::tools::register_all_tools;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "lode-mcp",
    about = "LODE MCP server — exposes tools, resources, and prompts over MCP"
)]
struct Args {
    #[arg(long)]
    stdio: bool,

    #[arg(long)]
    list_tools: bool,

    #[arg(long)]
    list_resources: bool,

    #[arg(long)]
    list_prompts: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.list_tools {
        let tools = register_all_tools();
        for tool in &tools {
            println!("{}: {}", tool.name, tool.description);
        }
        return ExitCode::SUCCESS;
    }

    if args.list_resources {
        let resources = lode_mcp::resources::list_resource_uris();
        for uri in &resources {
            println!("{uri}");
        }
        return ExitCode::SUCCESS;
    }

    if args.list_prompts {
        let prompts = lode_mcp::prompts::list_prompt_names();
        for name in &prompts {
            println!("{name}");
        }
        return ExitCode::SUCCESS;
    }

    let server = McpServer::new();

    if args.stdio {
        lode_mcp::transport::run_stdio_transport(&server);
        ExitCode::SUCCESS
    } else {
        eprintln!("HTTP transport is not yet implemented. Use --stdio instead.");
        ExitCode::from(1)
    }
}
