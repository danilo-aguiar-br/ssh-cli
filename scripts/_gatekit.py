# SPDX-License-Identifier: MIT OR Apache-2.0
"""Shared helpers for the gate scripts (stdlib only, Python 3.9+).

Why this module exists
----------------------
The gate scripts used to be bash. Bash gives timeouts, process-group kills and UTF-8
for free on Linux and none of them on Windows or macOS, so every script would have
re-implemented the same fragile subprocess code. It lives here once.

Rules every caller gets by using this module
--------------------------------------------
* Child processes run with ``shell=False`` and explicit stdin/stdout/stderr.
* A timeout kills the WHOLE process tree. ``subprocess.run(timeout=)`` kills only the
  direct child, so a ``cargo`` that spawned ``rustc`` would be left running; the tree
  is killed first and the output is read afterwards.
* POSIX uses ``start_new_session`` plus ``os.killpg`` (SIGTERM, short grace, SIGKILL).
  Windows uses ``CREATE_NEW_PROCESS_GROUP`` plus ``taskkill /PID <pid> /T /F``.
* SIGINT and SIGTERM kill every live child, then exit with 130 and 143.
* Text files are UTF-8 with ``\\n`` newlines, never ``utf-8-sig``.
* stdout carries payload only. Diagnostics go to stderr.

Importing this file never writes ``__pycache__`` next to the scripts: the callers set
``sys.dont_write_bytecode`` before importing it.
"""

from __future__ import annotations

import atexit
import json
import os
import shutil
import signal
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any, List, Mapping, NamedTuple, Optional, Sequence

IS_WINDOWS = os.name == "nt"

# Seconds a process group gets between SIGTERM and SIGKILL.
_TERM_GRACE_SECS = 3.0

# Seconds to wait for the pipes to close after the tree was killed.
_DRAIN_SECS = 5.0

_live_lock = threading.Lock()
_live: List["subprocess.Popen[str]"] = []
_aborted = threading.Event()


class RunResult(NamedTuple):
    """Outcome of :func:`run_tree`.

    ``code`` is ``None`` when the run timed out. A command that could not be started
    reports 127, like a shell would.
    """

    code: Optional[int]
    timed_out: bool
    stdout: str
    stderr: str


def setup_utf8() -> None:
    """Force UTF-8 and LF on stdout and stderr, whatever the console code page is."""
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", newline="\n")  # type: ignore[union-attr]
        except (AttributeError, ValueError, OSError):
            pass


def repo_root() -> Path:
    """Repository root: the parent of the ``scripts/`` directory holding this file."""
    return Path(__file__).resolve().parent.parent


def which(name: str) -> Optional[str]:
    """Resolve ``name`` on PATH. ``shutil.which`` also covers ``.exe`` on Windows."""
    return shutil.which(name)


def eprint(message: str = "") -> None:
    """Write one diagnostic line to stderr."""
    sys.stderr.write(message + "\n")
    sys.stderr.flush()


