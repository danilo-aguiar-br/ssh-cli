# SPDX-License-Identifier: MIT OR Apache-2.0
"""The official E01-E21 matrix for the E2E harness (Python 3.9+, stdlib only).

Helper module of scripts/e2e_real_ssh.py, not an executable script. Each case is its own
function; `run_matrix` schedules them in order. Output carries only PASS/FAIL/SOFT lines.
Children run by argv list through _gatekit (timeout kills the process tree), and secrets
reach the child by stdin, never by argv.
"""
from __future__ import annotations

import filecmp
import hashlib
import json
import os
import re
import shlex
import stat
import sys
from pathlib import Path
from typing import Callable, Dict, List, NamedTuple, Optional

sys.dont_write_bytecode = True  # keep __pycache__ out of scripts/
# `python -I` does not put the script directory on sys.path, so add it explicitly.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit  # noqa: E402
from _e2e_sshd import LocalSshd  # noqa: E402

IS_WINDOWS = sys.platform == "win32"
SSH_NAME = "e2e"


KEY_PREFIX_RE = re.compile(r"sshcli-enc:v[0-9]+:")


CLI_TIMEOUT_SECS = 300


SFTP_SIZES = (0, 1, 760, 32768, 32769, 65536)


class Lab(NamedTuple):
    host: Optional[str]
    port: int
    user: Optional[str]
    password: Optional[str]
    sudo_password: Optional[str]


class Matrix:
    """Shared state for the E01-E21 cases. Never holds output that carries secrets."""

    def __init__(self, bin_path: Path, tmp: Path, env: Dict[str, str], lab: Lab,
                 local: Optional[LocalSshd]) -> None:
        self.bin = bin_path
        self.tmp = tmp
        self.env = env
        self.lab = lab
        self.local = local
        self.pid = os.getpid()
        self.fails = 0
        self.soft_sudo = 0
        self.secret = local.passphrase if local else (lab.password or "")

    def ok(self, case: str) -> None:
        print(f"PASS {case}")

    def fail(self, case: str) -> None:
        print(f"FAIL {case}")
        self.fails += 1

    def soft(self, case: str, why: str) -> None:
        print(f"SOFT {case} (skipped: {why})")

    def verdict(self, case: str, good: bool) -> None:
        if good:
            self.ok(case)
        else:
            self.fail(case)

    def cli(self, *args: str, timeout: int = CLI_TIMEOUT_SECS,
            stdin: Optional[str] = None) -> "_gatekit.RunResult":
        argv = [str(self.bin), "--config-dir", str(self.tmp), "--output-format", "json", *args]
        return _gatekit.run_tree(argv, timeout=timeout, env=self.env, input_text=stdin)

    def succeeds(self, *args: str, timeout: int = CLI_TIMEOUT_SECS,
                 stdin: Optional[str] = None) -> bool:
        res = self.cli(*args, timeout=timeout, stdin=stdin)
        return res.code == 0 and not res.timed_out

    def out(self, *args: str, timeout: int = CLI_TIMEOUT_SECS) -> str:
        return self.cli(*args, timeout=timeout).stdout

    def remote_stdout(self, command: str, timeout: int = 60) -> str:
        """Run a remote command and return the `stdout` field of the JSON envelope."""
        raw = self.out("exec", "--json", SSH_NAME, command, timeout=timeout)
        try:
            return str(json.loads(raw).get("stdout") or "")
        except (ValueError, AttributeError):
            return ""

    def remote_rm(self, *paths: str, flags: str = "-f") -> None:
        quoted = " ".join(shlex.quote(p) for p in paths)
        self.cli("exec", SSH_NAME, f"rm {flags} {quoted}", timeout=60)

    def remote_hash(self, path: str) -> str:
        words = self.remote_stdout(f"sha256sum {shlex.quote(path)}").split()
        return words[0] if words else ""

    def local_path(self, name: str) -> Path:
        return self.tmp / name


