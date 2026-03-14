# AgenticWorkflow

**Universal orchestration engine for agentic systems.**

AgenticWorkflow provides a declarative workflow engine with 24 inventions and 124 MCP tools, enabling AI agents to compose, execute, and monitor complex multi-step workflows through the `.awf` file format.

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://github.com/agentralabs/agentic-workflow/actions/workflows/ci.yml/badge.svg)](https://github.com/agentralabs/agentic-workflow/actions/workflows/ci.yml)

---

## Install

### Quick install (recommended)

```bash
# Default (desktop profile)
curl -fsSL https://agentralabs.tech/install/workflow | bash

# Desktop profile (GUI + MCP server)
curl -fsSL https://agentralabs.tech/install/workflow/desktop | bash

# Terminal profile (CLI + MCP server)
curl -fsSL https://agentralabs.tech/install/workflow/terminal | bash

# Server profile (MCP server only, token-gated)
curl -fsSL https://agentralabs.tech/install/workflow/server | bash
```

### From source

```bash
cargo install agentic-workflow-cli
cargo install agentic-workflow-mcp
```

### Standalone guarantee

AgenticWorkflow is fully standalone. It requires no external runtime, no cloud service, and no other Agentra sister to function. Install it, run it, and it works. When other sisters (Memory, Codebase, Vision, etc.) are present, AgenticWorkflow can orchestrate them -- but they are never required.

---

## Quickstart

### Create a workflow

```bash
# Create a new workflow file
awf new my-pipeline.awf

# Run a workflow
awf run my-pipeline.awf

# List available workflow templates
awf templates
```

### Use via MCP

Add to your MCP client configuration:

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

Then use tools like:

```
workflow_create       — Create a new workflow definition
workflow_run          — Execute a workflow
workflow_status       — Check workflow execution status
workflow_step_add     — Add a step to a workflow
workflow_compose      — Compose workflows together
```

### The .awf file format

AgenticWorkflow uses the `.awf` (Agentic Workflow Format) file format -- a declarative YAML-based format for defining multi-step workflows:

```yaml
# my-pipeline.awf
name: data-pipeline
version: 1
description: Process and analyze data

steps:
  - id: fetch
    action: http_fetch
    params:
      url: "https://api.example.com/data"

  - id: transform
    action: map_transform
    depends_on: [fetch]
    params:
      expression: "item.value * 2"

  - id: store
    action: persist
    depends_on: [transform]
    params:
      target: results.json
```

---

## How It Works

AgenticWorkflow is built around **24 inventions** -- composable primitives that cover the full lifecycle of workflow orchestration:

| # | Invention | Description |
|---|-----------|-------------|
| 1 | Workflow Engine | Core execution runtime with DAG scheduling |
| 2 | Step Registry | Typed step definitions with validation |
| 3 | Dependency Resolver | Topological ordering and cycle detection |
| 4 | Parallel Executor | Concurrent step execution with resource limits |
| 5 | Conditional Router | Branch and merge logic for dynamic flows |
| 6 | Retry Engine | Configurable retry with backoff strategies |
| 7 | Checkpoint Manager | Durable checkpointing for long workflows |
| 8 | Template System | Reusable workflow templates and composition |
| 9 | Variable Resolver | Dynamic variable interpolation and scoping |
| 10 | Event Emitter | Workflow lifecycle event hooks |
| 11 | AWF Parser | .awf file format parser and validator |
| 12 | AWF Serializer | Workflow to .awf export |
| 13 | Schema Validator | JSON Schema validation for step params |
| 14 | Timeout Manager | Per-step and global timeout enforcement |
| 15 | Resource Pool | Shared resource allocation across steps |
| 16 | Audit Logger | Immutable execution audit trail |
| 17 | Metrics Collector | Timing, throughput, and error metrics |
| 18 | Hook System | Pre/post step hooks for side effects |
| 19 | Error Classifier | Categorize failures for smart recovery |
| 20 | Workflow Composer | Nest and chain workflows as sub-steps |
| 21 | Dry Run Engine | Validate workflows without execution |
| 22 | Diff Engine | Compare workflow versions |
| 23 | Migration Engine | Upgrade .awf files across schema versions |
| 24 | MCP Bridge | Expose all inventions as MCP tools |

Each invention is implemented as a standalone module that can be used independently or composed through the workflow engine.

---

## Architecture

AgenticWorkflow is organized as a Cargo workspace with 4 crates:

```
agentic-workflow/
  crates/
    agentic-workflow-core/     # Core engine, inventions, .awf parser
    agentic-workflow-mcp/      # MCP server exposing 124 tools
    agentic-workflow-cli/      # CLI binary (awf)
    agentic-workflow-ffi/      # C FFI for embedding
```

