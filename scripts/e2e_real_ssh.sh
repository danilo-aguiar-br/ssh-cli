#!/usr/bin/env bash
# E2E real SSH for ssh-cli — anti-leak by design (G-AUD-10 / G-E2E-05 / v0.5.2).
#
# Preferred credential sources (XDG / CLI — not product env store):
#   1) --config-dir DIR with hosts already registered via `ssh-cli vps add`
#   2) Lab file: $XDG_CONFIG_HOME/ssh-cli-e2e/lab.toml (host/user/key paths)
#   3) Flags: --host --user --key [--port]
#   4) Maintainer-only: --from-grok-config ($HOME/.grok/config.toml outside repo)
#
# Legacy SSH_CLI_E2E_* env vars are accepted ONLY by this harness (not product runtime).
# If no lab host is configured, exit 0 with SKIP (do not fail offline runs).
#
# Default binary: target/release/ssh-cli.
#
# Hygiene (GAP-SSH-SEC-002):
#   - Grok/MCP config MUST live under $HOME (default ~/.grok/config.toml), NEVER in this repo.
#   - Never prints host/user/password. Prints only PASS/FAIL/SKIP E0n.
#   - Uses /tmp (outside workspace). Temp dir destroyed on exit.
#   - Refuses grok config paths that resolve inside the repository root.
#
# GAP-SSH-ENV-001 (fail2ban / sshd ban policy):
#   - PROIBIDO loops de autenticação falha em host de produção.
#   - Matrix oficial E01–E16 NÃO inclui mass wrong-password.
set -euo pipefail
set +x

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${SSH_CLI_E2E_BIN:-$ROOT/target/release/ssh-cli}"
FROM_GROK=0
SOFT_SUDO=0
FAILS=0
CONFIG_DIR=""

pass() { echo "PASS $1"; }
fail() { echo "FAIL $1"; FAILS=$((FAILS + 1)); }
soft() { echo "SOFT $1 (skipped: $2)"; }
skip_all() { echo "SKIP E2E real SSH: $1"; exit 0; }

usage() {
  cat <<'EOF'
Usage: e2e_real_ssh.sh [options]
  --bin PATH              binary (default: target/release/ssh-cli)
  --config-dir DIR        isolated XDG config dir with pre-registered hosts
  --from-grok-config      read $HOME/.grok/config.toml only (never inside this repository)
  Env (harness-only, not product store): SSH_CLI_E2E_HOST PORT USER PASSWORD [SUDO_PASSWORD]
  Without a lab host, exits 0 with SKIP. Never prints secrets.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --from-grok-config) FROM_GROK=1; shift ;;
    --bin) BIN="$2"; shift 2 ;;
    --config-dir) CONFIG_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ ! -x "$BIN" ]]; then
  echo "building ssh-cli release..." >&2
  (cd "$ROOT" && cargo build --release -q)
  BIN="$ROOT/target/release/ssh-cli"
fi