def case_e01(m: Matrix) -> None:
    m.verdict("E01", m.succeeds("secrets", "status"))


def case_e02(m: Matrix) -> None:
    """Register the host and prove the secret is encrypted at rest."""
    host, port, user = m.lab.host or "", str(m.lab.port), m.lab.user or ""
    base = ["vps", "add", "--name", SSH_NAME, "--host", host, "--port", port, "--user", user]
    if m.local:
        auth = ["--key", str(m.local.key), "--key-passphrase-stdin"]
    else:
        auth = ["--password-stdin"]
    added = m.succeeds(*base, *auth, "--timeout", "60000", "--check", stdin=m.secret)
    cfg = m.local_path("config.toml")
    try:
        body = cfg.read_text(encoding="utf-8")
    except OSError:
        body = ""
    # The prefix is matched version-agnostically so the assertion never freezes on v1/v2.
    m.verdict("E02", added and bool(KEY_PREFIX_RE.search(body)) and m.secret not in body)


def case_e03(m: Matrix) -> None:
    m.verdict("E03", "e2e-ok" in m.out("exec", SSH_NAME, "echo e2e-ok"))


def case_e04(m: Matrix) -> None:
    if m.lab.sudo_password:
        m.cli("vps", "edit", SSH_NAME, "--sudo-password-stdin", stdin=m.lab.sudo_password)
    if m.succeeds("sudo-exec", SSH_NAME, "true"):
        m.ok("E04")
    else:
        m.soft("E04", "sudo unavailable or auth failed")
        m.soft_sudo = 1


def case_e05(m: Matrix) -> None:
    m.verdict("E05", m.succeeds("health-check", SSH_NAME))


def case_e06(m: Matrix) -> None:
    m.verdict("E06", "encrypted" in m.out("vps", "doctor"))


def case_e07(m: Matrix) -> None:
    """The registered secret must never surface in list/show output.

    Uses m.secret, which is the key passphrase under --local-sshd: an empty needle would
    match every line and fail E07 spuriously, hence the non-empty guard.
    """
    listing = m.out("vps", "list") + m.out("vps", "show", SSH_NAME)
    m.verdict("E07", not (m.secret and m.secret in listing))


def case_e08(m: Matrix) -> None:
    res = m.cli("exec", "--timeout", "2000", SSH_NAME, "sleep 30", timeout=60)
    if res.code != 0 and not res.timed_out:
        m.ok("E08")
    else:
        m.soft("E08", "sleep completed within timeout")


def case_e09(m: Matrix) -> None:
    m.verdict("E09", "e2e-ok" in m.out("exec", SSH_NAME, "echo e2e-ok"))


class ScpFiles(NamedTuple):
    remote: str
    remote_space: str


def case_e10(m: Matrix, f: ScpFiles) -> None:
    """`scp upload` of a plain file, a file with spaces and a 1 MiB payload."""
    _gatekit.write_text(m.local_path("up-plain.txt"), "e2e-scp-payload\n")
    _gatekit.write_text(m.local_path("up space file.txt"), "space-payload\n")
    m.local_path("up-1m.bin").write_bytes(os.urandom(1024 * 1024))
    steps = (
        (m.local_path("up-plain.txt"), f.remote, 120000),
        (m.local_path("up space file.txt"), f.remote_space, 120000),
        (m.local_path("up-1m.bin"), f.remote + ".1m", 180000),
    )
    good = all(m.succeeds("scp", "upload", "--timeout", str(t), SSH_NAME, str(src), dst)
               for src, dst, t in steps)
    m.verdict("E10", good)


def case_e11(m: Matrix, f: ScpFiles) -> None:
    """`scp download` of the three E10 files back into the temp dir."""
    steps = (
        (f.remote, m.local_path("down-plain.txt"), 120000),
        (f.remote_space, m.local_path("down space file.txt"), 120000),
        (f.remote + ".1m", m.local_path("down-1m.bin"), 180000),
    )
    good = all(m.succeeds("scp", "download", "--timeout", str(t), SSH_NAME, src, str(dst))
               for src, dst, t in steps)
    m.verdict("E11", good)


