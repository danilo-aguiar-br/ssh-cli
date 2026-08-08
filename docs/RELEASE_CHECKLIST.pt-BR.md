# Checklist de release — ssh-cli

> **0.5.4** — release de segurança e agent-native. Corrige DoS remoto pré-auth no banner SSH (A1), impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3), fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2) e adiciona flags de redução de payload (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) aplicadas antes da serialização. BREAKING: falha parcial multi-host agora sai com exit **1** (era 65); `--bind` fora do loopback exige `--i-accept-network-exposure`. Novo evento `tunnel_closed`.


> Gates obrigatórios antes de marcar release. O inventário local `gaps.md` é só do mantenedor (não publicado).

- Leia este documento em [inglês](RELEASE_CHECKLIST.md).
- Alvo de release / linha de produto: **0.5.4**.
- Gate histórico: **0.4.1** DOC-041 / honestidade AUD-POST (export empty, tunnel exit 0, paridade de auth, evento scp-transfer).
- Inventário de auditoria local (não publicado): `gaps.md` — **gitignored** e excluído do cargo; mantenedores podem manter localmente. Testes não devem assertar seu texto FIXED (G13/G15).
- Suites residuais: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG); `tests/gaps_v040_integration.rs` (SCP-010..023, DOC-004, IO-007/007b/008, REL-004, TEST-004); `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006); `tests/gaps_v042_integration.rs` (AUD-E2E); `tests/gaps_v051_integration.rs` (wire/export/secrets 0.5.2); `tests/gaps_v056_ssh.rs` + `tests/gaps_v057_sftp.rs` (G-SSH / G-SFTP); `tests/gaps_v058_e2e_residual.rs` (residual G-E2E: schema/doctor root, um único `vps-added`, `--use-agent`, purge de env help/clap, FIXED_MASK, ACME 64).


## Propósito
- Impedir ship com gaps abertos, docs de product line defasados ou waivers de supply-chain.
- Manter evidência de release honesta (notas pré/pós-fix locais, sem segredos em logs).
- Alinhar versão Cargo, `--version`, product line de docs, tags e âncoras do CHANGELOG.


## Gates (obrigatórios)

0. Bateria completa de gates — `bash scripts/check_all_gates.sh` exit 0 com os dez gates verdes e zero pulados. Ela cobre os itens 1 a 5 mais a checagem cross-target, o gate de frescor de advisories e o build `--no-default-features` numa invocação. Rode isso antes de marcar qualquer item abaixo: `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila, então um único arquivo de teste quebrado esconde o estado de todo gate atrás dele. `scripts/check_advisory_freshness.sh` não tem outro chamador, então pular a bateria pula esse gate inteiro.
1. Build de release — `cargo build --release` exit 0.
2. Clippy limpo — `cargo clippy --all-targets -- -D warnings` exit 0.
3. Identificadores em inglês — `bash scripts/check_en_identifiers.sh` exit 0.
4. Supply chain deny (DENY-002) — `cargo deny check` exit 0; sem `ignore` de CVE russh; `yanked=deny`; `ignore = []` vazio.
4b. **Política crypto G-TLS (gates locais apenas — sem GitHub Actions obrigatório):**
    - `deny.toml` bane `openssl`, `openssl-sys`, `native-tls`, `libssh2-sys`, `ring`, `rustls`.
    - `cargo tree -i rustls`, `-i openssl`, `-i ring`, `-i native-tls` sem package.
    - `cargo tree -i flate2` sem package (compressão SSH só `none`; sem feature flate2 do russh).
    - `cargo test --locked --test gaps_v052_tls_policy` verde.
    - SECURITY.md / SECURITY.pt-BR.md com **Transport & crypto policy (G-TLS)**.
4c. **Build cross-target (B1 — obrigatório):** `bash scripts/check_cross_targets.sh` exit 0.
    - Type-check de `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` e `x86_64-apple-darwin`.
    - Todo outro gate desta lista roda somente no triple do host.
    - Código sob `#[cfg(target_os = ...)]` alheio é descartado antes do type-check e fica invisível para eles.
    - O alvo Windows já falhou com seis erros com esta checklist inteira verde.
    - Windows usa `--no-default-features`: a pilha TLS padrão puxa `aws-lc-sys`, que compila C (A8).
    - `docs/CROSS_PLATFORM.pt-BR.md` não pode declarar alvo que este script não verifica.
