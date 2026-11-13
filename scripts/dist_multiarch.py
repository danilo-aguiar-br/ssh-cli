#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""G-22: multi-arch release build matrix via cross-rs (see Cross.toml).

Builds release binaries for the documented target list. Does NOT push, sign, or upload.
The maintainer attaches the artifacts to a release after the local tag.

Usage:
    python3 scripts/dist_multiarch.py
    python3 scripts/dist_multiarch.py --targets x86_64-unknown-linux-musl
    python3 scripts/dist_multiarch.py --jobs 1        # serial

Requires: cargo, docker (for cross), optional `cross` binary:
    cargo install cross --git https://github.com/cross-rs/cross

G-PAR-46: parallel builds are bounded by --jobs (default 2, because Docker/cross is
heavy). Each target builds in its own worker thread and its output is printed to stderr
as one block when that target finishes, so concurrent builds never interleave.

Contract: stdout carries one TSV record per target (`target  status  artifact`).
Progress and build output go to stderr. Exit 0 when every target built, 1 otherwise,
2 on usage error.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import List, NamedTuple, Optional, Sequence

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

DEFAULT_TARGETS = [
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
]
DEFAULT_JOBS = 2
DEFAULT_TIMEOUT_SECS = 3600
STRIP_TIMEOUT_SECS = 60.0
BIN_NAME = "ssh-cli"
TRIPLE_RE = re.compile(r"^[A-Za-z0-9_][A-Za-z0-9_.-]*$")


class BuildOutcome(NamedTuple):
    target: str
    ok: bool
    artifact: str
    log: str


def parse_targets(text: str) -> List[str]:
    """Split a space- or comma-separated list, rejecting anything that is not a triple."""
    targets = [t for t in text.replace(",", " ").split() if t]
    if not targets:
        raise argparse.ArgumentTypeError("no targets given")
    for target in targets:
        if TRIPLE_RE.match(target) is None:
            raise argparse.ArgumentTypeError(f"not a target triple: {target!r}")
    return targets


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_one(
    target: str, root: Path, out_dir: Path, timeout_secs: int
) -> BuildOutcome:
    log: List[str] = [f"=== building --release --target {target} ==="]
    runner = "cross" if gk.which("cross") is not None else "cargo"
    if runner == "cargo":
        log.append("cross not installed; trying cargo (needs local linker/sysroot)")

    result = gk.run_tree(
        [runner, "build", "--release", "--locked", "--target", target],
        timeout=gk.timeout_or_none(timeout_secs),
        cwd=str(root),
        merge_stderr=True,
    )
    log.append(result.stdout.rstrip("\n"))
    if result.timed_out:
        log.append(f"FAIL {target}: timed out after {timeout_secs}s")
        return BuildOutcome(target, False, "", "\n".join(log))
    if result.code != 0:
        log.append(f"FAIL {target}: {runner} build exited {result.code}")
        return BuildOutcome(target, False, "", "\n".join(log))

    suffix = ".exe" if "windows" in target else ""
    binary = root / "target" / target / "release" / (BIN_NAME + suffix)
    if not binary.is_file():
        log.append(f"missing binary: {binary}")
        return BuildOutcome(target, False, "", "\n".join(log))

    dest = out_dir / f"{BIN_NAME}-{target}{suffix}"
    shutil.copyfile(binary, dest)
    # Strip only when a strip tool exists (musl static is often already stripped by the
    # profile). A host strip may reject a foreign architecture, which is not an error.
    strip = gk.which("strip")
    if strip is not None and "linux" in target:
        gk.run_tree([strip, str(dest)], timeout=STRIP_TIMEOUT_SECS)

    sums = out_dir / f"{dest.name}.sha256"
    gk.write_text(sums, f"{sha256_file(dest)}  {dest.name}\n")
    log.append(f"OK: {dest}")
    return BuildOutcome(target, True, str(dest), "\n".join(log))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="dist_multiarch.py",
        description="Build release binaries for a list of targets (cross when available).",
    )
    parser.add_argument(
        "--targets",
        type=parse_targets,
        default=list(DEFAULT_TARGETS),
        help="space- or comma-separated target triples (default: " + " ".join(DEFAULT_TARGETS) + ")",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help="where artifacts and .sha256 files go (default: <repo>/target/dist)",
    )
    parser.add_argument(
        "--jobs",
        type=gk.positive_int,
        default=DEFAULT_JOBS,
        help=f"maximum parallel builds (default {DEFAULT_JOBS}; 1 is serial)",
    )
    parser.add_argument(
        "--timeout-secs",
        type=gk.non_negative_int,
        default=DEFAULT_TIMEOUT_SECS,
        help=f"per-target build budget in seconds, 0 for none (default {DEFAULT_TIMEOUT_SECS})",
    )
    parser.add_argument(
        "--quiet", action="store_true", help="print build output only for failed targets"
    )
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    root = gk.repo_root()
    out_dir: Path = (
        args.out_dir.resolve() if args.out_dir is not None else root / "target" / "dist"
    )
    out_dir.mkdir(parents=True, exist_ok=True)

    targets: List[str] = list(dict.fromkeys(args.targets))
    outcomes: List[BuildOutcome] = []
    # No `with` block: on SIGINT the SystemExit must cancel queued targets, not start them.
    executor = ThreadPoolExecutor(max_workers=args.jobs)
    try:
        futures = [
            executor.submit(build_one, t, root, out_dir, args.timeout_secs)
            for t in targets
        ]
        for future in as_completed(futures):
            outcome = future.result()
            outcomes.append(outcome)
            if outcome.ok and args.quiet:
                continue
            gk.eprint(outcome.log)
    finally:
        executor.shutdown(wait=True, cancel_futures=True)

    by_target = {o.target: o for o in outcomes}
    for target in targets:
        outcome = by_target[target]
        status = "ok" if outcome.ok else "fail"
        sys.stdout.write(f"{target}\t{status}\t{outcome.artifact}\n")

    failed = [o.target for o in outcomes if not o.ok]
    gk.eprint(f"G-22 multi-arch artifacts under {out_dir} (jobs={args.jobs})")
    if failed:
        gk.eprint(f"multi-arch build FAILED for: {', '.join(failed)}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
