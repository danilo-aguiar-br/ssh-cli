#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Assert `cargo audit` finds zero RUSTSEC advisories against this lockfile.

A4: `check_advisory_freshness.py` proves the database is current, and
`cargo deny check advisories` reads a database of its own. Neither one states the
intersection that matters: which advisories actually hit the crates in `Cargo.lock`.
The two databases are separate clones, so a fresh one can vouch for a stale one.

This gate closes that hole. It runs `cargo audit` against the live lockfile and
requires `vulnerabilities.count == 0`. The freshness precondition is enforced by
reusing `check_advisory_freshness.py` instead of restating its logic, so a stale
database fails here too rather than reporting a clean intersection nobody can trust.

Local gate. No CI, no product env store, no network beyond the clone cargo-audit
already maintains (the run passes `--no-fetch`, so the gate never mutates the clone).

Contract: stdout carries only the final OK line. Everything else goes to stderr.
Exit 0 clean, 1 advisory found or audit failed, 2 usage error.
"""

from __future__ import annotations

import argparse
import contextlib
import json
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Sequence

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402
import check_advisory_freshness as freshness  # noqa: E402

AUDIT_TIMEOUT_SECS = 120.0
TEXT_LIMIT = 300


def _one_line(value: Any) -> str:
    """Collapse an advisory string to a single readable line."""
    text = " ".join(str(value).split())
    if len(text) > TEXT_LIMIT:
        return text[:TEXT_LIMIT] + "..."
    return text


def _describe(entry: Dict[str, Any]) -> str:
    """One stderr line naming package, version, advisory id and the fixed version."""
    advisory = entry.get("advisory") or {}
    package = entry.get("package") or {}
    # `cargo audit --json` puts the fixed range on the entry (`versions.patched`), not
    # inside `advisory`; reading `advisory.patched_versions` printed "no patched
    # release" for RUSTSEC-2026-0492 while `>=0.63.1` existed.
    versions = entry.get("versions") or {}
    patched = ", ".join(versions.get("patched") or []) or "no patched release"
    return (
        f"GAP: {package.get('name', '?')} {package.get('version', '?')} "
        f"{advisory.get('id', '?')} - patched in: {patched}"
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="check_advisory_hits.py",
        description="Fail when cargo audit reports any advisory against Cargo.lock.",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="crate directory holding Cargo.toml and Cargo.lock (default: repository root)",
    )
    parser.add_argument(
        "--max-age-days",
        type=gk.non_negative_int,
        default=freshness.DEFAULT_MAX_AGE_DAYS,
        metavar="MAX_AGE_DAYS",
        help=f"maximum accepted advisory database age in days (default {freshness.DEFAULT_MAX_AGE_DAYS})",
    )
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    root = args.manifest if args.manifest is not None else gk.repo_root()

    # The intersection is only meaningful over a current database, so reuse the
    # freshness gate rather than duplicating its rules. Its OK line describes the
    # precondition, not this gate's result, so it goes to stderr and keeps stdout
    # carrying exactly one line.
    with contextlib.redirect_stdout(sys.stderr):
        fresh = freshness.main([str(args.max_age_days)])
    if fresh != 0:
        gk.eprint("     The advisory intersection would have been measured against a stale clone.")
        return 1

    cargo = gk.which("cargo")
    if cargo is None:
        gk.eprint("GAP: cargo not found on PATH")
        return 1

    result = gk.run_tree(
        [cargo, "audit", "--json", "--no-fetch", "--file", str(root / "Cargo.lock")],
        timeout=AUDIT_TIMEOUT_SECS,
        cwd=str(root),
    )
    if result.timed_out:
        gk.eprint(f"GAP: cargo audit exceeded {AUDIT_TIMEOUT_SECS:.0f}s")
        return 1

    payload: Optional[Dict[str, Any]] = None
    if result.stdout.strip():
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as err:
            gk.eprint(f"GAP: cargo audit printed unparseable JSON: {err}")
            return 1

    if payload is None:
        gk.eprint("GAP: cargo audit produced no JSON on stdout")
        if result.code == 127:
            gk.eprint("     cargo-audit is not installed. Install with: cargo install cargo-audit --locked")
        for line in gk.tail_lines(result.stderr, 5).splitlines():
            gk.eprint(f"     {line}")
        return 1

    violations = payload.get("vulnerabilities") or {}
    entries = violations.get("list") or []
    count = int(violations.get("count") or len(entries))
    if count or entries:
        gk.eprint(f"GAP: cargo audit found {max(count, len(entries))} advisory hit(s) against Cargo.lock")
        for entry in entries:
            gk.eprint("     " + _describe(entry))
            detail = _one_line((entry.get("advisory") or {}).get("title", ""))
            if detail:
                gk.eprint(f"     {detail}")
        gk.eprint("     Fix by raising the pin in Cargo.toml, never by adding an ignore in deny.toml.")
        return 1

    # Warnings are not failures by default, but yanked crates are: `deny.toml` sets
    # `yanked = "deny"`, and this gate must not be softer than the policy next door.
    yanked = (payload.get("warnings") or {}).get("yanked") or []
    if yanked:
        gk.eprint(f"GAP: cargo audit found {len(yanked)} yanked crate(s) in Cargo.lock")
        for entry in yanked:
            package = entry.get("package") or entry
            gk.eprint(
                f"     {package.get('name', '?')} {package.get('version', '?')} "
                f"{_one_line(entry.get('advisory', ''))}"
            )
        return 1

    advisory_count = (payload.get("database") or {}).get("advisory-count")
    sys.stdout.write(
        f"Advisory hits gate: OK (0 hits, {advisory_count if advisory_count is not None else '?'} advisories checked)\n"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