5. Install resolve — `bash scripts/verify_install_resolve.sh` exit 0; russh no piso de segurança (≥ 0.60.3; linha de produto usa 0.62.5).
6. Testes completos — `cargo test --locked --all-targets` verde (lib + integration + gaps_v037…v042 + **gaps_v051** + **gaps_v052** + **gaps_v056** + **gaps_v057** + **gaps_v058**).
6b. Formatação (G10) — `cargo fmt --check` exit 0.
7. Suites residuais de gaps verdes — todos os testes em `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, `tests/gaps_v040_integration.rs`, `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs`, `tests/gaps_v052_tls_policy.rs`, `tests/gaps_v056_ssh.rs`, `tests/gaps_v057_sftp.rs` `tests/gaps_v058_e2e_residual.rs` e `tests/gaps_v064_gate_runner.rs` passam (incluindo testes que não se chamam `gap_*`).
8. E2e local (sem VPS real) — help, CRUD fake de VPS, completions se comportam como documentado.
9. Smoke VPS real (quando disponível) — `health-check` / `exec` mais matriz SCP **E10–E14** e SFTP **E17/E18** (matriz completa **E01–E18**) via `scripts/e2e_real_ssh.sh` quando houver credenciais; prefira sshd local / VPS throwaway; sem tempestade de auth em produção; registrar resultado em notas locais do mantenedor sem segredos (`gaps.md` é só local).
10. Inventário não publicado — `gaps.md` **NÃO DEVE** ser publicado; está **gitignored** e excluído do cargo (`/gaps.md` em `.gitignore` e exclude do `Cargo.toml`). `git check-ignore gaps.md` **deve bater** (exit 0). PROIBIDO: testes circulares que assertam texto FIXED dentro de `gaps.md` (G13/G15).
11. Evidência pré/pós-fix honesta para mantenedores (DOC-002 / integridade) — notas só locais; não embarque segredos.
12. String de versão (REL-002) — `ssh-cli --version` bate com versão Cargo + hash git; reporta `-dirty` quando a tree está suja.
13. Commit e tag locais de release (REL-003) — `git status` limpo no commit de release; mensagem HEAD de Release; tag local `vX.Y.Z` (para 0.5.4: `v0.5.4`); sem push remoto sem autorização.
14. Sem telemetria — `vps doctor --json` reporta `"telemetry": false`; sem SDKs de métricas/telemetria na tree.
15. Probes temporários removidos — sem artefatos `_probe_*` sobrando na tree.
16. Tracing default error (LOG-001) — nível default é error (não info); stderr em modo tunnel/JSON é só envelope (sem banners INFO como "Tunnel SSH:" / "iniciando tunnel"). Verbosidade graduada `-v`/`-vv`/`-vvv` com escopo na crate (G2/G14); `RUST_LOG` ambiente ignorado.
17. Docs de product line = versão Cargo (DOC-003) — toda superfície de product line declara **0.5.4**, incluindo:
    - `llms.txt`, `llms.pt-BR.txt`, `llms-full.txt`
    - `README.md`, `README.pt-BR.md`
    - `INTEGRATIONS.md`, `INTEGRATIONS.pt-BR.md`
    - `docs/AGENTS.md`, `docs/AGENTS.pt-BR.md`
    - `docs/HOW_TO_USE.md`, `docs/HOW_TO_USE.pt-BR.md`
    - `docs/COOKBOOK.md`, `docs/COOKBOOK.pt-BR.md`
    - `docs/MIGRATION.md`, `docs/MIGRATION.pt-BR.md`
    - `docs/TESTING.md`, `docs/TESTING.pt-BR.md`
    - `docs/CROSS_PLATFORM.md`, `docs/CROSS_PLATFORM.pt-BR.md`
    - `docs/schemas/README.md`
    - `docs/RELEASE_CHECKLIST.md`, `docs/RELEASE_CHECKLIST.pt-BR.md`
18. Senha vazia em JSON é null (JSON-001) — runtime: `vps show|list --json` em host só-chave emite `"password": null` (não `"***"`); não vazia permanece mascarada `***`. Schema: `docs/schemas/vps-show.schema.json` (e list via `$ref`) declara tipo de `password` como `string` | `null`.
19. Timeout de health-check (CLI-004) — `health-check --timeout <ms>` é aceito (parse clap), alinhado a overrides do exec; coberto por gaps_v039.
20. Âncoras do CHANGELOG (CHG-001) — `CHANGELOG.md` tem seção `## [0.5.4]` e âncoras de compare/rodapé para 0.5.4 (e 0.5.3 / 0.5.2 / 0.4.x / 0.3.9 anteriores conforme necessário).
21. Package dry-run opcional — `cargo package --allow-dirty --list` ok; nunca auto-publish.
22. DOC-004 / honestidade SCP (0.4.0+) — superfícies de product line documentam:
    - SCP **somente arquivos regulares** (sem diretórios / sem `-r`); árvores via `sftp --recursive`
    - crates.io **0.3.9** anunciava SCP mas o wire estava quebrado; não prometa SCP funcional em 0.3.9
    - `docs/schemas/scp-transfer.schema.json` existe e está indexado (`docs/schemas/README.md`, `llms-full.txt`)
    - sufixo partial de download **`.ssh-cli.partial`**
    - `tunnel --json` / `tunnel_listening` e/ou superfície JSON de scp em README/INTEGRATIONS/AGENTS
    - skills bilíngues `skills/ssh-cli-en` e `skills/ssh-cli-pt` ensinam scp-transfer, tunnel_listening, file-only, partial, 32 KiB, matriz de timeout (DOC-004d)
    - SECURITY Supported Versions marca **0.5.x** como linha atual (não 0.3.x)
    - `cargo test --locked --test gaps_v040_integration` + `gaps_v041_integration` verde