def emit_json(obj: Any) -> None:
    """Write ``obj`` as one compact JSON line on stdout."""
    sys.stdout.write(json.dumps(obj, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def write_text(path: Path, text: str) -> None:
    """Write UTF-8 text with LF newlines, creating the parent directory."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(text)


def read_text(path: Path) -> str:
    """Read a UTF-8 text file with universal newlines."""
    with open(path, "r", encoding="utf-8") as handle:
        return handle.read()


def tail_lines(text: str, count: int) -> str:
    """Last ``count`` lines of ``text``."""
    return "\n".join(text.splitlines()[-count:])


def positive_int(value: str) -> int:
    """argparse type: integer >= 1."""
    import argparse

    try:
        number = int(value)
    except ValueError:
        raise argparse.ArgumentTypeError(f"not an integer: {value!r}") from None
    if number < 1:
        raise argparse.ArgumentTypeError(f"must be >= 1: {value!r}")
    return number


def non_negative_int(value: str) -> int:
    """argparse type: integer >= 0."""
    import argparse

    try:
        number = int(value)
    except ValueError:
        raise argparse.ArgumentTypeError(f"not an integer: {value!r}") from None
    if number < 0:
        raise argparse.ArgumentTypeError(f"must be >= 0: {value!r}")
    return number


def timeout_or_none(secs: int) -> Optional[float]:
    """Map the ``--timeout-secs`` convention (0 means no limit) to a Popen timeout."""
    return float(secs) if secs > 0 else None


def _kill_tree_posix(proc: "subprocess.Popen[Any]") -> None:
    # The child was started with start_new_session, so its pid is the group id.
    pgid = proc.pid
    try:
        os.killpg(pgid, signal.SIGTERM)
    except (ProcessLookupError, PermissionError):
        pass
    try:
        proc.wait(timeout=_TERM_GRACE_SECS)
    except subprocess.TimeoutExpired:
        pass
    # Always KILL the group: the leader may be gone while a descendant ignored TERM.
    try:
        os.killpg(pgid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass
    try:
        proc.wait(timeout=_TERM_GRACE_SECS)
    except subprocess.TimeoutExpired:
        pass


def _kill_tree_windows(proc: "subprocess.Popen[Any]") -> None:
    taskkill = which("taskkill")
    if taskkill is not None:
        try:
            subprocess.run(
                [taskkill, "/PID", str(proc.pid), "/T", "/F"],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=15,
                shell=False,
                check=False,
            )
        except (OSError, subprocess.SubprocessError):
            pass
    try:
        proc.kill()
    except OSError:
        pass
    try:
        proc.wait(timeout=_TERM_GRACE_SECS)
    except subprocess.TimeoutExpired:
        pass


def kill_tree(proc: "subprocess.Popen[Any]") -> None:
    """Kill ``proc`` and every descendant, then reap ``proc``."""
    if IS_WINDOWS:
        _kill_tree_windows(proc)
    else:
        _kill_tree_posix(proc)


def _kill_all_live() -> None:
    with _live_lock:
        procs = list(_live)
    workers = [threading.Thread(target=kill_tree, args=(p,)) for p in procs]
    for worker in workers:
        worker.start()
    for worker in workers:
        worker.join()


atexit.register(_kill_all_live)


def _coerce_text(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="replace")
    return str(value).replace("\r\n", "\n")


def _stderr_target() -> int:
    try:
        return sys.stderr.fileno()
    except (AttributeError, OSError, ValueError):
        return 2


def run_tree(
    argv: Sequence[str],
    timeout: Optional[float] = None,
    cwd: Optional[str] = None,
    env: Optional[Mapping[str, str]] = None,
    capture: bool = True,
    merge_stderr: bool = False,
    input_text: Optional[str] = None,
) -> RunResult:
    """Run ``argv`` without a shell and kill the whole process tree on timeout.

    ``capture=True`` collects stdout and stderr as UTF-8 text (invalid bytes become
    U+FFFD, CRLF becomes LF). With ``merge_stderr=True`` stderr is folded into stdout, in order, and
    ``RunResult.stderr`` is empty.

    ``capture=False`` sends both child streams to OUR stderr, so a child can never
    pollute the payload on our stdout. The result carries empty strings.

    ``input_text`` is written to the child's stdin as UTF-8 (no newline is added) and
    stdin is then closed, all under the same timeout and tree kill. Without it stdin is
    /dev/null. The command is resolved through PATH first, so ``cargo`` finds
    ``cargo.exe`` on Windows.
    """
    args = [str(a) for a in argv]
    if _aborted.is_set():
        return RunResult(130, False, "", "")

    exe = args[0]
    if not os.path.dirname(exe):
        resolved = which(exe)
        if resolved is None:
            return RunResult(127, False, "", f"command not found: {exe}\n")
        exe = resolved
    args[0] = exe

    kwargs: dict = {
        "stdin": subprocess.DEVNULL if input_text is None else subprocess.PIPE,
        "cwd": cwd,
        "env": None if env is None else dict(env),
        "shell": False,
    }
    if capture:
        kwargs["stdout"] = subprocess.PIPE
        kwargs["stderr"] = subprocess.STDOUT if merge_stderr else subprocess.PIPE
    else:
        target = _stderr_target()
        kwargs["stdout"] = target
        kwargs["stderr"] = target
    if IS_WINDOWS:
        kwargs["creationflags"] = getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
    else:
        kwargs["start_new_session"] = True

    sys.stdout.flush()
    sys.stderr.flush()
    try:
        proc = subprocess.Popen(args, **kwargs)
    except OSError as exc:
        return RunResult(127, False, "", f"cannot start {args[0]}: {exc}\n")

    with _live_lock:
        _live.append(proc)

    timed_out = False
    out: Any = ""
    err: Any = ""
    try:
        try:
            if capture or input_text is not None:
                payload = None if input_text is None else input_text.encode("utf-8")
                out, err = proc.communicate(input=payload, timeout=timeout)
            else:
                proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            # Kill the tree BEFORE reading: a surviving grandchild keeps the pipes open.
            kill_tree(proc)
            if capture or input_text is not None:
                try:
                    out, err = proc.communicate(timeout=_DRAIN_SECS)
                except subprocess.TimeoutExpired as exc:
                    out, err = exc.output, exc.stderr
                    for pipe in (proc.stdout, proc.stderr):
                        if pipe is not None:
                            try:
                                pipe.close()
                            except OSError:
                                pass
    except BaseException:
        kill_tree(proc)
        raise
    finally:
        with _live_lock:
            if proc in _live:
                _live.remove(proc)

    code = None if timed_out else proc.returncode
    return RunResult(code, timed_out, _coerce_text(out), _coerce_text(err))


def install_sigint_exit() -> None:
    """SIGINT and SIGTERM kill every live child, then exit with 130 and 143.

    The exit is a ``SystemExit`` raised in the main thread, so ``finally`` blocks and
    context managers (temporary directories, executors) still run. Once the signal
    arrived no new child is started.
    """

    def handler(signum: int, _frame: Any) -> None:
        _aborted.set()
        _kill_all_live()
        raise SystemExit(130 if signum == signal.SIGINT else 143)

    names = ["SIGINT", "SIGTERM", "SIGBREAK"]
    for name in names:
        number = getattr(signal, name, None)
        if number is None:
            continue
        try:
            signal.signal(number, handler)
        except (ValueError, OSError):
            pass
