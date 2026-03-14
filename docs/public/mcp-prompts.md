# MCP Prompts

AgenticWorkflow provides MCP prompts for common workflow operations.

## Available Prompts

### `workflow-design`

Guide the user through designing a new workflow.

**Arguments:**
| Name | Required | Description |
|------|----------|-------------|
| `goal` | yes | What the workflow should accomplish |
| `complexity` | no | `simple`, `moderate`, `complex` |

### `workflow-debug`

Help diagnose a failed workflow execution.

**Arguments:**
| Name | Required | Description |
|------|----------|-------------|
| `execution_id` | yes | The failed execution ID |

### `workflow-optimize`

Suggest optimizations for an existing workflow.

**Arguments:**
| Name | Required | Description |
|------|----------|-------------|
| `workflow_id` | yes | The workflow to optimize |
| `focus` | no | `speed`, `reliability`, `cost` |

### `awf-explain`

Explain the contents and structure of a .awf file.

**Arguments:**
| Name | Required | Description |
|------|----------|-------------|
| `content` | yes | The .awf file content |
