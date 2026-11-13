#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Assert the RustSec advisory database is fresh before trusting `cargo deny`.

D9: `cargo deny check advisories` returns `advisories ok` regardless of how old the
local database is. The result is only as good as the clone behind it, and nothing in
the gate says how old that clone was. This host carried two clones a week apart
(`~/.cargo/advisory-db` and `~/.cargo/advisory-dbs`), and the gate never named which
one it read.

`deny.toml` already records the lesson in prose: a green `cargo deny check advisories`
does not prove the absence of a known CVE, after A1 shipped a vulnerable russh under a
green gate. This script puts that lesson in code: a green advisories run now carries a
proven upper bound on the database age.

Local gate. No CI, no product env store, no network beyond the clone cargo-deny
already maintains.

Contract: stdout carries only the final OK line. Everything else goes to stderr.
Exit 0 fresh, 1 stale or unreadable, 2 usage error.
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path
from typing import List, Optional, Sequence

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

# Declared exception (operational staleness budget, not a quality score): RustSec ships
# new advisories without a schedule, so no source publishes a freshness reference; a week
# bounds how stale a "no hits" verdict may be, and the MAX_AGE_DAYS argument overrides it.
DEFAULT_MAX_AGE_DAYS = 7
GIT_TIMEOUT_SECS = 30.0


def default_db_root() -> Path:
    """cargo-deny keeps its clones under `advisory-dbs` (plural).

    The singular `advisory-db` is cargo-audit's and must NOT be read here: trusting it
    is how a stale clone silently vouches for a fresh run.
    """
    return Path.home() / ".cargo" / "advisory-dbs"


def head_commit_epoch(db: Path, git: str) -> int:
    """Committer date of the clone's HEAD in epoch seconds, or 0 when unreadable."""
    result = gk.run_tree(
        [git, "-C", str(db), "log", "-1", "--format=%ct"], timeout=GIT_TIMEOUT_SECS
    )
    if result.code != 0:
        return 0
    try:
        return int(result.stdout.strip())
    except ValueError:
        return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="check_advisory_freshness.py",
        description="Fail when the RustSec advisory database clone is older than MAX_AGE_DAYS.",
    )
    parser.add_argument(
        "max_age_days",
        nargs="?",
        type=gk.non_negative_int,
        default=DEFAULT_MAX_AGE_DAYS,
        metavar="MAX_AGE_DAYS",
        help=f"maximum accepted age in days (default {DEFAULT_MAX_AGE_DAYS})",
    )
    parser.add_argument(
        "--db-root",
        type=Path,
        default=None,
        help="directory holding the cargo-deny clones (default: ~/.cargo/advisory-dbs)",
    )
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    max_age = args.max_age_days
    db_root: Path = args.db_root if args.db_root is not None else default_db_root()

    if not db_root.is_dir():
        gk.eprint(
            f"GAP: advisory database absent at {db_root} - run "
            "'cargo deny check advisories' once to populate it"
        )
        return 1

    clones: List[Path] = sorted(
        p for p in db_root.iterdir() if p.is_dir() and (p / ".git").is_dir()
    )
    if not clones:
        gk.eprint(f"GAP: no git-backed advisory database under {db_root}")
        return 1

    git = gk.which("git")
    if git is None:
        gk.eprint("GAP: git not found on PATH, so the HEAD date of the advisory database cannot be read")
        return 1

    stalest_days = -1
    stalest_path: Optional[Path] = None
    now = int(time.time())
    for db in clones:
        commit_epoch = head_commit_epoch(db, git)
        if commit_epoch == 0:
            gk.eprint(f"GAP: cannot read HEAD date of advisory database {db}")
            return 1
        age_days = (now - commit_epoch) // 86400
        if age_days > stalest_days:
            stalest_days = age_days
            stalest_path = db

    if stalest_days > max_age:
        gk.eprint(
            f"GAP: advisory database is {stalest_days}d old (max {max_age}d): {stalest_path}"
        )
        gk.eprint("     A green 'cargo deny check advisories' over a stale database proves nothing.")
        gk.eprint("     Refresh with: cargo deny fetch advisories")
        return 1

    sys.stdout.write(
        f"Advisory freshness gate: OK ({stalest_days}d old, max {max_age}d)\n"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
