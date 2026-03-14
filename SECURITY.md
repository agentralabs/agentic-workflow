# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x | Yes |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email: security@agentralabs.tech
3. Include: description, reproduction steps, impact assessment
4. You will receive a response within 48 hours

## Security Measures

- All MCP inputs are validated; invalid parameters return explicit errors
- Per-project isolation prevents cross-project state contamination
- Server profile requires `AGENTIC_TOKEN` for authentication
- No arbitrary code execution from .awf files
- Checkpoint data is scoped to project hash
- Installer uses merge-only MCP config updates (never overwrites)

## Scope

This policy covers the AgenticWorkflow codebase and its release artifacts. Third-party dependencies are monitored via `cargo audit`.
