# Contributing to AgenticWorkflow

## Development Setup

```bash
git clone https://github.com/agentralabs/agentic-workflow.git
cd agentic-workflow
cargo build -j 1
cargo test -j 1
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` for lint checks
- Use conventional commit prefixes: `feat:`, `fix:`, `chore:`, `docs:`
- Keep files under 400 lines; split by responsibility

## Testing

- Unit tests go in the source file (`#[cfg(test)]`)
- Integration tests go in `tests/suite/` with mod entries in `main.rs`
- Never spawn real infrastructure in unit tests
- Use `-j 1` to prevent OOM on constrained machines

## Pull Requests

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes with tests
4. Run guardrails: `bash scripts/check-canonical-sister.sh`
5. Submit a pull request

## Guardrails

Before pushing, run:

```bash
bash scripts/check-canonical-sister.sh
bash scripts/check-install-commands.sh
bash scripts/check-runtime-hardening.sh
```

## Architecture

See [docs/public/architecture.md](docs/public/architecture.md) for crate responsibilities and data flow.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
