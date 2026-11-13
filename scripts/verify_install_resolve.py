#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""GAP-SSH-DEP-002 (0.3.8): prove that dependency resolution without the repo Cargo.lock
still compiles (the `cargo install` re-resolve path) with patched russh and stable crypto.

Build artefacts go to real disk, not into the scratch directory
---------------------------------------------------------------
The scratch directory honours TMPDIR, which on a typical Linux desktop is a `tmpfs`
under a per-user quota. This gate is the only one that compiles the whole dependency
tree from scratch, so it is the one that hits that ceiling (measured as
`Disk quota exceeded (os error 122)` on `aws-lc-sys`). That is a red light caused by
the host rather than by the code, and a gate that cries wolf teaches the reader to skip
it.

Redirecting only the target dir is deliberate: what this gate proves is that the
dependency graph *re-resolves* without the repository lockfile, and that proof lives
entirely in the generated `Cargo.lock` inside the scratch directory. The default target
dir sits under `target/`, in a subdirectory of its own, because Cargo namespaces
fingerprints by target-dir path: it is a separate cache and cannot invalidate the
project's own build artefacts.

Contract: stdout carries only the final OK line. Progress goes to stderr.
Exit 0 passed, 1 failed, 2 usage error.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from pathlib import Path
from typing import Optional, Sequence, Tuple

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

DEFAULT_TIMEOUT_SECS = 3600

# Crypto RC COMPAT pins that block a clean resolve.
RC_PINS = re.compile(
    r'^(elliptic-curve|primefield|primeorder|ecdsa|pkcs5)\s*=\s*"=0\.[0-9]+\.0-rc',
    re.MULTILINE,
)

# Security floor from the advisories, not a preferred line: RUSTSEC-2026-0495 is patched
# in `>=0.65.0` (RUSTSEC-2026-0492 in `>=0.63.1`), so 0.60.3 to 0.64.x now resolve to a
# vulnerable client and must fail this gate.
RUSSH_FLOOR = (0, 65, 0)


def locked_version(lock_text: str, package: str) -> Optional[str]:
    """Version of the first `[[package]]` named ``package``, or None when absent."""
    pattern = re.compile(
        r'^name = "' + re.escape(package) + r'"\r?\nversion = "([^"]+)"', re.MULTILINE
    )
    found = pattern.search(lock_text)
    return found.group(1) if found else None


def parse_version(text: str) -> Optional[Tuple[int, int, int]]:
    found = re.match(r"(\d+)\.(\d+)\.(\d+)", text)
    if found is None:
        return None
    return int(found.group(1)), int(found.group(2)), int(found.group(3))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="verify_install_resolve.py",
        description="Prove that dependencies re-resolve without Cargo.lock and still compile.",
    )
    parser.add_argument(
        "--target-dir",
        type=Path,
        default=None,
        help="cargo target dir for the compile (default: <repo>/target/install-resolve-gate)",
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


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    def note(message: str) -> None:
        if not args.quiet:
            gk.eprint(message)

    def fail(message: str) -> int:
        gk.eprint(f"FAIL: {message}")
        return 1

    root = gk.repo_root()
    target_dir: Path = (
        args.target_dir.resolve()
        if args.target_dir is not None
        else root / "target" / "install-resolve-gate"
    )
    timeout = gk.timeout_or_none(args.timeout_secs)
    target_dir.mkdir(parents=True, exist_ok=True)

    # Fail if root still freezes crypto RC COMPAT pins (blocks clean resolve).
    manifest = gk.read_text(root / "Cargo.toml")
    pins = [m.group(0) for m in RC_PINS.finditer(manifest)]
    if pins:
        for line in pins:
            gk.eprint(line)
        return fail("COMPAT RC pins still present in Cargo.toml (DEP-002)")

    with tempfile.TemporaryDirectory(prefix="ssh-cli-install-gate.") as tmp_name:
        tmp = Path(tmp_name)
        # json.dumps yields a valid TOML basic string; forward slashes work on Windows.
        root_toml = json.dumps(root.as_posix())
        gk.write_text(
            tmp / "Cargo.toml",
            "[package]\n"
            'name = "install_gate"\n'
            'version = "0.0.0"\n'
            'edition = "2021"\n'
            "\n"
            "[dependencies]\n"
            f"ssh-cli = {{ path = {root_toml} }}\n",
        )
        gk.write_text(tmp / "src" / "main.rs", "fn main() {}\n")

        note("==> generate-lockfile (re-resolve like cargo install)")
        lock = gk.run_tree(
            ["cargo", "generate-lockfile"],
            timeout=timeout,
            cwd=str(tmp),
            capture=False,
        )
        if lock.timed_out:
            return fail(f"cargo generate-lockfile timed out after {args.timeout_secs}s")
        if lock.code != 0:
            return fail(f"cargo generate-lockfile exited {lock.code}")

        lock_text = gk.read_text(tmp / "Cargo.lock")

        note("==> russh patched check (>= 0.65.0)")
        if re.search(r'^name = "russh"\r?$', lock_text, re.MULTILINE) is None:
            return fail("russh missing from resolved lock")
        russh = locked_version(lock_text, "russh")
        if russh is None:
            return fail("could not parse russh version")
        note(f"resolved russh = {russh}")
        parsed = parse_version(russh)
        if parsed is None:
            return fail("could not parse russh version")
        if parsed < RUSSH_FLOOR:
            if parsed >= (0, 60, 0):
                return fail(
                    f"russh {russh} below security floor 0.65.0 (RUSTSEC-2026-0492/0495)"
                )
            return fail(f"russh {russh} too old")

        note("==> primefield may be stable 0.14.0 (allowed in 0.3.8)")
        primefield = locked_version(lock_text, "primefield")
        if primefield is not None:
            note(f"resolved primefield = {primefield}")

        note("==> cargo check")
        check = gk.run_tree(
            ["cargo", "check", "-q", "--target-dir", str(target_dir)],
            timeout=timeout,
            cwd=str(tmp),
            capture=False,
        )
        if check.timed_out:
            return fail(f"cargo check timed out after {args.timeout_secs}s")
        if check.code != 0:
            return fail(f"cargo check exited {check.code}")

    sys.stdout.write(f"OK: install resolve gate passed (russh {russh})\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