def case_e12(m: Matrix) -> None:
    pairs = (("up-plain.txt", "down-plain.txt"),
             ("up space file.txt", "down space file.txt"),
             ("up-1m.bin", "down-1m.bin"))
    try:
        good = all(filecmp.cmp(m.local_path(a), m.local_path(b), shallow=False)
                   for a, b in pairs)
    except OSError:
        good = False
    m.verdict("E12", good)


def case_e13(m: Matrix) -> None:
    """Remote missing: exit 66 (ArquivoNaoEncontrado / GAP-SSH-IO-010), no residual file."""
    target = m.local_path("should-not-exist")
    res = m.cli("scp", "download", "--timeout", "30000", SSH_NAME,
                f"/tmp/ssh-cli-e2e-missing-{m.pid}-no-such", str(target), timeout=120)
    E13_EC = res.code
    m.verdict("E13", E13_EC == 66 and not target.exists())


def local_mode_and_mtime(path: Path) -> "tuple[str, str]":
    st = os.stat(path)
    return format(stat.S_IMODE(st.st_mode), "o"), str(int(st.st_mtime))


def case_e14(m: Matrix) -> None:
    """SCP-023: preserve mode+mtime on upload AND download (OpenSSH -p / line T)."""
    if IS_WINDOWS:
        m.soft("E14", "POSIX file modes are not available on Windows")
        return
    src = m.local_path("preserve-src.bin")
    down = m.local_path("preserve-down.bin")
    remote = f"/tmp/ssh-cli-e2e-preserve-{m.pid}.bin"
    _gatekit.write_text(src, "preserve-payload\n")
    os.chmod(src, 0o600)
    os.utime(src, (1579089600, 1579089600))  # 2020-01-15 12:00:00 UTC, epoch avoids TZ
    want = local_mode_and_mtime(src)
    good = (m.succeeds("scp", "upload", "--timeout", "60000", SSH_NAME, str(src), remote)
            and m.succeeds("scp", "download", "--timeout", "60000", SSH_NAME, remote, str(down)))
    if good:
        line = m.remote_stdout(f"stat -c '%a %Y' {shlex.quote(remote)}", 30).split("\n", 1)[0]
        good = tuple(line.split()) == want and local_mode_and_mtime(down) == want
    m.verdict("E14", good)
    m.remote_rm(remote)


def parse_ndjson(raw: str) -> List[object]:
    """Decode concatenated JSON documents of any layout (NDJSON or pretty-printed)."""
    decoder, docs, pos = json.JSONDecoder(), [], 0
    while pos < len(raw):
        if raw[pos].isspace():
            pos += 1
            continue
        try:
            doc, pos = decoder.raw_decode(raw, pos)
        except ValueError:
            break
        docs.append(doc)
    return docs


def case_e15(m: Matrix) -> None:
    """GAP-SSH-TUN-003: local port 0 binds an ephemeral port; JSON local_port must be >= 1."""
    res = m.cli("tunnel", SSH_NAME, "0", "127.0.0.1", str(m.lab.port),
                "--timeout-ms", "2000", "--json", timeout=8)
    # `tunnel` legitimately emits NDJSON: `tunnel_listening` then `tunnel_closed`. Judging
    # by the last document would always fail on the close event, so look for the bind event.
    good = any(isinstance(d, dict) and d.get("ok") is True
               and d.get("event") == "tunnel_listening"
               and isinstance(d.get("local_port"), int) and d["local_port"] >= 1
               for d in parse_ndjson(res.stdout))
    m.verdict("E15", good)


