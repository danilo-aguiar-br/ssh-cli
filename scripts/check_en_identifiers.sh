#!/usr/bin/env bash
# Fail if product Rust sources still use banned Portuguese identifiers.
# Allows: Message::pt() UI strings, serde(rename="…") wire keys, legacy erros re-export, test fixture password data.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Identifier patterns that must not appear as Rust identifiers in src/ (outside string literals is hard in shell;
# we grep roughly and allowlist known-safe lines).
PATTERN='\\b(fn|let|mut|struct|enum|const|pub fn|async fn|type)\\s+(cliente|saida|resultado|carregar|salvar|cancelado|terminado|idioma|conteudo|escrever_atomico|aplicar_overrides|OpcoesScp|ClienteFake|CamadaConfig|formatar_|mascarar_|empacotar_|validar_e_normalizar|normalizar_nfc|obter_flag)\\b'

if rg -n --type rust -e "$PATTERN" src/ 2>/dev/null; then
  echo "GAP: Portuguese-like identifiers found in src/" >&2
  exit 1
fi

# Hardcoded Portuguese UI outside i18n::pt and Message pt arms
if rg -n --type rust '"(erro ao |Senha:|\\(não definida\\)|falha ao )' src/ | rg -v 'src/i18n.rs'; then
  echo "GAP: Portuguese UI/error literals outside i18n" >&2
  exit 1
fi

# Hardcoded PT product literals that must go through Message or EN technical errors
if rg -n --type rust 'primary-key pronta|"ausente"\|nome de file|nome de VPS inválido' src/ | rg -v 'src/i18n.rs'; then
  echo "GAP: Portuguese product literals outside i18n" >&2
  exit 1
fi

# Residual PT function/const names in product code
if rg -n --type rust -e '\\b(fn|const|pub fn|async fn)\\s+(verificar_tofu|comando_scp_remoto|plaintext_permitido|PREFIXO_ENC|gerar_completions|ler_stdin_se|cifrar|decifrar|mapear_exit_status|interpretar_status_scp|parse_linha_t_scp)\\b' src/ 2>/dev/null; then
  echo "GAP: residual Portuguese function/const names in src/" >&2
  exit 1
fi

echo "EN identifier gate: OK"
