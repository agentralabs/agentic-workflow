# Architecture

## Workspace Structure

```
agentic-workflow/
  crates/
    agentic-workflow-core/     # Core engine, inventions, .awf parser
    agentic-workflow-mcp/      # MCP server (124 tools)
    agentic-workflow-cli/      # CLI binary (awf)
    agentic-workflow-ffi/      # C FFI bindings
  docs/
  scripts/
```

## Crate Responsibilities

### agentic-workflow-core

The foundation crate containing all 24 inventions. Pure logic with no I/O dependencies. Includes:

- Workflow engine with DAG-based step scheduling
- .awf file format parser and serializer
- Dependency resolver with cycle detection
- Parallel executor with configurable resource limits
- Checkpoint manager for durable long-running workflows
- Template system for reusable workflow composition
- Variable resolver with scoping and interpolation
- Retry engine with configurable backoff strategies
- Schema validator for step parameter validation
- Audit logger, metrics collector, and event emitter

### agentic-workflow-mcp

MCP stdio server wrapping all core functionality into 124 tools. Handles:

- JSON-RPC 2.0 protocol over stdin/stdout
- Tool dispatch to core invention modules
- Resource exposure for workflow state inspection
- Input validation with explicit error responses
- Token-based auth gating for server profile

### agentic-workflow-cli

The `awf` command-line binary providing:

- `awf new` -- create workflow files
- `awf run` -- execute workflows
- `awf validate` -- check workflow correctness
- `awf status` -- inspect execution state
- `awf templates` -- manage templates
- `awf diff` -- compare workflow versions

### agentic-workflow-ffi

C-compatible FFI surface for embedding in non-Rust applications. Exposes core workflow operations through `extern "C"` functions with opaque handle types.

## Data Flow

```
Input (.awf file or MCP tool call)
  |
  v
Parser / Tool Dispatch
  |
  v
Dependency Resolution (topological sort)
  |
  v
Execution Engine (sequential or parallel)
  |
  v
Step Execution (with retry, timeout, checkpoint)
  |
  v
Output (results, metrics, audit log)
```

## Per-Project Isolation

Each workflow execution is scoped to a project via deterministic path hashing. Same folder names in different locations never share state. Checkpoint and audit data are stored under `$AWF_DATA_DIR/<project_hash>/`.
