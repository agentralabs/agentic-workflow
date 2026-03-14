#!/usr/bin/env bash
set -euo pipefail

# AgenticWorkflow Installer
# Usage: curl -fsSL https://agentralabs.tech/install/workflow | bash
# Profiles: desktop (default), terminal, server

VERSION="${AWF_VERSION:-latest}"
PROFILE="${1:-desktop}"
INSTALL_DIR="${AWF_INSTALL_DIR:-$HOME/.local/bin}"
REPO="agentralabs/agentic-workflow"
PROJECT="agentic-workflow"
MCP_BIN="agentic-workflow-mcp"
CLI_BIN="awf"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${BLUE}[info]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ok]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC}  $*"; }
err()   { echo -e "${RED}[error]${NC} $*" >&2; }
die()   { err "$@"; exit 1; }

# --- Parse arguments ---
for arg in "$@"; do
  case "$arg" in
    --profile=*) PROFILE="${arg#--profile=}" ;;
    desktop|terminal|server) PROFILE="$arg" ;;
    --version=*) VERSION="${arg#--version=}" ;;
    --help|-h)
      echo "Usage: install.sh [--profile=desktop|terminal|server] [--version=X.Y.Z]"
      echo ""
      echo "Profiles:"
      echo "  desktop   GUI + CLI + MCP server (default)"
      echo "  terminal  CLI + MCP server"
      echo "  server    MCP server only (token-gated)"
      exit 0
      ;;
  esac
done

# --- Detect OS and arch ---
detect_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    *)      die "Unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)             die "Unsupported architecture: $arch" ;;
  esac

  echo "${os}-${arch}"
}

PLATFORM="$(detect_platform)"
info "Platform: $PLATFORM"
info "Profile:  $PROFILE"
info "Install:  $INSTALL_DIR"

# --- Ensure install dir exists ---
mkdir -p "$INSTALL_DIR"

# --- Resolve version ---
resolve_version() {
  if [ "$VERSION" = "latest" ]; then
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
      | grep '"tag_name"' | head -1 | sed 's/.*"v\([^"]*\)".*/\1/' 2>/dev/null)" || true
    if [ -z "$VERSION" ]; then
      warn "Could not resolve latest version from GitHub"
      return 1
    fi
  fi
  info "Version: v$VERSION"
}

# --- Binary install from GitHub releases ---
install_from_release() {
  resolve_version || return 1

  local base_url="https://github.com/$REPO/releases/download/v${VERSION}"
  local artifact="${PROJECT}-${VERSION}-${PLATFORM}.tar.gz"
  local url="${base_url}/${artifact}"

  info "Downloading $url"
  local tmpdir
  tmpdir="$(mktemp -d)"
  trap "rm -rf '$tmpdir'" EXIT

  if ! curl -fsSL "$url" -o "$tmpdir/$artifact"; then
    warn "Release artifact not found: $artifact"
    return 1
  fi

  tar -xzf "$tmpdir/$artifact" -C "$tmpdir"

  # Install binaries based on profile
  case "$PROFILE" in
    desktop|terminal)
      if [ -f "$tmpdir/$CLI_BIN" ]; then
        install -m 755 "$tmpdir/$CLI_BIN" "$INSTALL_DIR/$CLI_BIN"
        ok "Installed $CLI_BIN -> $INSTALL_DIR/$CLI_BIN"
      fi
      if [ -f "$tmpdir/$MCP_BIN" ]; then
        install -m 755 "$tmpdir/$MCP_BIN" "$INSTALL_DIR/$MCP_BIN"
        ok "Installed $MCP_BIN -> $INSTALL_DIR/$MCP_BIN"
      fi
      ;;
    server)
      if [ -f "$tmpdir/$MCP_BIN" ]; then
        install -m 755 "$tmpdir/$MCP_BIN" "$INSTALL_DIR/$MCP_BIN"
        ok "Installed $MCP_BIN -> $INSTALL_DIR/$MCP_BIN"
      fi
      ;;
  esac

  return 0
}

# --- Source build fallback ---
install_from_source() {
  info "Building from source (this may take a few minutes)..."

  if ! command -v cargo &>/dev/null; then
    die "Rust toolchain not found. Install from https://rustup.rs"
  fi

  case "$PROFILE" in
    desktop|terminal)
      cargo install --git "https://github.com/$REPO" agentic-workflow-cli -j 1
      cargo install --git "https://github.com/$REPO" agentic-workflow-mcp -j 1
      ;;
    server)
      cargo install --git "https://github.com/$REPO" agentic-workflow-mcp -j 1
      ;;
  esac

  ok "Built and installed from source"
}

