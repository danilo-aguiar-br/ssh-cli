#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""E2E real SSH for ssh-cli: anti-leak by design (G-AUD-10 / G-E2E-05 / v0.5.2).

Port of the former shell harness to Python 3 (stdlib only, Python 3.9+).

Credential sources (flags only, no environment variables):
  1) Flags: --host --user --password-file [--port] [--sudo-password-file]
  2) Maintainer-only: --from-grok-config [PATH] (default ~/.grok/config.toml, outside repo)
  3) --local-sshd: an unprivileged sshd on loopback with a throwaway passphrase key
If no lab host is configured and --local-sshd is absent, exit 0 with SKIP (offline runs
never fail).

Default binary: target/release/ssh-cli (target/release/ssh-cli.exe on Windows).

Hygiene (GAP-SSH-SEC-002):
  - Grok config MUST live under $HOME (default ~/.grok/config.toml), NEVER in this repo.
  - Never prints host/user/password. Prints only PASS/FAIL/SOFT/SKIP E0n.
  - Uses the system temp dir (outside the workspace). The temp dir is destroyed on exit.
  - Refuses grok config paths that resolve inside the repository root.

GAP-SSH-ENV-001 (fail2ban / sshd ban policy):
  - PROIBIDO: authentication-failure loops against a production host.
  - The official matrix E01-E21 does NOT include mass wrong-password attempts.

Platform support:
  - Linux and macOS: full matrix, including --local-sshd.
  - Windows: --local-sshd is Unix-only (no unprivileged sshd, no sftp-server, no ssh-keygen
    contract) and fails explicitly. Against a lab host, E14 (POSIX mode bits) and E16
    (symlink creation needs a privilege) are reported as SOFT with the reason.
  - --from-grok-config needs Python 3.11+ (tomllib).
  - Remote commands (sha256sum, stat -c, rm) run on the remote POSIX host, quoted with
    shlex; the local side needs only Python and the ssh-cli binary (plus sshd and
    ssh-keygen for --local-sshd). Every child runs by argv list, never through a shell.

Exit codes: 0 = pass or SKIP, 1 = at least one FAIL, 2 = usage error.
"""
from __future__ import annotations

import argparse
import getpass
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Optional

sys.dont_write_bytecode = True  # keep __pycache__ out of scripts/
# `python -I` does not put the script directory on sys.path, so add it explicitly.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit  # noqa: E402
from _e2e_cases import Lab, Matrix, run_matrix  # noqa: E402
from _e2e_sshd import LocalSshd, sshd_binary, start_local_sshd  # noqa: E402

IS_WINDOWS = sys.platform == "win32"
DEFAULT_GROK_CONFIG = "~/.grok/config.toml"


GROK_FLAG_MAP = {
    "--host": "host",
    "--port": "port",
    "--user": "user",
    "--password": "password",
    "--sudoPassword": "sudo_password",
}


def skip_all(reason: str) -> None:
    print(f"SKIP E2E real SSH: {reason}")
    sys.exit(0)


def usage_epilog() -> str:
    return (
        "Without a lab host and without --local-sshd, exits 0 with SKIP. "
        "Never prints secrets. With --local-sshd, an unusable sshd is a FAILURE, "
        "never a silent skip."
    )


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="e2e_real_ssh.py",
        description="Real-SSH E01-E21 matrix for ssh-cli.",
        epilog=usage_epilog(),
        allow_abbrev=False,
    )
    p.add_argument("--bin", default=None, help="binary (default: target/release/ssh-cli)")
    p.add_argument("--local-sshd", action="store_true", help="spawn an unprivileged sshd on "
                   "loopback and run the full matrix (Unix only)")
    p.add_argument("--from-grok-config", nargs="?", const=DEFAULT_GROK_CONFIG, default=None,
                   metavar="PATH", help="read the Grok config only (default "
                   f"{DEFAULT_GROK_CONFIG}; never inside this repository; needs Python 3.11+)")
    p.add_argument("--host", default=None, help="lab host")
    p.add_argument("--port", type=int, default=22, help="lab port (default 22)")
    p.add_argument("--user", default=None, help="lab user")
    p.add_argument("--password-file", default=None, metavar="PATH",
                   help="file with the lab password, first line ('-' reads stdin)")
    p.add_argument("--sudo-password-file", default=None, metavar="PATH",
                   help="file with the optional sudo password ('-' reads stdin)")
    return p


def read_secret(path: Optional[str]) -> Optional[str]:
    if path is None:
        return None
    raw = sys.stdin.read() if path == "-" else Path(path).read_text(encoding="utf-8")
    return raw.split("\n", 1)[0].rstrip("\r")


def parse_grok_args(srv_args: List[object]) -> Dict[str, str]:
    vals: Dict[str, str] = {}
    i = 0
    while i < len(srv_args):
        a = srv_args[i]
        if isinstance(a, str):
            for flag, key in GROK_FLAG_MAP.items():
                if a.startswith(flag + "="):
                    vals[key] = a[len(flag) + 1:]
                    break
                nxt = srv_args[i + 1] if i + 1 < len(srv_args) else None
                if a == flag and isinstance(nxt, str):
                    vals[key] = nxt
                    i += 1
                    break
        i += 1
    return vals


def grok_lab(raw_path: str, root: Path) -> Lab:
    """Read host/user/password from the Grok config; exit 1 on any hygiene failure."""
    if sys.version_info < (3, 11):
        _gatekit.eprint("FAIL E00: --from-grok-config needs Python 3.11+ (tomllib)")
        sys.exit(2)
    import tomllib

    grok_cfg = Path(raw_path).expanduser()
    if not grok_cfg.is_file():
        _gatekit.eprint("FAIL E00: grok config missing")
        sys.exit(1)
    # Refuse configs that live inside the repository (must stay outside the repo).
    GROK_CFG_ABS = grok_cfg.resolve()
    ROOT_ABS = root.resolve()
    if GROK_CFG_ABS == ROOT_ABS or ROOT_ABS in GROK_CFG_ABS.parents:
        _gatekit.eprint(f"FAIL E00: grok config must not live inside the repository ({GROK_CFG_ABS})")
        sys.exit(1)
    with open(GROK_CFG_ABS, "rb") as fh:
        data = tomllib.load(fh)
    servers = data.get("mcp_servers") or {}
    srv = servers.get("ssh-flowaiper") or {}
    vals = parse_grok_args(srv.get("args") or [])
    if not all(vals.get(k) for k in ("host", "user", "password")):
        _gatekit.eprint("FAIL E00: incomplete daemon args")
        sys.exit(1)
    return Lab(vals["host"], int(vals.get("port") or 22), vals["user"],
               vals["password"], vals.get("sudo_password"))


def resolve_bin(args: argparse.Namespace, root: Path) -> Path:
    default = root / "target" / "release" / ("ssh-cli.exe" if IS_WINDOWS else "ssh-cli")
    if args.bin is None:
        return default
    named = Path(args.bin)
    # The operator named this binary. Substituting the default would run the whole
    # E01-E21 matrix against a different artifact and print PASS for every case: a green
    # report about untested code.
    if not (named.is_file() and os.access(named, os.X_OK)):
        _gatekit.eprint(f"FAIL E00: named binary is not executable: {named}")
        sys.exit(2)
    return named


def ensure_default_bin(bin_path: Path, root: Path) -> None:
    if bin_path.is_file() and os.access(bin_path, os.X_OK):
        return
    _gatekit.eprint("building ssh-cli release...")
    if _gatekit.which("cargo") is None:
        _gatekit.eprint("FAIL E00: default binary missing and cargo not found")
        sys.exit(2)
    res = _gatekit.run_tree(["cargo", "build", "--release", "-q"], timeout=3600, cwd=str(root))
    if res.code != 0 or not bin_path.is_file():
        _gatekit.eprint(res.stderr)
        _gatekit.eprint("FAIL E00: cargo build --release failed")
        sys.exit(2)


def child_env(tmp: Path) -> Dict[str, str]:
    """Isolate the CLI under test from the operator's real configuration."""
    env = dict(os.environ)
    env.pop("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", None)
    for key in ("SSH_CLI_HOME", "HOME", "XDG_CONFIG_HOME"):
        env[key] = str(tmp)
    if IS_WINDOWS:
        for key in ("USERPROFILE", "APPDATA", "LOCALAPPDATA"):
            env[key] = str(tmp)
    return env


