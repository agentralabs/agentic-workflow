# Quickstart Guide

## Install

```bash
curl -fsSL https://agentralabs.tech/install/workflow | bash
```

Or from source:

```bash
cargo install agentic-workflow-cli
cargo install agentic-workflow-mcp
```

## Create Your First Workflow

```bash
awf new hello.awf
```

This creates a minimal workflow file. Edit it:

```yaml
# hello.awf
name: hello-world
version: 1

steps:
  - id: greet
    action: echo
    params:
      message: "Hello from AgenticWorkflow!"

  - id: timestamp
    action: echo
    depends_on: [greet]
    params:
      message: "Completed at $(date)"
```

## Run It

```bash
awf run hello.awf
```

## Validate Without Running

```bash
awf validate hello.awf
```

## Use via MCP

Add AgenticWorkflow to your MCP client:

```json
{
  "mcpServers": {
    "agentic-workflow": {
      "command": "agentic-workflow-mcp",
      "args": []
    }
  }
}
```

Then call tools like `workflow_create`, `workflow_run`, and `workflow_status` from your AI agent.

## Next Steps

- [Key Concepts](concepts.md) -- understand the mental model
- [CLI Reference](public/cli-reference.md) -- all commands
- [MCP Tools](public/mcp-tools.md) -- all 124 tools
- [Integration Guide](integration-guide.md) -- embed in your systems
