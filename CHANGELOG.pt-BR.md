# Changelog

- Leia este documento em [Inglês (en)](CHANGELOG.md).

Todas as mudanças notáveis deste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2] - 2026-07-19

### Adicionado
- **`--json` global** (G-AUD-01): alias agent-friendly que força JSON (clap `from_global` nos subcomandos).
- **`exec` / `sudo-exec` / `su-exec` com VPS ativo** (G-AUD-04): um posicional = COMMAND no host ativo de `connect`.
- **Envelope JSON de `vps path`** (G-AUD-02): `event: vps-path` quando o formato é JSON.
- **Módulo `fs_perm`** (G-AUD-24): fonte única para modos Unix de arquivos/dirs de segredo.
- **Comandos root `schema` + `doctor`** (G-E2E-02/03): descoberta de contrato por agentes; `doctor` é alias de `vps doctor`.
- **`vps add --use-agent` / `--agent-socket`** (G-E2E-19): triplo de auth no inventário (senha / chave / agent).

### Corrigido
- **Warning falso de password em argv** (G-AUD-08): inspeciona `Option` real, não strings `Debug`.
- **TLS PEM ausente** (G-AUD-05): `FileNotFound` / classe permanente (não exit 74 retryable).
- **`vps export` honra formato JSON global** (G-AUD-03).
- **ACME account create exige `--contact mailto:…`** (G-AUD-06/28).
- **Exclusão mútua de auth primária** na gravação (G-AUD-07): exatamente um de password / key / agent.
- **Mensagens de secrets** não anunciam mais stores env (G-AUD-21).
- **Filtro de log só via CLI** (G-AUD-22 / G-E2E-09): `RUST_LOG` ambiente é **ignorado**; use `-v`.
- **Cap de concorrência** com fonte única (G-AUD-19/23): `constants::MAX_CONCURRENCY`.
- **Skill description ≤1024** chars (G-AUD-15).
- **ACME validação permanente** (G-E2E-01): `invalidContact` / tipos de problema 4xx → exit **64** não-retryable (`tls/acme_error_map.rs`).
- **Um único JSON em `vps add` com auto-key** (G-E2E-04): campo `secrets_key_auto_created` embutido em `vps-added` (um documento; nunca dois eventos).
- **Stamp de versão `-dirty` com `.commit_hash`** (G-E2E-06): proveniência honesta em trees dirty.
- **Feature clap `env` removida** (G-E2E-08); help não ensina mais stores env (G-E2E-07).
- **Máscara de export redacted** (G-E2E-10): `***` via `FIXED_MASK`, não string vazia.
- **Harness E2E offline SKIP** + bin release default (G-E2E-05); identificadores de teste em EN (G-E2E-13).

### Removido
- **`.github/workflows`** (G-AUD-11 / G-E2E-11): só gates locais; sem CI/GH Actions de produto na tree.
- **Shim PT `src/erros.rs`** (G-AUD-14).
- **Leituras env de config de produto** `SSH_CLI_HOME` / `SSH_CLI_LANG` / `SSH_CLI_FORCE_TEXT` (G-AUD-12).

### Alterado
- Versão **0.5.1 → 0.5.2**.
- Testes de integração alinhados a config só CLI/XDG (sem store env de secrets/formato).
- Gate residual: `tests/gaps_v058_e2e_residual.rs` (G-E2E-01…15,17,19 FIXED; 16/18 MITIGATED).

### G-SFTP residual harden R01–R15

### Segurança
- Validação de **basename de entry** + `ensure_local_under` em download recursivo/multi-file (servidor SFTP malicioso não escapa destino local).
- **Cleanup de partial** em qualquer erro de download SFTP (paridade SCP).
- **Root de upload tree** com `symlink_metadata` (no-follow).

### Alterado
- **Timeout wall-clock** (`under_timeout`) em multi-file e FS ops.
- **`cli/scp_args.rs`** extraído (SRP).
- Docs/skills: SCP = arquivos regulares; árvores/FS = **`sftp`**.

### G-SFTP: subsistema SFTP

### Adicionado
- **`russh-sftp` 2.3** + `ssh-cli sftp` (upload/download/`--recursive`, ls/mkdir/rmdir/rm/stat/rename).
- Schemas JSON `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`.
- Gate `tests/gaps_v057_sftp.rs`.
- Agent em `ScpOptions`/`SftpOptions` (CLI/XDG).

### Segurança
- Stream 32 KiB (sem heap full-file); paths validados; recursive depth cap; symlink no-follow.

### G-SSH: regras SSH / russh

