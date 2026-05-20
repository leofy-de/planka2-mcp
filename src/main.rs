mod mcp;
mod planka;
mod tools;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use mcp::McpServer;
use planka::PlankaClient;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--dump-tools") {
        let tools = tools::list_tools();
        match serde_json::to_string_pretty(&tools) {
            Ok(json) => {
                println!("{json}");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to serialize tools: {e}");
                std::process::exit(1);
            }
        }
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("planka-mcp {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        std::process::exit(0);
    }

    // Initialize logging (writes to stderr to keep stdout clean for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    info!("Starting planka-mcp server v{}", env!("CARGO_PKG_VERSION"));

    let client = match PlankaClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to initialize Planka client: {e}");
            std::process::exit(1);
        }
    };

    let server = McpServer::new(client);

    if let Err(e) = server.run().await {
        error!("Server error: {e}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!(
        "planka-mcp {} — MCP server for Planka v2 kanban boards\n\n\
         USAGE:\n    planka-mcp [FLAGS]\n\n\
         FLAGS:\n    \
             --dump-tools    Print the JSON-RPC tools/list catalog to stdout and exit\n    \
             -V, --version   Print version and exit\n    \
             -h, --help      Show this help and exit\n\n\
         With no flags, planka-mcp speaks JSON-RPC 2.0 over stdin/stdout (MCP server mode).\n\n\
         Required environment variables in server mode:\n    \
             PLANKA_URL                              Base URL of your Planka instance\n    \
             PLANKA_TOKEN                            Bearer token (preferred)\n    \
             PLANKA_EMAIL + PLANKA_PASSWORD          Alternative to PLANKA_TOKEN\n\n\
         Optional:\n    \
             PLANKA_DEFAULT_CARD_TYPE                \"project\" (default) or \"story\"\n",
        env!("CARGO_PKG_VERSION")
    );
}