def offline_skip() -> None:
    """G-E2E-05: offline / no lab -> SKIP exit 0 (not FAIL), and the skip names its cause."""
    if sshd_binary():
        skip_all("no lab host, but sshd IS available here; re-run with --local-sshd "
                 "to execute E01-E21")
    if IS_WINDOWS:
        skip_all("no lab host and no sshd: --local-sshd is Unix-only on Windows "
                 "(use --host/--user/--password-file or --from-grok-config)")
    skip_all("no lab host and no sshd (use --host/--user/--password-file, "
             "--from-grok-config or --local-sshd)")


def build_lab(args: argparse.Namespace, root: Path) -> Lab:
    if args.from_grok_config is not None:
        return grok_lab(args.from_grok_config, root)
    return Lab(args.host, args.port, args.user,
               read_secret(args.password_file), read_secret(args.sudo_password_file))


def execute(args: argparse.Namespace, root: Path, bin_path: Path, lab: Lab, tmp: Path,
            holder: List["subprocess.Popen[bytes]"]) -> int:
    local: Optional[LocalSshd] = None
    if args.local_sshd:
        # Explicit mode never degrades to SKIP: an unusable sshd is a hard failure.
        local = start_local_sshd(tmp, holder)
        if local is None:
            print("---")
            print("fails=1 local_sshd=requested_but_unusable")
            return 1
        lab = Lab("127.0.0.1", local.port, getpass.getuser(), None, None)
    elif not (lab.host and lab.user and lab.password):
        offline_skip()
    if args.bin is None:
        ensure_default_bin(bin_path, root)
    m = Matrix(bin_path, tmp, child_env(tmp), lab, local)
    run_matrix(m)
    return finish(m, tmp, holder)


def finish(m: Matrix, tmp: Path, holder: List["subprocess.Popen[bytes]"]) -> int:
    release(tmp, holder)
    print("---")
    print(f"fails={m.fails} soft_sudo={m.soft_sudo} tmp_destroyed=yes")
    return 1 if m.fails > 0 else 0


def release(tmp: Path, holder: List["subprocess.Popen[bytes]"]) -> None:
    """Kill sshd's process tree and destroy the temp dir; safe to call twice."""
    while holder:
        _gatekit.kill_tree(holder.pop())
    shutil.rmtree(tmp, ignore_errors=True)


def main(argv: Optional[List[str]] = None) -> int:
    _gatekit.setup_utf8()
    _gatekit.install_sigint_exit()
    args = build_parser().parse_args(argv)
    root = _gatekit.repo_root()
    bin_path = resolve_bin(args, root)
    lab = build_lab(args, root)
    tmp = Path(tempfile.mkdtemp(prefix="ssh-cli-e2e."))
    holder: List["subprocess.Popen[bytes]"] = []
    try:
        return execute(args, root, bin_path, lab, tmp, holder)
    finally:
        release(tmp, holder)


if __name__ == "__main__":
    sys.exit(main())
