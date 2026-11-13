#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""G-PROC-SLSA: release attestation helper (one-shot; no telemetry).

Prints the attestation checklist on stdout and exits 0. With --require-cosign it also
fails when the `cosign` binary is not on PATH. Nothing is signed or uploaded here.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Optional, Sequence

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

CHECKLIST = [
    "ssh-cli release attestation checklist",
    "1. cargo test --lib && cargo clippy --all-targets -- -D warnings",
    "2. python3 scripts/generate_sbom.py  (Windows: py -3 scripts/generate_sbom.py)",
    "3. cargo package --allow-dirty (review)",
    "4. git tag -a vX.Y.Z && git push origin vX.Y.Z  (only with human approval)",
    "5. Optional: cosign sign-blob --key <path-to-cosign.key> target/package/<name>.crate",
    "6. Optional: SLSA generator for provenance of the published release",
]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="release_attest.py",
        description="Print the release attestation checklist.",
    )
    parser.add_argument(
        "--require-cosign",
        action="store_true",
        help="exit 1 when the cosign binary is not on PATH",
    )
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    sys.stdout.write("\n".join(CHECKLIST) + "\n")
    if args.require_cosign and gk.which("cosign") is None:
        gk.eprint("cosign missing")
        return 1
    sys.stdout.write("OK (documentation path; no secrets uploaded)\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
