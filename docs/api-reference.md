# Rust API Reference

## Core Types

### `Engine`

The main entry point for workflow execution.

```rust
use agentic_workflow_core::Engine;

let engine = Engine::new();
let engine = Engine::with_config(config);
```

### `Workflow`

A validated workflow definition ready for execution.

```rust
use agentic_workflow_core::{Workflow, WorkflowBuilder};

let workflow = WorkflowBuilder::new("pipeline")
    .description("Data processing pipeline")
    .add_step(step)
    .build()?;
```

### `Step`

An individual step within a workflow.

```rust
use agentic_workflow_core::Step;

let step = Step::new("fetch", Action::HttpFetch { url })
    .with_timeout(Duration::from_secs(30))
    .with_retry(RetryConfig::exponential(3));
```

### `Execution`

Represents a running or completed workflow execution.

```rust
let execution = engine.run(&workflow)?;
let status = execution.status();
let metrics = execution.metrics();
```

### `AwfFile`

Parse and serialize .awf files.

```rust
use agentic_workflow_core::AwfFile;

let awf = AwfFile::parse_file("pipeline.awf")?;
let workflow = awf.to_workflow()?;
awf.write_file("output.awf")?;
```

## Configuration

```rust
use agentic_workflow_core::Config;

let config = Config::builder()
    .max_parallel(8)
    .data_dir("/custom/path")
    .checkpoint_enabled(true)
    .build();
```

## Error Types

All fallible operations return `Result<T, AwfError>`.

```rust
use agentic_workflow_core::AwfError;

match result {
    Err(AwfError::CycleDetected(nodes)) => { /* handle */ }
    Err(AwfError::StepTimeout(id)) => { /* handle */ }
    Err(AwfError::ValidationFailed(msg)) => { /* handle */ }
    _ => {}
}
```

## Full Documentation

Generate full API docs locally:

```bash
cargo doc --open -p agentic-workflow-core
```
