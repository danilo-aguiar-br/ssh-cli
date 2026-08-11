#!/usr/bin/env bash
# Fail if Rust sources still use banned Portuguese identifiers (src/ and tests/).
# Allows: Message::pt() UI strings, serde(rename="…") wire keys, legacy erros re-export, test fixture password data.
#
# This gate has FIVE independent checks. Falsifying one proves only that one:
#   1. banned Portuguese identifier declarations   (PATTERN)
#   2. Portuguese UI/error literals outside i18n
#   3. Portuguese product literals outside i18n
#   4. residual Portuguese fn/const names
#   5. Portuguese grammar in identifiers, across src/ AND tests/
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

# Check 5: Portuguese *grammar* in test function names, across src/ and tests/.
#
# Why this is separate from checks 1 and 4: those are denylists of specific words,
# so they can only catch a name somebody already thought to ban. Measured before
# this check existed: about thirty test functions carried names like
# `gap_scp_019_upload_diretorio_rejeita` and `export_redacted_nao_contem_senha`.
# None appeared in either denylist, and neither denylist looked at `tests/` at all
# — the four checks above scan `src/` only, which is where the product lives and
# is exactly half of where its identifiers live.
#
# This targets connective tissue instead of vocabulary: `_nao_`, `_sem_`, `_contem`
# and friends are how Portuguese joins words, and they are rare enough in English
# identifiers to be a usable signal. `testa_` is the Portuguese habit of prefixing
# the word "test" onto a `#[test]` function, which is redundant in either language.
#
# It is a heuristic and does not pretend otherwise: a single Portuguese noun with
# no connective still passes. A gate that catches the common shape is worth more
# than a denylist that catches only the shapes already fixed.
#
# Extended after two survivors were found by reading, not by any gate:
# `gap_sec_001b_todo_sidecar_pontuado_esta_barrado_do_pacote` and
# `gap_sec_002_e2e_recusa_grok_config_dentro_do_repo`. Neither carried a listed
# connective, and neither carried an accent, so the diacritic gate in
# `tests/gaps_v068_source_language.rs` could not see them either.
#
# The obvious additions are the Portuguese contractions, and two of them are traps.
# Measured on this tree before adding: `_do_` occurs four times, two of them in
# real English (`rejections_do_not`, `software_do_not_collide`), and `_no_` occurs
# thirty-nine times, essentially all English (`no_active_vps`). Both are homographs
# and are deliberately absent — a signal that fires on correct code gets the whole
# gate disabled. `_da_`, `_dos_`, `_das_`, `_dentro_` and `_esta_` have no English
# homograph in identifier position, so they are added and they measure zero today.
PT_GRAMMAR='\bfn\s+[a-z0-9_]*(_nao_|_sem_|_com_|_contem_|_rejeita|_falha\b|_arquivo_|_senha\b|_diretorio|_ambigua|_invalida\b|_valida\b|_vazio\b|_lixo\b|_curto\b|_longo\b|_da_|_dos_|_das_|_dentro_|_esta_|^testa_)[a-z0-9_]*'
if rg -n --type rust -e "$PT_GRAMMAR" src/ tests/ 2>/dev/null; then
  echo "GAP: Portuguese grammar in Rust identifiers (src/ or tests/)" >&2
  exit 1
fi

echo "EN identifier gate: OK"
