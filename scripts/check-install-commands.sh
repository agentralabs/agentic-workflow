#!/usr/bin/env bash
set -euo pipefail

# Install Command Guardrail Check
# Validates install routes are correct and consistent

PASS=0
FAIL=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Install Command Guardrail Check ==="
echo ""

# --- Check installer script exists and is executable ---
echo "--- Installer script ---"

if [ -f "scripts/install.sh" ]; then
  pass "scripts/install.sh exists"
  if [ -x "scripts/install.sh" ]; then
    pass "scripts/install.sh is executable"
  else
    fail "scripts/install.sh is not executable"
  fi
else
  fail "scripts/install.sh missing"
fi

# --- Check README install commands reference correct URLs ---
echo ""
echo "--- README install commands ---"

if grep -q "agentralabs.tech/install/workflow" README.md 2>/dev/null; then
  pass "README has canonical install URL"
else
  fail "README missing canonical install URL"
fi

for profile in desktop terminal server; do
  if grep -q "install/workflow/$profile" README.md 2>/dev/null; then
    pass "README has $profile profile URL"
  else
    fail "README missing $profile profile install URL"
  fi
done

# --- Check installer handles all profiles ---
echo ""
echo "--- Installer profile support ---"

for profile in desktop terminal server; do
  if grep -q "$profile" scripts/install.sh 2>/dev/null; then
    pass "Installer handles $profile profile"
  else
    fail "Installer missing $profile profile"
  fi
done

# --- Check installer has merge-only MCP config ---
echo ""
echo "--- MCP config behavior ---"

if grep -q "merge" scripts/install.sh 2>/dev/null || grep -q "already" scripts/install.sh 2>/dev/null; then
  pass "Installer uses merge-only MCP config"
else
  fail "Installer may overwrite MCP config (merge-only required)"
fi

# --- Check installer has completion block ---
echo ""
echo "--- Completion block ---"

if grep -q "installed successfully" scripts/install.sh 2>/dev/null; then
  pass "Installer has completion block"
else
  fail "Installer missing completion block"
fi

if grep -q "MCP client" scripts/install.sh 2>/dev/null; then
  pass "Installer has MCP client summary"
else
  fail "Installer missing MCP client summary"
fi

# --- Check binary names ---
echo ""
echo "--- Binary names ---"

if grep -q "awf" scripts/install.sh 2>/dev/null; then
  pass "Installer references awf binary"
else
  fail "Installer missing awf binary reference"
fi

if grep -q "agentic-workflow-mcp" scripts/install.sh 2>/dev/null; then
  pass "Installer references agentic-workflow-mcp binary"
else
  fail "Installer missing agentic-workflow-mcp binary reference"
fi

# --- Check source fallback ---
echo ""
echo "--- Source build fallback ---"

if grep -q "cargo" scripts/install.sh 2>/dev/null; then
  pass "Installer has cargo source fallback"
else
  fail "Installer missing source build fallback"
fi

# --- Summary ---
echo ""
echo "=== Summary ==="
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo ""

if [ "$FAIL" -gt 0 ]; then
  echo "RESULT: FAILED ($FAIL failures)"
  exit 1
else
  echo "RESULT: PASSED"
  exit 0
fi
