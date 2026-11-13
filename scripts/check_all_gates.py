#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Runs the whole mandatory gate battery in one invocation.
#
# Why this exists
# ---------------
# Every gate in this repository was individually well designed, and the 0.5.4 audit
# still found four red ones plus a test target that did not compile, while the local
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
# `scripts/check_*.py` in this repository.
#
# Contract (agent-native)
# -----------------------
#   stdout : one TSV record per gate (`id  status  exit  ms`), then a summary record.
#            With --json, one NDJSON object per gate instead. Nothing else. Ever.
#   stderr : progress, per-gate log paths, diagnostics.
#   exit   : 0 when every gate passed, 1 when any failed, 2 on usage error.
#
# Portability
# -----------
# Pure Python 3 standard library: no bash, rg, fd, awk, jq or sha256sum, so it runs the
# same on Linux, macOS and Windows. Sibling gates are started with `sys.executable`.
# A gate that exceeds --timeout-secs has its whole process tree killed and is reported
# as failed with exit 124.
#
# Declared non-gates
# ------------------
# A battery that presents itself as complete and omits scripts in silence reads as
# total coverage. These `scripts/*.py` are deliberately NOT gates here, and each one
# is named so the omission is a declaration rather than an oversight:
#
#   dist_multiarch.py        release artefact build; needs cross images, not a local gate
#   generate_sbom.py         release inventory generation; produces artefacts, asserts nothing
#   release_attest.py        release attestation; runs once per published tag
#   e2e_real_ssh.py          real-SSH end-to-end harness; needs a lab host or local sshd
#
# `tests/gaps_v064_gate_runner.rs` enforces this list: every public `scripts/*.py`
# (file name not starting with `_`) must be either a gate below or named in this
# block, so a new script cannot slip in silently.
#
# Sequential by design
# --------------------
# Most of the fourteen gates invoke cargo (`fmt`, `build-release`, `build-no-default`,
# `clippy`, `doc`, `test`, `exec-target`, `deny`, plus `install-resolve` and `cross-targets`
# indirectly). Concurrent cargo invocations block on the same `target/` directory
# lock, so running them in parallel would buy no wall-clock and would interleave the
# per-gate report. The sequence is the correct design here, not a missing optimisation.
#
# By default every gate runs even after a failure, because the point is to see the
# whole board. --fail-fast stops at the first red when you only want the next thing
# to fix.

from __future__ import annotations

import argparse
import os
import sys
import tempfile
import time
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

ROOT = gk.repo_root()
SCRIPTS = ROOT / "scripts"
MANIFEST = str(ROOT / "Cargo.toml")

# Per-gate budget. Generous on purpose: install-resolve compiles the whole dependency
# tree from scratch and cross-targets budgets three cargo checks of its own.
DEFAULT_TIMEOUT_SECS = 3600
TIMEOUT_EXIT = 124
TAIL_LINES = 20


def _py(script: str) -> List[str]:
    """argv that runs a sibling script with the interpreter running this one."""
    return [sys.executable, str(SCRIPTS / script)]


# (id, description, argv)
# Order matters: fmt and the build come first because they are the cheapest signals,
# and because a tree that does not compile makes every later gate meaningless.
#
# `exec-target` runs a suite `test` already covers, and that redundancy is the point.
# GAP-SSH-EXEC-ARGC-001 was an argv that executed a day of configuration on a machine
# nobody named; a regression there is not one red test among nine hundred, it is a
# safety property coming back off. Naming it as its own gate means the report says
# which property broke instead of "the suite failed", and `--only exec-target` gives
# a sub-second check of that one property.
GATES = [
    ("fmt", "rustfmt is clean", ["cargo", "fmt", "--manifest-path", MANIFEST, "--all", "--check"]),
    ("build-release", "release build succeeds", ["cargo", "build", "--manifest-path", MANIFEST, "--release"]),
    ("build-no-default", "builds without default features (Windows path, A8)", ["cargo", "build", "--manifest-path", MANIFEST, "--no-default-features"]),
    ("clippy", "clippy clean with warnings denied, doc_markdown included (A5)", ["cargo", "clippy", "--manifest-path", MANIFEST, "--all-targets", "--all-features", "--", "-D", "warnings", "-W", "clippy::doc_markdown"]),
    # Gap 29 proved rustdoc with this exact command, but nothing in the battery ran it,
    # so a broken intra-doc link could land with every gate green. The optional fourth
    # field adds variables on top of the inherited environment for this gate only.
    ("doc", "rustdoc builds with warnings denied (Gap 29)", ["cargo", "doc", "--manifest-path", MANIFEST, "--locked", "--no-deps", "--all-features"], {"RUSTDOCFLAGS": "-D warnings"}),
    ("test", "full test suite green", ["cargo", "test", "--manifest-path", MANIFEST, "--locked", "--all-features", "--no-fail-fast"]),
    ("exec-target", "the dangerous exec argv stays rejected (GAP-SSH-EXEC-ARGC-001)", ["cargo", "test", "--manifest-path", MANIFEST, "--locked", "--all-features", "--test", "gaps_v065_exec_target_designation"]),
    ("deny", "supply chain clean", ["cargo", "deny", "--manifest-path", MANIFEST, "check"]),
    ("cross-targets", "windows and macos type-check (B1)", _py("check_cross_targets.py")),
    ("advisory-freshness", "advisory database is current", _py("check_advisory_freshness.py")),
    ("advisory-hits", "no RUSTSEC advisory hits Cargo.lock (A4)", _py("check_advisory_hits.py")),
    ("en-identifiers", "identifiers are English", _py("check_en_identifiers.py")),
    ("markdown-format", "published documents follow the formatting contract (Gaps 25 and 37)", _py("check_markdown_format.py")),
    ("install-resolve", "cargo install --locked resolves", _py("verify_install_resolve.py")),
]


