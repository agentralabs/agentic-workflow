# Configuration

## Environment Variables

All AgenticWorkflow environment variables use the `AWF_` prefix. Cross-sister shared variables use the `AGENTRA_` prefix.

| Variable | Default | Allowed Values | Effect |
|----------|---------|---------------|--------|
| `AWF_LOG_LEVEL` | `info` | `trace`, `debug`, `info`, `warn`, `error` | Log verbosity |
| `AWF_DATA_DIR` | `~/.local/share/agentic-workflow` | any path | Data storage root |
| `AWF_CHECKPOINT_DIR` | `$AWF_DATA_DIR/checkpoints` | any path | Checkpoint storage |
| `AWF_TEMPLATE_DIR` | `$AWF_DATA_DIR/templates` | any path | Template search path |
| `AWF_MAX_PARALLEL` | `4` | `1`-`64` | Max parallel step execution |
| `AWF_DEFAULT_TIMEOUT` | (none) | seconds | Global default step timeout |
| `AWF_AUDIT_ENABLED` | `true` | `true`, `false` | Enable audit logging |
| `AWF_METRICS_ENABLED` | `true` | `true`, `false` | Enable metrics collection |
| `AGENTIC_TOKEN` | (none) | any string | Auth token for server profile |
| `AGENTRA_LOG_LEVEL` | (none) | same as AWF_LOG_LEVEL | Cross-sister log override |

## Config Files

AgenticWorkflow reads configuration from (in priority order):

1. Environment variables (highest priority)
2. `$AWF_DATA_DIR/config.toml`
3. Built-in defaults (lowest priority)

### config.toml example

```toml
[engine]
max_parallel = 8
default_timeout = 300

[audit]
enabled = true
retention_days = 30

[templates]
search_paths = ["~/.local/share/agentic-workflow/templates", "./templates"]

[server]
token_required = true
```

## Runtime Modes

- **Desktop** -- full GUI + CLI + MCP server
- **Terminal** -- CLI + MCP server (no GUI)
- **Server** -- MCP server only, requires `AGENTIC_TOKEN` for auth gating
