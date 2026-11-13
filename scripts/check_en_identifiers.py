#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Fail if Rust sources still use banned Portuguese identifiers (src/ and tests/).

Allows: Message::pt() UI strings, serde(rename="...") wire keys, legacy erros re-export,
test fixture password data.

This gate has FIVE independent checks. Falsifying one proves only that one:
  1. banned Portuguese identifier declarations   (PATTERN)
  2. Portuguese UI/error literals outside i18n
  3. Portuguese product literals outside i18n
  4. residual Portuguese fn/const names
  5. Portuguese grammar in identifiers, across src/ AND tests/

Checks 1 and 4 were dead from introduction until D5 (a double-escaped regex written in
shell single quotes, where no escape processing happens). Each check must be probed
separately before this script may be reported as green.

Contract: stdout carries only the final OK line. Matches and the GAP verdicts go to
stderr. Exit 0 when clean, 1 when any check found something, 2 on usage error. All five
checks always run, so one invocation shows the whole board.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterator, List, Optional, Pattern, Sequence, Tuple

sys.dont_write_bytecode = True
# Explicit sys.path entry: `python -I` and other-directory runs do not add this folder.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import _gatekit as gk  # noqa: E402

# Identifier declarations that must not appear in src/. Matching is per line, so the
# allowlist is the pattern itself: only declaration keywords followed by a banned name.
PATTERN = re.compile(
    r"\b(fn|let|mut|struct|enum|const|pub fn|async fn|type)\s+"
    r"(cliente|saida|resultado|carregar|salvar|cancelado|terminado|idioma|conteudo"
    r"|escrever_atomico|aplicar_overrides|OpcoesScp|ClienteFake|CamadaConfig"
    r"|formatar_|mascarar_|empacotar_|validar_e_normalizar|normalizar_nfc|obter_flag)\b"
)

# The i18n subsystem is the one place Portuguese literals are legitimate. C3 split the
# tables into src/i18n/{en,pt}.rs, so the allowlist must match the directory too.
I18N_ALLOW = re.compile(r"src/i18n(\.rs|/)")

# Hardcoded Portuguese UI outside i18n::pt and Message pt arms.
UI_LITERALS = re.compile(r'"(erro ao |Senha:|\(não definida\)|falha ao )')

# Hardcoded PT product literals that must go through Message or EN technical errors.
# The escaped pipe is a literal `|` inside the second alternative.
PRODUCT_LITERALS = re.compile(
    r'primary-key pronta|"ausente"\|nome de file|nome de VPS inválido'
)

# Residual PT function/const names in product code.
RESIDUAL_NAMES = re.compile(
    r"\b(fn|const|pub fn|async fn)\s+"
    r"(verificar_tofu|comando_scp_remoto|plaintext_permitido|PREFIXO_ENC|gerar_completions"
    r"|ler_stdin_se|cifrar|decifrar|mapear_exit_status|interpretar_status_scp"
    r"|parse_linha_t_scp)\b"
)

# Check 5: Portuguese *grammar* in function names, across src/ and tests/.
#
# Checks 1 and 4 are denylists of specific words, so they only catch a name somebody
# already thought to ban. This one targets connective tissue instead of vocabulary:
# `_nao_`, `_sem_`, `_contem_` and friends are how Portuguese joins words and are rare
# in English identifiers. `testa_` is the Portuguese habit of prefixing the word "test"
# onto a `#[test]` function.
#
# `_do_` and `_no_` are deliberately absent: they are homographs of real English
# (`rejections_do_not`, `no_active_vps`), and a signal that fires on correct code gets
# the whole gate disabled. `_da_`, `_dos_`, `_das_`, `_dentro_` and `_esta_` have no
# English homograph in identifier position.
#
# It is a heuristic and does not pretend otherwise: a single Portuguese noun with no
# connective still passes.
PT_GRAMMAR = re.compile(
    r"\bfn\s+(testa_[a-z0-9_]*|[a-z0-9_]*"
    r"(_nao_|_sem_|_com_|_contem_|_rejeita|_falha\b|_arquivo_|_senha\b|_diretorio"
    r"|_ambigua|_invalida\b|_valida\b|_vazio\b|_lixo\b|_curto\b|_longo\b"
    r"|_da_|_dos_|_das_|_dentro_|_esta_)[a-z0-9_]*)"
)


def rust_files(root: Path, subdir: str) -> Iterator[Path]:
    """Every ``*.rs`` under ``root/subdir``, sorted, skipping hidden directories."""
    base = root / subdir
    if not base.is_dir():
        return
    for path in sorted(base.rglob("*.rs")):
        rel_parts = path.relative_to(root).parts
        if any(part.startswith(".") for part in rel_parts):
            continue
        yield path


def scan(
    root: Path,
    subdirs: Sequence[str],
    regex: Pattern[str],
    allow: Optional[Pattern[str]] = None,
) -> List[str]:
    """Return ``path:line:text`` for every matching line, minus allowlisted ones."""
    hits: List[str] = []
    for subdir in subdirs:
        for path in rust_files(root, subdir):
            rel = path.relative_to(root).as_posix()
            text = gk.read_text(path)
            for number, line in enumerate(text.splitlines(), start=1):
                if regex.search(line) is None:
                    continue
                record = f"{rel}:{number}:{line}"
                if allow is not None and allow.search(record) is not None:
                    continue
                hits.append(record)
    return hits


def run_checks(root: Path) -> List[Tuple[str, List[str]]]:
    """Run the five checks and return ``(verdict, matches)`` for each one that failed."""
    checks = [
        ("Portuguese-like identifiers found in src/", ["src"], PATTERN, None),
        (
            "Portuguese UI/error literals outside i18n",
            ["src"],
            UI_LITERALS,
            I18N_ALLOW,
        ),
        (
            "Portuguese product literals outside i18n",
            ["src"],
            PRODUCT_LITERALS,
            I18N_ALLOW,
        ),
        (
            "residual Portuguese function/const names in src/",
            ["src"],
            RESIDUAL_NAMES,
            None,
        ),
        (
            "Portuguese grammar in Rust identifiers (src/ or tests/)",
            ["src", "tests"],
            PT_GRAMMAR,
            None,
        ),
    ]
    failed: List[Tuple[str, List[str]]] = []
    for verdict, subdirs, regex, allow in checks:
        hits = scan(root, subdirs, regex, allow)
        if hits:
            failed.append((verdict, hits))
    return failed


def build_parser() -> argparse.ArgumentParser:
    return argparse.ArgumentParser(
        prog="check_en_identifiers.py",
        description="Fail if Rust sources under src/ and tests/ use Portuguese identifiers.",
    )


def main(argv: Optional[Sequence[str]] = None) -> int:
    gk.setup_utf8()
    gk.install_sigint_exit()
    build_parser().parse_args(argv)

    failed = run_checks(gk.repo_root())
    for verdict, hits in failed:
        for record in hits:
            gk.eprint(record)
        gk.eprint(f"GAP: {verdict}")
    if failed:
        return 1

    sys.stdout.write("EN identifier gate: OK\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
