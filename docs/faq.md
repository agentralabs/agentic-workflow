# Frequently Asked Questions

## General

### What is AgenticWorkflow?

A universal orchestration engine that lets AI agents compose, execute, and monitor multi-step workflows through the `.awf` file format and 124 MCP tools.

### Do I need other Agentra sisters installed?

No. AgenticWorkflow is fully standalone. Other sisters enhance capabilities when present but are never required.

### What is the .awf file format?

A YAML-based declarative format for defining workflows. Files are human-readable, version-controlled, and composable.

## Installation

### Which install profile should I use?

- **desktop** -- if you want GUI + CLI + MCP server
- **terminal** -- if you work in the terminal (CLI + MCP server)
- **server** -- if you only need the MCP server (e.g., headless deployment)

### Can I install from crates.io?

```bash
cargo install agentic-workflow-cli
cargo install agentic-workflow-mcp
```

## Usage

### How many steps can a workflow have?

There is no hard limit. Workflows with hundreds of steps are supported. Performance depends on step complexity and parallelism settings.

### Can workflows call other workflows?

Yes. Use `compose_nest` or `compose_chain` to nest workflows as sub-steps.

### How do I retry failed steps?

Configure retry in the .awf file or use `exec_retry_config` via MCP. Supports exponential backoff, fixed delay, and custom strategies.

### How do I resume a failed workflow?

Enable checkpointing with `--checkpoint`, then resume with `awf run --resume <execution_id>`.

## MCP

### How many MCP tools are available?

124 tools across 24 invention categories.

### Can I use AgenticWorkflow with Claude, Cursor, or other MCP clients?

Yes. Any MCP-compatible client works. The installer auto-detects Claude, Cursor, Windsurf, and Cody.
