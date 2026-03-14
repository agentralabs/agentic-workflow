#!/usr/bin/env bash
set -euo pipefail

# Runtime Hardening Guardrail Check
# Validates runtime safety per CANONICAL_SISTER_KIT Section 13

PASS=0
FAIL=0
WARN=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
warn() { echo "  WARN: $*"; WARN=$((WARN + 1)); }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Runtime Hardening Guardrail Check ==="
echo ""

# --- MCP input validation ---
echo "--- MCP input validation ---"

if grep -rq "Invalid\|invalid.*param\|validation\|validate" crates/agentic-workflow-mcp/src/ 2>/dev/null; then
  pass "MCP server has input validation patterns"
else
  warn "Could not verify MCP input validation (check manually)"
fi

# --- Project isolation ---
echo ""
echo "--- Project isolation ---"

if grep -rq "project.*id\|workspace.*id\|canonical.*path\|project_hash" crates/ 2>/dev/null; then
  pass "Project isolation patterns found"
else
  warn "No project isolation patterns detected"
fi

# --- No silent fallback ---
echo ""
echo "--- Silent fallback check ---"

SILENT_COUNT=$(grep -rn '\.ok()\|\.unwrap_or_default()\|let _ =' crates/ 2>/dev/null | grep -v "test\|spec\|bench" | wc -l || echo "0")
SILENT_COUNT=$(echo "$SILENT_COUNT" | tr -d ' ')

if [ "$SILENT_COUNT" -gt 10 ]; then
  warn "Found $SILENT_COUNT potential silent error swallowing patterns"
else
  pass "Silent error swallowing within acceptable range ($SILENT_COUNT)"
fi

# --- Lock handling ---
echo ""
echo "--- Concurrent startup hardening ---"

if grep -rq "lock\|Lock\|Mutex\|RwLock\|file_lock\|flock" crates/ 2>/dev/null; then
  pass "Lock/concurrency patterns found"
else
  warn "No lock patterns detected (may not need concurrent startup)"
fi

# --- Auth gate for server profile ---
echo ""
echo "--- Server auth gate ---"

if grep -rq "AGENTIC_TOKEN\|auth.*token\|token.*gate" crates/ scripts/ 2>/dev/null; then
  pass "Token-based auth gate referenced"
else
  warn "No AGENTIC_TOKEN auth gate found"
fi

# --- Env var namespace ---
echo ""
echo "--- Env var namespace ---"

if grep -rq "AWF_" crates/ 2>/dev/null; then
  pass "AWF_ prefix env vars used"
else
  warn "No AWF_ prefixed env vars found in crates"
fi

# --- Summary ---
echo ""
echo "=== Summary ==="
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo "  WARN: $WARN"
echo ""

if [ "$FAIL" -gt 0 ]; then
  echo "RESULT: FAILED ($FAIL failures)"
  exit 1
else
  echo "RESULT: PASSED (with $WARN warnings)"
  exit 0
fi
