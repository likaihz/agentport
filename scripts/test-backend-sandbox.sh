#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
REAL_HOME="${HOME:-}"
SANDBOX_DIR="${AGENTPORT_TEST_SANDBOX:-$(mktemp -d)}"

cleanup() {
  if [[ -z "${AGENTPORT_KEEP_TEST_SANDBOX:-}" && -z "${AGENTPORT_TEST_SANDBOX:-}" ]]; then
    rm -rf "$SANDBOX_DIR"
  else
    echo "Keeping AgentPort test sandbox: $SANDBOX_DIR"
  fi
}
trap cleanup EXIT

mkdir -p \
  "$SANDBOX_DIR/home" \
  "$SANDBOX_DIR/config" \
  "$SANDBOX_DIR/data" \
  "$SANDBOX_DIR/cache" \
  "$SANDBOX_DIR/tmp"

export HOME="$SANDBOX_DIR/home"
export XDG_CONFIG_HOME="$SANDBOX_DIR/config"
export XDG_DATA_HOME="$SANDBOX_DIR/data"
export XDG_CACHE_HOME="$SANDBOX_DIR/cache"
export TMPDIR="$SANDBOX_DIR/tmp"
export AGENTPORT_TEST_REAL_HOME="$REAL_HOME"

unset SSH_AUTH_SOCK
unset GPG_AGENT_INFO

echo "AgentPort backend test sandbox:"
echo "  HOME=$HOME"
echo "  XDG_CONFIG_HOME=$XDG_CONFIG_HOME"
echo "  XDG_DATA_HOME=$XDG_DATA_HOME"
echo "  XDG_CACHE_HOME=$XDG_CACHE_HOME"
echo "  TMPDIR=$TMPDIR"

cargo test \
  --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" \
  --locked \
  -- \
  --test-threads=1
