#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""B1 gate: type-check the targets this CLI claims to support.

Why this exists
---------------
`cargo fmt`, `cargo clippy`, `cargo test` and `cargo deny` all run for the host triple.
Code behind `#[cfg(target_os = "windows")]` is discarded by cfg expansion *before*
type-check on a Linux host, so a Windows-only compile error can sit in the tree while
every gate reports green. That is exactly how `src/platform/windows.rs` reached a state
where six errors blocked the whole target while `docs/CROSS_PLATFORM.md` still
advertised "Supported".

A green gate only proves what the gate COMPILES.

Every target below uses `--no-default-features` on purpose: the default TLS stack pulls
`aws-lc-sys`, which compiles C and needs a cross toolchain this host does not ship
(recorded as A8, structural, mandated by G-TLS-02). Dropping default features still
type-checks 100% of the product's own `cfg(windows)` code, which is what this gate is
for.

Local only. No CI, no network beyond `rustup target add`.

Contract: stdout is empty. Progress and cargo output go to stderr.
Exit 0 when every target type-checks, 1 when any fails, 2 on usage error.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import List, Optional, Sequence, Tuple

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

# (triple, extra cargo flags)
TARGETS: List[Tuple[str, List[str]]] = [
    ("x86_64-pc-windows-msvc", ["--no-default-features"]),
    ("aarch64-pc-windows-msvc", ["--no-default-features"]),
    ("x86_64-apple-darwin", ["--no-default-features"]),
]

DEFAULT_TIMEOUT_SECS = 600
RUSTUP_TIMEOUT_SECS = 600.0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="check_cross_targets.py",
        description="Type-check the Windows and macOS targets with cargo check.",
    )
    parser.add_argument(
        "--timeout-secs",
        type=gk.non_negative_int,
        default=DEFAULT_TIMEOUT_SECS,
        help=f"per-target cargo check budget in seconds, 0 for none (default {DEFAULT_TIMEOUT_SECS})",
    )
    parser.add_argument(
        "--quiet", action="store_true", help="suppress progress lines on stderr"
    )
    return parser


def installed_targets(rustup: str) -> Optional[List[str]]:
    result = gk.run_tree([rustup, "target", "list", "--installed"], timeout=60.0)
    if result.code != 0:
        return None
    return [line.strip() for line in result.stdout.splitlines()]


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    def note(message: str) -> None:
        if not args.quiet:
            gk.eprint(message)

    rustup = gk.which("rustup")
    if rustup is None:
        gk.eprint("FAIL: rustup not found on PATH; it is needed to install the cross targets")
        return 1

    root = str(gk.repo_root())
    timeout = gk.timeout_or_none(args.timeout_secs)
    installed = installed_targets(rustup)
    if installed is None:
        gk.eprint("FAIL: 'rustup target list --installed' failed")
        return 1

    failed: List[str] = []
    for triple, flags in TARGETS:
        if triple not in installed:
            note(f"==> installing target {triple}")
            added = gk.run_tree(
                [rustup, "target", "add", triple],
                timeout=RUSTUP_TIMEOUT_SECS,
                capture=False,
            )
            if added.code != 0:
                gk.eprint(f"FAIL {triple}: rustup target add failed")
                failed.append(f"{triple} (target unavailable)")
                continue

        note(f"==> cargo check --target {triple} {' '.join(flags)}")
        result = gk.run_tree(
            ["cargo", "check", "--target", triple, *flags],
            timeout=timeout,
            cwd=root,
            capture=False,
        )
        if result.timed_out:
            gk.eprint(f"FAIL {triple}: timed out after {args.timeout_secs}s")
            failed.append(f"{triple} (timeout)")
        elif result.code != 0:
            gk.eprint(f"FAIL {triple}: cargo check exited {result.code}")
            failed.append(f"{triple} (exit {result.code})")
        else:
            note(f"OK   {triple}")

    if failed:
        gk.eprint("")
        gk.eprint(f"cross-target gate FAILED for: {', '.join(failed)}")
        gk.eprint(
            "docs/CROSS_PLATFORM.md must not claim support for a target that does not compile."
        )
        return 1

    note(f"cross-target gate OK for {len(TARGETS)} targets")
    return 0


if __name__ == "__main__":
    sys.exit(main())
