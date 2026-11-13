# SPDX-License-Identifier: MIT OR Apache-2.0
"""Unprivileged local sshd lab for the E2E harness (Unix only, Python 3.9+, stdlib only).

Helper module of scripts/e2e_real_ssh.py, not an executable script. It starts an sshd on
loopback with a throwaway passphrase-protected ed25519 key, so the E01-E21 matrix can run
without a lab host, a network or any secret on disk outside the harness temp dir.
On Windows there is no unprivileged sshd, so start_local_sshd reports the failure.
"""
from __future__ import annotations

import os
import secrets
import socket
import subprocess
import sys
from pathlib import Path
from typing import List, NamedTuple, Optional

sys.dont_write_bytecode = True  # keep __pycache__ out of scripts/
# `python -I` does not put the script directory on sys.path, so add it explicitly.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit  # noqa: E402

IS_WINDOWS = sys.platform == "win32"
SFTP_SERVER_CANDIDATES = (
    "/usr/libexec/openssh/sftp-server",
    "/usr/lib/openssh/sftp-server",
    "/usr/libexec/sftp-server",
    "/usr/lib/ssh/sftp-server",
)


# --- unprivileged local sshd (Unix only) --------------------------------------------
# Auth is a passphrase-protected ed25519 key rather than a password: a system account
# password cannot be driven without PAM/root, and the passphrase keeps E02's real
# assertion (secrets are encrypted at rest) while exercising --key-passphrase-stdin.


def sshd_binary() -> Optional[str]:
    if IS_WINDOWS:
        return None
    found = _gatekit.which("sshd")
    if found:
        return str(found)
    for cand in ("/usr/sbin/sshd", "/usr/bin/sshd"):
        if os.access(cand, os.X_OK):
            return cand
    return None


def sftp_server_binary() -> Optional[str]:
    for cand in SFTP_SERVER_CANDIDATES:
        if os.access(cand, os.X_OK):
            return cand
    return None


def port_is_free(port: int) -> bool:
    """A refused connect means nothing is listening there."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.settimeout(0.5)
        return sock.connect_ex(("127.0.0.1", port)) != 0


def free_loopback_port() -> Optional[int]:
    # sshd rejects Port 0, so probe upward from a randomized offset.
    base = 20000 + secrets.randbelow(20000)
    for offset in range(200):
        if port_is_free(base + offset):
            return base + offset
    return None


def sshd_config_text(port: int, tmp: Path, sftp: str) -> str:
    lines = [
        f"Port {port}",
        "ListenAddress 127.0.0.1",
        f"HostKey {tmp / 'e2e_hostkey'}",
        f"PidFile {tmp / 'sshd.pid'}",
        f"AuthorizedKeysFile {tmp / 'authorized_keys'}",
        "PubkeyAuthentication yes",
        "PasswordAuthentication no",
        "KbdInteractiveAuthentication no",
        "UsePAM no",
        "StrictModes no",
        "AllowTcpForwarding yes",
        "PermitTTY yes",
        "LogLevel ERROR",
        f"Subsystem sftp {sftp}",
    ]
    return "\n".join(lines) + "\n"


def generate_keys(tmp: Path, passphrase: str) -> Optional[Path]:
    keygen = _gatekit.which("ssh-keygen")
    if keygen is None:
        _gatekit.eprint("FAIL E00: --local-sshd requested but ssh-keygen not found")
        return None
    user_key = tmp / "e2e_id_ed25519"
    for path, phrase, comment in ((tmp / "e2e_hostkey", "", "e2e-host"),
                                  (user_key, passphrase, "e2e-user")):
        res = _gatekit.run_tree([str(keygen), "-q", "-t", "ed25519", "-f", str(path),
                                 "-N", phrase, "-C", comment], timeout=60)
        if res.code != 0:
            _gatekit.eprint("FAIL E00: ssh-keygen failed")
            return None
    public_key = (tmp / "e2e_id_ed25519.pub").read_text(encoding="utf-8")
    _gatekit.write_text(tmp / "authorized_keys", public_key)
    for path in (tmp / "e2e_hostkey", user_key, tmp / "authorized_keys"):
        os.chmod(path, 0o600)
    return user_key


def wait_for_listener(proc: "subprocess.Popen[bytes]", port: int) -> bool:
    # proc.wait(timeout) is both the poll delay and the early-exit detector.
    for _ in range(60):
        try:
            proc.wait(timeout=0.1)
            _gatekit.eprint("FAIL E00: local sshd exited during startup")
            return False
        except subprocess.TimeoutExpired:
            pass
        if not port_is_free(port):
            return True
    _gatekit.eprint(f"FAIL E00: local sshd never listened on 127.0.0.1:{port}")
    return False


class LocalSshd(NamedTuple):
    proc: "subprocess.Popen[bytes]"
    port: int
    key: Path
    passphrase: str


def start_local_sshd(tmp: Path, holder: List["subprocess.Popen[bytes]"]) -> Optional[LocalSshd]:
    """Start sshd on loopback and register it in `holder` so it is always reaped.
    Returns None (after logging) when the host cannot honour the request."""
    if IS_WINDOWS:
        _gatekit.eprint("FAIL E00: --local-sshd is Unix-only (no unprivileged sshd on Windows)")
        return None
    sshd = sshd_binary()
    if sshd is None:
        _gatekit.eprint("FAIL E00: --local-sshd requested but no sshd binary found")
        return None
    sftp = sftp_server_binary()
    if sftp is None:
        _gatekit.eprint("FAIL E00: --local-sshd requested but no sftp-server binary found")
        return None
    passphrase = secrets.token_hex(16)
    key = generate_keys(tmp, passphrase)
    port = free_loopback_port()
    if key is None:
        return None
    if port is None:
        _gatekit.eprint("FAIL E00: no free loopback port for local sshd")
        return None
    cfg = tmp / "sshd_config"
    _gatekit.write_text(cfg, sshd_config_text(port, tmp, sftp))
    check = _gatekit.run_tree([sshd, "-t", "-f", str(cfg)], timeout=30)
    if check.code != 0:
        _gatekit.eprint(check.stderr.strip())
        _gatekit.eprint("FAIL E00: local sshd config rejected")
        return None
    proc = subprocess.Popen([sshd, "-D", "-f", str(cfg), "-E", str(tmp / "sshd.log")],
                            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL, start_new_session=True)
    holder.append(proc)
    if not wait_for_listener(proc, port):
        return None
    return LocalSshd(proc, port, key, passphrase)