23. DOC-041 / honestidade AUD-POST (0.4.x histórico) — superfícies de product line e agentes documentam:
    - `vps export` redacted **nunca** documenta nem espera `sshcli-enc:` para segredos vazios
    - deadline pós-bind do tunnel sai com exit **0** após `tunnel_listening` (sucesso one-shot; não 74)
    - paridade de flags de auth de `tunnel` / `health-check` documentada (`--password-stdin`, overrides de chave / passphrase conforme aplicável)
    - schema `scp-transfer` **exige** `event: "scp-transfer"`
    - `cargo test --locked --test gaps_v041_integration` verde
24. DOC-051 / honestidade 0.5.2 (histórico) — superfícies de product line documentam:
    - o corpo de `vps export` segue o formato resolvido: JSON `event: "vps-export"` em qualquer stdout non-TTY; TOML somente com `--output-format text`
    - wire **schema v3** dual-read (serialize EN / aliases PT no load)
    - schemas de secrets `secrets-init.schema.json` / `secrets-reencrypt.schema.json` indexados
    - tunnel `--bind` padrão `127.0.0.1`
    - exit **77** para auth; exit **65** para `TomlDe` / import ruim
    - flags de secrets `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferidas ao env
    - `--include-secrets` exige `-o` ou `--i-understand-secrets-on-stdout`
    - `cargo test --locked --test gaps_v042_integration` + `gaps_v051_integration` verde
25. DOC-053 / honestidade 0.5.3 — superfícies de product line documentam:
    - integridade de upload SFTP corrigida (G1); prefira 0.5.3+; verifique com `sha256` no destino
    - SETSTAT atime+mtime (G3), set_metadata fail-closed (G4), máscara de perms (G12)
    - verbosidade graduada `-v`/`-vv`/`-vvv` com escopo na crate (G2/G14); `RUST_LOG` ignorado
    - cardinalidade de cancel em batch (G5/G17); `exec --json` um único objeto (G8); SCP `sync_data` (G9)
    - E2E **E17/E18** SFTP checksum/árvore (G7)
    - sem testes circulares de `gaps.md` (G13/G15); `gaps.md` gitignored
    - `cargo fmt --check` verde (G10)
    - `cargo test --locked --test gaps_v056_ssh` + `gaps_v057_sftp` + `gaps_v058_e2e_residual` verde
    - árvore completa de comandos da CLI documentada em HOW_TO_USE + AGENTS + COOKBOOK (incluindo `locale show|set|clear` + `tls mtls/*` / `tls acme/*` completos)
    - G6 `serial_test` / isolamento determinístico da suite residual
    - G11 baseline verde (`cargo test --locked --all-targets` + suites residuais)
    - G16 identificadores SCP em inglês (wire/schema/docs usam nomes EN)
    - G18 `set_permissions` fail-closed no download SFTP
    - G19 constante nomeada `SFTP_PERM_MASK` (`0o7777`, saída/upload) e `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`, entrada/download; A3)
    - descoberta `ssh-cli commands` documentada (superfície de inventário para agentes)
    - docs de product line listam `locale` + árvore `tls` completa (mtls + acme account/issue/complete/status/list)
26. DOC-054 / honestidade 0.5.4 — superfícies de product line documentam:
    - os quatro modos de `tunnel`: forward local padrão, `--reverse` (G-TUN-R01), `--socks5` (G-TUN-R02), `--remote-socket <PATH>` (G-TUN-R03); as três flags são mutuamente exclusivas
    - o rótulo JSON `mode` é um de `local` / `reverse` / `socks5` / `streamlocal`
    - `--reverse` com `REMOTE_PORT 0` deixa o servidor alocar e informar a porta; um forward local não aceita 0
    - `--socks5` e `--remote-socket` omitem `REMOTE_HOST`/`REMOTE_PORT` (passá-los é exit **64**); `--remote-socket` exige caminho absoluto
    - um `--bind` roteável exige `--i-accept-network-exposure` (G-TUN-R13), **inclusive** em forward local — não só em `--reverse`
    - sob `--reverse` o guard cobre o bind do **servidor** (o posicional `<remote_host>`), que é comparado como texto e não parseado como IP (typo → exit 64, não exit 2); o próprio `--bind` é aceito e descartado em silêncio
    - evento `tunnel_closed` com `reason`, `forwards_served`, `capacity_waits` — o único discriminador entre os três finais com exit 0
    - redução de payload aplicada antes da serialização: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
    - `--dry-run` é honrado por `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init`, `secrets reencrypt` — e **somente** por esses; nos demais é exit 64
    - seletor de frota `--tags` só em `exec` / `sudo-exec` / `su-exec`, mutuamente exclusivo com `--all` e `--hosts`
    - BREAKING: falha parcial multi-host sai com exit **1** (era 65; G-ERR-R02)
    - A1 aplica o teto de log de 512 caracteres do banner, que já existia, em fronteira de caractere (remove um abort disparável remotamente sob `panic = "abort"`); A2 cria chaves ACME/mTLS em `0600`; A3 mascara os modos de **entrada** com `SFTP_PERM_MASK_UNTRUSTED`
    - `cargo test --locked --test docs_conformance` verde (incluindo os gates de superfície 0.5.4, seletor de frota e máscara direcional)


## Como verificar residuals rápido

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v056_ssh
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo fmt --check
cargo deny check
bash scripts/check_en_identifiers.sh
bash scripts/verify_install_resolve.sh
ssh-cli --version
git check-ignore gaps.md   # deve bater (exit 0)
```

- LOG-001: tunnel com `--output-format json` falha sem conectar; stderr tem envelope JSON e sem prosa INFO.
- JSON-001: show JSON de host só-chave contém `"password": null`; arquivo de schema contém null no tipo de password.
- CLI-004: `health-check --timeout 50` não é "unexpected argument".
- DOC-003: arquivos de product line (incluindo este par de checklists) contêm `0.5.4`.
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mencionam SCP file-only e aviso de wire 0.3.9; schema scp-transfer presente.
- DOC-004d: `skills/ssh-cli-en` e `skills/ssh-cli-pt` ensinam scp-transfer, tunnel_listening, file-only, partial, stream 32 KiB e matriz de timeout; evals cobrem a superfície.
- DOC-041: export redacted de segredos vazios sem `sshcli-enc:`; deadline pós-bind do tunnel exit 0 após `tunnel_listening`; paridade de flags de auth tunnel/health documentada; schema scp-transfer exige `event`; gaps_v041 verde.
- DOC-051: redaction do export; formato segue --output-format; schema v3; schemas secrets; `--bind` loopback; exit 77; flags secrets; include-secrets; gaps_v042 + gaps_v051 verdes.
- DOC-053 / 0.5.3: integridade SFTP G1; verbosidade G2/G14; E17/E18; gaps.md gitignored; `cargo fmt --check`; gaps_v056/v057 verdes; árvore CLI completa (locale + tls mtls/acme) em HOW_TO_USE/AGENTS/COOKBOOK; G6 serial_test; G11 baseline; G16 ids SCP EN; G18 set_permissions; G19 `SFTP_PERM_MASK`; descoberta `ssh-cli commands`.
- DENY-002: `deny.toml` tem `yanked = "deny"`, `ignore = []`, política multiple-versions documentada.
- CHG-001 / REL: seção `## [0.5.4]` no CHANGELOG + tag local `v0.5.4` sem push não autorizado.
- TEST-004 / SCP: gaps_v040 cobre wire, schema, path partial, preserve, script e2e E10–E14.
- AUD-POST / gaps_v041: suite residual EXP-001, TUN-002, CLI-005/006, IO-009, REL-006 verde.
- 0.5.2 / gaps_v051: roundtrip de redaction do export, JSON `vps-export`, dual-read, evento secrets-init, guarda include-secrets, CRUD `vps-added`, empty command, import exit 65.
- G-E2E / gaps_v058: root `schema` / `doctor`, um único `vps-added` + `secrets_key_auto_created`, `--use-agent`, `RUST_LOG` ambiente ignorado, FIXED_MASK `***`, ACME exit 64.
- G10: `cargo fmt --check` verde.
- Identificadores EN: `scripts/check_en_identifiers.sh` exit 0.


## Política

- PROIBIDO: declarar inventário Fechado enquanto qualquer gap permanecer Aberto (notas locais do mantenedor).
- PROIBIDO: waiver eterno de RUSTSEC / CVE sem tracking fechado na mesma release.
- PROIBIDO: `git push` ou publish no crates.io sem autorização explícita do mantenedor.
- PROIBIDO: logar ou colar segredos reais em notas do inventário, checklist ou logs de CI.
- PROIBIDO: publicar `gaps.md` ou assertar seu texto FIXED em testes de produto (G13/G15).
- OBRIGATÓRIO: escritas multi-linha de inventário local / CHANGELOG usam atomwrite (ou escrita atômica equivalente).
- OBRIGATÓRIO: Status Resolvido só com código + teste + nota de versão (evidência de destino, não texto de inventário).


## G-22 — Distribuição, SBOM, multi-arch (processo)

> Identidade do produto permanece **CLI one-shot** (crates.io + tag local). Multi-arch
> e SBOM assinado são gates de **processo de release**, não features de runtime.
> Scripts em `scripts/`; nunca fazem push ou publish.

### 28. Binários multi-arch de release (G-22)

- Config: [`Cross.toml`](../Cross.toml) na raiz — alvos:
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`
  - `aarch64-unknown-linux-gnu`
- Ferramentas: instale [`cross`](https://github.com/cross-rs/cross) + Docker.
- Matriz de build (local, sem push):

```bash
bash scripts/dist_multiarch.sh
# ou um único alvo:
TARGETS="x86_64-unknown-linux-musl" bash scripts/dist_multiarch.sh
```

- Artefatos: `target/dist/ssh-cli-<triple>` + sidecars `.sha256`.
- cargo-dist opcional: mantenedores **podem** adicionar `dist-workspace.toml` /
  `release.yml` do GitHub depois; até lá `cross` + `dist_multiarch.sh` é o caminho suportado.
- PROIBIDO: anexar builds debug unstripped como “release” sem documentar.

### 29. Geração de SBOM (G-22)

```bash
# Preferido: CycloneDX JSON
cargo install cargo-cyclonedx
bash scripts/generate_sbom.sh
# → target/sbom/ssh-cli.cdx.json (ou path argumento)
```

- Fallback sem cargo-cyclonedx: o script grava inventário `cargo tree` e avisa
  que **não** é um SBOM CycloneDX.
- Assinatura (mantenedor, offline):

```bash
# Exemplo com cosign keyless (precisa OIDC) ou chave local — escolha um padrão da org:
# cosign sign-blob --bundle target/sbom/ssh-cli.cdx.json.bundle target/sbom/ssh-cli.cdx.json
# gpg --detach-sign --armor target/sbom/ssh-cli.cdx.json
```

- Anexar SBOM + assinatura + binários multi-arch ao GitHub Release da tag `vX.Y.Z`
  **somente** após autorização explícita do mantenedor (mesma regra de `git push`).
- Publish no crates.io permanece separado (`cargo publish`); SBOM é evidência de release, não arquivo do crate.

### 30. Critérios de aceite G-22 (fechar inventário)

- [ ] `bash scripts/dist_multiarch.sh` produz pelo menos artefato musl x86_64 + sha256 (ou skip documentado quando Docker indisponível).
- [ ] `bash scripts/generate_sbom.sh` produz CycloneDX JSON **ou** inventário fallback documentado.
- [ ] Release notes / GitHub Release (quando autorizado) listam triples de binário + caminho do SBOM.
- [ ] Sem auto-push / sem auto-publish a partir dos scripts.


## Referência

- `gaps.md` — inventário de auditoria **somente local** do mantenedor (gitignored / excluído do cargo; não publicado)
- [../Cross.toml](../Cross.toml) — imagens de alvo cross-rs
- [../scripts/dist_multiarch.sh](../scripts/dist_multiarch.sh) — build multi-arch
- [../scripts/generate_sbom.sh](../scripts/generate_sbom.sh) — SBOM / inventário
- [../deny.toml](../deny.toml) — política de supply-chain
- [../scripts/check_all_gates.sh](../scripts/check_all_gates.sh) — runner da bateria obrigatória de gates (os dez gates, uma invocação)
- [../scripts/check_advisory_freshness.sh](../scripts/check_advisory_freshness.sh) — gate de frescor do banco de advisories
- [../scripts/verify_install_resolve.sh](../scripts/verify_install_resolve.sh) — gate de re-resolve de install
- [../scripts/check_en_identifiers.sh](../scripts/check_en_identifiers.sh) — gate residual de identificadores em inglês
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs) — gates residuais LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs) — gates residuais SCP/IO/DOC-004/REL-004
- [../tests/gaps_v041_integration.rs](../tests/gaps_v041_integration.rs) — gates residuais EXP-001/TUN-002/CLI-005/006/IO-009/REL-006 (DOC-041)
- [../tests/gaps_v042_integration.rs](../tests/gaps_v042_integration.rs) — gates residuais AUD-E2E (TUN-003, IO-010, ENV-001, SCP-024, …)
- [../tests/gaps_v051_integration.rs](../tests/gaps_v051_integration.rs) — gates residuais 0.5.2 export/schema v3/secrets
- [../tests/gaps_v056_ssh.rs](../tests/gaps_v056_ssh.rs) — gates residuais G-SSH
- [../tests/gaps_v057_sftp.rs](../tests/gaps_v057_sftp.rs) — gates residuais G-SFTP
- [../tests/gaps_v058_e2e_residual.rs](../tests/gaps_v058_e2e_residual.rs) — gates residuais G-E2E (schema/doctor root, um único `vps-added`, `--use-agent`, FIXED_MASK, ACME 64)
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json) — password `null` | mascarado `***`
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json) — JSON de sucesso SCP (só arquivos; exige `event`)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json) — evento de bind do tunnel
- [schemas/vps-export.schema.json](schemas/vps-export.schema.json) — só `vps export --json` (`event: "vps-export"`)
- [schemas/secrets-init.schema.json](schemas/secrets-init.schema.json) — `secrets init --json`
- [schemas/secrets-reencrypt.schema.json](schemas/secrets-reencrypt.schema.json) — `secrets reencrypt --json`
- [schemas/README.md](schemas/README.md) — índice de schemas (linha de produto 0.5.4)

## Gate residual de docs G-E2E (v0.5.3)

- [ ] `RUST_LOG` ambiente ignorado documentado; só `-v`/`-vv`/`-vvv` controla verbosidade (G2/G14)
- [ ] Um único JSON `vps-added` com campo `secrets_key_auto_created` (família G8)
- [ ] Root `schema` / `doctor` documentados
- [ ] `vps add --use-agent` documentado
- [ ] ACME `invalidContact` → exit 64 permanente documentado
- [ ] Export redacted `***` (`FIXED_MASK`) documentado
- [ ] E2E XDG-first + SKIP offline documentado em TESTING
- [ ] E2E **E17/E18** SFTP checksum/árvore documentado (G7)
- [ ] Integridade SFTP G1 + preferir 0.5.3+ documentado
- [ ] Árvore completa de comandos da CLI (incluindo `locale` + `tls mtls` / `tls acme` completos) documentada em HOW_TO_USE + AGENTS + COOKBOOK
- [ ] Descoberta `ssh-cli commands` documentada
- [ ] G6 `serial_test` / suite determinística; G11 baseline verde; G16 identificadores SCP em inglês
- [ ] G18 `set_permissions` fail-closed no download SFTP; G19 constante nomeada `SFTP_PERM_MASK`
- [ ] Docs de product line listam `locale` + árvore `tls` completa
- [ ] `gaps.md` gitignored (`git check-ignore gaps.md` bate); sem testes circulares de texto FIXED (G13/G15)
- [ ] `cargo fmt --check` nos gates de publish (G10)
- [ ] Suites `tests/gaps_v056_ssh.rs` + `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs` nos gates de publish


## Matriz multi-OS local (G-E2E-18)

- Código de produto: `src/platform/{linux,macos,windows}.rs` — sem matriz cloud de GitHub Actions.
- Multi-arch local: `scripts/dist_multiarch.sh` quando toolchains cross estiverem instalados.
- Valide notas de path length / agent socket em macOS e Windows antes de taggear um release.
- **Não** reintroduza `.github/workflows` para CI (política: CLI one-shot, sem CI cloud de produto).
