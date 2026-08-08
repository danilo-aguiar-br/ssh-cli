#!/usr/bin/env bash
# B1 gate: type-check the targets this CLI claims to support.
#
# Why this exists
# ---------------
# `cargo fmt`, `cargo clippy`, `cargo test` and `cargo deny` all run for the
# host triple. Code behind `#[cfg(target_os = "windows")]` is discarded by cfg
# expansion *before* type-check on a Linux host, so a Windows-only compile error
# can sit in the tree while every gate reports green. That is exactly how
# `src/platform/windows.rs` reached a state where six errors blocked the whole
# target while `docs/CROSS_PLATFORM.md` still advertised "Supported".
#
# A green gate only proves what the gate COMPILES.
#
# Windows uses `--no-default-features` on purpose: the default TLS stack pulls
# `aws-lc-sys`, which compiles C and needs a cross toolchain this host does not
# ship (recorded as A8, structural, mandated by G-TLS-02). Dropping default
# features still type-checks 100% of the product's own `cfg(windows)` code,
# which is what this gate is for.
#
# Local only. No CI, no network beyond `rustup target add`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Per-target check budget in whole seconds (the Rust `timeout` binary rejects
# suffixes such as `5m`).
CHECK_TIMEOUT_SECS="${CHECK_TIMEOUT_SECS:-600}"

# triple:extra-cargo-flags
TARGETS=(
  "x86_64-pc-windows-msvc:--no-default-features"
  "aarch64-pc-windows-msvc:--no-default-features"
  "x86_64-apple-darwin:--no-default-features"
)

failed=()

for entry in "${TARGETS[@]}"; do
  triple="${entry%%:*}"
  flags="${entry#*:}"

  if ! rustup target list --installed | rg -q "^${triple}$"; then
    echo "==> installing target ${triple}" >&2
    if ! rustup target add "${triple}" >&2; then
      echo "FAIL ${triple}: rustup target add failed" >&2
      failed+=("${triple} (target unavailable)")
      continue
    fi
  fi

  echo "==> cargo check --target ${triple} ${flags}" >&2
  # shellcheck disable=SC2086
  if timeout "${CHECK_TIMEOUT_SECS}" cargo check --target "${triple}" ${flags} >&2; then
    echo "OK   ${triple}" >&2
  else
    rc=$?
    if [ "${rc}" -eq 124 ]; then
      echo "FAIL ${triple}: timed out after ${CHECK_TIMEOUT_SECS}s" >&2
      failed+=("${triple} (timeout)")
    else
      echo "FAIL ${triple}: cargo check exited ${rc}" >&2
      failed+=("${triple} (exit ${rc})")
    fi
  fi
done

if [ "${#failed[@]}" -ne 0 ]; then
  echo >&2
  echo "cross-target gate FAILED for: ${failed[*]}" >&2
  echo "docs/CROSS_PLATFORM.md must not claim support for a target that does not compile." >&2
  exit 1
fi

echo "cross-target gate OK for ${#TARGETS[@]} targets" >&2
