# CLI Reference

## Global Options

| Flag | Description |
|------|-------------|
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version |
| `--verbose`, `-v` | Increase log verbosity |
| `--quiet`, `-q` | Suppress non-error output |
| `--data-dir <PATH>` | Override data directory |

## Commands

### `awf new <FILE>`

Create a new workflow file.

| Option | Default | Description |
|--------|---------|-------------|
| `--template <NAME>` | `blank` | Template to use |
| `--name <NAME>` | filename | Workflow name |

### `awf run <FILE>`

Execute a workflow.

| Option | Default | Description |
|--------|---------|-------------|
| `--parallel <N>` | `4` | Max parallel steps |
| `--dry-run` | `false` | Validate without executing |
| `--checkpoint` | `false` | Enable checkpointing |
| `--timeout <SECS>` | none | Global timeout |
| `--resume <ID>` | none | Resume from checkpoint |

### `awf validate <FILE>`

Check workflow correctness without executing.

### `awf status [ID]`

Show execution status. Without ID, lists recent executions.

### `awf templates`

List available workflow templates.

| Option | Description |
|--------|-------------|
| `--import <FILE>` | Import a template |
| `--export <NAME>` | Export a template |

### `awf diff <FILE1> <FILE2>`

Compare two workflow files and show differences.

### `awf migrate <FILE>`

Upgrade a workflow file to the latest .awf schema version.

### `awf list`

List all known workflows in the data directory.

### `awf inspect <FILE>`

Show detailed workflow structure including dependency graph.

### `awf metrics [ID]`

Show execution metrics for a completed workflow run.

### `awf audit [ID]`

Show audit log for a workflow execution.
