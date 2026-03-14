use clap::Parser;
use tracing_subscriber::EnvFilter;

use agentic_workflow_mcp::protocol::ProtocolHandler;
use agentic_workflow_mcp::transport::StdioTransport;

#[derive(Parser)]
#[command(name = "agentic-workflow-mcp")]
#[command(about = "MCP server for AgenticWorkflow — universal workflow orchestration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser)]
enum Command {
    /// Start the MCP server (stdio transport)
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("AWF_LOG")
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Serve) | None => {
            let handler = ProtocolHandler::new();
            let transport = StdioTransport::new(handler);
            transport.run().await?;
        }
    }

    Ok(())
}
