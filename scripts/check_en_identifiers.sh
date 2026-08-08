#!/usr/bin/env bash
# Fail if product Rust sources still use banned Portuguese identifiers.
# Allows: Message::pt() UI strings, serde(rename="…") wire keys, legacy erros re-export, test fixture password data.
#
# This gate has FOUR independent checks. Falsifying one proves only that one:
#   1. banned Portuguese identifier declarations   (PATTERN)
#   2. Portuguese UI/error literals outside i18n
#   3. Portuguese product literals outside i18n
#   4. residual Portuguese fn/const names
# Checks 1 and 4 were dead from introduction until D5 (double-escaped regex in
# single quotes). Each of the four must be probed separately before this script
# may be reported as green — see tests/gaps_v063_gate_falsification.rs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Identifier patterns that must not appear as Rust identifiers in src/ (outside string literals is hard in shell;
# we search roughly and allowlist known-safe lines).
#
# D5: this pattern used to be written with `\\b` and `\\s` inside SINGLE quotes.
# The shell performs no escape processing inside '…', so `rg` received a literal
# backslash followed by `b` — a regex that matches a backslash character, which
# never occurs in these identifiers. Checks 1 and 4 of this script therefore
# could not fail, no matter what the source contained. Verified by control:
# `rg -e '\\b(fn)\\s+(main)\\b' src/main.rs` exits 1, while `\bfn\s+main\b`
# matches once. Single-escape form below is the one `rg` actually understands.
PATTERN='\b(fn|let|mut|struct|enum|const|pub fn|async fn|type)\s+(cliente|saida|resultado|carregar|salvar|cancelado|terminado|idioma|conteudo|escrever_atomico|aplicar_overrides|OpcoesScp|ClienteFake|CamadaConfig|formatar_|mascarar_|empacotar_|validar_e_normalizar|normalizar_nfc|obter_flag)\b'

if rg -n --type rust -e "$PATTERN" src/ 2>/dev/null; then
  echo "GAP: Portuguese-like identifiers found in src/" >&2
  exit 1
fi

# The i18n subsystem is the one place Portuguese literals are legitimate.
# C3 split the tables into src/i18n/{en,pt}.rs; an allowlist naming only
# src/i18n.rs turned the split into a false positive. Match the directory too.
I18N_ALLOW='src/i18n(\.rs|/)'

# Hardcoded Portuguese UI outside i18n::pt and Message pt arms
if rg -n --type rust '"(erro ao |Senha:|\\(não definida\\)|falha ao )' src/ | rg -v "$I18N_ALLOW"; then
  echo "GAP: Portuguese UI/error literals outside i18n" >&2
  exit 1
fi

# Hardcoded PT product literals that must go through Message or EN technical errors
if rg -n --type rust 'primary-key pronta|"ausente"\|nome de file|nome de VPS inválido' src/ | rg -v "$I18N_ALLOW"; then
  echo "GAP: Portuguese product literals outside i18n" >&2
  exit 1
fi

# Residual PT function/const names in product code (D5: same double-escape bug as check 1).
if rg -n --type rust -e '\b(fn|const|pub fn|async fn)\s+(verificar_tofu|comando_scp_remoto|plaintext_permitido|PREFIXO_ENC|gerar_completions|ler_stdin_se|cifrar|decifrar|mapear_exit_status|interpretar_status_scp|parse_linha_t_scp)\b' src/ 2>/dev/null; then
  echo "GAP: residual Portuguese function/const names in src/" >&2
  exit 1
fi

echo "EN identifier gate: OK"
