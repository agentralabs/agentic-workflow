# MCP Resources

AgenticWorkflow exposes workflow state and metadata as MCP resources.

## URI Scheme

All resources use the `awf://` URI scheme.

| URI Pattern | Description |
|-------------|-------------|
| `awf://workflows` | List of all workflow definitions |
| `awf://workflows/{id}` | Single workflow definition |
| `awf://executions` | List of recent executions |
| `awf://executions/{id}` | Single execution state |
| `awf://executions/{id}/metrics` | Execution metrics |
| `awf://executions/{id}/audit` | Execution audit log |
| `awf://templates` | List of available templates |
| `awf://templates/{name}` | Single template definition |
| `awf://resources` | Shared resource pool state |
| `awf://health` | Engine health status |

## Response Format

All resources return JSON with consistent envelope:

```json
{
  "uri": "awf://workflows/abc123",
  "mimeType": "application/json",
  "text": "{...}"
}
```

## Cross-Sister References

When other Agentra sisters are available, AgenticWorkflow resources can reference:

- `amem://` -- Memory sister for workflow execution history
- `acb://` -- Codebase sister for code-generation step outputs
- `avis://` -- Vision sister for image-processing step outputs