### Adicionado
- **`client_handler` / `client_connect` / `key_material`:** TOFU tipado, cadeia de auth, perms de chave.
- **Agent CLI/XDG:** `--use-agent`, `--agent-socket` (sem env como store).
- **Gates:** `tests/gaps_v056_ssh.rs`.

### Alterado
- client_id genérico `SSH-2.0-ssh-cli`; rekey/window/TCP keepalive explícitos; deny ban ssh2/thrussh.

### Segurança
- `HostKeyChanged` tipado; fail-closed known_hosts; RSA ≥2048; password só se secret non-empty no inventário.

### G-UNSAFE: unsafe code e FFI

### Adicionado
- **`test_util::env`:** encapsula `set_var`/`remove_var` com `// SAFETY:`.
- **`vps/config_io.rs`:** split de path/load/save (SRP).
- **Gates:** `tests/gaps_v055_unsafe_ffi.rs`.

### Alterado
- **`main`:** `register_handler` **antes** do Tokio multi_thread (G-UNSAFE-13).
- SAFETY de SIGTERM expandido; docs Windows FFI; testes plaintext via `set_runtime_flags`.
- Docs secrets/concurrency sem env-as-store; `forbid(unsafe_code)` em módulos puros.

### Segurança
- Allowlist de `unsafe` de produto: windows console + signals; env de teste encapsulado.

### G-ERR: tratamento de erros

### Adicionado
- Variantes `Domain`/`Crypto`/`Config`; TLS/canal com `source`; `error_code` no envelope JSON; gates `gaps_v054`; split do client SSH.

### Alterado
- Display minúsculo; `paths` tipado; validate de VPS com `DomainError`; secrets sem env-as-store; concurrency sem env store.

### G-DOM: tipos de domínio chrono/uuid/rust_decimal/url

### Adicionado
- **Quatro crates de domínio (coordenadas):** `chrono` 0.4.45, `uuid` 1.24 (v4+v7+serde), `rust_decimal` 1.42 (serde-with-str), `url` 2.5 (serde).
- **`src/domain/` dividido (SRP):** time, ids, http_url, money, names, ports, limits, command, error.
- **`Rfc3339Utc`:** timestamps VPS/ACME como `DateTime<Utc>` (wire RFC 3339).
- **`HttpsUrl` / `AcmeOrderUrl`:** parse HTTPS para resume ACME no XDG.
- **`BatchRunId` (v7):** campo `batch_run_id` nos JSON batch multi-host.
- **`Money<C>`:** biblioteca decimal (sem superfície monetária no VPS).
- **Gates:** `tests/gaps_v053_domain_types.rs` + proptest.

### Alterado
- Schemas batch exigem `batch_run_id`; import valida RFC 3339 em `added_at`.

### Segurança
- Sem `Local::now`; sem `serde-float`; URLs ACME só `https`.

### G-TLS produto: rustls / SSH-over-TLS / mTLS / ACME

### Adicionado
- **Feature `tls` (padrão):** `rustls` ≥ 0.23.18 + `aws_lc_rs`, `tokio-rustls`, `webpki-roots`, `rustls-pki-types`, `instant-acme`.
- **`CryptoProvider::install_default`** no `main` do binário (somente aws_lc_rs).
- **SSH-over-TLS**, **mTLS** (XDG) e **ACME** DNS-01 em dois passos (agent-friendly).
- Subcomando `ssh-cli tls …` e campos VPS `tls` / `tls_sni` / cert+key.

### Alterado
- `deny.toml` permite rustls de produto; ban mantém OpenSSL/native-tls/ring.
- PEM via `rustls-pki-types` (sem `rustls-pemfile`).

### G-TLS / política rustls — sessão anterior

### Adicionado
- **`src/ssh/connect.rs`:** helper único de Config + dial Happy Eyeballs (G-TLS-07/09).
- **Suite residual** `tests/gaps_v052_tls_policy.rs` (G-TLS-03).
- **SECURITY Política de transporte e crypto (G-TLS)** — SSH ≠ TLS; aws-lc-rs; rustls futuro só com ADR.

### Alterado
- **Compressão SSH só `none`** (G-TLS-04).
- **russh:** remove feature `flate2` (G-TLS-05); mantém `aws-lc-rs`.
- **`deny.toml`:** ban `openssl`, `ring`, `rustls` além de `openssl-sys` / `native-tls` / `libssh2-sys` (G-TLS-02).
- README / CROSS_PLATFORM / RELEASE_CHECKLIST / llms: superfícies de política crypto (G-TLS-01/06/08/11/12).

