#!/usr/bin/env bash
set -euo pipefail

# AgenticWorkflow Release Publisher
# Usage: scripts/publish.sh <version>
# Example: scripts/publish.sh 0.1.0

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  echo "Usage: scripts/publish.sh <version>"
  echo "Example: scripts/publish.sh 0.1.0"
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Publishing AgenticWorkflow v$VERSION ==="
echo ""

# --- Pre-flight checks ---
echo "--- Pre-flight checks ---"

# Clean working tree
if ! git diff --quiet 2>/dev/null; then
  echo "ERROR: Working tree has uncommitted changes. Commit or stash first."
  exit 1
fi
echo "  OK: Working tree clean"

# Run guardrails
echo "  Running guardrails..."
bash scripts/check-canonical-sister.sh || { echo "ERROR: Canonical sister check failed"; exit 1; }
bash scripts/check-install-commands.sh || { echo "ERROR: Install command check failed"; exit 1; }
echo "  OK: All guardrails passed"

# Run tests
echo "  Running tests..."
cargo test -j 1 || { echo "ERROR: Tests failed"; exit 1; }
echo "  OK: Tests passed"

# Lint
echo "  Running lint..."
cargo fmt -- --check || { echo "ERROR: Format check failed"; exit 1; }
cargo clippy -j 1 -- -D warnings || { echo "ERROR: Clippy failed"; exit 1; }
echo "  OK: Lint passed"

# --- Tag and push ---
echo ""
echo "--- Creating release ---"

git tag -a "v$VERSION" -m "Release v$VERSION"
echo "  Created tag v$VERSION"

git push origin "v$VERSION"
echo "  Pushed tag v$VERSION"

echo ""
echo "Release v$VERSION tagged and pushed."
echo "GitHub Actions will build release artifacts automatically."
echo ""
echo "To publish to crates.io (when ready):"
echo "  cargo publish -p agentic-workflow-core"
echo "  cargo publish -p agentic-workflow-mcp"
echo "  cargo publish -p agentic-workflow-cli"
echo "  cargo publish -p agentic-workflow-ffi"
