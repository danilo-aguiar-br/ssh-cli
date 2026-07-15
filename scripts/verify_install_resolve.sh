#!/usr/bin/env bash
# Gate GAP-014: prove that dependency resolve without the repo Cargo.lock
# still compiles (cargo install re-resolve path).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

mkdir -p "$TMP/src"
cat >"$TMP/Cargo.toml" <<TOML
[package]
name = "install_gate"
version = "0.0.0"
edition = "2021"

[dependencies]
ssh-cli = { path = "$ROOT" }
TOML
echo 'fn main() {}' >"$TMP/src/main.rs"

echo "==> generate-lockfile (re-resolve like cargo install)"
(cd "$TMP" && cargo generate-lockfile)

echo "==> primefield pin check"
if ! grep -q 'name = "primefield"' "$TMP/Cargo.lock"; then
  echo "primefield missing" >&2
  exit 1
fi
# Must not be stable 0.14.0 without rc
if grep -A2 'name = "primefield"' "$TMP/Cargo.lock" | grep -q 'version = "0.14.0"'; then
  echo "FAIL: primefield stable 0.14.0 resolved" >&2
  exit 1
fi

echo "==> cargo check"
(cd "$TMP" && cargo check -q)

echo "OK: install resolve gate passed"
