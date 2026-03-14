# Changelog

All notable changes to AgenticWorkflow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Core workflow engine with DAG-based step scheduling
- 24 inventions as composable primitives
- .awf file format parser and serializer
- MCP server with 124 tools
- CLI binary (`awf`) with workflow management commands
- C FFI bindings for non-Rust embedding
- Dependency resolver with cycle detection
- Parallel executor with configurable resource limits
- Checkpoint manager for durable workflows
- Template system for reusable workflow composition
- Retry engine with configurable backoff strategies
- Audit logger and metrics collector
- Desktop, terminal, and server install profiles
- Canonical sister kit compliance (scripts, CI, docs)
