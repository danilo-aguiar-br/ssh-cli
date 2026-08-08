#!/usr/bin/env bash
# Assert the RustSec advisory database is fresh before trusting `cargo deny`.
#
# D9: `cargo deny check advisories` returns `advisories ok` regardless of how old
# the local database is. The result is only as good as the clone behind it, and
# nothing in the gate says how old that clone was. This host carried two clones a
# week apart (`~/.cargo/advisory-db` at 2026-07-30, `~/.cargo/advisory-dbs` at
# 2026-08-06), and the gate never named which one it read.
#
# `deny.toml` already records the lesson in prose — "cargo deny check advisories
# verde não prova ausência de CVE conhecido" — after A1 shipped a vulnerable russh
# under a green gate. This script puts that lesson in code: a green advisories run
# now carries a proven upper bound on the database age.
#
# Local gate. No CI, no product env store, no network beyond the clone cargo-deny
# already maintains.
set -euo pipefail

MAX_AGE_DAYS="${1:-7}"

if ! [[ "$MAX_AGE_DAYS" =~ ^[0-9]+$ ]]; then
  echo "usage: check_advisory_freshness.sh [MAX_AGE_DAYS]" >&2
  exit 2
fi

# cargo-deny keeps its clones under `advisory-dbs` (plural). The singular
# `advisory-db` is cargo-audit's and must NOT be read here: trusting it is how a
# stale clone silently vouches for a fresh run.
DB_ROOT="${CARGO_HOME:-$HOME/.cargo}/advisory-dbs"

if [[ ! -d "$DB_ROOT" ]]; then
  echo "GAP: advisory database absent at $DB_ROOT — run 'cargo deny check advisories' once to populate it" >&2
  exit 1
fi

FOUND=0
STALEST_DAYS=-1
STALEST_PATH=""

while IFS= read -r db; do
  [[ -d "$db/.git" ]] || continue
  FOUND=1
  COMMIT_EPOCH="$(git -C "$db" log -1 --format=%ct 2>/dev/null || echo 0)"
  if [[ "$COMMIT_EPOCH" -eq 0 ]]; then
    echo "GAP: cannot read HEAD date of advisory database $db" >&2
    exit 1
  fi
  NOW_EPOCH="$(date +%s)"
  AGE_DAYS=$(( (NOW_EPOCH - COMMIT_EPOCH) / 86400 ))
  if [[ "$AGE_DAYS" -gt "$STALEST_DAYS" ]]; then
    STALEST_DAYS="$AGE_DAYS"
    STALEST_PATH="$db"
  fi
done < <(fd -H -t d -d 1 . "$DB_ROOT" 2>/dev/null || true)

if [[ "$FOUND" -eq 0 ]]; then
  echo "GAP: no git-backed advisory database under $DB_ROOT" >&2
  exit 1
fi

if [[ "$STALEST_DAYS" -gt "$MAX_AGE_DAYS" ]]; then
  echo "GAP: advisory database is ${STALEST_DAYS}d old (max ${MAX_AGE_DAYS}d): $STALEST_PATH" >&2
  echo "     A green 'cargo deny check advisories' over a stale database proves nothing." >&2
  echo "     Refresh with: cargo deny fetch advisories" >&2
  exit 1
fi

echo "Advisory freshness gate: OK (${STALEST_DAYS}d old, max ${MAX_AGE_DAYS}d)"
