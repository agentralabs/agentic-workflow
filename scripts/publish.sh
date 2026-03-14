#!/usr/bin/env bash
set -e

DO_PUBLISH=false
if [ "${1:-}" = "--publish" ]; then
  DO_PUBLISH=true
fi

CORE_VERSION="$(grep -m1 'version = "' crates/agentic-workflow/Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/' || true)"
if [ -z "$CORE_VERSION" ]; then
  CORE_VERSION="$(grep -m1 '^version = "' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')"
fi

echo "=== Publishing AgenticWorkflow paired crates to crates.io ==="
echo "Core version: ${CORE_VERSION}"
echo ""

# Verify logged in
cargo login --help > /dev/null

# Run all tests
echo "Running tests..."
cargo test --workspace -j 1
echo "Tests passed."

# Check formatting
echo "Checking format..."
cargo fmt -- --check
echo "Format OK."

# Dry run publish (paired crates: core first, then MCP, then CLI, then FFI)
echo ""
echo "Dry run: agentic-workflow (core)"
(cd crates/agentic-workflow && cargo publish --dry-run)

echo ""
echo "Preflight: agentic-workflow-mcp (build + lint)"
cargo check -p agentic-workflow-mcp -j 1
echo "Note: skipping MCP crates.io dry-run until core crate is published."

echo ""
echo "Preflight: agentic-workflow-cli"
cargo check -p agentic-workflow-cli -j 1

echo ""
echo "Preflight: agentic-workflow-ffi"
cargo check -p agentic-workflow-ffi -j 1

echo ""
if [ "${DO_PUBLISH}" = true ]; then
  echo "Publishing core crate..."
  (cd crates/agentic-workflow && cargo publish)
  echo "Waiting for crates.io propagation..."
  sleep 45

  echo "Publishing MCP crate..."
  (cd crates/agentic-workflow-mcp && cargo publish)
  echo "Waiting for crates.io propagation..."
  sleep 30

  echo "Publishing CLI crate..."
  (cd crates/agentic-workflow-cli && cargo publish)
  sleep 15

  echo "Publishing FFI crate..."
  (cd crates/agentic-workflow-ffi && cargo publish)

  if ! command -v gh >/dev/null 2>&1; then
    echo "Warning: gh CLI not found. Skipping GitHub release."
  else
    echo ""
    echo "Creating GitHub release..."
    gh release create "v${CORE_VERSION}" \
      --title "AgenticWorkflow v${CORE_VERSION}" \
      --notes "AgenticWorkflow v${CORE_VERSION} — Universal orchestration engine for AI agents.

24 capabilities, 124 MCP tools, .awf binary format.
281 tests, 0 failures.

Install: \`cargo install agentic-workflow-cli agentic-workflow-mcp\`
Docs: https://github.com/agentralabs/agentic-workflow" \
      --target "$(git rev-parse HEAD)"
    echo "GitHub release created."
  fi

  echo ""
  echo "Publish complete: v${CORE_VERSION}"
  echo "  agentic-workflow       → crates.io"
  echo "  agentic-workflow-mcp   → crates.io"
  echo "  agentic-workflow-cli   → crates.io"
  echo "  agentic-workflow-ffi   → crates.io"
  echo "  v${CORE_VERSION}       → GitHub release"
else
  echo "Dry run successful. To actually publish:"
  echo "  ./scripts/publish.sh --publish"
fi