### Crate responsibilities

- **agentic-workflow-core** -- All 24 inventions, the .awf parser/serializer, workflow execution engine, dependency resolution, and the complete domain model. Zero I/O dependencies.
- **agentic-workflow-mcp** -- MCP stdio server that wraps core functionality into 124 tools. Handles JSON-RPC protocol, tool dispatch, and resource exposure.
- **agentic-workflow-cli** -- The `awf` command-line binary. Subcommands for creating, running, inspecting, and managing workflows.
- **agentic-workflow-ffi** -- C-compatible FFI surface for embedding AgenticWorkflow in non-Rust applications (Python, Swift, Node.js, etc.).

### Data flow

```
User / Agent
    |
    v
MCP Client  --->  agentic-workflow-mcp  --->  agentic-workflow-core
    |                                              |
    v                                              v
awf CLI     --->  agentic-workflow-core  --->  .awf files
    |                                              |
    v                                              v
C FFI       --->  agentic-workflow-ffi  --->  agentic-workflow-core
```

---

## MCP Tools

AgenticWorkflow exposes **124 MCP tools** organized across its 24 inventions. Key tool categories:

### Workflow lifecycle (12 tools)
`workflow_create`, `workflow_run`, `workflow_pause`, `workflow_resume`, `workflow_cancel`, `workflow_status`, `workflow_list`, `workflow_delete`, `workflow_clone`, `workflow_export`, `workflow_import`, `workflow_validate`

### Step management (10 tools)
`step_add`, `step_remove`, `step_update`, `step_reorder`, `step_enable`, `step_disable`, `step_status`, `step_output`, `step_retry`, `step_skip`

### Dependency management (6 tools)
`dep_add`, `dep_remove`, `dep_list`, `dep_validate`, `dep_cycle_check`, `dep_topological_sort`

### Template operations (8 tools)
`template_list`, `template_create`, `template_apply`, `template_export`, `template_import`, `template_validate`, `template_compose`, `template_params`

### Execution control (10 tools)
`exec_start`, `exec_stop`, `exec_checkpoint`, `exec_restore`, `exec_dry_run`, `exec_parallel_limit`, `exec_timeout_set`, `exec_retry_config`, `exec_resource_bind`, `exec_metrics`

### AWF file operations (8 tools)
`awf_parse`, `awf_serialize`, `awf_validate`, `awf_migrate`, `awf_diff`, `awf_merge`, `awf_schema`, `awf_format`

### Monitoring and audit (10 tools)
`monitor_events`, `monitor_metrics`, `monitor_audit_log`, `monitor_errors`, `monitor_throughput`, `monitor_duration`, `monitor_resource_usage`, `monitor_step_timeline`, `monitor_health`, `monitor_export`

And 60 additional tools across variable management, conditional routing, hooks, error handling, composition, resource pooling, and schema validation.

See [docs/public/mcp-tools.md](docs/public/mcp-tools.md) for the complete reference.

---

## Configuration

AgenticWorkflow uses the `AWF_` prefix for all environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `AWF_LOG_LEVEL` | `info` | Log verbosity (trace, debug, info, warn, error) |
| `AWF_DATA_DIR` | `~/.local/share/agentic-workflow` | Data storage directory |
| `AWF_MAX_PARALLEL` | `4` | Maximum parallel step execution |
| `AWF_CHECKPOINT_DIR` | `$AWF_DATA_DIR/checkpoints` | Checkpoint storage |
| `AWF_TEMPLATE_DIR` | `$AWF_DATA_DIR/templates` | Template search path |
| `AGENTIC_TOKEN` | (none) | Auth token for server profile |

See [docs/public/configuration.md](docs/public/configuration.md) for full configuration reference.

---

## Documentation

- [Quickstart Guide](docs/quickstart.md)
- [Key Concepts](docs/concepts.md)
- [Integration Guide](docs/integration-guide.md)
- [Architecture](docs/public/architecture.md)
- [CLI Reference](docs/public/cli-reference.md)
- [MCP Tools Reference](docs/public/mcp-tools.md)
- [Configuration](docs/public/configuration.md)
- [FFI Reference](docs/public/ffi-reference.md)
- [Troubleshooting](docs/public/troubleshooting.md)
- [FAQ](docs/faq.md)
- [Benchmarks](docs/benchmarks.md)
- [API Reference](docs/api-reference.md)

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Security

See [SECURITY.md](SECURITY.md) for the security policy and reporting instructions.

## License

MIT License. See [LICENSE](LICENSE) for details.

Copyright (c) 2025-2026 Agentra Labs
