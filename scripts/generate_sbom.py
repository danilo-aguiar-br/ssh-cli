#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""G-22: generate a software bill of materials (SBOM) for release evidence.

Preferred tool: cargo-cyclonedx (CycloneDX JSON).
Fallback: `cargo tree --edges normal` (human inventory; NOT a signed SBOM).

Usage:
    python3 scripts/generate_sbom.py
    python3 scripts/generate_sbom.py /tmp/ssh-cli.cdx.json
    python3 scripts/generate_sbom.py --timeout-secs 900

Does NOT publish, push, or sign. Signing is a maintainer step (cosign / gpg) documented
in docs/RELEASE_CHECKLIST.md.

Contract: stdout carries only the final result line (`OK: <path> (<n> bytes)` or
`OK (fallback inventory): <path>`). Progress and notes go to stderr.
Exit 0 on success, 1 on failure, 2 on usage error.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
import time
from pathlib import Path
from typing import List, Optional, Sequence

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

DEFAULT_TIMEOUT_SECS = 600
PROBE_TIMEOUT_SECS = 30.0
SEARCH_DEPTH = 3


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="generate_sbom.py",
        description="Generate a CycloneDX SBOM (or a cargo tree inventory as fallback).",
    )
    parser.add_argument(
        "out",
        nargs="?",
        type=Path,
        default=None,
        help="output file (default: <repo>/target/sbom/ssh-cli.cdx.json)",
    )
    parser.add_argument(
        "--timeout-secs",
        type=gk.non_negative_int,
        default=DEFAULT_TIMEOUT_SECS,
        help=f"budget per cargo invocation in seconds, 0 for none (default {DEFAULT_TIMEOUT_SECS})",
    )
    parser.add_argument(
        "--quiet", action="store_true", help="suppress progress lines on stderr"
    )
    return parser


def cyclonedx_available() -> bool:
    if gk.which("cargo-cyclonedx") is not None:
        return True
    probe = gk.run_tree(["cargo", "cyclonedx", "-h"], timeout=PROBE_TIMEOUT_SECS)
    return probe.code == 0


def fresh_bom_files(root: Path, since: float, skip: Path) -> List[Path]:
    """BOM files written at or after ``since`` within SEARCH_DEPTH levels of ``root``.

    Older cargo-cyclonedx releases ignore --output-cdx and write next to the package.
    Only files touched by THIS run count, so a stale BOM from an earlier run is never
    mistaken for the output.
    """
    found: List[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        depth = len(Path(dirpath).relative_to(root).parts)
        if depth + 1 >= SEARCH_DEPTH:
            # Files one level deeper than this directory would exceed the depth cap.
            dirnames[:] = []
        for name in filenames:
            if not (name.endswith(".cdx.json") or name == "bom.xml"):
                continue
            path = Path(dirpath) / name
            if path != skip and path.stat().st_mtime >= since:
                found.append(path)
    return sorted(found, key=lambda p: p.stat().st_mtime, reverse=True)


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    def note(message: str) -> None:
        if not args.quiet:
            gk.eprint(message)

    root = gk.repo_root()
    out: Path = (
        args.out.resolve()
        if args.out is not None
        else root / "target" / "sbom" / "ssh-cli.cdx.json"
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    timeout = gk.timeout_or_none(args.timeout_secs)

    if cyclonedx_available():
        note(f"G-22: generating CycloneDX SBOM via cargo-cyclonedx -> {out}")
        started = time.time() - 1.0
        first = gk.run_tree(
            ["cargo", "cyclonedx", "--format", "json", "--output-cdx", str(out)],
            timeout=timeout,
            cwd=str(root),
        )
        if first.timed_out:
            gk.eprint(f"cargo-cyclonedx timed out after {args.timeout_secs}s")
            return 1
        if first.code != 0:
            # Older CLI: it has no --output-cdx, so run it plain and collect its file.
            second = gk.run_tree(
                ["cargo", "cyclonedx", "--format", "json"],
                timeout=timeout,
                cwd=str(root),
            )
            if second.code != 0:
                gk.eprint("cargo-cyclonedx failed")
                gk.eprint(gk.tail_lines(second.stderr or first.stderr, 20))
                return 1
            produced = fresh_bom_files(root, started, out)
            if not produced:
                gk.eprint("cargo-cyclonedx ran but no output file found; see tool docs")
                return 1
            shutil.copyfile(produced[0], out)
        if not out.is_file():
            gk.eprint(f"cargo-cyclonedx reported success but {out} does not exist")
            return 1
        sys.stdout.write(f"OK: {out} ({out.stat().st_size} bytes)\n")
        return 0

    text = str(out)
    stem = text[: -len(".json")] if text.endswith(".json") else text
    tree_out = Path(stem + ".tree.txt")
    gk.eprint(f"G-22: cargo-cyclonedx not installed; writing cargo tree inventory to {tree_out}")
    gk.eprint("Install: cargo install cargo-cyclonedx")
    tree = gk.run_tree(
        ["cargo", "tree", "--edges", "normal", "--prefix", "none"],
        timeout=timeout,
        cwd=str(root),
    )
    if tree.timed_out:
        gk.eprint(f"cargo tree timed out after {args.timeout_secs}s")
        return 1
    if tree.code != 0:
        gk.eprint(gk.tail_lines(tree.stderr, 20))
        gk.eprint(f"cargo tree exited {tree.code}")
        return 1
    gk.write_text(tree_out, tree.stdout)
    sys.stdout.write(f"OK (fallback inventory): {tree_out}\n")
    gk.eprint("NOTE: fallback is NOT a CycloneDX SBOM - install cargo-cyclonedx for release.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