### Segurança
- Sem stack TLS de produto; sem OpenSSL/`native-tls`/`ring`/`rustls` no grafo.
- Sem OTEL de produto.

### Sistema de Tipos

### Adicionado
- **Newtypes de domínio (G-TYPE-01…20):** `src/domain/` com `VpsName`, `SshHost`, `SshUser`, `SshPort(NonZeroU16)`, `TimeoutMs`, `HostTag`, `CharLimit`, `RemoteCommand`, `KeyPath`, `BindPort`.
- **`ssh/session_io.rs`:** extração de helpers UTF-8 (G-TYPE-14).
- Testes de layout zero-cost para `SshPort`.

### Alterado
- **`VpsRecord` / `ConnectionConfig`:** campos com prova de tipo; `try_new` no lugar de `new` infalível.
- **`HostSelection`:** tipado com `VpsName` / `HostTag`.
- **`ExecOptions` / `ScpOptions`:** `TimeoutMs` e `RemoteCommand`.
- **CLI:** portas SSH com range 1..=65535; bind local ainda aceita 0.
- **`validate_and_normalize` → `VpsName`**.
- **Import JSON:** host/user vazios rejeitados na fronteira.

### Segurança
- Helper único `secret_nonempty`; sem OTEL de produto.

### Notas de sessão (validação / serde)


### Adicionado
- **Pipeline de validação (G-SERDE-01…14):** `validator` 0.20 + `serde_with` 3 + `serde_path_to_error` + `serde_ignored`; módulo `src/validation.rs`.
- **Tags no JSON agent (G-SERDE-06):** list/export/import com round-trip.
- **Validação estrutural no load (G-SERDE-04).**
- **Fuzz** `import_envelope` (G-SERDE-12).
- **`ssh/connection.rs`** e **`cli/tests.rs`** (G-COMP-R).

### Alterado
- **deny_unknown_fields** no TOML crítico; import JSON Must-Ignore com warn.
- **Arc\<ScpOptions\>** no fan-out multi-host SCP (G-MEM-SCP).
- **Actions CI pinados por SHA** (G-PROC-PIN).

### Segurança
- Sem telemetria de produto. Secrets em `SecretString`.

- Read this document in [English](CHANGELOG.md).

Todas as mudanças notáveis deste projeto são documentadas neste arquivo.

