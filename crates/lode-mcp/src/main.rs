use std::env;

use lode_mcp::server::McpServer;
use lode_mcp::tools::register_all_tools;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut use_stdio = false;
    let mut use_http = false;
    let mut _port = 3000;
    let mut _host = "127.0.0.1".to_string();
    let mut list_tools = false;
    let mut list_resources = false;
    let mut list_prompts = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--stdio" => use_stdio = true,
            "--http" => use_http = true,
            "--port" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    _port = val.parse().unwrap_or(3000);
                }
            }
            "--host" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    _host = val.clone();
                }
            }
            "--list-tools" => list_tools = true,
            "--list-resources" => list_resources = true,
            "--list-prompts" => list_prompts = true,
            "--help" | "-h" => {
                eprintln!("Usage: lode-mcp [OPTIONS]");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  --stdio            Run MCP over stdio (JSON-RPC)");
                eprintln!("  --http             Run MCP over HTTP");
                eprintln!("  --port <PORT>      HTTP port (default: 3000)");
                eprintln!("  --host <HOST>      HTTP host (default: 127.0.0.1)");
                eprintln!("  --list-tools       List all registered tools and exit");
                eprintln!("  --list-resources   List all registered resources and exit");
                eprintln!("  --list-prompts     List all registered prompts and exit");
                eprintln!("  -h, --help         Print this help message");
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Use --help for usage information.");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if list_tools {
        let tools = register_all_tools();
        for tool in &tools {
            println!("{}: {}", tool.name, tool.description);
        }
        return;
    }

    if list_resources {
        let resources = lode_mcp::resources::list_resource_uris();
        for uri in &resources {
            println!("{uri}");
        }
        return;
    }

    if list_prompts {
        let prompts = lode_mcp::prompts::list_prompt_names();
        for name in &prompts {
            println!("{name}");
        }
        return;
    }

    let server = McpServer::new();

    if !use_stdio && !use_http {
        use_stdio = true;
    }

    if use_stdio {
        lode_mcp::transport::run_stdio_transport(&server);
    } else if use_http {
        eprintln!("HTTP transport not yet implemented");
        std::process::exit(1);
    }
}