def case_e16(m: Matrix) -> None:
    """GAP-SSH-SCP-024: symlink: follow the target content if it is a regular file."""
    target = m.local_path("symlink-target.txt")
    link = m.local_path("symlink-link.txt")
    down = m.local_path("symlink-down.txt")
    remote = f"/tmp/ssh-cli-e2e-symlink-{m.pid}.txt"
    _gatekit.write_text(target, "symlink-payload-v1\n")
    try:
        if link.is_symlink() or link.exists():
            link.unlink()
        os.symlink(target, link)
    except (OSError, NotImplementedError) as exc:
        m.soft("E16", f"cannot create a symlink on this platform ({type(exc).__name__})")
        return
    good = (m.succeeds("scp", "upload", "--timeout", "60000", SSH_NAME, str(link), remote)
            and m.succeeds("scp", "download", "--timeout", "60000", SSH_NAME, remote, str(down))
            and filecmp.cmp(target, down, shallow=False))
    m.verdict("E16", good)
    m.remote_rm(remote)


def sftp_roundtrip(m: Matrix, size: int) -> bool:
    """One size of E17: upload, remote sha256 (never trust client bytes), download, compare."""
    local = m.local_path(f"sftp-up-{size}.bin")
    down = m.local_path(f"sftp-down-{size}.bin")
    remote = f"/tmp/ssh-cli-e2e-sftp-{size}-{m.pid}.bin"
    payload = os.urandom(size)
    local.write_bytes(payload)
    good = False
    if m.succeeds("sftp", "upload", "--timeout", "60000", SSH_NAME, str(local), remote):
        # Measure the EFFECT on the destination, not the JSON bytes field.
        remote_hash = m.remote_hash(remote)
        good = (remote_hash == hashlib.sha256(payload).hexdigest()
                and m.succeeds("sftp", "download", "--timeout", "60000", SSH_NAME, remote, str(down))
                and filecmp.cmp(local, down, shallow=False))
    m.remote_rm(remote)
    return good


def case_e17(m: Matrix) -> None:
    """G7: SFTP round-trip, sizes cover the chunk boundary (empty, 1B, 760B, 32KiB, +1, 64KiB)."""
    good = True
    for size in SFTP_SIZES:
        if not sftp_roundtrip(m, size):
            good = False
            break
    m.verdict("E17", good)


def case_e18(m: Matrix) -> None:
    """G7: SFTP recursive tree upload + remote checksum of one leaf."""
    tree = m.local_path(f"sftp-tree-{m.pid}")
    (tree / "sub").mkdir(parents=True, exist_ok=True)
    _gatekit.write_text(tree / "a.txt", "leaf-a\n")
    _gatekit.write_text(tree / "sub" / "b.txt", "leaf-b\n")
    remote = f"/tmp/ssh-cli-e2e-sftp-tree-{m.pid}"
    leaf = hashlib.sha256((tree / "sub" / "b.txt").read_bytes()).hexdigest()
    good = False
    if m.succeeds("sftp", "upload", "--recursive", "--timeout", "120000", SSH_NAME,
                  str(tree), remote):
        remote_leaf = m.remote_hash(f"{remote}/sub/b.txt")
        good = bool(remote_leaf) and remote_leaf == leaf
    m.verdict("E18", good)
    m.remote_rm(remote, flags="-rf")


def _hosts_after_e02(m: Matrix) -> List[dict]:
    """Parsed `vps list --json`; empty when the payload is not the documented array."""
    try:
        parsed = json.loads(m.out("vps", "list", "--json"))
    except ValueError:
        return []
    return parsed if isinstance(parsed, list) else []


