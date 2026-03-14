#!/usr/bin/env bash
set -euo pipefail

# Canonical Sister Guardrail Check
# Validates that this repo complies with CANONICAL_SISTER_KIT.md

PASS=0
FAIL=0
WARN=0

pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
warn() { echo "  WARN: $*"; WARN=$((WARN + 1)); }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Canonical Sister Guardrail Check ==="
echo ""

# --- Section 1: Required files ---
echo "--- Required files ---"

for f in README.md Cargo.toml LICENSE CONTRIBUTING.md SECURITY.md CHANGELOG.md; do
  if [ -f "$f" ]; then pass "$f exists"; else fail "$f missing"; fi
done

for d in scripts docs docs/public; do
  if [ -d "$d" ]; then pass "$d/ exists"; else fail "$d/ missing"; fi
done

# --- Section 2: Required scripts ---
echo ""
echo "--- Required scripts ---"

for s in scripts/install.sh scripts/check-canonical-sister.sh scripts/check-install-commands.sh; do
  if [ -f "$s" ]; then
    pass "$s exists"
    if [ -x "$s" ]; then pass "$s is executable"; else fail "$s is not executable"; fi
  else
    fail "$s missing"
  fi
done

# --- Section 3: Required CI workflows ---
echo ""
echo "--- Required CI workflows ---"

for w in .github/workflows/ci.yml .github/workflows/release.yml \
         .github/workflows/canonical-sister-guardrails.yml \
         .github/workflows/install-command-guardrails.yml; do
  if [ -f "$w" ]; then pass "$w exists"; else fail "$w missing"; fi
done

# --- Section 4: docs/public mandatory pages (8 pages) ---
echo ""
echo "--- Standard reference doc pages (docs/public/) ---"

MANDATORY_PAGES=(
  architecture.md
  cli-reference.md
  configuration.md
  ffi-reference.md
  mcp-tools.md
  mcp-resources.md
  mcp-prompts.md
  troubleshooting.md
)

for page in "${MANDATORY_PAGES[@]}"; do
  if [ -f "docs/public/$page" ]; then
    pass "docs/public/$page exists"
  else
    fail "docs/public/$page missing"
  fi
done

# --- Section 5: docs/ baseline pages ---
echo ""
echo "--- Docs baseline pages ---"

for page in quickstart.md concepts.md integration-guide.md faq.md benchmarks.md api-reference.md; do
  if [ -f "docs/$page" ]; then pass "docs/$page exists"; else fail "docs/$page missing"; fi
done

# --- Section 6: README required sections ---
echo ""
echo "--- README required sections ---"

for section in "## Install" "## Quickstart" "## How It Works"; do
  if grep -q "$section" README.md 2>/dev/null; then
    pass "README has '$section'"
  else
    fail "README missing '$section'"
  fi
done

# Standalone guarantee
if grep -qi "standalone" README.md 2>/dev/null; then
  pass "README mentions standalone guarantee"
else
  fail "README missing standalone guarantee"
fi

# Install profiles
for profile in desktop terminal server; do
  if grep -q "$profile" README.md 2>/dev/null; then
    pass "README mentions $profile profile"
  else
    warn "README does not mention $profile profile"
  fi
done

# --- Section 7: Ecosystem doc ---
echo ""
echo "--- Ecosystem ---"

if [ -f "docs/ecosystem/CANONICAL_SISTER_KIT.md" ]; then
  pass "CANONICAL_SISTER_KIT.md present"
else
  fail "docs/ecosystem/CANONICAL_SISTER_KIT.md missing"
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
  echo "RESULT: PASSED"
  exit 0
fi