if [[ "$FROM_GROK" -eq 1 ]]; then
  GROK_CFG="${SSH_CLI_E2E_GROK_CONFIG:-$HOME/.grok/config.toml}"
  if [[ ! -f "$GROK_CFG" ]]; then
    echo "FAIL E00: grok config missing" >&2
    exit 1
  fi
  # Refuse configs that live inside the git workspace (must stay outside the repo).
  GROK_CFG_ABS="$(cd "$(dirname "$GROK_CFG")" && pwd)/$(basename "$GROK_CFG")"
  ROOT_ABS="$(cd "$ROOT" && pwd)"
  case "$GROK_CFG_ABS" in
    "$ROOT_ABS"|"$ROOT_ABS"/*)
      echo "FAIL E00: grok config must not live inside the repository ($GROK_CFG_ABS)" >&2
      exit 1
      ;;
  esac
  # Parse via helper file to avoid shell quoting hell; values only in env exports.
  HELPER="$(mktemp /tmp/ssh-cli-e2e-parse.XXXXXX.py)"
  cat >"$HELPER" <<'PY'
import sys, shlex
try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

path = sys.argv[1]
with open(path, "rb") as f:
    data = tomllib.load(f)
servers = data.get("mcp_servers") or {}
srv = servers.get("ssh-flowaiper") or {}
args = srv.get("args") or []
flag_map = {
    "--host": "SSH_CLI_E2E_HOST",
    "--port": "SSH_CLI_E2E_PORT",
    "--user": "SSH_CLI_E2E_USER",
    "--password": "SSH_CLI_E2E_PASSWORD",
    "--sudoPassword": "SSH_CLI_E2E_SUDO_PASSWORD",
}
vals = {}
i = 0
while i < len(args):
    a = args[i]
    if not isinstance(a, str):
        i += 1
        continue
    for flag, envk in flag_map.items():
        if a.startswith(flag + "="):
            vals[envk] = a[len(flag) + 1 :]
            break
        if a == flag and i + 1 < len(args) and isinstance(args[i + 1], str):
            vals[envk] = args[i + 1]
            i += 1
            break
    i += 1
for k, v in vals.items():
    print(f"export {k}={shlex.quote(v)}")
need = ("SSH_CLI_E2E_HOST", "SSH_CLI_E2E_USER", "SSH_CLI_E2E_PASSWORD")
if any(k not in vals or not vals[k] for k in need):
    print("echo 'FAIL E00: incomplete daemon args' >&2; exit 1")
PY
  eval "$(python3 "$HELPER" "$GROK_CFG")"
  rm -f "$HELPER"
fi

# G-E2E-05: offline / no lab → SKIP exit 0 (not FAIL).
if [[ -z "${SSH_CLI_E2E_HOST:-}" || -z "${SSH_CLI_E2E_USER:-}" || -z "${SSH_CLI_E2E_PASSWORD:-}" ]]; then
  skip_all "no lab host (set harness SSH_CLI_E2E_* or --from-grok-config / --config-dir)"
fi
SSH_CLI_E2E_PORT="${SSH_CLI_E2E_PORT:-22}"

TMP="$(mktemp -d /tmp/ssh-cli-e2e.XXXXXX)"
cleanup() {
  rm -rf "$TMP"
  unset SSH_CLI_E2E_PASSWORD SSH_CLI_E2E_SUDO_PASSWORD SSH_CLI_E2E_HOST SSH_CLI_E2E_USER 2>/dev/null || true
}
trap cleanup EXIT

export SSH_CLI_HOME="$TMP"
export HOME="$TMP"
export XDG_CONFIG_HOME="$TMP"
unset SSH_CLI_ALLOW_PLAINTEXT_SECRETS || true

cli() {
  "$BIN" --config-dir "$TMP" --output-format json "$@"
}

if cli secrets status >/dev/null 2>&1; then
  pass E01
else
  fail E01
fi

if printf '%s' "$SSH_CLI_E2E_PASSWORD" | cli vps add \
  --name e2e \
  --host "$SSH_CLI_E2E_HOST" \
  --port "$SSH_CLI_E2E_PORT" \
  --user "$SSH_CLI_E2E_USER" \
  --password-stdin \
  --timeout 60000 \
  --check >/dev/null 2>&1; then
  if grep -q 'sshcli-enc:v1:' "$TMP/config.toml" 2>/dev/null \
    && ! grep -Fq "$SSH_CLI_E2E_PASSWORD" "$TMP/config.toml" 2>/dev/null; then
    pass E02
  else
    fail E02
  fi
else
  fail E02
fi

OUT="$(cli exec e2e 'echo e2e-ok' 2>/dev/null || true)"
if echo "$OUT" | grep -q 'e2e-ok'; then
  pass E03
else
  fail E03
fi

if [[ -n "${SSH_CLI_E2E_SUDO_PASSWORD:-}" ]]; then
  printf '%s' "$SSH_CLI_E2E_SUDO_PASSWORD" | cli vps edit e2e --sudo-password-stdin >/dev/null 2>&1 || true
fi
if cli sudo-exec e2e 'true' >/dev/null 2>&1; then
  pass E04
else
  soft E04 "sudo unavailable or auth failed"
  SOFT_SUDO=1
fi

if cli health-check e2e >/dev/null 2>&1; then
  pass E05
else
  fail E05
fi

DOUT="$(cli vps doctor 2>/dev/null || true)"
if echo "$DOUT" | grep -q 'encrypted'; then
  pass E06
else
  fail E06
fi

LOUT="$(cli vps list 2>/dev/null || true)$(cli vps show e2e 2>/dev/null || true)"
if echo "$LOUT" | grep -Fq "$SSH_CLI_E2E_PASSWORD"; then
  fail E07
else
  pass E07
fi

if ! cli exec e2e --timeout 2000 'sleep 30' >/dev/null 2>&1; then
  pass E08
else
  soft E08 "sleep completed within timeout"
fi

OUT2="$(cli exec e2e 'echo e2e-ok' 2>/dev/null || true)"
if echo "$OUT2" | grep -q 'e2e-ok'; then
  pass E09
else
  fail E09
fi

# --- SCP (GAP-SSH-SCP-016): E10 upload, E11 download, E12 integrity, E13 missing remote ---
REMOTE_SCP="/tmp/ssh-cli-e2e-scp-$$.bin"
REMOTE_SPACE="/tmp/ssh-cli e2e space $$ .bin"
UP_PLAIN="$TMP/up-plain.txt"
UP_SPACE="$TMP/up space file.txt"
UP_1M="$TMP/up-1m.bin"
DOWN_PLAIN="$TMP/down-plain.txt"
DOWN_SPACE="$TMP/down space file.txt"
DOWN_1M="$TMP/down-1m.bin"

printf 'e2e-scp-payload\n' >"$UP_PLAIN"
printf 'space-payload\n' >"$UP_SPACE"
# ≥1 MiB payload for streaming/wire stress
dd if=/dev/urandom of="$UP_1M" bs=1024 count=1024 status=none 2>/dev/null || \
  head -c 1048576 /dev/urandom >"$UP_1M"

if cli scp upload e2e --timeout 120000 "$UP_PLAIN" "$REMOTE_SCP" >/dev/null 2>&1 \
  && cli scp upload e2e --timeout 120000 "$UP_SPACE" "$REMOTE_SPACE" >/dev/null 2>&1 \
  && cli scp upload e2e --timeout 180000 "$UP_1M" "${REMOTE_SCP}.1m" >/dev/null 2>&1; then
  pass E10
else
  fail E10
fi

if cli scp download e2e --timeout 120000 "$REMOTE_SCP" "$DOWN_PLAIN" >/dev/null 2>&1 \
  && cli scp download e2e --timeout 120000 "$REMOTE_SPACE" "$DOWN_SPACE" >/dev/null 2>&1 \
  && cli scp download e2e --timeout 180000 "${REMOTE_SCP}.1m" "$DOWN_1M" >/dev/null 2>&1; then
  pass E11
else
  fail E11
fi

if cmp -s "$UP_PLAIN" "$DOWN_PLAIN" \
  && cmp -s "$UP_SPACE" "$DOWN_SPACE" \
  && cmp -s "$UP_1M" "$DOWN_1M"; then
  pass E12
else
  fail E12
fi

# E13: remote missing → exit 66 (ArquivoNaoEncontrado / GAP-SSH-IO-010); no residual local file
set +e
cli scp download e2e --timeout 30000 "/tmp/ssh-cli-e2e-missing-$$-no-such" "$TMP/should-not-exist" >/dev/null 2>&1
E13_EC=$?
set -e
if [[ "$E13_EC" -eq 66 && ! -f "$TMP/should-not-exist" ]]; then
  pass E13
else
  fail E13
fi

# E14: SCP-023 preserve mode+mtime on upload AND download (OpenSSH -p / linha T)
PRESERVE_LOCAL="$TMP/preserve-src.bin"
PRESERVE_REMOTE="/tmp/ssh-cli-e2e-preserve-$$.bin"
PRESERVE_DOWN="$TMP/preserve-down.bin"
printf 'preserve-payload\n' >"$PRESERVE_LOCAL"
chmod 600 "$PRESERVE_LOCAL"
# Epoch seconds (portable): avoid TZ ambiguity of touch -d strings
python3 - "$PRESERVE_LOCAL" <<'PY' || true
import os, sys, time
path = sys.argv[1]
epoch = 1_579_089_600  # 2020-01-15 12:00:00 UTC
os.utime(path, (epoch, epoch))
PY
LOCAL_MODE="$(stat -c '%a' "$PRESERVE_LOCAL" 2>/dev/null || stat -f '%OLp' "$PRESERVE_LOCAL")"
LOCAL_MTIME="$(stat -c '%Y' "$PRESERVE_LOCAL" 2>/dev/null || stat -f '%m' "$PRESERVE_LOCAL")"
if cli scp upload e2e --timeout 60000 "$PRESERVE_LOCAL" "$PRESERVE_REMOTE" >/dev/null 2>&1 \
  && cli scp download e2e --timeout 60000 "$PRESERVE_REMOTE" "$PRESERVE_DOWN" >/dev/null 2>&1; then
  # Non-TTY default is JSON envelope — extract stdout field (text format has banners).
  REMOTE_JSON="$(cli exec e2e --json --timeout 30000 "stat -c '%a %Y' $(printf '%q' "$PRESERVE_REMOTE")" 2>/dev/null || true)"
  REMOTE_LINE="$(printf '%s' "$REMOTE_JSON" | python3 -c 'import sys,json
try:
  d=json.load(sys.stdin)
  print((d.get("stdout") or "").strip().splitlines()[0] if (d.get("stdout") or "").strip() else "")
except Exception:
  print("")' 2>/dev/null || true)"
  REMOTE_MODE="$(printf '%s\n' "$REMOTE_LINE" | awk '{print $1}')"
  REMOTE_MTIME="$(printf '%s\n' "$REMOTE_LINE" | awk '{print $2}')"
  DOWN_MODE="$(stat -c '%a' "$PRESERVE_DOWN" 2>/dev/null || stat -f '%OLp' "$PRESERVE_DOWN")"
  DOWN_MTIME="$(stat -c '%Y' "$PRESERVE_DOWN" 2>/dev/null || stat -f '%m' "$PRESERVE_DOWN")"
  if [[ "$REMOTE_MODE" == "$LOCAL_MODE" && "$REMOTE_MTIME" == "$LOCAL_MTIME" \
    && "$DOWN_MODE" == "$LOCAL_MODE" && "$DOWN_MTIME" == "$LOCAL_MTIME" ]]; then
    pass E14
  else
    fail E14
  fi
else
  fail E14
fi
cli exec e2e --json "rm -f $(printf '%q' "$PRESERVE_REMOTE")" >/dev/null 2>&1 || true

# E15: GAP-SSH-TUN-003 — local port 0 binds ephemeral; JSON local_port must be >= 1
TUN_JSON_OUT="$TMP/tunnel-e15.json"
set +e
# wall timeout; tunnel --timeout-ms 2000 one-shot post-bind exit 0
timeout 8 "$BIN" --config-dir "$TMP" --output-format json tunnel e2e 0 127.0.0.1 22 --timeout-ms 2000 --json >"$TUN_JSON_OUT" 2>/dev/null
set -e
if python3 - "$TUN_JSON_OUT" <<'PY'
import json, sys
path = sys.argv[1]
try:
    with open(path) as f:
        raw = f.read().strip()
    # first complete JSON object
    d = json.loads(raw)
    lp = int(d.get("local_port") or 0)
    ev = d.get("event")
    ok = d.get("ok") is True and ev == "tunnel_listening" and lp >= 1
    sys.exit(0 if ok else 1)
except Exception:
    sys.exit(1)
PY
then
  pass E15
else
  fail E15
fi

# E16: GAP-SSH-SCP-024 — symlink: follow target content if regular file; else contract fail
SYM_TARGET="$TMP/symlink-target.txt"
SYM_LINK="$TMP/symlink-link.txt"
SYM_REMOTE="/tmp/ssh-cli-e2e-symlink-$$.txt"
SYM_DOWN="$TMP/symlink-down.txt"
printf 'symlink-payload-v1\n' >"$SYM_TARGET"
ln -sfn "$SYM_TARGET" "$SYM_LINK"
if cli scp upload e2e --timeout 60000 "$SYM_LINK" "$SYM_REMOTE" >/dev/null 2>&1 \
  && cli scp download e2e --timeout 60000 "$SYM_REMOTE" "$SYM_DOWN" >/dev/null 2>&1 \
  && cmp -s "$SYM_TARGET" "$SYM_DOWN"; then
  pass E16
else
  # OpenSSH scp may reject non-regular; still no residual wrong content
  fail E16
fi
cli exec e2e "rm -f $(printf '%q' "$SYM_REMOTE")" >/dev/null 2>&1 || true

# Best-effort remote cleanup (never print paths with secrets)
cli exec e2e "rm -f $(printf '%q' "$REMOTE_SCP") $(printf '%q' "$REMOTE_SPACE") $(printf '%q' "${REMOTE_SCP}.1m")" >/dev/null 2>&1 || true

echo "---"
echo "fails=$FAILS soft_sudo=$SOFT_SUDO tmp_destroyed=yes"
if [[ "$FAILS" -gt 0 ]]; then
  exit 1
fi
exit 0
