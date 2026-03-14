# Integration Guide

## MCP Integration (Recommended)

The simplest way to integrate AgenticWorkflow is through MCP. Any MCP-compatible client can use all 124 tools without writing code.

### Setup

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

### Usage Pattern

1. Call `workflow_create` with a name and description
2. Call `step_add` to add steps with actions and parameters
3. Call `dep_add` to wire step dependencies
4. Call `workflow_validate` to check correctness
5. Call `workflow_run` to execute
6. Call `workflow_status` or `monitor_metrics` to track progress

## Rust API Integration

Add to your `Cargo.toml`:

```toml
[dependencies]
agentic-workflow-core = "0.1"
```

```rust
use agentic_workflow_core::{Engine, WorkflowBuilder};

let engine = Engine::new();
let workflow = WorkflowBuilder::new("my-pipeline")
    .add_step("fetch", Action::HttpFetch { url: "https://..." })
    .add_step("process", Action::Transform { expr: "..." })
    .depends_on("process", "fetch")
    .build()?;

let execution = engine.run(&workflow)?;
```

## C FFI Integration

For non-Rust applications, use the FFI bindings. See [FFI Reference](public/ffi-reference.md).

## CLI Scripting

Use `awf` in shell scripts:

```bash
awf new pipeline.awf --template data-pipeline
awf run pipeline.awf --parallel 8 --checkpoint
awf status "$EXECUTION_ID" --wait
```

## Sister Orchestration

When other Agentra sisters are available, workflows can invoke sister tools as step actions. AgenticWorkflow discovers available sisters at runtime and exposes them as step action types.
