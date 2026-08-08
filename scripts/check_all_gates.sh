#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Runs the whole mandatory gate battery in one invocation.
#
# Why this exists
# ---------------
# Every gate in this repository was individually well designed, and the 0.5.4 audit
# still found four red ones plus a test target that did not compile — while the local
# inventory declared 835 green and zero red. The gates had not regressed; they had
# never been RUN together. `cargo clippy` and `cargo test` both abort on the first
# unbuildable target, so one broken test file silently hid the state of every other
# gate behind it.
#
# The lesson is not "add more gates". It is that a gate nobody executes is not a gate.
# This script is the missing step: one command, every gate, all results, non-zero exit
# if any of them is red.
#
# NOT CI. There is no workflow, no runner, no scheduler and no remote push. This is a
# local script a maintainer invokes by hand, in the same family as the other
# `scripts/check_*.sh` in this repository.
#
# Contract (agent-native)
# -----------------------
#   stdout : one TSV record per gate (`id  status  exit  ms`), then a summary record.
#            With --json, one NDJSON object per gate instead. Nothing else. Ever.
#   stderr : progress, per-gate log paths, diagnostics.
#   exit   : 0 when every gate passed, 1 when any failed, 2 on usage error.
#
# Declared non-gates
# ------------------
# A battery that presents itself as complete and omits scripts in silence reads as
# total coverage. These `scripts/*.sh` are deliberately NOT gates here, and each one
# is named so the omission is a declaration rather than an oversight:
#
#   dist_multiarch.sh        release artefact build; needs cross images, not a local gate
#   generate_sbom.sh         release inventory generation; produces artefacts, asserts nothing
#   release_attest.sh        release attestation; runs once per published tag
#   e2e_real_ssh.sh          real-SSH end-to-end harness; needs a lab host or local sshd
#
# `tests/gaps_v064_gate_runner.rs` enforces this list: every `scripts/*.sh` must be
# either a gate below or named in this block, so a new script cannot slip in silently.
#
# Sequential by design
# --------------------
# Six of the ten gates invoke cargo (`fmt`, `build-release`, `build-no-default`,
# `clippy`, `test`, `deny`, plus `install-resolve` and `cross-targets` indirectly).
# Concurrent cargo invocations block on the same `target/` directory lock, so running
# them in parallel would buy no wall-clock and would interleave the per-gate report.
# The sequence is the correct design here, not a missing optimisation.
#
# By default every gate runs even after a failure, because the point is to see the
# whole board. --fail-fast stops at the first red when you only want the next thing
# to fix.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/Cargo.toml"
FAIL_FAST=0
AS_JSON=0
ONLY=""
LOG_DIR=""

usage() {
  cat <<'EOF'
Usage: check_all_gates.sh [options]
  --fail-fast        stop at the first red gate (default: run all, report all)
  --json             emit NDJSON on stdout instead of TSV
  --only ID[,ID...]  run only the named gates (see --list)
  --log-dir DIR      keep per-gate output here (default: a fresh mktemp -d)
  --list             print gate ids and exit
  -h, --help         this text
Exit: 0 all green, 1 at least one red, 2 usage error.
EOF
}

# id|description|command
# Order matters: fmt and the build come first because they are the cheapest signals,
# and because a tree that does not compile makes every later gate meaningless.
GATES=(
  "fmt|rustfmt is clean|cargo fmt --manifest-path '$MANIFEST' --all --check"
  "build-release|release build succeeds|cargo build --manifest-path '$MANIFEST' --release"
  "build-no-default|builds without default features (Windows path, A8)|cargo build --manifest-path '$MANIFEST' --no-default-features"
  "clippy|clippy clean with warnings denied|cargo clippy --manifest-path '$MANIFEST' --all-targets --all-features -- -D warnings"
  "test|full test suite green|cargo test --manifest-path '$MANIFEST' --locked --all-features"
  "deny|supply chain clean|cargo deny --manifest-path '$MANIFEST' check"
  "cross-targets|windows and macos type-check (B1)|bash '$ROOT/scripts/check_cross_targets.sh'"
  "advisory-freshness|advisory database is current|bash '$ROOT/scripts/check_advisory_freshness.sh'"
  "en-identifiers|identifiers are English|bash '$ROOT/scripts/check_en_identifiers.sh'"
  "install-resolve|cargo install --locked resolves|bash '$ROOT/scripts/verify_install_resolve.sh'"
)

gate_ids() {
  local g
  for g in "${GATES[@]}"; do printf '%s\n' "${g%%|*}"; done
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fail-fast) FAIL_FAST=1; shift ;;
    --json) AS_JSON=1; shift ;;
    --only) ONLY="${2:-}"; shift 2 ;;
    --log-dir) LOG_DIR="${2:-}"; shift 2 ;;
    --list) gate_ids; exit 0 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -n "$ONLY" ]]; then
  for want in ${ONLY//,/ }; do
    gate_ids | rg -qx -- "$want" || { echo "unknown gate id: $want" >&2; exit 2; }
  done
fi

if [[ -z "$LOG_DIR" ]]; then
  LOG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ssh-cli-gates.XXXXXX")"
fi
mkdir -p "$LOG_DIR"
echo "gate logs: $LOG_DIR" >&2

TOTAL=0
FAILED=0
SKIPPED=0

for entry in "${GATES[@]}"; do
  id="${entry%%|*}"
  rest="${entry#*|}"
  desc="${rest%%|*}"
  cmd="${rest#*|}"

  if [[ -n "$ONLY" ]] && ! printf '%s' ",$ONLY," | rg -q ",$id,"; then
    SKIPPED=$((SKIPPED + 1))
    continue
  fi

  TOTAL=$((TOTAL + 1))
  log="$LOG_DIR/$id.log"
  echo "==> $id: $desc" >&2

  start_ns="$(date +%s%N)"
  # Gate output goes to its log, never to stdout: stdout is the machine contract.
  eval "$cmd" >"$log" 2>&1
  code=$?
  ms=$(( ( $(date +%s%N) - start_ns ) / 1000000 ))

  if [[ $code -eq 0 ]]; then
    status=pass
  else
    status=fail
    FAILED=$((FAILED + 1))
    # Surface the tail on stderr so the operator sees why without opening the log.
    echo "--- $id FAILED (exit $code); last 20 lines of $log ---" >&2
    tail -20 "$log" >&2
  fi

  if [[ $AS_JSON -eq 1 ]]; then
    printf '{"gate":"%s","status":"%s","exit":%d,"duration_ms":%d,"log":"%s"}\n' \
      "$id" "$status" "$code" "$ms" "$log"
  else
    printf '%s\t%s\t%d\t%d\n' "$id" "$status" "$code" "$ms"
  fi

  if [[ $status == fail && $FAIL_FAST -eq 1 ]]; then
    echo "--fail-fast: stopping at $id" >&2
    break
  fi
done

if [[ $AS_JSON -eq 1 ]]; then
  printf '{"gate":"_summary","total":%d,"failed":%d,"skipped":%d,"log_dir":"%s"}\n' \
    "$TOTAL" "$FAILED" "$SKIPPED" "$LOG_DIR"
else
  printf '_summary\ttotal=%d\tfailed=%d\tskipped=%d\n' "$TOTAL" "$FAILED" "$SKIPPED"
fi

[[ $FAILED -eq 0 ]] || exit 1
exit 0