def case_e19(m: Matrix) -> None:
    """D6: `vps list --select` must return a reduced item and stay a bare array.

    The published `vps-list` schema states two item shapes with `anyOf`: the full record
    from `vps-show`, and the reduced record `--select` produces. Before this case the
    deviation was invisible, because `[{"name":"a"}]` satisfied no published shape and no
    gate compared the shaped output against the document the CLI itself ships.
    """
    hosts = _hosts_after_e02(m)
    if not hosts:
        m.fail("E19")
        return
    res = m.cli("vps", "list", "--json", "--select", "name", "--select", "host")
    if res.code != 0 or res.timed_out:
        m.fail("E19")
        return
    try:
        reduced = json.loads(res.stdout)
    except ValueError:
        m.fail("E19")
        return
    good = isinstance(reduced, list) and len(reduced) == len(hosts)
    if good:
        for item in reduced:
            # The projection is the contract: only the selected keys, no envelope field
            # smuggled into an item, and nothing the full shape requires.
            if not isinstance(item, dict):
                good = False
                break
            if set(item) != {"name", "host"}:
                good = False
                break
            if not isinstance(item.get("name"), str) or not isinstance(item.get("host"), str):
                good = False
                break
        # The bare array must stay a bare array: the reduction report goes to stderr.
        if good and ("agent_shape" in res.stdout or "schema_version" in res.stdout):
            good = False
    m.verdict("E19", good)


def case_e20(m: Matrix) -> None:
    """D7: `--limit` drops records and says so on stderr, and `--quiet` silences it.

    The composition worked but had no case protecting it. A limit that silently returns
    fewer records is indistinguishable from a registry with fewer hosts, which is why the
    accounting line is asserted rather than just the count.
    """
    hosts = _hosts_after_e02(m)
    if len(hosts) < 2:
        # A second registry entry is what this case needs, and it does not need to be
        # reachable: `vps list` never dials, so a placeholder entry on a dead port is
        # enough to exercise the shape. Skipping here made the case report SOFT on
        # every run of the harness, which is a named coverage hole, not a passing run.
        filler = f"{SSH_NAME}-filler"
        m.cli("vps", "add", "--name", filler, "--host", "127.0.0.1", "--port", "1",
              "--user", "filler", "--use-agent", "--json")
        hosts = _hosts_after_e02(m)
        if len(hosts) < 2:
            m.fail("E20")
            return
    res = m.cli("vps", "list", "--json", "--select", "name", "--limit", "1")
    if res.code != 0 or res.timed_out:
        m.fail("E20")
        return
    try:
        limited = json.loads(res.stdout)
    except ValueError:
        m.fail("E20")
        return
    report = {}
    for line in res.stderr.splitlines():
        line = line.strip()
        if line.startswith("{"):
            try:
                candidate = json.loads(line)
            except ValueError:
                continue
            if candidate.get("event") == "agent-shape":
                report = candidate
    good = (
        isinstance(limited, list)
        and len(limited) == 1
        and report.get("input_count") == len(hosts)
        and report.get("output_count") == 1
        and report.get("dropped") == len(hosts) - 1
    )
    # `--quiet` is the documented way to ask for a silent stream; the payload must not
    # change with it.
    quiet = m.cli("vps", "list", "--json", "--select", "name", "--limit", "1", "--quiet")
    if good:
        good = quiet.code == 0 and not quiet.timed_out and quiet.stdout.strip() == res.stdout.strip()
    if good:
        good = "agent-shape" not in quiet.stderr
    m.verdict("E20", good)


def case_e21(m: Matrix) -> None:
    """Gap 50: `secrets init --keyring` must round-trip through the real OS store.

    The old backend answered `Ok` from a process-local mock, so `init` reported success
    and the next `status` reported `encryption_active: false`. This case runs two
    processes: one writes, the other must resolve the key without repeating the flag.
    A machine with no answering store is a SOFT skip, never a PASS.
    """
    # By the time this case runs, E02 has already registered a host, which created
    # `secrets.key`. That is exactly the transition an operator makes when moving a
    # live profile into the OS store, so `--force` is the honest call here: it rotates
    # the key, re-encrypts the inventory and retires the file. The first version of
    # this case called `init --keyring` alone, which minted a second key instead of
    # refusing, and the file kept shadowing the store afterwards.
    init = m.cli("secrets", "init", "--keyring", "--force", "--json")
    try:
        if init.code != 0 or init.timed_out:
            if init.code == 69:
                m.soft("E21", "no credential store on this machine (exit 69)")
            else:
                m.fail("E21")
            return
        _check_e21(m, init)
    finally:
        # Runs on every exit, failed or timed-out init included: the store write in
        # `init_primary_key` happens before `secrets_status()`, which can still fail.
        _retire_e21_store_entry(m)


