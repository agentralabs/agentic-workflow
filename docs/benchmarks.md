# Benchmarks

## Overview

Performance benchmarks for AgenticWorkflow core operations. All benchmarks run on Apple M1 with 16GB RAM unless noted otherwise.

## Workflow Parsing

| Workflow Size | Parse Time | Memory |
|---------------|-----------|--------|
| 10 steps | <1ms | ~50KB |
| 100 steps | ~5ms | ~500KB |
| 1000 steps | ~50ms | ~5MB |

## Dependency Resolution

| Graph Size | Topological Sort | Cycle Detection |
|-----------|-----------------|-----------------|
| 10 nodes | <1ms | <1ms |
| 100 nodes | ~2ms | ~1ms |
| 1000 nodes | ~20ms | ~10ms |

## Execution Engine

| Steps | Sequential | Parallel (4) | Parallel (8) |
|-------|-----------|-------------|-------------|
| 10 no-op steps | ~1ms | ~1ms | ~1ms |
| 100 no-op steps | ~10ms | ~5ms | ~3ms |
| 10 x 100ms steps | ~1s | ~300ms | ~200ms |

## Checkpoint Operations

| Operation | Time |
|-----------|------|
| Create checkpoint (10 steps) | ~2ms |
| Restore checkpoint (10 steps) | ~3ms |
| Create checkpoint (100 steps) | ~15ms |
| Restore checkpoint (100 steps) | ~20ms |

## MCP Tool Dispatch

| Operation | Latency |
|-----------|---------|
| Tool list | <1ms |
| Simple tool call | ~2ms |
| Workflow execution tool | depends on workflow |

## Running Benchmarks

```bash
make bench
# or
cargo bench
```

Benchmark results are stored in `target/criterion/` for comparison across runs.
