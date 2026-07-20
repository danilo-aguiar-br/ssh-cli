#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# G-22 — multi-arch release build matrix via cross-rs (see Cross.toml).
#
# Builds release binaries for the documented target list. Does NOT push, sign,
# or upload. Maintainer attaches artifacts to a GitHub Release after local tag.
#
# Usage:
#   bash scripts/dist_multiarch.sh
#   TARGETS="x86_64-unknown-linux-musl" bash scripts/dist_multiarch.sh
#
# Requires: cargo, docker (for cross), optional `cross` binary
#   cargo install cross --git https://github.com/cross-rs/cross

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGETS="${TARGETS:-x86_64-unknown-linux-musl aarch64-unknown-linux-musl aarch64-unknown-linux-gnu}"
OUT_DIR="${OUT_DIR:-$ROOT/target/dist}"
# G-PAR-46: bound parallel builds (default 2 — Docker/cross is heavy).
# Override: PARALLEL_JOBS=1 bash scripts/dist_multiarch.sh  (serial)
PARALLEL_JOBS="${PARALLEL_JOBS:-2}"
mkdir -p "$OUT_DIR"

run_build() {
  local target="$1"
  echo "=== building --release --target $target ==="
  if command -v cross >/dev/null 2>&1; then
    cross build --release --locked --target "$target"
  else
    echo "cross not installed; trying cargo (needs local linker/sysroot)" >&2
    cargo build --release --locked --target "$target"
  fi
  local bin="target/${target}/release/ssh-cli"
  if [[ -f "$bin" ]]; then
    local dest="$OUT_DIR/ssh-cli-${target}"
    cp -f "$bin" "$dest"
    # Strip only when strip tool exists (musl static often already stripped by profile).
    if command -v strip >/dev/null 2>&1 && [[ "$target" == *linux* ]]; then
      strip "$dest" 2>/dev/null || true
    fi
    (cd "$OUT_DIR" && sha256sum "ssh-cli-${target}" >"ssh-cli-${target}.sha256")
    echo "OK: $dest"
  else
    echo "missing binary: $bin" >&2
    return 1
  fi
}

export -f run_build
export OUT_DIR ROOT

# shellcheck disable=SC2086
printf '%s\n' $TARGETS | xargs -P "$PARALLEL_JOBS" -I{} bash -c 'run_build "$@"' _ {}

echo "G-22 multi-arch artifacts under $OUT_DIR (PARALLEL_JOBS=$PARALLEL_JOBS):"
ls -la "$OUT_DIR"