O formato segue [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e o versionamento segue [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### Prior closeout / process notes

### Changed
- **O1–O6 + processo (obrigatório):** `--fail-fast`; tags de host; multi-cmd `--step` na mesma sessão; SCP `--scp-file-concurrency`; Arc de options no fan-out; proptest/fuzz; CI miri/geiger/sbom; `scripts/release_attest.sh`. Zero telemetria de produto.
- **Componentização profunda (G-COMP-05 / G-COMP-06a–d / G-CLOSE-09 / G-DRY-01 / G-EN-R01):** extraídos `vps/exec_ops.rs` (exec/sudo/su + DRY `finish_execution_output`); `ssh/scp_wire.rs`; `scp/{mod,batch}.rs`; `output/{mod,batch}.rs`; `cli/{mod,dispatch}.rs`; `commands/*` com reexports reais. Renomes EN residuais. OPEN de segurança de produto permanece 0; monólitos inventoriáveis fatiados.
- **Componentização (G-COMP-02…04):** extraídos `vps/doctor.rs`, `vps/import_export.rs` e `vps/health.rs` do monólito `vps` (`mod.rs` ~2428 → ~1698 LOC); reexport de `HostHealthResult` / `run_health_check`.

### Segurança
- **Meta-auditoria de fechamento (G-CLOSE):** casts de doctor/concurrency/SCP via `TryFrom`; `forbid(unsafe_code)` nos módulos puros restantes; extração `vps/selection.rs` (SRP); reexecução context7 + docsrs-cli + duckduckgo para conformidade da skill.
- **Auditoria de segurança de desenvolvimento (G-SECDEV):** secrets atravessam a fronteira CLI como `SecretString` (`read_secret_stdin` + overrides de exec/scp/tunnel/health); módulos puros com `#![forbid(unsafe_code)]`; deny de `clippy::mem_forget` + unsafe sem SAFETY / multi-op; mapa STRIDE + preferência CVSS v4 em `SECURITY.md` (+ pt-BR).
- **Auditoria de segurança defensiva (G-SEC):** `deny(unsafe_op_in_unsafe_fn)`;
  `overflow-checks` em release; comparação constant-time de fingerprint TOFU;
  caminhos de produto sem `.unwrap`/`.expect`/`unreachable!` em parsers CLI,
  admissão de concorrência e ramos single-host; porta de import via
  `u16::try_from`; `SshCliError` `#[non_exhaustive]`; modelo de ameaça em
  `SECURITY.md` (+ pt-BR); job CI `cargo deny check` (`deny.toml`).

### Added
- **Auditoria de retry (G-RETRY):** classificação tipada de erros (`ErrorClass` /
  `ErrorLayer` / `RetryKind`, `is_retryable` / `is_permanent` / `suggestion`) em
  `SshCliError`; `retry::RetryConfig` nomeado com backoff full-jitter e defaults
  de agente (máx. 2 retries no exit 74); envelope JSON com `error_class`,
  `retryable`, `suggestion` + schema. Auto-retry in-process de ops remotas não
  idempotentes permanece **desligado** (agente reinvoca o processo).

### Corrigido
- **Auditoria de rede (G-NET):** dial SSH com DNS assíncrono + corrida multi-endereço
  Happy Eyeballs (`net::dial_tcp` + `russh::client::connect_stream`); `TCP_NODELAY` e
  keepalives SSH (`15s` / máx. `3`); carga de chave privada e TOFU de known_hosts em
  `spawn_blocking`; accept do tunnel resiste a erros transitórios e aplica nodelay nos
  forwards locais.


### Alterado
- **Auditoria de hardcode (G-HC):** módulo central `constants` (nomes XDG, env keys,
  identidade do app, defaults de rede, timing de processo, AEAD/keyring); helper
  único `paths::xdg_config_dir()`. Sem segredos/URLs de produto no binário; hosts
  continuam no registry/CLI.

### Corrigido
- **Auditoria de processos externos (G-PROC):** probes `git` em `build.rs` com
  `Stdio` explícito (null/piped); comandos remotos rejeitam NUL antes do packing
  de exec SSH; fixtures de teste `ssh-keygen` usam argv direto + stdio explícito
  e fazem skip se o binário estiver ausente.
- Docs: política de fronteira de processo em CROSS_PLATFORM / AGENTS (sem spawn
  local OpenSSH; MSRV ≥ 1.77.2 BatBadBut; packing remoto `sh -c` só no host alvo).

### Adicionado
- **Concorrência multi-host com bound (modus operandi):** `health-check|exec|sudo-exec|su-exec|scp --all` faz fan-out com `Semaphore` + `JoinSet` (cap de `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` / fórmula auto CPUs×RAM, clamp 1..=64). JSON batch: `health-check-batch` / `exec-batch` / `scp-batch` (`docs/schemas/*-batch.schema.json`). Forwards de accept do tunnel usam o mesmo gate.
- **Seleção seletiva `--hosts a,b,c`:** mesmo fan-out e JSON batch que `--all` (mesmo com um nome); unificado via `HostSelection` + `resolve_host_jobs`.
- **SCP multi-arquivo (single-host, G-PAR-47):** uma **sessão SSH** e transfers seriais (auth uma vez).
- **SCP multi-host × multi-arquivo (G-PAR-48):** `scp upload --all f1 f2 … REMOTE_DIR` — bound por sessão host; arquivos seriais na sessão.
- **TOFU flock (G-PAR-49):** mutações de `known_hosts` com lock exclusivo + reload-merge.
- **`vps doctor --probe-ssh [--hosts a,b]`:** um único root JSON `event: vps-doctor` com `local` + `ssh_probe` opcional (sem dual roots).
- **`map_bounded` cancel:** para admissão em SIGINT/SIGTERM; `force_exit` aborta JoinSet; span `fan_out_unit` + `available_permits`.
- Docs/skills de agente: frota multi-host + multi-arquivo / cartesiano SCP + envelope doctor.

### Alterado
- Path SCP (validação e pós-download) usa `tokio::fs` / `spawn_blocking` (não bloqueia workers sob fan-out).
- `scripts/dist_multiarch.sh` suporta `PARALLEL_JOBS` (default 2) via `xargs -P`.

## [0.5.1] - 2026-07-17

### Corrigido
- **Roundtrip export/import agent-first**: corpo default de `vps export` é **TOML** mesmo em non-TTY; JSON só com `--json`. Import aceita TOML (chaves EN+PT) e envelopes JSON `vps-export` (GAP-AUD-001/022).
- **Wire dual-read**: deserializa EN + aliases PT legados; serializa chaves em inglês; schema **v3**; default `added_at` quando ausente (GAP-AUD-002/021). Substitui a nota de wire 0.5.0 (chaves PT só via `serde(rename)`).
- **JSON de `secrets init` / `reencrypt`** (`event: secrets-init|secrets-reencrypt`) via `--json` ou `--output-format json` (GAP-AUD-003).
- Erro de comando vazio é técnico em inglês (`empty command`) em qualquer locale (GAP-AUD-004).
- Caminhos de sucesso CRUD/connect/import emitem JSON estruturado quando o formato é JSON (GAP-AUD-008).
- Mensagem SCP remoto ausente normalizada para `file not found: <path>` (GAP-AUD-025); EC 66 mantido.
- Erros de parse TOML no import mapeiam para sysexits **65** (`TomlDe`) (GAP-AUD-012).
- Exit de `SshAuthentication` alinhado a **77** (GAP-AUD-020).
- Timeouts `< 1000` ms emitem warning em stderr (GAP-AUD-009).
- `--include-secrets` em pipe/non-TTY exige `--output` ou `--i-understand-secrets-on-stdout` (GAP-AUD-011).
- Doctor `secrets_plaintext_opt_out` é JSON **bool** (GAP-AUD-013).
- Hardcodes/tracing residuais em inglês técnico (GAP-AUD-005).

### Adicionado
- Flags CLI: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (camadas env depreciadas, ainda funcionam) (GAP-AUD-006).
- Evento `secrets-key-auto-created` quando a primary-key é provisionada na primeira gravação (GAP-AUD-007).
- Tunnel `--bind` (default `127.0.0.1`) (GAP-AUD-018).
- Warning em stderr de password em argv (GAP-AUD-010).

### Alterado
- Versão **0.5.0 → 0.5.1**.
- Tracing / identificadores residuais padronizados em inglês (GAP-AUD-005).
- Aliases de tipo em português no módulo `erros` marcados como deprecated (GAP-AUD-017).

### Notas
- Sem publish crates.io/GitHub sem OK explícito do maintainer.
- Contratos reais de transferência SCP de 0.5.0 §1.1 não devem regredir.

## [0.5.0] - 2026-07-15

### Corrigido
- **CRÍTICO**: `secrets init --force` reencripta hosts existentes e grava `secrets.key.bak` (GAP-AUD-SEC-001).
- Doctor `permissions` em inglês (`"missing"`).
- Mensagens técnicas, help clap e identificadores residualmente em EN.
- Nomes de VPS com whitespace interno rejeitados (GAP-AUD-VAL-001).

### Alterado
- Semver **0.5.0** por renomeações de API em inglês. Wire TOML ainda usava chaves PT via `serde(rename)` nesta release (**supersedido em 0.5.1** por serialize EN + dual-read EN/PT, schema v3).
- `secrets init` / `reencrypt` via `Message` i18n.

### Notas
- Sem publish crates.io/GitHub sem OK explícito.

## [0.4.2] - 2026-07-15

### Corrigido
- **Tunnel porta efêmera** (`local_port=0`): após bind, JSON/banner reportam a porta **atribuída pelo SO** via `local_addr()` (nunca `0` pós-bind) (GAP-SSH-TUN-003). Schema `local_port.minimum` = 1.
- **SCP remote missing** agora sai com **66** `ArquivoNaoEncontrado` (paridade com missing local) em vez de **74** `CanalFalhou` quando o OpenSSH reporta `No such file` / `not found` (GAP-SSH-IO-010). Erros de protocolo/permissão permanecem 74.

### Adicionado
- `vps export --json` envelope agent-first: `event: "vps-export"`, hosts redacted por padrão, sem `sshcli-enc:` para secrets vazios (GAP-SSH-UX-001 / paridade EXP-001); schema `docs/schemas/vps-export.schema.json`
- Embed de commit hash no pack crates.io: `build.rs` com precedência env → `.commit_hash` → git → `unknown` (GAP-SSH-REL-007)
- e2e oficial **E15** (tunnel porta 0) + **E16** (symlink) + E13 exige exit **66**; política ENV-001/fail2ban no header do script
- Suite `tests/gaps_v042_integration.rs`

### Alterado
- Versão 0.4.1 → **0.4.2**
- Docs/skills: tunnel continua com args **posicionais**; porta `0` = efêmera; confiar em `local_port` do JSON; nunca inventar `--local-port` (GAP-SSH-DOC-042)

### Segurança / honestidade
- Ban TCP na VPS após e2e de auditoria foi **fail2ban** por senhas erradas intencionais (ENV-001), **não** TUN-003.
- Sem telemetria

### Notas
- CLI one-shot: nascer → executar → morrer
- Contratos agent aditivos (PATCH)


## [0.4.1] - 2026-07-15

### Corrigido
- **Export redacted com secret vazio** não emite mais ciphertext `sshcli-enc:v1:…` para senha `""` (GAP-SSH-EXP-001).
- **Deadline do tunnel** após bind local não retorna mais exit **74** quando o agente já recebeu `tunnel_listening` (GAP-SSH-TUN-002). Timeout pré-bind permanece 74.

### Adicionado
- Paridade de flags auth em `tunnel`: `--password-stdin`, `--key-passphrase`, `--key-passphrase-stdin` (GAP-SSH-CLI-005)
- Paridade de flags auth em `health-check`: `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin` (GAP-SSH-CLI-006)
- Campo JSON SCP `event: \"scp-transfer\"` + schema obrigatório (GAP-SSH-IO-009)
- Suite `tests/gaps_v041_integration.rs`
- `health-check` honra `--replace-host-key` global e envelope JSON de erro com `--json`

### Alterado
- Versão 0.4.0 → **0.4.1**
- Docs/skills de product line com paridade auth e event scp-transfer

### Segurança / honesty
- **Se instalou 0.4.0 do crates.io:** export redacted podia mostrar ciphertext falso de senha vazia; tunnel podia emitir `ok:true` e sair 74. Atualize para **0.4.1**.
- Sem telemetria

### Notas
- CLI one-shot: nascer → executar → morrer
- Contratos agent aditivos apenas (PATCH)

## [0.4.0] - 2026-07-15

### Corrigido
- **Protocolo wire SCP** quebrado no crates.io **0.3.9** (header com `\\n` literal em vez de newline real `0x0a`; ACK/EOF com data vazia em vez do byte `0x00`; status remoto não validado; download com header/terminador incorretos) — SCP-010..013
- Escape shell do path remoto SCP para espaços e meta-caracteres (SCP-014)
- Unit tests não cristalizam mais o header quebrado (SCP-015)
- Download não deixa arquivo final parcial em falha: grava `{path}.ssh-cli.partial` e faz rename atômico (SCP-022); mode/times aplicados no **partial** antes do rename (SCP-022b)
- Upload não carrega o arquivo inteiro em RAM (`fs::read`); stream em chunks de 32 KiB (SCP-018)
- `scp --json` habilita envelope JSON de erro em stderr (paridade com tunnel; IO-007b)
- Mensagens de validação file-only do SCP em i18n EN/PT (SCP-020b)

### Adicionado
- E2E oficial E10–E14 SCP em `scripts/e2e_real_ssh.sh` (upload, download, `cmp`, remoto ausente, preserve mode/mtime) (SCP-016, SCP-023)
- Paridade de flags scp com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json` (SCP-017)
- JSON estruturado de sucesso SCP + `docs/schemas/scp-transfer.schema.json` (IO-007, SCP-021)
- Preserve mtime/mode bi-direcional: remoto `scp -tp`/`-fp`, linha `T` + parse mode `C`, set_permissions + set_times (SCP-023/023b; e2e E14)
- `tunnel --json` emite evento estruturado `tunnel_listening` após bind local (IO-008)
- Mensagens i18n EN/PT de sucesso SCP (SCP-020)
- Suite `tests/gaps_v040_integration.rs` (TEST-004)

### Alterado
- Versão 0.3.9 → **0.4.0**
- Docs de product line documentam **somente arquivos regulares** (sem `-r` / sem SFTP) e a regressão wire SCP de 0.3.9 (DOC-004, SCP-019, REL-004)
- Honestidade da raiz (SECURITY 0.4.x atual, INTEGRATIONS superfície real 0.4.0, CONTRIBUTING gaps_v040) (DOC-004b)
- Honestidade de `docs/*`: AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION/TESTING/RELEASE_CHECKLIST/CROSS_PLATFORM + índice de schemas cobrem SCP file-only, partial, stream 32 KiB, preserve, `scp --json`, `tunnel --json` / `tunnel_listening` e aviso wire 0.3.9 (DOC-004c)
- Honestidade de `skills/*`: skills bilíngues + evals ensinam SCP file-only, JSON scp-transfer, `.ssh-cli.partial`, stream 32 KiB, preserve mtime/mode, tunnel `--json` / `tunnel_listening`, matriz de flags de timeout (DOC-004d)
- Adicionado `docs/schemas/tunnel-listening.schema.json` para o contrato de agente IO-008
- `scp` honra `--replace-host-key` global e `--output-format json` global

### Segurança / honestidade
- **Se você instalou 0.3.9 do crates.io e usou `scp`:** essa release anunciava SCP, mas o wire era inoperante (upload frequentemente gerava arquivo remoto 0 bytes ou timeout). Atualize para **0.4.0**.
- Sem telemetria

### Notas
- CLI one-shot: conectar → transferir → desconectar → sair
- Arquivos grandes: aumente `--timeout` (cobre connect + transferência completa)

## [0.3.9] - 2026-07-15

### Corrigido
- Residuais da auditoria pós-0.3.8: LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL-003, CHG-001
- Tracing default **error** (agent-first); `-v` ativa debug (LOG-001)
- stderr JSON sem prosa INFO por omissão (LOG-001)
- VPS só-chave: `password: null` no JSON (não `"***"`) (JSON-001)
- `health-check --timeout <ms>` alinhado ao exec (CLI-004)
- Docs de product line em **0.3.9** e comportamentos residuais documentados em README, `llms*.txt`, INTEGRATIONS, `docs/*` e skills (auditoria profunda DOC-003)
- Âncoras de compare do CHANGELOG para 0.3.8/0.3.9 (CHG-001)
- `deny.toml` documenta warns multi-version esperados sem ignore de CVE (DENY-002)
- `docs/schemas/vps-show.schema.json` permite `password` com tipo `string | null` (paridade JSON-001)
- Higiene de exposição SEC-001..003: ignore `.setting.cyber/`, E2E recusa grok config no repo, docs usam `demo-password-not-real`

### Adicionado
- Suite `tests/gaps_v039_integration.rs` para gaps residuais de auditoria (incl. SEC-001..003)

### Alterado
- Versão 0.3.8 → 0.3.9
- `exclude` do Cargo inclui `.setting.cyber/` e sidecars sqlite do enrich-queue

### Notas
- Sem telemetria
- Credenciais reais ficam fora da árvore (`~/.config/ssh-cli/`, `$HOME/.grok/config.toml`)

## [0.3.8] - 2026-07-15

### Corrigido
- Gaps residuais pós-auditoria 0.3.7 (IO-006, EXIT-002, VAL-004, TEST-004, DOC-001, REL-001/002, DENY-001, PROC-001, E2E-001)
- Banners do tunnel não poluem stdout de agentes (IO-006)
- Sem VPS ativa retorna exit 66 tipado (EXIT-002)
- Parse OpenSSH de key_path no write-path (VAL-004)
- Suite `gaps_v038_integration` 1:1 (TEST-004)
- Version string com `-dirty` se tree suja (REL-002)
- Inventário `gaps.md` versionado; checklist `docs/RELEASE_CHECKLIST.md`

### Segurança
- Upgrade **russh 0.62.2** (piso ≥0.60.3); remove pins COMPAT RC (DEP-002)
- `cargo deny` sem waivers CVE/yanked; remove license morta Unicode-DFS-2016
- Gate install exige russh patched; permite primefield estável
- crossbeam-epoch ≥0.9.20 (RUSTSEC-2026-0204)

### Alterado
- Versão 0.3.7 → 0.3.8
- Política de `verify_install_resolve.sh` invertida

### Notas
- Sem telemetria
- Fixes de produto 0.3.7 não commitados entram neste commit de release


### Adicionado
- Framework completo de documentação bilíngue (README, CONTRIBUTING, SECURITY, INTEGRATIONS, guias docs, schemas, skills)
- Arquivos de licença dual `LICENSE-MIT` e `LICENSE-APACHE` com MIT OR Apache-2.0

## [0.3.7] - 2026-07-15

### Corrigido
- Todos os 23 gaps de `gaps.md` (VAL/IO/TUN/SCP/STATE/PERM/CLI/TEST/EXIT/SEC/DEP/IMP)
- Write-path de domínio: `validar_e_normalizar`, porta 1..=65535, chave existente (VAL-001..003)
- I/O: `--output-format` no CRUD VPS, `health-check --json`, envelope JSON de erro, `--quiet` silencia sucesso humano, `println!` só em `output` (IO-001..005)
- Tunnel: `--timeout-ms` cobre connect + loop (TUN-001)
- SCP valida arquivo local antes do connect (SCP-001)
- `vps remove` limpa `active` órfão; lock `0o600` (STATE-001, PERM-001)
- `su-exec --password-stdin`; conflitos clap password/*_stdin; completions EPIPE seguro (CLI-001..003)
- Testes de sinais `#[serial]`; snapshot help; assert real de abort (TEST-001..003)
- Falha de comando remoto → exit do processo `1` (não o código remoto) (EXIT-001)
- Senha sudo/su no stdin do canal, não na argv; máscara sempre `***` (SEC-001, SEC-002)
- Import redacted com UX + `--allow-incomplete` (IMP-001)
- `cargo deny` verde com política de pins datada (DEP-001)

### Alterado
- Versão 0.3.6 → 0.3.7
- **Quebra de contrato (agentes):** senhas longas não expõem 12+4; exit remoto ≠0 vira processo `1` com `remote_exit_code` no envelope
- `SSH_CLI_FORCE_TEXT=1` força formato texto

### Segurança
- Sem senha sudo/su em `ps` remoto
- Sem vazamento de prefixo de senha em list/show

## [0.3.6] - 2026-07-15

### Adicionado
- Cifragem at-rest por padrão: auto `secrets.key` (0o600) na primeira gravação
- CLI `secrets status|init|reencrypt` (nunca imprime master-key)
- Opt-out `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1` para testes
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`
- Script `scripts/e2e_real_ssh.sh` para E2E real sem logar credenciais
- Mensagem de auth falha orienta stdin/key

### Alterado
- Versão 0.3.5 → 0.3.6
- GAP-009 residual: cifragem default (não só opcional)
- Documentação de pin freeze russh/crypto (R-PINS)

### Segurança
- Segredos no TOML cifrados por padrão
- Protocolo E2E proíbe vazar host/user/password

## [0.3.5] - 2026-07-15

### Corrigido
- Residual GAP-007: `vps export` atômico
- Residual GAP-006: abort remoto TERM+KILL
- Residual GAP-009/012: cifragem opcional at-rest (env/file/keyring)
- README sem install sem `--locked`
- Matriz de paridade do gaps.md atualizada

### Adicionado
- Overrides `--key-passphrase` em exec/sudo-exec/su-exec
- JSON automático fora de TTY
- Doctor com `secrets_at_rest` / `secrets_key_source`
- Testes `tests/gaps_v035_integration.rs`

### Alterado
- Versão 0.3.4 → 0.3.5

## [0.3.4] - 2026-07-15

### Fixed
- Grafo crypto de `cargo install`: pin `primefield`, `primeorder`, `ecdsa`, `pkcs5`, `russh = 0.60.0` exato (GAP-014)
- Packing de `sudo-exec` com `sh -c`  (GAP-005)
- Escrita atômica de `config.toml` com tempfile + fsync + flock (GAP-007)
- Host key TOFU via `known_hosts` XDG (GAP-008)
- Dual `max_command_chars` / `max_output_chars` (GAP-004)
- Abort remoto best-effort no timeout (GAP-006)
- Validação de credencial: password ou key obrigatório (GAP-011)

### Added
- Auth por chave privada (`--key`, `key_path`) via russh `load_secret_key` (GAP-002)
- `su-exec` one-shot consumindo `senha_su` (GAP-003)
- Segredos via stdin (`--password-stdin` e pares sudo/su) (GAP-009)
- `vps doctor`, `vps export`, `vps import` (GAP-012)
- Tunnel com `--timeout-ms` obrigatório (GAP-010)
- `--disable-sudo`, `--description`, `--replace-host-key`
- Schema v2 multi-host XDG
- Gate de install: `scripts/verify_install_resolve.sh`

### Changed
- Timeout default 60000 ms 
- `directories` 5 → 6 (GAP-013)
- Versão 0.3.3 → 0.3.4
- Dual license MIT OR Apache-2.0

## [0.3.3] - 2026-07-15

### Changed
- Migração de ownership e repositório para `danilo-aguiar-br` após ban da conta GitHub anterior.
- `repository` / `homepage` apontam para `https://github.com/danilo-aguiar-br/ssh-cli`.
- Metadados de autor atualizados para `Danilo Aguiar <daniloaguiarbr@proton.me>`.
- Workflows GitHub Actions e badges de CI removidos.

### Note
- crates.io já tinha versões até `0.3.2` da conta anterior; este release é o primeiro sob o novo owner.

## [0.2.1] - 2026-04-16

### Fixed
- Pin `elliptic-curve = "=0.14.0-rc.30"` para corrigir falha de `cargo install ssh-cli`

## [0.2.0] - 2026-04-15

### Added
- Fix de piping de senha sudo-exec com `printf '%s\n'`
- Overrides de runtime em exec/sudo-exec/scp/tunnel
- Aliases camelCase para LLMs

## [0.1.0] - 2026-04-14

Release inicial.

[Unreleased]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.2...HEAD
[0.5.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.9...v0.4.0
[0.3.9]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.3.3
[0.2.1]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/danilo-aguiar-br/ssh-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/danilo-aguiar-br/ssh-cli/releases/tag/v0.1.0
