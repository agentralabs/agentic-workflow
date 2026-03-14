# Key Concepts

## Workflows

A workflow is a directed acyclic graph (DAG) of steps. Each step has an action, parameters, and optional dependencies on other steps. Workflows are defined in `.awf` files.

## Steps

A step is the atomic unit of execution. Each step has:

- **id** -- unique identifier within the workflow
- **action** -- what the step does
- **params** -- configuration for the action
- **depends_on** -- list of step IDs that must complete first
- **timeout** -- optional per-step timeout
- **retry** -- optional retry configuration

## The .awf File Format

AgenticWorkflow uses `.awf` (Agentic Workflow Format) -- a YAML-based declarative format. Files are human-readable, version-controlled, and composable.

## Inventions

The 24 inventions are the composable primitives that power AgenticWorkflow. Each invention is a standalone module (workflow engine, step registry, dependency resolver, etc.) that can be used independently or composed together.

## Templates

Templates are reusable workflow patterns with parameterized placeholders. Create a template from any workflow and apply it with different parameters.

## Checkpoints

Checkpoints capture execution state at a point in time, enabling resume after failure. Useful for long-running workflows.

## Variable Scoping

Variables are resolved at execution time with lexical scoping. Steps can reference outputs from upstream steps using interpolation syntax.

## Resource Pools

Shared resources (database connections, API rate limits, file locks) can be managed through the resource pool, ensuring safe concurrent access across parallel steps.

## MCP Integration

All 24 inventions are exposed as MCP tools, making AgenticWorkflow usable from any MCP-compatible AI agent without writing code.
