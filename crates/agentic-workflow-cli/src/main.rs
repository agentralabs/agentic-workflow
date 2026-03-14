use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "awf")]
#[command(about = "AgenticWorkflow CLI — universal workflow orchestration from the terminal")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new workflow
    Create {
        /// Workflow name
        name: String,
        /// Description
        #[arg(short, long, default_value = "")]
        description: String,
    },

    /// List all workflows
    List,

    /// Run a workflow
    Run {
        /// Workflow ID
        workflow_id: String,
    },

    /// Show workflow status
    Status {
        /// Execution ID
        execution_id: String,
    },

    /// Validate a workflow DAG
    Validate {
        /// Workflow ID
        workflow_id: String,
    },

    /// Visualize a workflow as Mermaid diagram
    Visualize {
        /// Workflow ID
        workflow_id: String,
    },

    /// Start the MCP server
    Serve,

    /// Show version information
    Info,
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
        Command::Create { name, description } => {
            let wf = agentic_workflow::Workflow::new(&name, &description);
            println!("Created workflow: {} ({})", wf.name, wf.id);
        }

        Command::List => {
            println!("No workflows loaded. Use 'awf create <name>' to create one.");
        }

        Command::Run { workflow_id } => {
            println!("Would run workflow: {}", workflow_id);
            println!("(Full execution requires MCP server mode)");
        }

        Command::Status { execution_id } => {
            println!("Would show status for execution: {}", execution_id);
        }

        Command::Validate { workflow_id } => {
            println!("Would validate workflow: {}", workflow_id);
        }

        Command::Visualize { workflow_id } => {
            println!("Would visualize workflow: {}", workflow_id);
        }

        Command::Serve => {
            let handler = agentic_workflow_mcp::ProtocolHandler::new();
            let transport = agentic_workflow_mcp::StdioTransport::new(handler);
            transport.run().await?;
        }

        Command::Info => {
            println!("AgenticWorkflow v{}", env!("CARGO_PKG_VERSION"));
            println!("Universal orchestration engine");
            println!("24 inventions | 124 MCP tools | .awf format");
            println!();
            println!("https://agentralabs.tech");
        }
    }

    Ok(())
}
