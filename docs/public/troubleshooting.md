# Troubleshooting

## Installation Issues

### Binary not found after install

Ensure `~/.local/bin` is in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add this line to your `~/.bashrc`, `~/.zshrc`, or equivalent.

### Source build fails with OOM

Limit Cargo parallelism:

```bash
cargo install agentic-workflow-cli -j 1
```

### Cargo not found

Install the Rust toolchain from [rustup.rs](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## MCP Server Issues

### Tool not found errors

Verify the MCP server binary is accessible:

```bash
which agentic-workflow-mcp
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | agentic-workflow-mcp
```

### MCP client not picking up config

1. Verify config file location for your client
2. Ensure the JSON is valid (no trailing commas)
3. Restart your MCP client after config changes

### Server auth failures (server profile)

Set the `AGENTIC_TOKEN` environment variable:

```bash
export AGENTIC_TOKEN="your-secret-token"
```

## Common Errors

### "Cycle detected in dependency graph"

Your workflow has circular dependencies. Use `awf inspect <file>` to visualize the dependency graph and remove the cycle.

### "Step timeout exceeded"

Increase the timeout:

```bash
awf run workflow.awf --timeout 600
```

Or set per-step timeouts in the .awf file:

```yaml
steps:
  - id: slow_step
    timeout: 300
```

### "Checkpoint directory not writable"

Check permissions on the checkpoint directory:

```bash
ls -la "$HOME/.local/share/agentic-workflow/checkpoints"
```

## Performance Tips

- Use `--parallel` to increase concurrent step execution
- Enable checkpointing for long workflows to allow resume on failure
- Use `awf validate` before `awf run` to catch errors early
- Set `AWF_LOG_LEVEL=warn` in production to reduce log overhead