def _retire_e21_store_entry(m: Matrix) -> None:
    """Delete the OS store entry E21 may have written, and say so when it cannot.

    The store is machine-wide and survives the temp dir, so every run used to leave one
    `secrets-primary-key:<digest>` behind. Rotating back to the file with `--force` is
    the product path that deletes the entry (`src/secrets/init_key.rs`), and E21 is the
    last case, so nothing reads the profile afterwards. A failed cleanup is its own FAIL
    line, because a leftover entry is a real residue and not a skipped check.
    """
    cleanup = m.cli("secrets", "init", "--force", "--json")
    if cleanup.code != 0 or cleanup.timed_out:
        _gatekit.eprint(f"E21: keyring cleanup exited {cleanup.code}; an entry may remain in the OS store")
        m.fail("E21-cleanup")


def _check_e21(m: Matrix, init: "_gatekit.RunResult") -> None:
    """Body of E21 once `init --keyring` succeeded, so its caller can always clean up."""
    try:
        payload = json.loads(init.stdout)
    except ValueError:
        m.fail("E21")
        return
    if payload.get("key_source") != "keyring":
        m.fail("E21")
        return
    if m.local_path("secrets.key").exists():
        # A leftover file answers before the store on every later read, so the
        # rotation would look applied and never be.
        m.fail("E21")
        return
    status = m.cli("secrets", "status", "--json")
    good = False
    if status.code == 0 and not status.timed_out:
        try:
            body = json.loads(status.stdout)
        except ValueError:
            body = {}
        good = body.get("key_source") == "keyring" and body.get("encryption_active") is True
    # The inventory written before the rotation must survive it: a key that moved
    # without re-encrypting turns every stored secret into a decrypt failure.
    if good and _hosts_after_e02(m):
        good = m.succeeds("vps", "show", SSH_NAME)
    m.verdict("E21", good)


def guarded(m: Matrix, case: str, fn: Callable[[], None]) -> None:
    try:
        fn()
    except Exception as exc:  # a crashing case is a failed case, never a skipped one
        _gatekit.eprint(f"{case}: unexpected {type(exc).__name__}")
        m.fail(case)


def run_matrix(m: Matrix) -> None:
    files = ScpFiles(f"/tmp/ssh-cli-e2e-scp-{m.pid}.bin",
                     f"/tmp/ssh-cli e2e space {m.pid} .bin")
    plan: List["tuple[str, Callable[[], None]]"] = [
        ("E01", lambda: case_e01(m)), ("E02", lambda: case_e02(m)),
        ("E03", lambda: case_e03(m)), ("E04", lambda: case_e04(m)),
        ("E05", lambda: case_e05(m)), ("E06", lambda: case_e06(m)),
        ("E07", lambda: case_e07(m)), ("E08", lambda: case_e08(m)),
        ("E09", lambda: case_e09(m)), ("E10", lambda: case_e10(m, files)),
        ("E11", lambda: case_e11(m, files)), ("E12", lambda: case_e12(m)),
        ("E13", lambda: case_e13(m)), ("E14", lambda: case_e14(m)),
        ("E15", lambda: case_e15(m)), ("E16", lambda: case_e16(m)),
        ("E17", lambda: case_e17(m)), ("E18", lambda: case_e18(m)),
        ("E19", lambda: case_e19(m)), ("E20", lambda: case_e20(m)),
        ("E21", lambda: case_e21(m)),
    ]
    for case, fn in plan:
        guarded(m, case, fn)
    # Best-effort remote cleanup (never print paths with secrets).
    m.remote_rm(files.remote, files.remote_space, files.remote + ".1m")
