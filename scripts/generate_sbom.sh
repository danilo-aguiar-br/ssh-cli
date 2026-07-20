#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# G-22 — generate a software bill of materials (SBOM) for release evidence.
#
# Preferred tool: cargo-cyclonedx (CycloneDX JSON).
# Fallback: cargo tree --edges normal (human inventory; not a signed SBOM).
#
# Usage:
#   bash scripts/generate_sbom.sh
#   bash scripts/generate_sbom.sh /tmp/ssh-cli.cdx.json
#
# Does NOT publish, push, or sign. Signing is a maintainer step (cosign / gpg)
# documented in docs/RELEASE_CHECKLIST.md.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="${1:-"$ROOT/target/sbom/ssh-cli.cdx.json"}"
mkdir -p "$(dirname "$OUT")"

if command -v cargo-cyclonedx >/dev/null 2>&1 || cargo cyclonedx -h >/dev/null 2>&1; then
  echo "G-22: generating CycloneDX SBOM via cargo-cyclonedx → $OUT"
  # cargo-cyclonedx writes next to the package by default; force path when supported.
  if cargo cyclonedx --format json --output-cdx "$OUT" 2>/dev/null; then
    :
  elif cargo cyclonedx --format json 2>/dev/null; then
    # Older CLI: move generated file if present
    found="$(find . -maxdepth 3 -name '*.cdx.json' -o -name 'bom.xml' 2>/dev/null | head -1 || true)"
    if [[ -n "${found:-}" && -f "$found" ]]; then
      cp -f "$found" "$OUT"
    else
      echo "cargo-cyclonedx ran but no output file found; see tool docs" >&2
      exit 1
    fi
  else
    echo "cargo-cyclonedx failed" >&2
    exit 1
  fi
  echo "OK: $OUT ($(wc -c <"$OUT") bytes)"
  exit 0
fi

echo "G-22: cargo-cyclonedx not installed; writing cargo tree inventory to ${OUT%.json}.tree.txt" >&2
echo "Install: cargo install cargo-cyclonedx" >&2
TREE_OUT="${OUT%.json}.tree.txt"
cargo tree --edges normal --prefix none >"$TREE_OUT"
echo "OK (fallback inventory): $TREE_OUT"
echo "NOTE: fallback is NOT a CycloneDX SBOM — install cargo-cyclonedx for release."
exit 0