# --- MCP client config detection and merge ---
detect_mcp_clients() {
  local clients_found=()

  # Claude Desktop
  local claude_config=""
  if [ "$(uname -s)" = "Darwin" ]; then
    claude_config="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
  else
    claude_config="$HOME/.config/claude/claude_desktop_config.json"
  fi
  if [ -d "$(dirname "$claude_config")" ]; then
    clients_found+=("claude:$claude_config")
  fi

  # Cursor
  local cursor_config="$HOME/.cursor/mcp.json"
  if [ -d "$(dirname "$cursor_config")" ]; then
    clients_found+=("cursor:$cursor_config")
  fi

  # Windsurf
  local windsurf_config="$HOME/.windsurf/mcp.json"
  if [ -d "$(dirname "$windsurf_config")" ]; then
    clients_found+=("windsurf:$windsurf_config")
  fi

  # Cody
  local cody_config="$HOME/.config/cody/mcp.json"
  if [ -d "$(dirname "$cody_config")" ]; then
    clients_found+=("cody:$cody_config")
  fi

  echo "${clients_found[@]}"
}

merge_mcp_config() {
  local config_path="$1"
  local mcp_bin_path
  mcp_bin_path="$(command -v "$MCP_BIN" 2>/dev/null || echo "$INSTALL_DIR/$MCP_BIN")"

  local mcp_entry
  mcp_entry=$(cat <<ENTRY
{
  "command": "$mcp_bin_path",
  "args": []
}
ENTRY
)

  if [ ! -f "$config_path" ]; then
    # Create new config
    mkdir -p "$(dirname "$config_path")"
    cat > "$config_path" <<NEWCONF
{
  "mcpServers": {
    "agentic-workflow": $mcp_entry
  }
}
NEWCONF
    ok "Created MCP config: $config_path"
    return
  fi

  # Check if already configured
  if grep -q '"agentic-workflow"' "$config_path" 2>/dev/null; then
    info "MCP config already has agentic-workflow entry: $config_path"
    return
  fi

  # Merge into existing config (never overwrite)
  if command -v python3 &>/dev/null; then
    python3 -c "
import json, sys
with open('$config_path', 'r') as f:
    cfg = json.load(f)
if 'mcpServers' not in cfg:
    cfg['mcpServers'] = {}
cfg['mcpServers']['agentic-workflow'] = json.loads('''$mcp_entry''')
with open('$config_path', 'w') as f:
    json.dump(cfg, f, indent=2)
" 2>/dev/null && ok "Merged MCP config: $config_path" || warn "Could not merge config: $config_path"
  else
    warn "python3 not available for JSON merge. Add manually to: $config_path"
    echo "  Entry: \"agentic-workflow\": $mcp_entry"
  fi
}

# --- Check PATH ---
check_path() {
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) return 0 ;;
  esac
  warn "$INSTALL_DIR is not in your PATH"
  echo ""
  echo "  Add to your shell profile:"
  echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
  echo ""
}

# --- Main ---
main() {
  echo ""
  echo -e "${BOLD}AgenticWorkflow Installer${NC}"
  echo "========================"
  echo ""

  # Try release install first, fall back to source
  if ! install_from_release; then
    warn "Release install failed, falling back to source build"
    install_from_source
  fi

  # Configure MCP clients
  echo ""
  info "Detecting MCP clients..."
  local clients
  clients="$(detect_mcp_clients)"

  local configured_clients=()
  for entry in $clients; do
    local name="${entry%%:*}"
    local path="${entry#*:}"
    merge_mcp_config "$path"
    configured_clients+=("$name")
  done

  check_path

  # --- Completion block ---
  echo ""
  echo -e "${GREEN}========================================${NC}"
  echo -e "${GREEN} AgenticWorkflow installed successfully ${NC}"
  echo -e "${GREEN}========================================${NC}"
  echo ""

  # MCP client summary
  if [ ${#configured_clients[@]} -gt 0 ]; then
    echo -e "${BOLD}MCP clients configured:${NC}"
    for client in "${configured_clients[@]}"; do
      echo "  - $client"
    done
  else
    echo "  No MCP clients detected."
  fi
  echo ""

  # Generic MCP guidance
  echo -e "${BOLD}For any MCP client${NC} (Claude, Cursor, Windsurf, Cody, VS Code, Codex, Cline):"
  echo ""
  echo "  Add to your MCP configuration:"
  echo ""
  echo "    {\"mcpServers\": {\"agentic-workflow\": {\"command\": \"$INSTALL_DIR/$MCP_BIN\", \"args\": []}}}"
  echo ""

  # Server profile note
  if [ "$PROFILE" = "server" ]; then
    echo -e "${YELLOW}Server profile:${NC} Set AGENTIC_TOKEN for auth gating."
    echo "  export AGENTIC_TOKEN=\"your-secret-token\""
    echo ""
  fi

  # Quick test
  echo -e "${BOLD}Quick test:${NC}"
  if [ "$PROFILE" = "server" ]; then
    echo "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}' | $MCP_BIN"
  else
    echo "  $CLI_BIN --help"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}' | $MCP_BIN"
  fi
  echo ""

  # Restart guidance
  echo -e "${YELLOW}Restart your MCP client to pick up the new configuration.${NC}"
  echo ""
}

main
