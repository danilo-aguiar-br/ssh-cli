#!/usr/bin/env bash
# GAP-SSH-DEP-002 (0.3.8): prove that dependency resolve without the repo Cargo.lock
# still compiles (cargo install re-resolve path) with patched russh and stable crypto.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

# Fail if root still freezes crypto RC COMPAT pins (blocks clean resolve).
if grep -E '^(elliptic-curve|primefield|primeorder|ecdsa|pkcs5)\s*=\s*"=0\.[0-9]+\.0-rc' "$ROOT/Cargo.toml"; then
  echo "FAIL: COMPAT RC pins still present in Cargo.toml (DEP-002)" >&2
  exit 1
fi

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

echo "==> russh patched check (>= 0.60.3, prefer 0.62.x)"
if ! grep -q 'name = "russh"' "$TMP/Cargo.lock"; then
  echo "FAIL: russh missing from resolved lock" >&2
  exit 1
fi
# Extract first russh package version after name = "russh"
RUSSH_VER="$(awk '
  $0 == "name = \"russh\"" { getline; if ($1 == "version") { gsub(/"/, "", $3); print $3; exit } }
' "$TMP/Cargo.lock")"
if [[ -z "${RUSSH_VER}" ]]; then
  echo "FAIL: could not parse russh version" >&2
  exit 1
fi
echo "resolved russh = ${RUSSH_VER}"
# Reject known unpatched floor: 0.60.0, 0.60.1, 0.60.2 and anything < 0.60.3 in 0.60 line
case "${RUSSH_VER}" in
  0.60.0|0.60.1|0.60.2)
    echo "FAIL: russh ${RUSSH_VER} below security floor 0.60.3 (RUSTSEC-2026-0153/0154)" >&2
    exit 1
    ;;
  0.5*|0.4*|0.3*|0.2*|0.1*|0.0*)
    echo "FAIL: russh ${RUSSH_VER} too old" >&2
    exit 1
    ;;
esac
# Prefer 0.62+ but accept any >= 0.60.3 in 0.60.x and all 0.61+/0.62+
if [[ "${RUSSH_VER}" == 0.60.* ]]; then
  MINOR_PATCH="${RUSSH_VER#0.60.}"
  if [[ "${MINOR_PATCH}" -lt 3 ]]; then
    echo "FAIL: russh ${RUSSH_VER} < 0.60.3" >&2
    exit 1
  fi
fi

echo "==> primefield may be stable 0.14.0 (allowed in 0.3.8)"
if grep -q 'name = "primefield"' "$TMP/Cargo.lock"; then
  PF_VER="$(awk '
    $0 == "name = \"primefield\"" { getline; if ($1 == "version") { gsub(/"/, "", $3); print $3; exit } }
  ' "$TMP/Cargo.lock")"
  echo "resolved primefield = ${PF_VER:-unknown}"
fi

echo "==> cargo check"
(cd "$TMP" && cargo check -q)

echo "OK: install resolve gate passed (russh ${RUSSH_VER})"