def gate_ids() -> List[str]:
    return [gate[0] for gate in GATES]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="check_all_gates.py",
        description="Run the whole mandatory gate battery. Exit: 0 all green, 1 at least one red, 2 usage error.",
    )
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="stop at the first red gate (default: run all, report all)",
    )
    parser.add_argument(
        "--json", action="store_true", help="emit NDJSON on stdout instead of TSV"
    )
    parser.add_argument(
        "--only",
        default="",
        metavar="ID[,ID...]",
        help="run only the named gates (see --list)",
    )
    parser.add_argument(
        "--log-dir",
        type=Path,
        default=None,
        metavar="DIR",
        help="keep per-gate output here (default: a fresh temporary directory)",
    )
    parser.add_argument("--list", action="store_true", help="print gate ids and exit")
    parser.add_argument(
        "--timeout-secs",
        type=gk.non_negative_int,
        default=DEFAULT_TIMEOUT_SECS,
        help=f"per-gate budget in seconds, 0 for none (default {DEFAULT_TIMEOUT_SECS})",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="suppress progress lines on stderr (failure tails are still printed)",
    )
    return parser


def run_gate(
    argv: List[str], timeout_secs: int, extra_env: Optional[Dict[str, str]] = None
) -> Tuple[int, str]:
    """Run one gate and return ``(exit code, combined output)``.

    ``extra_env`` is layered over ``os.environ``: ``run_tree`` replaces the whole
    environment when ``env`` is given, and a gate must keep ``PATH`` and ``HOME``.
    """
    env = None if not extra_env else {**os.environ, **extra_env}
    result = gk.run_tree(
        argv,
        timeout=gk.timeout_or_none(timeout_secs),
        cwd=str(ROOT),
        env=env,
        merge_stderr=True,
    )
    if result.timed_out:
        note = f"\n[gate killed after the {timeout_secs}s budget]\n"
        return TIMEOUT_EXIT, result.stdout + note
    return (result.code if result.code is not None else 1), result.stdout


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    args = build_parser().parse_args(argv)

    ids = gate_ids()
    if args.list:
        sys.stdout.write("".join(f"{gid}\n" for gid in ids))
        return 0

    only = [x for x in args.only.replace(",", " ").split() if x]
    for want in only:
        if want not in ids:
            gk.eprint(f"unknown gate id: {want}")
            return 2

    if args.log_dir is not None:
        log_dir = args.log_dir.resolve()
        log_dir.mkdir(parents=True, exist_ok=True)
    else:
        log_dir = Path(tempfile.mkdtemp(prefix="ssh-cli-gates."))
    gk.eprint(f"gate logs: {log_dir}")

    total = 0
    failed = 0
    skipped = 0

    for gid, desc, command, *gate_env in GATES:
        if only and gid not in only:
            skipped += 1
            continue

        total += 1
        log = log_dir / f"{gid}.log"
        if not args.quiet:
            gk.eprint(f"==> {gid}: {desc}")

        started = time.time_ns()
        # Gate output goes to its log, never to stdout: stdout is the machine contract.
        code, output = run_gate(command, args.timeout_secs, gate_env[0] if gate_env else None)
        ms = (time.time_ns() - started) // 1_000_000
        gk.write_text(log, output)

        if code == 0:
            status = "pass"
        else:
            status = "fail"
            failed += 1
            # Surface the tail on stderr so the operator sees why without opening the log.
            gk.eprint(f"--- {gid} FAILED (exit {code}); last {TAIL_LINES} lines of {log} ---")
            gk.eprint(gk.tail_lines(output, TAIL_LINES))

        if args.json:
            gk.emit_json(
                {
                    "gate": gid,
                    "status": status,
                    "exit": code,
                    "duration_ms": ms,
                    "log": str(log),
                }
            )
        else:
            sys.stdout.write(f"{gid}\t{status}\t{code}\t{ms}\n")
            sys.stdout.flush()

        if status == "fail" and args.fail_fast:
            gk.eprint(f"--fail-fast: stopping at {gid}")
            break

    if args.json:
        gk.emit_json(
            {
                "gate": "_summary",
                "total": total,
                "failed": failed,
                "skipped": skipped,
                "log_dir": str(log_dir),
            }
        )
    else:
        sys.stdout.write(f"_summary\ttotal={total}\tfailed={failed}\tskipped={skipped}\n")

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
