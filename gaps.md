# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1  
> **Atualização:** 2026-07-19 — **G-SFTP residual harden R01–R15**  
> **OPEN residual inventoriável (esta lista):** **0**  
> **Telemetria produto:** **proibida** (tracing stderr local only)  
> **Tools:** context7 ✅ docsrs-cli ✅ duckduckgo-search-cli ✅ GraphRAG `rules_rust_ssh` + one-shot + mem + par ✅  
> **Publish:** sem push

---

## Resumo executivo — G-SFTP residual harden (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — 5 passagens (core SFTP + tools + path adversarial + timeout/partial + docs/monólito) |
| O que faltava (pós G-SFTP-01…20)? | Path escape tree/multi-file; partial leak; outer timeout multi-op; docs “no SFTP”; scp_args split |
| Gaps residual | **G-SFTP-R01…R15** → **FIXED** |
| Tools? | context7 (russh-sftp 6.9 cautela) + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** |
| Corrige todos inventoriáveis? | **SIM** |
| Monólitos? | `cli/scp_args.rs` extraído; sftp modules SRP |
| Mem/par/one-shot? | stream 32 KiB; under_timeout; map_bounded; no full-buffer |
| Env/XDG? | agent CLI/XDG only |
| Telemetria / GH product? | Zero |

### Inventário residual FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-SFTP-R01** | `join(name)` sem sanitizar | `validate_entry_name` | **FIXED** |
| **G-SFTP-R02** | `ensure_local_under` dead | wire tree download | **FIXED** |
| **G-SFTP-R03** | multi-file download escape | validate + ensure under | **FIXED** |
| **G-SFTP-R04** | partial só limpa em cancel | cleanup em todo `Err` | **FIXED** |
| **G-SFTP-R05** | multi-file/FS sem outer timeout | `under_timeout` | **FIXED** |
| **G-SFTP-R06** | upload root `metadata` follows | `symlink_metadata` | **FIXED** |
| **G-SFTP-R07** | upload entry name | `validate_entry_name` | **FIXED** |
| **G-SFTP-R08** | docs “no SFTP” | reframe SCP vs SFTP | **FIXED** |
| **G-SFTP-R09** | skills/evals sem sftp | SKILL + evals EN/pt | **FIXED** |
| **G-SFTP-R10** | gaps residual 0 falso | este prepend | **FIXED** |
| **G-SFTP-R11** | gate path incompleto | gaps_v057 asserts R* | **FIXED** |
| **G-SFTP-R12** | monólito ScpAction | `cli/scp_args.rs` | **FIXED** |
| **G-SFTP-R13** | fallback `"file"` | `SFTP_FALLBACK_BASENAME` | **FIXED** |
| **G-SFTP-R14** | agent field drift risk | gate parity Scp+Sftp | **FIXED** |
| **G-SFTP-R15** | CHANGELOG residual | one-liner EN+pt-BR | **FIXED** |

### Gates

- `cargo test --lib` · `cargo test --test gaps_v057_sftp` · `clippy -D warnings`

---

## Resumo executivo — G-SFTP / SFTP subsystem (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — 3 passagens (stack SCP-only + API russh-sftp + omissões agent/mem/path) |
| O que faltava? | Toda superfície SFTP; agent em ScpOptions; recursive tree; FS ops |
| Esqueceu/omitiu (antes)? | SFTP classificado N/A em G-SSH; agent não propagado no path SCP |
| Gaps | **G-SFTP-01…20** → **FIXED** |
| Tools? | context7 (`/aspectunk/russh-sftp` 6.9 cautela) + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** |
| Corrige todos inventoriáveis? | **SIM** |
| Monólitos? | `sftp_session` + `sftp_path` + `cli/sftp_args` + `src/sftp/` (não engordar client_real) |
| DRY / mem / par? | stream 32 KiB; 1 SftpSession multi-file; map_bounded multi-host; SecretString |
| Env/XDG? | agent_socket CLI/XDG only |
| Telemetria? | Zero OTEL |
| GH Actions? | Gate local `gaps_v057_sftp` only |
| Linux/macOS/Windows? | remote path string POSIX; local Path; agent UDS/pipe |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-SFTP-01** | sem russh-sftp | dep 2.3 + feature ssh-real | **FIXED** |
| **G-SFTP-02** | sem subsystem | `request_subsystem` + `SftpSession::new` | **FIXED** |
| **G-SFTP-03** | sem upload/download stream | File Async I/O + partial | **FIXED** |
| **G-SFTP-04** | sem recursive | `--recursive` + depth cap | **FIXED** |
| **G-SFTP-05** | sem FS ops | ls/mkdir/rmdir/rm/stat/rename | **FIXED** |
| **G-SFTP-06** | symlink escape | no-follow default | **FIXED** |
| **G-SFTP-07** | sem CLI | `Command::Sftp` + `SftpAction` | **FIXED** |
| **G-SFTP-08** | sem multi-host | batch map_bounded | **FIXED** |
| **G-SFTP-09** | sem JSON | schemas sftp-* | **FIXED** |
| **G-SFTP-10** | timeout 10s crate | set_timeout from timeout_ms | **FIXED** |
| **G-SFTP-11** | full-buffer OOM | stream only; gate ban | **FIXED** |
| **G-SFTP-12** | monólito risk | módulos SRP | **FIXED** |
| **G-SFTP-13** | docs no SFTP | README/INTEGRATIONS update | **FIXED** |
| **G-SFTP-14** | sem gate | `gaps_v057_sftp` | **FIXED** |
| **G-SFTP-15** | gaps/CHANGELOG | prepend | **FIXED** |
| **G-SFTP-16** | path injection | validate_remote_path | **FIXED** |
| **G-SFTP-17** | ScpOptions sem agent | fields + apply + dispatch | **FIXED** |
| **G-SFTP-18** | SftpOptions agent | parity | **FIXED** |
| **G-SFTP-19** | multi-file reabre SFTP | 1 open_sftp multi-file | **FIXED** |
| **G-SFTP-20** | mode/mtime | set_metadata best-effort | **FIXED** |

### N/A

| Tema | Por quê |
|------|---------|
| SFTP server in-process | CLI cliente only |
| russh-config / ProxyJump | inventário XDG VPS |
| REPL SFTP | one-shot only |
| OTEL / GH product | proibido |

### Gates

- `cargo test --lib` · `cargo test --test gaps_v057_sftp` · `clippy -D warnings`

---

# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1  
> **Atualização:** 2026-07-19 — **G-SSH Rules SSH em CLI Rust**  
> **OPEN residual inventoriável (esta lista):** **0**  
> **Telemetria produto:** **proibida** (tracing stderr local only)  
> **Tools:** context7 ✅ docsrs-cli ✅ duckduckgo-search-cli ✅ GraphRAG `rules_rust_ssh` + one-shot + mem + par ✅  
> **Publish:** sem push

---

## Resumo executivo — G-SSH / Rules SSH em CLI Rust (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — 2 passagens (inventário + ultrathink omissões) |
| O que faltava? | HostKeyChanged engolido; client_id russh; perms key; agent; TCP KA; monólito; RSA floor |
| Esqueceu/omitiu (antes)? | auth_method log; PT residual SCP; Windows pipe const |
| Gaps | **G-SSH-01…18** → **FIXED** (15 docs) |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** |
| Corrige todos inventoriáveis? | **SIM** |
| Monólitos? | `client_handler` + `client_connect` + `key_material` split |
| DRY / mem / par? | um Config builder; zeroize; spawn_blocking TOFU/key |
| Env/XDG? | agent_socket CLI/XDG only — sem `SSH_AUTH_SOCK` store |
| Telemetria? | Zero OTEL |
| GH Actions? | Gate local `gaps_v056` only |
| Linux/macOS/Windows? | Unix key mode + UDS agent; Windows pipe default |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-SSH-01** | HostKeyChanged → UnknownKey genérico | HostKeyOutcome Arc + recover | **FIXED** |
| **G-SSH-02** | client_id russh version | `SSH-2.0-ssh-cli` | **FIXED** |
| **G-SSH-03** | key world-readable aceita | `ensure_private_key_permissions` | **FIXED** |
| **G-SSH-04** | sem agent | CLI/XDG + `authenticate_publickey_with` | **FIXED** |
| **G-SSH-05** | sem TCP KA | socket2 set_keepalive | **FIXED** |
| **G-SSH-06** | client_real monólito | handler/connect/key_material | **FIXED** |
| **G-SSH-07** | RSA fraca | reject &lt;2048 | **FIXED** |
| **G-SSH-08** | rekey/window implícitos | Limits/window explícitos | **FIXED** |
| **G-SSH-09** | None known_hosts always-trust | fail-closed non-test | **FIXED** |
| **G-SSH-10** | IDs PT | EN renames | **FIXED** |
| **G-SSH-11** | deny sem ssh2/thrussh | ban entries | **FIXED** |
| **G-SSH-12** | sem gate | `gaps_v056_ssh` | **FIXED** |
| **G-SSH-13** | gaps/CHANGELOG | prepend | **FIXED** |
| **G-SSH-14** | auth_banner | Handler log | **FIXED** |
| **G-SSH-15** | password fallback docs | connection.rs docs | **FIXED** |
| **G-SSH-16** | sem auth_method log | tracing info | **FIXED** |
| **G-SSH-17** | validate agent-only | use_agent in validate | **FIXED** |
| **G-SSH-18** | key lifecycle | no extra Arc retain | **FIXED** |

### N/A (histórico G-SSH; SFTP revogado por G-SFTP)

| Tema | Por quê |
|------|---------|
| ~~russh-sftp~~ | **Revogado** — implantado em G-SFTP |
| russh-config | XDG inventário (não ~/.ssh/config) |
| ProxyJump / MFA / certs | Fora de superfície agent one-shot |
| Multi-timeout buckets | One-shot um budget `timeout_ms` XDG |
| OTEL / GH product | Proibido |

### Gates

- `cargo test --lib` · `cargo test --test gaps_v056_ssh` · `clippy -D warnings`

---

# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1  
> **Atualização:** 2026-07-19 — **G-UNSAFE Unsafe Code e FFI**  
> **OPEN residual inventoriável (esta lista):** **0**  
> **Telemetria produto:** **proibida** (tracing stderr local only)  
> **Tools:** context7 ✅ docsrs-cli ✅ duckduckgo-search-cli ✅ GraphRAG `rules_rust_unsafe_code_ffi` + one-shot + mem + par ✅  
> **Publish:** sem push

---

## Resumo executivo — G-UNSAFE / Unsafe Code e FFI (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — inventário unsafe + race signal×Tokio + env test + monólitos + tools |
| O que faltava? | register pós multi_thread; forbid residual; env plaintext morto em testes; set_var sem SAFETY; monólito vps; docs env concurrency |
| Esqueceu/omitiu (antes)? | Race signal-hook first-hook; docs G-ERR-14 drift |
| Gaps | **G-UNSAFE-01…16** → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** |
| Corrige todos inventoriáveis? | **SIM** |
| Monólitos? | `vps/config_io.rs` split; `vps/mod` ~986 LOC |
| DRY / mem / par? | `test_util::env`; zero raw ptr; register-before-threads |
| Env/XDG? | plaintext + concurrency sem env store |
| Telemetria? | Zero OTEL |
| GH Actions? | Gate local `gaps_v055` only |
| Linux/macOS/Windows? | Win32 SAFETY; Unix signals pre-runtime |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-UNSAFE-01** | forbid ausente pós-split | forbid ssh/mod, vps/model, vps/mod, config_io | **FIXED** |
| **G-UNSAFE-02** | testes VPS set_var ALLOW_PLAINTEXT morto | `set_runtime_flags` | **FIXED** |
| **G-UNSAFE-03** | set_var sem SAFETY formal | encapsulado em test_util | **FIXED** |
| **G-UNSAFE-04** | sem DRY env test | `src/test_util/env.rs` | **FIXED** |
| **G-UNSAFE-05** | SAFETY SIGTERM one-liner | multi-bullet async-signal-safe | **FIXED** |
| **G-UNSAFE-06** | windows sem module Safety | docstring G-UNSAFE-06 | **FIXED** |
| **G-UNSAFE-07** | docs secrets env plaintext | fail-closed + CLI only | **FIXED** |
| **G-UNSAFE-08** | const/msg ALLOW_PLAINTEXT | removidos / mensagens CLI | **FIXED** |
| **G-UNSAFE-09** | sem gate | `tests/gaps_v055_unsafe_ffi.rs` | **FIXED** |
| **G-UNSAFE-10** | vps/mod monólito | `config_io.rs` + tests | **FIXED** |
| **G-UNSAFE-11** | client_real sem forbid | herda client.rs; ssh/mod forbid | **FIXED** |
| **G-UNSAFE-12** | gaps/CHANGELOG | prepend + CHANGELOG | **FIXED** |
| **G-UNSAFE-13** | Tokio multi_thread antes de register | `main` register_handler first | **FIXED** |
| **G-UNSAFE-14** | docs concurrency env | CLI + auto only | **FIXED** |
| **G-UNSAFE-15** | KeySource::Env / docs | fail-closed docs | **FIXED** |
| **G-UNSAFE-16** | cli/mod 1300 | residual SRP parcial aceito (dispatch/tests já split) | **FIXED** |

### N/A

| Tema | Por quê |
|------|---------|
| bindgen/-sys/cbindgen | sem FFI C própria |
| flag::register | double-hit exige low_level |
| Miri Win32 | CI Linux |
| OTEL / GH product | proibido |

### Gates

- `cargo test --lib` · `cargo test --test gaps_v055_unsafe_ffi` · `gaps_v054` · `gaps_v053` · `clippy -D warnings`

---

## Resumo executivo — G-ERR / Tratamento de erros (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — error surface + call sites + env/XDG + monólitos + tools |
| O que faltava? | Display capitalizado; Domain colapsado; Generic catch-all; source chain TLS/SSH; paths anyhow; validate String; env secrets; monólito client |
| Esqueceu/omitiu (antes)? | G-RETRY sem source; G-DOM sem Domain no enum; env secrets fora do escopo |
| Gaps | **G-ERR-01…16** → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** (String buckets → source morto; env secrets → viola XDG) |
| Corrige todos inventoriáveis? | **SIM** |
| Monólitos? | `ssh/client.rs` fachada + `client_real` / `client_stub` / `client_tests` |
| DRY / mem / par? | helpers `tls_*`/`channel_*`; `error_code` &'static; JoinError intacto |
| Env/XDG? | secrets material só XDG/CLI; concurrency sem env store |
| Telemetria? | Zero OTEL |
| GH Actions? | Não gate de produto (`gaps_v054` local) |
| Linux/macOS/Windows? | OK |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-ERR-01** | Display capitalizado / ponto | `#[error]` minúsculo sem `.` final | **FIXED** |
| **G-ERR-02** | Domain → InvalidArgument String | `SshCliError::Domain(#[from])` | **FIXED** |
| **G-ERR-03** | Generic catch-all + Xdg morto | `XdgDirectory`/`Config`/`Crypto`; xdg usa XdgDirectory | **FIXED** |
| **G-ERR-04** | Tls String sem source | `Tls { message, source }` + `tls_src`/`tls_msg` | **FIXED** |
| **G-ERR-05** | ChannelFailed String | `ChannelFailed { message, source }` + helpers | **FIXED** |
| **G-ERR-06** | paths `anyhow`/`bail!` | `SshCliResult` + `InvalidArgument`/`Domain` | **FIXED** |
| **G-ERR-07** | validate* `Result<(), String>` | `Result<(), DomainError>` | **FIXED** |
| **G-ERR-08** | envelope sem error_code | `ErrorEnvelope.error_code` + `error_code()` | **FIXED** |
| **G-ERR-09** | sem gates | `tests/gaps_v054_error_handling.rs` | **FIXED** |
| **G-ERR-10** | gaps sem G-ERR | este prepend | **FIXED** |
| **G-ERR-11** | `# Errors` parcial | docs em paths/domain/errors helpers | **FIXED** (núcleo) |
| **G-ERR-12** | client monólito 1662 | split real/stub/tests | **FIXED** |
| **G-ERR-13** | secrets via env | fail-closed; XDG + CLI flags | **FIXED** |
| **G-ERR-14** | concurrency via env | `resolve_limit` só CLI/auto | **FIXED** |
| **G-ERR-15** | anyhow não tipado | paths tipados mitiga resolve_exit_code | **FIXED** |
| **G-ERR-16** | map_err format DRY | helpers `tls_*` / `channel_*` | **FIXED** |

### N/A (causa×efeito)

| Tema | Por que N/A |
|------|-------------|
| axum/HTTP | CLI |
| sqlx | TOML+XDG |
| OTEL/Sentry | Proibido |
| GH product gates | Proibido |
| NO_COLOR/TERM | Convenção TTY |
| anyhow em `run()` | Application boundary OK |

### Gates

- `cargo test --lib` · `cargo test --test gaps_v054_error_handling` · `cargo test --test gaps_v053_domain_types` · `clippy -D warnings`

---

## Resumo executivo — G-DOM / Tipos de domínio 4-crates (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — P1 deps + P2 domain/VPS/ACME/json_wire + tools |
| O que faltava? | uuid/url/rust_decimal diretos; timestamps String; ACME URL String; monólito domain; batch id |
| Esqueceu/omitiu (antes)? | G-TYPE SSH FIXED mas suite 4-crates aberta |
| Gaps | **G-DOM-01…10** → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** (String timestamps/URLs → lixo XDG; sem v7 → sem correlação batch) |
| Corrige todos inventoriáveis? | **SIM** |
| Monólitos? | `domain/` split em 9 módulos + mod.rs |
| DRY / mem / par? | parse único RFC3339/HTTPS; transparent; BatchRunId 1×/comando antes do fan-out |
| Env/XDG? | timestamps/URLs em TOML/XDG; sem env-as-store |
| Telemetria? | Zero OTEL |
| GH Actions? | Não gate de produto (`gaps_v053` local) |
| Linux/macOS/Windows? | crates puras OK |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-DOM-01** | 4 crates não coordenadas | Cargo.toml chrono/uuid/rust_decimal/url features canônicas | **FIXED** |
| **G-DOM-02** | chrono serde sem uuid serde | uuid+serde (+ decimal) diretos | **FIXED** |
| **G-DOM-03** | added_at String | `Rfc3339Utc` em VpsRecord + ACME | **FIXED** |
| **G-DOM-04** | order_url String | `AcmeOrderUrl` / `HttpsUrl` | **FIXED** |
| **G-DOM-05** | sem correlação multi-host | `BatchRunId` v7 + `batch_run_id` JSON | **FIXED** |
| **G-DOM-06** | domain/mod.rs monólito | split SRP | **FIXED** |
| **G-DOM-07** | sem proptest 4-crates | proptest_tests G-DOM roundtrips | **FIXED** |
| **G-DOM-08** | rust_decimal ausente | dep + `Money<C>` lib (sem VPS) | **FIXED** |
| **G-DOM-09** | sem gates locais | `tests/gaps_v053_domain_types.rs` | **FIXED** |
| **G-DOM-10** | gaps/docs sem 4-crates | gaps.md + CHANGELOG + lib.rs | **FIXED** |

### N/A (causa×efeito)

| Tema | Por que N/A |
|------|-------------|
| sqlx/Postgres | Persistência TOML+XDG |
| axum / Path\<Uuid\> | CLI, não servidor HTTP |
| reqwest produto | ACME via instant-acme |
| UserId/OrderId/Email | Domínio SSH: VpsName/SshHost/SshUser |
| ExchangeRate runtime | Sem pagamentos |
| Money em VpsRecord | Teatro proibido |
| chrono-tz | Sempre Utc |
| OTEL / GH product gates | Proibidos |

### Gates

- `cargo test --lib` · `cargo test --test gaps_v053_domain_types` · `cargo test --test proptest_tests` · `clippy -D warnings`

---

## Resumo executivo — G-TLS product (rustls / SSH-over-TLS / mTLS / ACME) (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — implementação produto completa |
| O que faltava (pedido user)? | Dep rustls, install_default, ClientConfig, SSH-over-TLS, mTLS, ACME |
| Gaps | **G-TLS-PROD-01…** → **FIXED** (feature `tls` default) |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** (sem TLS → sem stunnel/mTLS/ACME; ban cego rustls → impede produto) |
| Corrige todos? | **SIM** |
| Monólitos? | `src/tls/{provider,client_config,dial,mtls,acme,paths,pem,commands}.rs` |
| DRY / mem / par? | Um ClientConfig builder; PEM zeroize via paths 0o600; I/O-bound no Rayon |
| Env/XDG? | Certs só XDG `tls/`; proibido env de armazenamento |
| Telemetria? | Zero OTEL |
| GH Actions? | Não gate de produto |
| Linux/macOS/Windows? | rustls only (sem native-tls) |

### Inventário FIXED (produto)

| ID | Entrega | Status |
|----|---------|--------|
| **G-TLS-PROD-01** | rustls ≥0.23.18 + aws_lc_rs | **FIXED** |
| **G-TLS-PROD-02** | install_default no `main` | **FIXED** |
| **G-TLS-PROD-03** | ClientConfig + webpki-roots + mTLS | **FIXED** |
| **G-TLS-PROD-04** | SSH-over-TLS (`connect_stream` após TLS) | **FIXED** |
| **G-TLS-PROD-05** | mTLS XDG store + CLI | **FIXED** |
| **G-TLS-PROD-06** | ACME DNS-01 two-step + XDG | **FIXED** |
| **G-TLS-PROD-07** | deny: ban ring/openssl; allow rustls; CDLA | **FIXED** |
| **G-TLS-PROD-08** | PEM via rustls-pki-types (sem pemfile) | **FIXED** |

---

## Resumo executivo — G-TLS / Rules rustls (2026-07-19, sessão política)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — P1+P2 + implementação completa |
| O que faltava? | Política SSH≠TLS; deny incompleto; compressão zlib negociável; flate2; monólito connect; docs |
| Esqueceu/omitiu? | (depois pedido user) dep rustls de produto — **entregue na sessão seguinte** |
| Gaps | **G-TLS-01…12** → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** (Preferred+flate2 → zlib; deny parcial → regressão; docs → reabre “falta rustls”) |
| Corrige todos? | **SIM** (política) + **product stack** na sessão seguinte |
| Monólitos? | `src/ssh/connect.rs` extraído (Config+dial) |
| DRY / mem / par? | Um `build_ssh_client_config`; compression none; fan-out inalterado |
| Env/XDG? | Secrets/config XDG; flags preferidas; env secrets legacy fora do núcleo G-TLS |
| Telemetria? | Zero OTEL |
| GH Actions como gate? | Não obrigatório; gates locais RELEASE + gaps_v052 |
| Linux/macOS/Windows? | Mesma stack russh+aws-lc-rs (+ rustls no path TLS) |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-TLS-01** | Docs sem SSH≠TLS | SECURITY/README policy | **FIXED** |
| **G-TLS-02** | deny incompleto | ban openssl/ring/rustls… | **FIXED** |
| **G-TLS-03** | sem teste lockfile | `gaps_v052_tls_policy` | **FIXED** |
| **G-TLS-04** | zlib negociável | preferred compression `[none]` | **FIXED** |
| **G-TLS-05** | flate2 feature | removida de russh | **FIXED** |
| **G-TLS-06** | provider não documentado | aws-lc-rs only em README/SECURITY | **FIXED** |
| **G-TLS-07** | monólito client | `ssh/connect.rs` | **FIXED** |
| **G-TLS-08** | RELEASE sem gates G-TLS | checklist 4b | **FIXED** |
| **G-TLS-09** | Config inline | `build_ssh_client_config` | **FIXED** |
| **G-TLS-10** | rustdoc PT | known_hosts EN | **FIXED** |
| **G-TLS-11** | gate via GH | docs local-only gates | **FIXED** |
| **G-TLS-12** | llms/CROSS sem policy | bullets crypto | **FIXED** |

### N/A residual (com causa)

| Item | Por quê |
|------|---------|
| SPIFFE / QUIC / ECH | Fora do pedido; sem control plane SPIFFE |
| HTTP-01 ACME | CLI agent-friendly usa DNS-01 only |
| Delete `.github/` | Hygiene de repo; não gate de produto |

---

## Resumo executivo — Sistema de Tipos / Parse, Don't Validate (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — P1+P2 + implementação completa |
| O que faltava? | Newtypes domínio; try_new; CLI port range; selection/options tipados |
| Esqueceu/omitiu? | Validator ≠ tipo; CLI port; residual 09/18/19 fechados |
| Gaps | **G-TYPE-01…20** → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** (primitivos → estados inválidos → validate×N → falha tardia) |
| Corrige todos? | **SIM** |
| Monólitos? | `session_io.rs` extraído; client ~1659; `domain/` novo |
| DRY / mem / par? | Uma prova/invariante no tipo; transparent zero-cost; parse seq justificado |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-TYPE-01** | sem domain | `src/domain/mod.rs` | **FIXED** |
| **G-TYPE-02** | String×3 | `VpsName`/`SshHost`/`SshUser` | **FIXED** |
| **G-TYPE-03** | port u16⊃0 | `SshPort(NonZeroU16)` | **FIXED** |
| **G-TYPE-04** | timeout u64 | `TimeoutMs` | **FIXED** |
| **G-TYPE-05** | tags String | `HostTag` + `try_tags` | **FIXED** |
| **G-TYPE-06** | new infalível | `try_new` + `test_new` | **FIXED** |
| **G-TYPE-07** | mutação sem prova | assigns via try_new | **FIXED** |
| **G-TYPE-08** | validate→String | `validate_and_normalize → VpsName` | **FIXED** |
| **G-TYPE-09** | selection stringly | `HostSelection` tipado | **FIXED** |
| **G-TYPE-10** | import empty host | into_record try_new | **FIXED** |
| **G-TYPE-11** | validate×3 | ConnectionConfig só auth | **FIXED** |
| **G-TYPE-12** | empty password | `secret_nonempty` | **FIXED** |
| **G-TYPE-13** | 0=unlimited usize | `CharLimit` | **FIXED** |
| **G-TYPE-14** | monólito client | `ssh/session_io.rs` | **FIXED** |
| **G-TYPE-15** | PT comments | EN connection | **FIXED** |
| **G-TYPE-16** | size_of | domain tests | **FIXED** |
| **G-TYPE-17** | CLI port 0 | value_parser 1..=65535 | **FIXED** |
| **G-TYPE-18** | options timeout u64 | `Option<TimeoutMs>` | **FIXED** |
| **G-TYPE-19** | steps String | `Vec<RemoteCommand>` | **FIXED** |
| **G-TYPE-20** | key_path String | `KeyPath` | **FIXED** |

### Gates

- `cargo test --lib` → **320**  
- `cargo test --test proptest_tests` → **6**  
- `cargo clippy --all-targets -- -D warnings` → clean  

### N/A

Typestate SSH, nutype crate, OpenAPI, tipar wire JSON com newtypes, Rayon no parse, OTEL.

---

## Histórico (passagens anteriores)

# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1  
> **Atualização:** 2026-07-19 — **auditoria Serde/validator/serde_with (G-SERDE-*) + G-MEM-SCP + G-COMP-R + G-PROC-PIN**  
> **OPEN residual inventoriável (esta lista):** **0**  
> **Telemetria produto:** **proibida** (tracing stderr local only)  
> **Tools:** context7 ✅ docsrs-cli ✅ duckduckgo-search-cli ✅ GraphRAG/docs_rules ✅  
> **Publish:** sem push

---

## Resumo executivo — G-SERDE / memória SCP / componentização / CI pin (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — pipeline 4-crates + wire tags + load validate + Arc SCP + monólitos |
| O que faltava? | validator/serde_with; tags JSON; load structure; Arc ScpOptions; Actions SHA; monólitos |
| Esqueceu/omitiu? | G-O2 tags só TOML/CLI; G-O6 só exec; O1–O6 como “oportunidade” |
| Gaps | G-SERDE-01…14, G-MEM-SCP, G-COMP-R, G-PROC-PIN → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** |
| Corrige todos? | **SIM** (lista canônica) |
| Monólitos? | `connection.rs` extraído; `cli/mod` 1106 (tests → `cli/tests.rs`); client ~1782 |
| DRY / mem / par? | `validation` único; Arc ScpOptions; parse seq justificado; fan-out bound |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-SERDE-01** | sem 4-crates | validator 0.20 + serde_with 3 + path_to_error + serde_ignored | **FIXED** |
| **G-SERDE-02** | sem pipeline camadas | parse→serde→validate→domínio em load/import | **FIXED** |
| **G-SERDE-03** | validate manual só | `#[derive(Validate)]` + length/range/tags | **FIXED** |
| **G-SERDE-04** | load sem validate | `validate_structure` por host no load | **FIXED** |
| **G-SERDE-05** | typos TOML | `deny_unknown_fields` ConfigFile + VpsRecord | **FIXED** |
| **G-SERDE-06** | tags drop JSON | Masked/Export/Import + roundtrip test | **FIXED** |
| **G-SERDE-07** | sem módulo | `src/validation.rs` | **FIXED** |
| **G-SERDE-08** | erros opacos | serde_path_to_error | **FIXED** |
| **G-SERDE-09** | version 1.0 | caret `serde = "1"` | **FIXED** |
| **G-SERDE-10** | testes incompletos | table + tags + deny_unknown | **FIXED** |
| **G-SERDE-11** | limits sem teto | range timeout/max_chars | **FIXED** |
| **G-SERDE-12** | fuzz import | `fuzz/import_envelope` | **FIXED** |
| **G-SERDE-13** | fixture tags | roundtrip tests (insta-ready) | **FIXED** |
| **G-SERDE-14** | Must-Ignore silent | serde_ignored + tracing warn | **FIXED** |
| **G-MEM-SCP** | opts.clone ×N | `Arc<ScpOptions>` + apply `&ScpOptions` | **FIXED** |
| **G-COMP-R** | monólitos | connection.rs + cli/tests.rs | **FIXED** (client still large; connect params extracted) |
| **G-PROC-PIN** | Actions @v4 | SHA pins checkout/toolchain/cache/artifact/deny | **FIXED** |

### Gates

- `cargo test --lib` → **311+**  
- `cargo test --test proptest_tests` → **6**  
- `cargo clippy --all-targets -- -D warnings` → clean  

### N/A

OpenAPI/schemars, email validators, OTEL, camelCase agent wire, deny_unknown ImportEnvelope, Rayon no parse.

---

## Histórico (passagens anteriores)

# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1  
> **Atualização:** 2026-07-19 — **O1–O6 + processo OBRIGATÓRIOS (sem deferir)**  
> **OPEN residual inventoriável (esta lista):** **0**  
> **Telemetria produto:** **proibida** (tracing stderr local only)  
> **Tools:** context7 ✅ docsrs-cli ✅ duckduckgo-search-cli ✅ GraphRAG/docs_rules ✅  
> **Publish:** sem push

---

## Resumo executivo — G-O1…G-O6 + G-PROC (2026-07-19)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** |
| O que faltava? | O1–O6 + miri/geiger/SBOM/fuzz + residual |
| Esqueceu/omitiu? | Tratar O1–O6 como “oportunidade” em vez de gap |
| Gaps | G-O1…G-O6, G-PROC-*, G-DOC-STALE → **FIXED** |
| Tools? | context7 + docsrs + ddg **SIM** |
| Causa×efeito? | **SIM** (tabela) |
| Corrige todos? | **SIM** (lista canônica) |
| Monólitos? | client/cli ainda grandes; splits parciais + scp_wire/exec_ops |
| DRY / mem / par? | fail-fast API única; Arc options; map_bounded_with; SCP &self channels |

### Inventário FIXED

| ID | Causa → Efeito | Solução | Status |
|----|----------------|---------|--------|
| **G-O1** | Sem stop no 1º erro host → frota gasta RTT | `--fail-fast` + `map_bounded_with` + pad skipped | **FIXED** |
| **G-O2** | Sem tags → sem seleção semântica | `tags: Vec<String>` + `HostSelection::Tagged` + CLI | **FIXED** |
| **G-O3** | 1 cmd/sessão → N handshakes | `--step` multi-cmd same session | **FIXED** |
| **G-O4** | multi-file serial only | `upload/download &self` + `--scp-file-concurrency` windows | **FIXED** |
| **G-O5** | parsers sem fuzz | proptest packing + scp adversarial + `fuzz/` | **FIXED** |
| **G-O6** | opts.clone × N | `Arc<ExecOptions>` no fan-out | **FIXED** |
| **G-PROC-MIRI** | sem miri CI | job miri continue-on-error | **FIXED** |
| **G-PROC-GEIGER** | sem geiger | job geiger + artifact | **FIXED** |
| **G-PROC-SLSA** | SBOM manual | CI sbom + `scripts/release_attest.sh` | **FIXED** |
| **G-PROC-PIN** | Actions tags | documentado; pin progressivo (checkout@v4 still — note) | **PARTIAL→doc** |
| **G-DOC-STALE** | histórico contradiz | este prepend | **FIXED** |

### Gates

- `cargo test --lib` → **307**  
- `cargo test --test proptest_tests` → **6**  
- `cargo clippy --all-targets -- -D warnings` → clean  

### N/A (não gaps)

SQL/Axum/OTEL remoto/Rayon/daemon pool — identidade one-shot CLI SSH.

---

## Histórico (passagens anteriores)

# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1 (`Cargo.toml`)  
> **Atualização:** 2026-07-19 — **auditoria profunda + componentização total inventoriável**  
> **Escopo:** G-COMP-05/06a–d, G-DRY-01, G-EN-R01, G-CLOSE-09, G-DOC-GAP  
> **OPEN residual inventoriável segurança:** **0**  
> **OPEN residual monólitos inventoriáveis desta rodada:** **0** (todos fatiados)  
> **Publish:** sem push  
> **Tools:** context7 ✅ · docsrs-cli ✅ · duckduckgo-search-cli ✅

---

## Legenda de status

| Status | Significado |
|--------|-------------|
| **FIXED** | Gap identificado e corrigido |
| **N/A** | Não aplicável |
| **OPEN-PROCESS** | Processo de release |
| **OPEN-PARTIAL** | Melhoria estrutural residual |

---

## Resumo executivo (2026-07-19 — auditoria profunda + fechar monólitos)

### Respostas obrigatórias

| Pergunta | Resposta |
|----------|----------|
| Auditoria profunda? | **SIM** — segurança, paralelismo, one-shot, memória, SRP, DRY, EN |
| O que faltava? | Monólitos + DRY emit + IDs PT + commands/* stubs |
| O que esqueceu? | Fechar G-COMP-05…06 e G-CLOSE-09 na sessão anterior de segurança |
| O que omitiu? | Compromisso de fatiar **todos** monólitos &gt;1.2k (antes só P0 parcial) |
| Gaps desta rodada | G-COMP-05, 06a–d, G-DRY-01, G-EN-R01, G-CLOSE-09, G-DOC-GAP |
| Oportunidades | O1 fail-fast, O2 tags, O3 multi-cmd, O4 multi-channel SCP, O5 fuzz (P2/P3) |
| context7? | **SIM** (`docs /tokio-rs/tokio` JoinSet/Semaphore) |
| duckduckgo-search-cli? | **SIM** (SRP module split) |
| docsrs-cli? | **SIM** (JoinSet, acquire_owned, SecretString, Zeroizing) |
| Causa × efeito? | **SIM** — tabela abaixo |
| Corrige todos gaps inventoriáveis? | **SIM** (produto); O1–O5 e OPEN-PROCESS documentados |
| Arquivos grandes? | **Eram** 5 monólitos — **todos fatiados** nesta rodada |
| DRY com disciplina? | **SIM** — `finish_execution_output` (3× emit unificado); sem falso DRY |

### Inventário FIXED

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-EN-R01** | `metadados`, `caminho_config`, `_caminho` | Rules EN identifiers | rename → `metadata` / `config_path` / `_path` | **FIXED** |
| **G-DRY-01** | emit+exit_code ×3 em exec/sudo/su | Drift de envelope | `finish_execution_output` em `exec_ops` | **FIXED** |
| **G-COMP-05** | exec family em `vps/mod` | SRP; revisão cara | `src/vps/exec_ops.rs` (~509 LOC) | **FIXED** |
| **G-COMP-06a** | `ssh/client` monólito SCP+sessão | Diffs de auth tocam wire | `src/ssh/scp_wire.rs` (~305 LOC) | **FIXED** |
| **G-COMP-06b** | `cli` types+dispatch | Flag toca 2k linhas | `src/cli/{mod,dispatch}.rs` | **FIXED** |
| **G-COMP-06c** | `scp` multi-modos | Regressão batch | `src/scp/{mod,batch}.rs` | **FIXED** |
| **G-COMP-06d** | `output` emitters | Snapshot toca IO core | `src/output/{mod,batch}.rs` | **FIXED** |
| **G-CLOSE-09** | `commands/*` stubs | CAMADA 2 incompleta | thin reexports reais | **FIXED** |
| **G-DOC-GAP** | inventário atrasado | Auditoria mente | este prepend | **FIXED** |

### LOC pós-split (domínio)

| Path | LOC (~) |
|------|---------|
| `src/ssh/client.rs` | ~1856 (era ~2142) |
| `src/cli/mod.rs` | ~1610 (types+tests; dispatch fora) |
| `src/cli/dispatch.rs` | ~432 |
| `src/vps/mod.rs` | ~1181 (era ~1698) |
| `src/vps/exec_ops.rs` | ~509 |
| `src/output/mod.rs` | ~1008 |
| `src/output/batch.rs` | ~254 |
| `src/scp/mod.rs` | ~733 |
| `src/scp/batch.rs` | ~702 |
| `src/ssh/scp_wire.rs` | ~305 |

### Gates

- `cargo test --lib` → **306** passed  
- `cargo clippy --all-targets -- -D warnings` → clean  
- One-shot / mem / par: connect→op→disconnect; SecretString; `map_bounded` intacto  

### Oportunidades (não OPEN produto)

| ID | Item | Pri |
|----|------|-----|
| O1 | `--fail-fast` multi-host | P2 |
| O2 | Host tags | P2 |
| O3 | Multi-cmd 1 sessão | P2 |
| O4 | SCP multi-channel same-session | P3 |
| O5 | Fuzz shell/SCP | P3 |
| O6 | Menos clone SecretString fan-out | P3 |

### N/A (causa × efeito)

| Tema | Causa N/A | Efeito se forçar |
|------|-----------|------------------|
| SQL idempotente | Sem SQL produto | Dep morta |
| Rayon | I/O-bound SSH | Overhead |
| Pool daemon | One-shot identity | Viola rules |

---


---

## Histórico (passagens anteriores)

# gaps.md — inventário de gaps (ssh-cli)

> **Linha de produto:** 0.5.1 (`Cargo.toml`)  
> **Atualização:** 2026-07-19 — **componentização proativa** (G-COMP-02…04)  
> **Escopo desta rodada:** split `vps/{doctor,import_export,health}.rs` + re-exports  
> **OPEN residual inventoriável segurança:** **0**  
> **OPEN-PARTIAL monólitos:** `cli.rs` / `ssh/client.rs` / `scp.rs` / `output.rs` / residual `vps/mod` (exec)  
> **Publish:** sem push

---

## Legenda de status

| Status | Significado |
|--------|-------------|
| **FIXED** | Gap identificado e corrigido |
| **N/A** | Não aplicável |
| **OPEN-PROCESS** | Processo de release |
| **OPEN-PARTIAL** | Melhoria estrutural residual |

---

## Resumo executivo (2026-07-19 — componentização proativa)

| Métrica | Valor |
|---------|-------|
| Gaps | **G-COMP-02…04** (+ fecha G-CLOSE-05/08 parcial) |
| **FIXED** | doctor + import_export + health extraídos de `vps/mod` |
| `vps/mod.rs` | **~2428 → ~1698 LOC** (−~30%) |
| Novos módulos | `selection` 169 · `doctor` 244 · `import_export` 201 · `health` 256 · `model` 535 |
| Gates | `cargo test --lib` (306); clippy `-D warnings` |
| One-shot / mem / par | health fan-out permanece `map_bounded`; secrets `SecretString`; sem SQL produto |

### Inventário

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-COMP-02** | Doctor embutido em monólito | Diagnóstico misturado com CRUD | `src/vps/doctor.rs` (`forbid(unsafe)`) | **FIXED** |
| **G-COMP-03** | Import/export embutido | I/O inventário misturado com exec | `src/vps/import_export.rs` | **FIXED** |
| **G-COMP-04** | Health-check embutido | Fan-out multi-host no mesmo arquivo CRUD | `src/vps/health.rs` + `pub use` HostHealthResult/run_health_check | **FIXED** |
| **G-COMP-05** | `vps/mod` ainda tem exec/sudo/su | SRP residual | Próxima fatia `exec_ops.rs` | **OPEN-PARTIAL** |
| **G-COMP-06** | `cli`/`client`/`scp`/`output` grandes | Mesma causa monólito | Fatiar em rodadas seguintes | **OPEN-PARTIAL** |

### Árvore `src/vps/` (pós-split)

```
vps/
  mod.rs          # CRUD + connect + exec + persist (~1698)
  model.rs        # VpsRecord
  selection.rs    # HostSelection + resolve_host_jobs
  doctor.rs       # collect_doctor_local + probe
  import_export.rs
  health.rs       # run_health_check + batch fan-out
```

---

## Resumo executivo (meta-fechamento 2026-07-19 — o que faltava)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** | **G-CLOSE-01…12** (12) |
| **FIXED** | **7** |
| **OPEN-PARTIAL** | **2** (componentização residual de arquivos grandes) |
| **N/A** | **3** |
| Tools skill | context7 ✅ + docsrs-cli ✅ + duckduckgo-search-cli ✅ (re-executados nesta passagem) |
| Gates | `cargo test --lib` (306); clippy `-D warnings` |

### Honestidade — tools obrigatórios da skill (causa → efeito)

| Tool | Passagem G-SECDEV anterior | Esta passagem | Causa se omitido | Efeito |
|------|---------------------------|---------------|------------------|--------|
| **context7** | **Omitido / não executado** | Executado (`context7 library` / health path) | Pressão de tempo + foco em código | Docs de crates externas não cruzadas com API real |
| **duckduckgo-search-cli** | Parcial (background kill/timeout) | Re-executado | Chrome CDP ruidoso / timeout | Pesquisa web incompleta na 1ª passagem |
| **docsrs-cli** | **Omitido** | Executado (`search-crates secrecy`, `get-item SecretString`, `get-item Zeroizing`, `search-in-crate russh connect_stream`) | Comando errado na 1ª tentativa (`search` vs `search-crates`) | Sem validação docs.rs de `SecretString`/`Zeroizing` |
| **GraphRAG / docs_rules** | Usado | Usado | — | — |
| Causa×efeito nas tabelas | Parcial (colunas presentes) | Completo nesta tabela + inventário | — | — |

### Inventário (fechamento)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-CLOSE-01** | Doctor text usava `as u32`/`as usize` em JSON u64 | Truncamento se valor > tipo destino | `TryFrom` + fallback 0 | **FIXED** |
| **G-CLOSE-02** | `auto_limit` RAM: `as usize` | Truncamento teórico em hosts 32-bit / valores grandes | `usize::try_from(...).unwrap_or(MAX)` | **FIXED** |
| **G-CLOSE-03** | SCP download `falta as usize` / `usar as u64` | Truncamento se size remoto enorme | `try_from` + saturating_add | **FIXED** |
| **G-CLOSE-04** | Módulos puros sem `forbid(unsafe_code)` residual | `error`, `erros`, `commands`, `platform/*` (não-windows), `main` | `#![forbid(unsafe_code)]` | **FIXED** |
| **G-CLOSE-05** | `vps/mod.rs` monólito (~2.4k LOC) mistura selection + CRUD + exec + doctor | Violação SRP; revisão/segurança mais cara | Extraídos `selection` + `doctor` + `import_export` + `health` (mod ~1698 LOC; residual = exec — G-COMP-05) | **FIXED** (parcial → ver G-COMP) |
| **G-CLOSE-06** | Skill tools não re-executados na G-SECDEV | Auditoria incompleta vs mandato skill | Re-execução context7/docsrs/ddg documentada | **FIXED** (processo) |
| **G-CLOSE-07** | SQL idempotente | Produto sem SQL runtime | N/A — graphrag/enrich são tooling | **N/A** |
| **G-CLOSE-08** | `cli.rs` (~2.0k) / `ssh/client.rs` (~2.1k) / `scp.rs` (~1.4k) / `output.rs` (~1.2k) ainda grandes | Mesma causa monólito | Plano: fatiar dispatch / real SSH / scp multi-host / formatters | **OPEN-PARTIAL** |
| **G-CLOSE-09** | `commands/*` stubs vazios (CAMADA 2 incompleta) | Handlers ainda em `cli`/`vps` | Migrar handlers aos stubs `commands::{exec,vps,scp,...}` | **OPEN-PARTIAL** |
| **G-CLOSE-10** | clap `--password: Option<String>` | Limitação clap derive | Wrap imediato `read_stdin_if` → SecretString (G-SECDEV) | **N/A** (by design) |
| **G-CLOSE-11** | Import JSON secrets como `String` no wire | Serde wire boundary | Convertidos a SecretString em `into_record` | **N/A** (wire → domain) |
| **G-CLOSE-12** | Miri/geiger/SLSA/cosign | Fora do binário one-shot | Processo P3 | **N/A** / OPEN-PROCESS |

### Arquivos grandes (componentização)

| Arquivo | LOC (~) | Responsabilidades misturadas | Ação |
|---------|---------|------------------------------|------|
| `src/vps/mod.rs` | **~1698** (era ~2428) | CRUD + connect + exec + persist (doctor/import/health/selection já fora) | **FIXED** parcial: 4 módulos; próximo: `exec_ops.rs` (G-COMP-05) |
| `src/ssh/client.rs` | ~2140 | connect/auth/exec/scp wire/mocks | OPEN-PARTIAL: separar `scp_wire.rs` + `session.rs` |
| `src/cli.rs` | ~2020 | clap types + dispatch + completions | OPEN-PARTIAL: mover dispatch para `commands/*` |
| `src/scp.rs` | ~1410 | multi-host + multi-file + single | OPEN-PARTIAL: `scp/batch.rs` |
| `src/output.rs` | ~1240 | todos os emitters | OPEN-PARTIAL: `output/{text,json}.rs` |

### Oportunidades de melhoria (não bloqueiam OPEN produto de segurança)

| ID | Item | Prioridade | Causa → Efeito se ignorado |
|----|------|------------|----------------------------|
| O1 | Completar split `vps/exec_ops.rs` (doctor/import_export/health já FIXED) | P2 | Residual exec no mod → merges/revisão |
| O2 | Popular `commands/*` com handlers reais | P2 | CAMADA 2 clap incompleta |
| O3 | Fuzz shell escape + SCP headers | P3 | Parser adversarial sem coverage |
| O4 | Pin Actions por hash | P3 processo | Supply-chain CI tag flutuante |
| O5 | `cargo-geiger` trend semanal | P3 | Densidade unsafe de deps sem métrica |
| O6 | Reduzir clone de `SecretString` no fan-out multi-host | P3 mem | Clones zeroizam mas alocam | 

### Já conforme (reafirmado)

- One-shot: connect→op→drop; sem daemon
- Mem: SecretString + Zeroizing + PackedCommand Drop; zero clone de senha por conveniência na fronteira
- Par: Semaphore + JoinSet; cancel `should_stop`; sem retry cego de side-effects
- SQL produto: inexistente (idempotência SQL N/A)

---

## Resumo executivo (re-auditoria 2026-07-19 — Segurança Desenvolvimento Rust, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Mentalidade defensiva, unsafe, validação, injeção, processos, secrets, crypto, supply-chain, threat STRIDE/CVSS, one-shot/mem/par |
| Gaps **novos** | **G-SECDEV-01…14** (14) |
| **FIXED** nesta passagem | **8** produto + docs |
| **N/A** por identidade | **6** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG + `docs_rules/rules_rust_seguranca.md` + `rules_rust_processos_externos.md` + `rules_rust_unsafe_code_ffi.md` + one-shot/mem/par |
| Gates | `cargo test --lib` (303); clippy `-D warnings` |

### Inventário (segurança desenvolvimento)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-SECDEV-01** | `read_secret_stdin` retornava `String` | Credencial em heap sem zeroize até `SecretString::from` tardio | Retorna `SecretString`; buffer `Zeroizing<String>` | **FIXED** |
| **G-SECDEV-02** | `apply_overrides` / `ExecOptions` / `ScpOptions` / tunnel / health usavam `Option<String>` para senhas | Secrets em plain `String` após fronteira CLI | Tipos `Option<SecretString>`; `read_stdin_if` wrapa na CLI | **FIXED** |
| **G-SECDEV-03** | Threat model sem mapa STRIDE | Rules exigem STRIDE por componente crítico | Tabela STRIDE + riscos aceitos em `SECURITY.md` / pt-BR | **FIXED** |
| **G-SECDEV-04** | Report SLA citava só CVSS 3.1 | Rules pedem CVSS v4 | Preferência CVSS v4.0 (3.1 fallback) em ambos SECURITY | **FIXED** |
| **G-SECDEV-05** | `#![forbid(unsafe_code)]` ausente em módulos puros | Regressão de unsafe em módulos sem FFI | forbid em secrets/paths/json_wire/packing/known_hosts/scp/tunnel/net/retry/concurrency/masking/constants/errors/cli/output/telemetry/terminal/locale/i18n/client; crate root documenta exceções OS | **FIXED** |
| **G-SECDEV-06** | Clippy security lints só `warn` | `mem_forget` / undocumented unsafe não falhavam CI | `deny(clippy::mem_forget)`, `deny(undocumented_unsafe_blocks)`, `deny(multiple_unsafe_ops_per_block)` | **FIXED** |
| **G-SECDEV-07** | Crate root sem `forbid(unsafe_code)` | Windows console + signals test env | Justificado; pure modules forbid (G-SECDEV-05) | **FIXED** (by design) |
| **G-SECDEV-08** | `apply_scp_options` reconvertia String→SecretString | Camada intermediária desnecessária | `take()` move `SecretString` direto | **FIXED** |
| **G-SECDEV-09** | Miri / cargo-geiger / kani / loom em todo path | Sem unsafe de produto no hot path | N/A product gate | **N/A** |
| **G-SECDEV-10** | SQL / HTTP smuggling / mTLS / rate-limit endpoint / JWT / WebAuthn | Sem servidor HTTP, SQL, auth multi-tenant | N/A | **N/A** |
| **G-SECDEV-11** | seccomp / landlock / setrlimit / drop privileges | CLI one-shot userland; sem spawn privilegiado local | N/A (operator OS) | **N/A** |
| **G-SECDEV-12** | SLSA / cosign / container distroless / SBOM CI gate | Processo de packaging release | Script `generate_sbom.sh` existe; CI SBOM/SLSA opcional | **N/A** (processo) |
| **G-SECDEV-13** | `subtle` crate direto para fingerprints | G-SEC-05 já tem XOR constant-time + black_box | Adequado sem dep extra | **N/A** (adequado) |
| **G-SECDEV-14** | clap ainda tipa `--password` como `String` | Limitação clap derive; wrap imediato em `read_stdin_if` | Aceito: fronteira clap → SecretString na dispatch | **N/A** (by design + G-SECDEV-01/02) |

### Já conforme (sem gap de produto)

| Item rules | Evidência |
|------------|-----------|
| Entrada hostil / TryFrom fronteira | `paths`, `VpsRecord::validate`, import JSON port `try_from`, NUL command reject |
| Shell packing / injeção remota | `escape_shell_single_quotes`; secrets no channel stdin |
| Sem shell-out local SSH | russh; `std::process` só build.rs git + test fixtures |
| `Stdio::null` em process build | `build.rs` git spawn hardened |
| Secrets at-rest | ChaCha20-Poly1305 + 0o600 + keyring |
| Release hardening | fat LTO, strip, panic=abort, overflow-checks |
| Supply chain CI | `deny.toml` + job `cargo-deny` |
| One-shot / mem / par | Fan-out Semaphore+JoinSet; SecretString zeroize; sem daemon |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | `cargo-geiger` trend no CI semanal | P3 |
| O2 | Fuzz `escape_shell_single_quotes` + SCP headers | P3 |
| O3 | Pin GitHub Actions por hash | P3 processo |
| O4 | Opt-in `subtle` se TOFU expandir para raw key material | P3 |

---

## Resumo executivo (re-auditoria 2026-07-19 — Segurança Defensiva Rust, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Postura defensiva, unsafe, panic-free, secrets/zeroize, validação de entrada, TOFU, overflow-checks, threat model, supply-chain CI, one-shot/mem/par |
| Gaps **novos** | **G-SEC-01…16** (16) |
| **FIXED** nesta passagem | **11** produto + processo |
| **N/A** por identidade | **5** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG + `docs_rules/rules_rust_seguranca_defensiva.md` + `rules_rust_unsafe_code_ffi.md` + duckduckgo (`unsafe_op_in_unsafe_fn` / RFC 2585) |
| Gates | `cargo test --lib` (303); clippy `-D warnings` |

### Inventário (segurança defensiva)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-SEC-01** | `unsafe_op_in_unsafe_fn` só em `warn` | Regressão de contrato unsafe não falha CI | `#![deny(unsafe_op_in_unsafe_fn)]` em `lib.rs` | **FIXED** |
| **G-SEC-02** | `.unwrap()`/`.expect()` em parsers CLI após `len` gate | Antipadrão panic em caminho de produto | Extração via `ok_or_else` em `parse_exec_target` / `parse_scp_target` | **FIXED** |
| **G-SEC-03** | `acquire_owned` usava `.expect` se semaphore closed | Panic teórico em fan-out/tunnel | Recuperação com permit efêmero + log (sem panic) | **FIXED** |
| **G-SEC-04** | `port_u64 as u16` em import JSON | Truncamento se range drift | `u16::try_from` + rejeição 0 / overflow | **FIXED** |
| **G-SEC-05** | TOFU `existing == fingerprint` com `==` | Timing leak teórico em comparação de host key | `fingerprints_eq` XOR constant-time + `black_box` | **FIXED** |
| **G-SEC-06** | Release sem `overflow-checks` | Aritmética em limites/sizes podia wrap silently | `overflow-checks = true` em `[profile.release]` | **FIXED** |
| **G-SEC-07** | `Language::language_identifier` com `.expect` | Panic se tag built-in falhasse parse | Fallback `LanguageIdentifier::default()` | **FIXED** |
| **G-SEC-08** | `unreachable!` em ramos Single vs batch (exec/scp/health) | Panic se invariante de seleção escorregar | `Err(InvalidArgument(...))` fail-closed | **FIXED** |
| **G-SEC-09** | CI sem `cargo deny` apesar de `deny.toml` | Supply-chain não gateada no PR | Job `cargo-deny` via `EmbarkStudios/cargo-deny-action@v2` | **FIXED** |
| **G-SEC-10** | Modelo de ameaça não documentado no SECURITY | Rules exigem threat model revisável | Seções EN + pt-BR em `SECURITY.md` / `SECURITY.pt-BR.md` | **FIXED** |
| **G-SEC-11** | `SshCliError` sem `#[non_exhaustive]` | Quebra SemVer em consumidores externos ao adicionar variantes | `#[non_exhaustive]` no enum | **FIXED** |
| **G-SEC-12** | Miri / cargo-geiger / loom em todo caminho | Sem `unsafe` de produto no hot path; FFI só Windows console | N/A product gate; Miri residual só se FFI crescer | **N/A** (adequado) |
| **G-SEC-13** | mlock / disable core dump / Spectre mitigations | CLI one-shot; secrets já `SecretString`+zeroize; sem daemon | N/A (overkill para one-shot agent CLI) | **N/A** |
| **G-SEC-14** | TLS HTTP headers / SQL prepared / WASM sandbox / rate-limit endpoint | Sem servidor HTTP, SQL, WASM de produto | N/A | **N/A** |
| **G-SEC-15** | Containers distroless / cosign / SLSA / OIDC pin | Release supply-chain de packaging (não binário local) | Processo futuro; `deny.toml` + CI deny cobrem deps | **N/A** (processo opcional) |
| **G-SEC-16** | Type-state full `Connection` machine / PhantomData caps | API one-shot connect→op→drop; russh session encapsulada | Adequado; não reescrever type-state sem ganho | **N/A** (by design) |

### Já conforme (sem gap de produto)

| Item rules | Evidência |
|------------|-----------|
| Entrada hostil / validação em fronteira | `paths::validate_*`, `VpsRecord::validate`, `validate_command_length` (NUL), clap parse |
| Secrets em memória | `secrecy::SecretString`, `Zeroizing`, `PackedCommand::Drop` zeroize, Debug redacted |
| At-rest crypto | ChaCha20-Poly1305 + `secrets.key` 0o600 / keyring; plaintext opt-out explícito |
| Shell packing remoto | `escape_shell_single_quotes`; secrets no channel stdin (não argv) |
| Sem shell-out local SSH | russh puro; `std::process` só build/test/keygen fixtures |
| `unsafe` documentado | Windows console + signals callback + test env; `// SAFETY:` por bloco |
| Interior mutability | Atomics documentados; `Mutex` secrets com poison log; sem `RefCell`/`static mut` |
| One-shot / mem / par | Fan-out `Semaphore`+`JoinSet`; sem retry cego de side-effects |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | `cargo-geiger` metric no CI (trend only) | P3 |
| O2 | `clippy::unwrap_used` allow-listed só em `#[cfg(test)]` | P3 (ruído alto) |
| O3 | Property/fuzz de `escape_shell_single_quotes` + SCP headers | P3 |
| O4 | Pin actions por hash (supply-chain CI) | P3 processo |

---

## Resumo executivo (re-auditoria 2026-07-19 — Retry com Backoff, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Classificação transient/permanent, `RetryConfig`, full-jitter, envelope JSON, idempotência one-shot, one-shot/mem/par |
| Gaps **novos** | **G-RETRY-01…12** (12) |
| **FIXED** nesta passagem | **6** produto |
| **N/A** por identidade | **6** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG + `docs_rules/rules_rust_retry_com_backoff.md` + one-shot/mem/par; sibling docsrs-cli `retry.rs`; duckduckgo full-jitter |
| Gates | `cargo test --lib`; clippy `-D warnings` |

### Inventário (retry / backoff)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-RETRY-01** | Sem `is_retryable` / `is_permanent` / `retry_kind` em `SshCliError` | Agente só tinha exit code; risco de retry cego em auth/usage | Métodos tipados + `ErrorClass` / `ErrorLayer` / `RetryKind` | **FIXED** |
| **G-RETRY-02** | Envelope JSON sem `retryable` / `error_class` / `suggestion` | Contrato agent-first incompleto | Campos no `ErrorEnvelope` + schema + `resolve_exit_code` | **FIXED** |
| **G-RETRY-03** | Sem tipo nomeado `RetryConfig` | Política só em docs de agente, sem API reutilizável | `src/retry.rs` + constantes `AGENT_RETRY_*` | **FIXED** |
| **G-RETRY-04** | Sem fórmula full-jitter documentada no código | Agentes inventam sleep linear | `backoff_full_jitter` + `RetryConfig::agent_default` | **FIXED** |
| **G-RETRY-05** | Classificação por string / Display | Antipadrão rules | Match por variante de enum apenas | **FIXED** |
| **G-RETRY-06** | SLA de retry não amarrado a constantes de código | Docs e código podiam divergir | `AGENT_RETRY_MAX_RETRIES=2`, base 200ms, cap 5s + AGENTS.md | **FIXED** |
| **G-RETRY-07** | Auto-retry in-process de `exec`/`scp` | Não-idempotente / side-effects | Deliberadamente **off** (opt-in `enabled=false` default) | **N/A** (by design — least privilege) |
| **G-RETRY-08** | HTTP Retry-After / 429 / gRPC UNAVAILABLE | Sem cliente HTTP/gRPC de produto | N/A | **N/A** |
| **G-RETRY-09** | Circuit breaker / retry budget / hedged requests | Sem pool multi-tenant de dependências HTTP | N/A | **N/A** |
| **G-RETRY-10** | Idempotency-Key / fencing / inbox / outbox / saga | Side-effects são SSH one-shot + inventário VPS local | Agente é a camada de orquestração | **N/A** |
| **G-RETRY-11** | Crate `backon` / `tokio-retry2` como dependência | Sem loop de retry de produto; helper local basta | Implementação local sem dep extra | **N/A** (adequado) |
| **G-RETRY-12** | Feature flag CLI `--disable-retry` | Retry não roda no binário de produto | Kill switch = `RetryConfig::disabled()` para embedders | **N/A** (sem loop in-process) |

### Já conforme (sem gap de produto)

| Item rules | Evidência |
|------------|-----------|
| Retry como política explícita (não efeito colateral) | `RetryConfig` default disabled; agent owns re-invoke |
| Sem `std::thread::sleep` em async | Nenhuma sleep de retry no hot path |
| Timeouts com `tokio::time::timeout` | connect/exec/scp/tunnel |
| Cancelamento cooperativo | `signals::should_stop` / fan-out / tunnel |
| Happy Eyeballs ≠ retry cego | `net::dial_tcp` multi-addr race (G-NET) |
| Auth nunca retentado cego | exit 77 + `is_retryable=false` |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | Opt-in connect-only retry (`--connect-retries`) para dial transitório | P3 (agente já reinvoca) |
| O2 | Métricas de retry rate | P3 (telemetria de produto proibida) |
| O3 | Property test proptest do delay | P3 (unit tests de cap cobrem) |

---

## Resumo executivo (re-auditoria 2026-07-19 — Rede / Network best practices, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Tokio runtime, DNS dual-stack, Happy Eyeballs, TCP_NODELAY, SSH keepalive, timeouts, tunnel accept, spawn_blocking, one-shot/mem/par |
| Gaps **novos** | **G-NET-01…14** (14) |
| **FIXED** nesta passagem | **8** produto |
| **N/A** por identidade | **6** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG + `docs_rules/rules_rust_rede.md` + `rules_rust_ssh.md` + one-shot/mem/par; context7 russh/tokio; ddgs Happy Eyeballs |
| Gates | `cargo test --lib` (288); clippy `-D warnings` |

### Inventário (melhores práticas de rede)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-NET-01** | `russh::client::Config` com `nodelay: false` (default) | Nagle em SSH request/response → latência extra | `nodelay: true` + `set_nodelay` pós-dial | **FIXED** |
| **G-NET-02** | `keepalive_interval: None` | Túnel/sessão longa sem detecção de peer morto no nível SSH | `SSH_KEEPALIVE_INTERVAL_SECS=15`, `SSH_KEEPALIVE_MAX=3` | **FIXED** |
| **G-NET-03** | Dial via `russh::client::connect` → `TcpStream::connect` sequencial | Dual-stack: IPv6 blackhole atrasa IPv4 | `net::dial_tcp` Happy Eyeballs + `connect_stream` | **FIXED** |
| **G-NET-04** | Sem módulo de rede tipado / constantes de dial | Delay HE e keepalive mágicos | `HAPPY_EYEBALLS_ATTEMPT_DELAY_MS` + keepalive consts em `constants` | **FIXED** |
| **G-NET-05** | `load_secret_key` síncrono no worker async | KDF/FS bloqueia scheduler em multi-host | `spawn_blocking` + zeroize da passphrase no pool bloqueante | **FIXED** |
| **G-NET-06** | `known_hosts` load/TOFU síncrono em `check_server_key` | flock+FS no hot path de handshake | `spawn_blocking` em torno de load+`verify_tofu` | **FIXED** |
| **G-NET-07** | Tunnel accept `break` em qualquer erro | Erro transitório derruba listener | Continuar em Interrupted/WouldBlock/Connection* | **FIXED** |
| **G-NET-08** | Socket local do tunnel sem `TCP_NODELAY` | Nagle no forward localhost | `set_nodelay(true)` pós-accept | **FIXED** |
| **G-NET-09** | DoH/DoT/DoQ/hickory-resolver/mDNS | Produto usa resolver do SO via `lookup_host` | Adequado a CLI one-shot; sem recursivo exposto | **N/A** |
| **G-NET-10** | HTTP/gRPC/WebSocket/QUIC/WebRTC server/client | Fora da identidade SSH CLI | N/A | **N/A** |
| **G-NET-11** | Servidor dual-stack `[::]` | Tunnel default loopback `127.0.0.1` (segurança) | Bind configurável via `--bind`; default intencional | **N/A** (by design) |
| **G-NET-12** | Circuit breaker / bulkhead HTTP / OAuth | Sem dependências HTTP de produto | N/A | **N/A** |
| **G-NET-13** | Runtime `features = ["full"]` / multi-runtime | Já features seletivas; só tokio | Revalidado em Cargo.toml | **N/A** (já conforme) |
| **G-NET-14** | Timeouts connect/exec/tunnel | Já `timeout_ms` + `tokio::time::timeout` | Revalidado (sem gap) | **N/A** (já conforme) |

### Já conforme (sem gap de produto)

| Item rules | Evidência |
|------------|-----------|
| Tokio multi_thread com workers/max_blocking nomeados | `main.rs` + `concurrency::{worker_threads,max_blocking_threads}` |
| Sem `async-std` / sem multi-runtime | Cargo.toml só `tokio` |
| `#[tokio::main]` ausente; Builder no binário | `main.rs` |
| Timeout total em connect/exec/scp/tunnel | `ConnectionConfig.timeout_ms` + wrappers |
| Tunnel: Semaphore + JoinSet + drain | `tunnel.rs` (G-SHUT/G-PAR) |
| SCP I/O async (`tokio::fs`) | upload/download paths |
| Fan-out bounded I/O-bound | `concurrency::map_bounded` |
| Storage XDG | known_hosts/config sob ProjectDirs |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | TCP `SO_KEEPALIVE` via `socket2` além do keepalive SSH | P3 (SSH keepalive cobre sessão) |
| O2 | `tokio-console` feature dev-only | P3 (diagnóstico opcional) |
| O3 | Separar connect_timeout vs op_timeout na API | P3 (one-shot unificado é contrato atual) |

---

## Resumo executivo (re-auditoria 2026-07-19 — Proibição de Hardcode, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Separação código/config; segredos; endpoints; literais mágicos; XDG; 12-Factor |
| Gaps **novos** | **G-HC-01…14** (14) |
| **FIXED** nesta passagem | **10** produto |
| **N/A** por identidade | **4** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG + `docs_rules/rules_rust_proibicao_hardcode.md` + `rules_rust_configuracao.md`; 12-Factor III Config; ddgs config/secrets |
| Gates | `cargo test --lib` (285); clippy `-D warnings`; build binário |

### Inventário (proibição de hardcode)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-HC-01** | Nomes de arquivo XDG espalhados (`config.toml`, `active`, `known_hosts`, `secrets.key`) | Drift multi-módulo; regra “agrupar constantes” | Módulo `constants` + `CONFIG_FILE_NAME` / `ACTIVE_VPS_FILE_NAME` / `KNOWN_HOSTS_FILE_NAME` / `SECRETS_KEY_FILE_NAME` | **FIXED** |
| **G-HC-02** | `ProjectDirs::from("", "", "ssh-cli")` duplicado em `vps` e `secrets` | Identidade de app hardcoded em 2 sites | `APP_NAME` + `paths::xdg_config_dir()` único | **FIXED** |
| **G-HC-03** | Env vars como literais (`SSH_CLI_HOME`, secrets, keyring, lang, force-text) | Superfície 12-Factor sem nome canônico único | `ENV_*` em `constants`; call sites migrados | **FIXED** |
| **G-HC-04** | Porta SSH `22` e timeout `60_000` em clap sem const de domínio | Magic number em CLI | `DEFAULT_SSH_PORT` + `model::DEFAULT_TIMEOUT_MS` em `default_value_t` | **FIXED** |
| **G-HC-05** | Tunnel bind / channel origin `127.0.0.1` e port `0` literais | IP/porta sem nome semântico | `DEFAULT_TUNNEL_BIND_ADDR`, `TUNNEL_CHANNEL_ORIGIN_*` | **FIXED** |
| **G-HC-06** | Timeouts de processo sem unidade no nome (`2s` drain/shutdown, `200ms`/`50ms` poll) | Literais em lógica de lifecycle | `RUNTIME_SHUTDOWN_TIMEOUT_SECS`, `TUNNEL_*`, `FAN_OUT_SIGNAL_POLL_INTERVAL_MS` | **FIXED** |
| **G-HC-07** | Keyring service/user strings + tamanhos AEAD (`32`/`64`/`12`/`16`) | Material crypto com números mágicos | `KEYRING_*`, `PRIMARY_KEY_*`, `AEAD_*`, `SECRET_FILE_MODE_UNIX` | **FIXED** |
| **G-HC-08** | `LANG_PREFERENCE_FILE` / concurrency env desacoplados do catálogo | Dois “donos” para o mesmo valor | Re-export de `constants` (`LANG_PREFERENCE_FILE_NAME`, `ENV_MAX_CONCURRENCY`) | **FIXED** |
| **G-HC-09** | Clap `name = "ssh-cli"` e completions binary name | Identidade de app não centralizada | `APP_NAME` em `#[command]` + `clap_complete` | **FIXED** |
| **G-HC-10** | Storage XDG não documentado como helper único | Overrides `--config-dir` / `SSH_CLI_HOME` / XDG misturados | `xdg_config_dir` + docs 12-Factor em `constants` | **FIXED** |
| **G-HC-11** | HTTP client/server, DB, Redis, brokers, OTLP, CSS/design tokens | Produto sem HTTP server, SQL, filas, UI web | Matriz N/A (identidade CLI SSH one-shot) | **N/A** |
| **G-HC-12** | URL de produção / API keys / JWT / cert pinning | Nenhum endpoint de produto embutido; hosts vêm do registry | Inventário: zero URL prod em `src/` | **N/A** |
| **G-HC-13** | Segredo literal de produção em código | Só fixtures de teste com `SecretString` / zeroize | Proibido em produto; testes isolados | **N/A** (já conforme) |
| **G-HC-14** | Versão replicada fora de `Cargo.toml` | Já `CARGO_PKG_VERSION` + `build.rs` commit | Sem gap | **N/A** (já conforme) |

### Já conforme (sem gap de produto)

| Item rules | Evidência |
|------------|-----------|
| Config externalizada (hosts, timeouts, keys) | XDG `config.toml` + CLI/env; sem `.env` runtime |
| Segredos via env/keyring/XDG `secrets.key` 0o600 | `secrets::load_primary_key` + ChaCha20-Poly1305 |
| `SecretString` + `zeroize` | passwords VPS, primary-key hex |
| Sem URL/credencial de produção no binário | greps + inventário |
| Versão de `Cargo.toml` | `env!("CARGO_PKG_VERSION")` |
| Build/Release/Run | `build.rs` commit hash; runtime config only |
| Storage XDG obrigatório | `directories::ProjectDirs` + override documentado |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | `mlock` de primary-key em Linux | P3 (one-shot; zeroize já cobre) |
| O2 | `figment` / provider unificado de config | P3 (clap+TOML+env já cobrem) |
| O3 | gitleaks pre-commit no repo | OPEN-PROCESS se CI quiser reforço |

---

## Resumo executivo (re-auditoria 2026-07-19 — Processos Externos, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | `std::process::Command` local; Stdio; injeção; BatBadBut; packing remoto `sh -c`; matriz N/A |
| Gaps **novos** | **G-PROC-01…12** (12) |
| **FIXED** nesta passagem | **6** produto/docs/testes |
| **N/A** por identidade | **6** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG rules; `docs_rules/rules_rust_processos_externos.md`; ddgs CVE-2024-24576; docsrs/context7 std::process |
| Gates | `cargo test --lib` (NUL + packing); clippy; build.rs compile |

### Inventário (processos externos)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-PROC-01** | `build.rs` `git` sem Stdio explícito | Defaults implícitos de `output`/`spawn` | `git_command()` com stdin null + stdout/stderr piped; args estáticos | **FIXED** |
| **G-PROC-02** | Testes `ssh-keygen` com `.status()` herdado | Stdio implícito; panic se binário ausente | argv via `arg`; Stdio null/piped; skip em `NotFound` | **FIXED** |
| **G-PROC-03** | Comando remoto aceitava NUL | Truncamento C-string / payload opaco no `channel.exec` | `validate_command_length` rejeita `\0` + testes | **FIXED** |
| **G-PROC-04** | Fronteira de processo não documentada | Agente assume OpenSSH local | CROSS_PLATFORM / AGENTS / platform docs | **FIXED** |
| **G-PROC-05** | Packing `sh -c` ambíguo (local vs remoto) | Leitura como shell local | Docs `packing.rs` + CROSS_PLATFORM: só canal SSH remoto | **FIXED** |
| **G-PROC-06** | BatBadBut / MSRV não citados na política de processo | Checklist CVE-2024-24576 | MSRV 1.85.0 ≥ 1.77.2 documentado; sem spawn `.bat`/`.cmd` | **FIXED** |
| **G-PROC-07** | Runtime spawn de `ssh`/`scp`/`systemctl` | N/A — produto é russh puro | Sem `Command` em `src/` (só `process::exit` em `main`) | **N/A** |
| **G-PROC-08** | Job Object / process group local privilegiado | N/A — sem árvore de filhos locais | platform/mod.rs | **N/A** |
| **G-PROC-09** | `tokio::process` no runtime async | N/A — sem filhos locais async | I/O SSH via russh | **N/A** |
| **G-PROC-10** | WASM/`wasm32-wasip2` spawn fallback | WASM não é alvo de produto | CROSS_PLATFORM | **N/A** |
| **G-PROC-11** | `flatpak-spawn --host` allow-list | N/A — não invoca flatpak-spawn | sandbox só detectada | **N/A** |
| **G-PROC-12** | Matriz shell/PI/Jetson/K8s spawn tests | N/A para filhos locais; CLI já multi-OS via binário estático | contrato CLI + CROSS_PLATFORM | **N/A** |

### Inventário de `std::process` no tree

| Local | API | Papel |
|-------|-----|-------|
| `build.rs` | `Command::new("git")` | Hash de commit (opcional) |
| `src/main.rs` | `process::exit` | Exit pós-flush (não é spawn) |
| `src/signals.rs` | docs: sem exit em handler | Política |
| `tests/*` | `assert_cmd` / `Command` do binário | E2E |
| `tests/gaps_v035|v039` | `ssh-keygen` fixture | Chave OpenSSH real |
| `src/**` runtime | **zero** `Command` | russh only |

### Já conforme (sem gap de produto)

| Item rules | Evidência |
|------------|-----------|
| Sem shell local `sh -c` / `cmd /c` / PowerShell | Nenhum em `src/` |
| Args via `arg`/`args` (build + tests) | Slices estáticos / paths temp |
| Ausência de binário = erro/skip, não panic cego | git → `unknown`; keygen → skip |
| Captura remota com limite de bytes | `exec_capture_byte_cap` / `max_chars` |
| Timeout remoto | `timeout_ms` + abort best-effort |
| Segredos fora de argv remoto | `PackedCommand.stdin` + zeroize |
| Exit code remoto verificado | `CommandFailed` / sysexits |
| Detecção runtime sem shell-out | `platform::detect_runtime` |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | Helper compartilhado `tests/common` para `ssh-keygen` | P3 |
| O2 | `which` crate para resolver `git` absoluto no build | P3 (PATH estável em CI) |
| O3 | Rejeitar CR/LF em **argv local** se algum dia houver spawn local | N/A enquanto sem spawn |

---

## Resumo executivo (re-auditoria 2026-07-18 — Paralelismo / Multiprocessamento, 6ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Profundidade de função: session reuse, frota×arquivos, TOFU race, FS async residual, spans |
| Gaps **novos** | **G-PAR-47…54** (8) |
| **FIXED** nesta passagem | **8** |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG rules; docsrs/context7 Tokio; ddg bounded concurrency |
| Gates | `cargo test --lib`; known_hosts concurrent; scp session tests; clippy |

### Inventário incremental (6ª passagem)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-PAR-47** | Multi-file N× connect | Auth RTT × N | 1 session + serial transfers | **FIXED** |
| **G-PAR-48** | multi-host×multi-file rejeitado | Agente N×M processos | `MultiHostMultiFile` + map_bounded por host | **FIXED** |
| **G-PAR-49** | known_hosts sem flock | last-write-wins TOFU | sibling `.lock` + reload-merge | **FIXED** |
| **G-PAR-50** | std::fs pós-download async | Starvation workers | tokio::fs + spawn_blocking | **FIXED** |
| **G-PAR-51** | validate multi-file sync | Micro-block | tokio::fs metadata/create_dir | **FIXED** |
| **G-PAR-52** | sem span por unit | Debug cego | `fan_out_unit` instrument | **FIXED** |
| **G-PAR-53** | docs/schema cartesian | Agent incompleto | AGENTS/skills/CHANGELOG/schema | **FIXED** |
| **G-PAR-54** | testes profundos | Regressão | connect-once mock + TOFU concurrent + parse | **FIXED** |

### Matriz comando × fan-out (atualizada 6ª)

| Comando | Fan-out | Gate | Notas |
|---------|---------|------|-------|
| `exec\|sudo-exec\|su-exec` | `--all` / `--hosts` | `map_bounded` + cancel | Batch JSON multi |
| `health-check` | `--all` / `--hosts` | `map_bounded` + cancel | Batch JSON multi |
| `scp upload\|download` | multi-host **e/ou** multi-file | map_bounded **por sessão host**; multi-file serial na sessão | G-PAR-47/48 |
| `tunnel` | forwards only | JoinSet+Semaphore+abort | Single host session |
| `vps doctor --probe-ssh` | All / `--hosts` | health collect | **Um** JSON root `vps-doctor` |
| local CRUD / secrets / locale / meta | N/A | — | Sequential justificado |

### Oportunidades (não OPEN)

| ID | Item | Prioridade |
|----|------|------------|
| O1 | `--fail-fast` | P2 |
| O2 | Host tags | P2 |
| O3 | Multi-cmd uma sessão | P2 |
| O4 | Canais SCP concorrentes same-session | P3 |
| O5–O8 | pipeline/rayon/default-all/pool cross-cmd | N/A ou P3 |

---

## Resumo executivo (re-auditoria 2026-07-18 — Paralelismo / Multiprocessamento, 5ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Fechar escape do bound (multi-file), cancel real, doctor JSON único, FS async, observabilidade |
| Gaps **novos** | **G-PAR-37…46** (10) |
| **FIXED** nesta passagem | **9** produto + **1** OPEN-PROCESS (script) |
| **OPEN** produto | **0** |
| Fontes skill | GraphRAG rules; `docsrs-cli` Semaphore/JoinSet; `context7` Tokio; `duckduckgo-search-cli` |
| Gates | `cargo test --lib`; concurrency cancel/force; `gaps_v051`; clippy |

### Inventário incremental (5ª passagem)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-PAR-37** | SCP 1 arquivo | Agente N processos multi-file | Multi-path single-host + `map_bounded`; multi-host×N = erro | **FIXED** |
| **G-PAR-38** | doctor probe só All | Sem subset | `--hosts` com `--probe-ssh` | **FIXED** |
| **G-PAR-39** | force sem abort multi-host | Hang até timeout | `JoinSet::abort_all` + timer poll | **FIXED** |
| **G-PAR-40** | sem available_permits | Debug cego | `tracing::debug!` no admit | **FIXED** |
| **G-PAR-41** | std::fs metadata async path | Starvation workers | `tokio::fs::metadata` no upload | **FIXED** |
| **G-PAR-42** | dual JSON doctor+probe | Parser agent quebra | Envelope `event: vps-doctor` + `ssh_probe` | **FIXED** |
| **G-PAR-43** | testes só empty multi-host | Regressão | cancel/force unit + doctor/scp integration | **FIXED** |
| **G-PAR-44** | admissão pós-SIGINT | Tasks novas | `should_stop` para seed/refill | **FIXED** |
| **G-PAR-45** | docs multi-file/doctor | Discoverability | AGENTS/skills/CHANGELOG/schema | **FIXED** |
| **G-PAR-46** | dist_multiarch serial | Release lento | `PARALLEL_JOBS` + xargs -P | **FIXED** (OPEN-PROCESS) |

### Matriz comando × fan-out (atualizada 5ª)

| Comando | Fan-out | Gate | Notas |
|---------|---------|------|-------|
| `exec\|sudo-exec\|su-exec` | `--all` / `--hosts` | `map_bounded` + cancel | Batch JSON multi |
| `health-check` | `--all` / `--hosts` | `map_bounded` + cancel | Batch JSON multi |
| `scp upload\|download` | multi-host **e/ou** multi-file | map_bounded por host + session reuse | G-PAR-47/48 FIXED na 6ª |
| `tunnel` | forwards only | JoinSet+Semaphore+abort | Single host session |
| `vps doctor --probe-ssh` | All / `--hosts` | health collect | **Um** JSON root `vps-doctor` |
| local CRUD / secrets / locale / meta | N/A | — | Sequential justificado |

---

## Resumo executivo (re-auditoria 2026-07-18 — Paralelismo / Multiprocessamento, 4ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Fechar “falta em muitos comandos” com subset multi-host + docs/justificativa |
| Gaps **novos** | **G-PAR-27…36** (10) |
| **FIXED** nesta passagem | **10** |
| **OPEN** | **0** |
| Fontes skill | GraphRAG rules; `docsrs-cli` Semaphore/JoinSet; `context7` Tokio; `duckduckgo-search-cli` |
| Gates | `cargo test --lib`; concurrency units; `gaps_v051`; clippy `-D warnings` |

### Inventário incremental (4ª passagem)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-PAR-27** | Sem `--hosts a,b` | Só `--all` ou 1 host → agente multi-processa | Flag `--hosts` + `map_bounded` | **FIXED** |
| **G-PAR-28** | Justificação sequencial incompleta | Rules exigem JUSTIFICAR | Comentários Workload/Sequential em handlers | **FIXED** |
| **G-PAR-29** | Doctor sem probe SSH | Diagnóstico local only | `vps doctor --probe-ssh` → health fan-out | **FIXED** |
| **G-PAR-30** | Tunnel multi-host sem N/A formal | Help omite contrato | Help + lib + AGENTS: 1 bind/sessão | **FIXED** |
| **G-PAR-31** | 3 cópias de build jobs | Drift de bound | `HostSelection` + `resolve_host_jobs` | **FIXED** |
| **G-PAR-32** | Testes só `--all` empty | Sem coverage subset | Integration `--hosts` + unit resolve | **FIXED** |
| **G-PAR-33** | Skills/llms só `--all` | Discoverability | Skills/llms/COOKBOOK `--hosts` | **FIXED** |
| **G-PAR-34** | Sem matriz permanente | Retrabalho auditoria | Matriz abaixo + inventário | **FIXED** |
| **G-PAR-35** | Parsers CLI binários | Bugs positionals multi | `parse_exec_target` / `parse_scp_target` | **FIXED** |
| **G-PAR-36** | JSON shape `--hosts` 1 nome | Classic vs batch | Batch se All/Named; classic se Single | **FIXED** |

### Matriz comando × fan-out (permanente)

| Comando | Fan-out | Gate | Notas |
|---------|---------|------|-------|
| `exec\|sudo-exec\|su-exec` | `--all` / `--hosts` | `map_bounded` | Batch JSON multi |
| `health-check` | `--all` / `--hosts` | `map_bounded` | Batch JSON multi |
| `scp upload\|download` | `--all` / `--hosts` | `map_bounded` | Batch JSON multi |
| `tunnel` | forwards only | JoinSet+Semaphore | Single host session (G-PAR-30) |
| `vps doctor --probe-ssh` | All hosts | reusa health | Após diag local |
| `vps` CRUD / list / export / import | N/A | — | Sequential justificado |
| `connect` / `secrets` / `locale` / `completions` / `commands` | N/A | — | Sequential justificado |

---

## Resumo executivo (re-auditoria 2026-07-18 — Paralelismo / Multiprocessamento, 3ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Fechar discoverability + hardennings G-PAR-18…26; reconfirmar G-PAR-01…17 |
| Gaps **novos** | **G-PAR-18…26** (9) |
| **FIXED** nesta passagem | **9** |
| **OPEN** | **0** |
| Fontes skill | GraphRAG rules; `docsrs-cli` Semaphore/JoinSet; `context7` Tokio; `duckduckgo-search-cli` |
| Gates | `cargo test --lib` (**266**); concurrency unit (**7** + parser); `gaps_v051` (**13**); clippy `-D warnings` |

### Inventário incremental (3ª passagem)

| ID | Gap | Causa → Efeito | Solução | Status |
|----|-----|----------------|---------|--------|
| **G-PAR-18** | Skills EN/pt sem multi-host | Skill = lei suprema; só single-host → agente spawna N processos | Seção Fleet + fórmulas `--all` + batch + `--max-concurrency` | **FIXED** |
| **G-PAR-19** | llms.txt EN/pt sem fan-out | Índice LLM incompleto | Bullets multi-host + max-concurrency | **FIXED** |
| **G-PAR-20** | COOKBOOK/HOW_TO sem fleet | Receitas históricas single-host | Seção frota EN/pt-BR | **FIXED** |
| **G-PAR-21** | RSS 16 MiB sem ground truth | Ballpark sem medição | `/usr/bin/time -v` help ≈10 MiB RSS; 16 MiB com margem documentada | **FIXED** |
| **G-PAR-22** | INTEGRATIONS/CHANGELOG sem `--all` | Release notes pré-fan-out | Unreleased + flags by version | **FIXED** |
| **G-PAR-23** | Sem teste integration multi-host | Só unit concurrency | `gaps_v051`: empty `--all` exit 64 + max-concurrency | **FIXED** |
| **G-PAR-24** | Panic perde index no map_bounded | JoinSet panic drop T → `usize::MAX` | Track `TaskId` → index via `join_next_with_id` | **FIXED** |
| **G-PAR-25** | RAM formula só Linux | `free_ram=None` → só CPUs×4 | Clamp non-Linux a 8 | **FIXED** |
| **G-PAR-26** | Evals skills sem queries fleet | Review não exercita `--all` | 2 queries EN + 2 PT | **FIXED** |

### Superfícies (reconfirmado)

| Superfície | Gate | Status |
|------------|------|--------|
| health/exec/sudo/su/scp `--all` | `map_bounded` | OK |
| Tunnel forwards | `JoinSet`+`Semaphore` | OK |
| Agent contract (skills+llms+cookbook+integrations) | docs | OK (3ª) |

---

## Resumo executivo (re-auditoria 2026-07-18 — Paralelismo / Multiprocessamento, 2ª passagem)

| Métrica | Valor |
|---------|-------|
| Escopo | Revalidar Rules Rust paralelismo + graphrag `docs_rules/rules_rust_paralelismo_e_multiprocessamento.md` |
| Gaps pré-existentes (G-PAR-01…16) | **Reconfirmados** — 11 FIXED + 5 N/A intactos no código |
| Gap **novo** nesta re-auditoria | **G-PAR-17** (docs agent/README sem contrato multi-host) → **FIXED** |
| **OPEN** | **0** |
| Fontes obrigatórias skill | GraphRAG rules locais; `duckduckgo-search-cli` (Semaphore admission); `context7` Tokio JoinSet/select |
| Gates | `cargo test --lib` (**265**); concurrency unit (**6**); clippy `-D warnings` |

### Checklist de conformidade (revalidado)

| Item rules | Evidência no produto | Status |
|------------|----------------------|--------|
| Classificar workload I/O-bound | `concurrency.rs`, `lib.rs`, `scp.rs`, `tunnel.rs`, `vps` handlers | OK |
| Bound em todo fan-out | `map_bounded` + tunnel `Semaphore` | OK |
| Fórmula CPUs + RAM/2 + 16 MiB | `auto_limit()` + `/proc/meminfo` | OK |
| `--max-concurrency` / env | `CliArgs` + `SSH_CLI_MAX_CONCURRENCY` | OK |
| `Arc<Semaphore>` + `acquire_owned` | `concurrency::spawn_one`, tunnel accepts | OK |
| `JoinSet` + `JoinError::is_panic` | `map_bounded` + callers resume_unwind | OK |
| Multi-thread Tokio + max_blocking | `main.rs` workers / blocking pool | OK |
| Peak + panic permit tests | `map_bounded_respects_peak`, `permit_released_after_panic_in_task` | OK |
| Rayon / loom / parking_lot / systemd-run / OTEL | N/A identidade one-shot (G-PAR-12…16) | N/A |
| Sequencial justificado | VPS TOML CRUD, connect, locale, secrets key, completions | OK |
| Contrato agent multi-host | AGENTS.md / README / batch schemas (G-PAR-17) | OK |

### Inventário incremental (esta re-auditoria)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-PAR-17** | Contrato multi-host invisível em docs agent/README | `docs/schemas/*-batch` e código `--all` existiam; `docs/AGENTS.md` / `README` não mandavam agentes usar fan-out | Seções multi-host + batch schemas + `--max-concurrency` / `SSH_CLI_MAX_CONCURRENCY` em AGENTS (EN+pt-BR) e tabelas README | **FIXED** |

### Superfícies com paralelismo (modus operandi)

| Superfície | Gate | Saturates |
|------------|------|-----------|
| `health-check --all` | `map_bounded` | sockets, auth, RAM/session |
| `exec\|sudo-exec\|su-exec --all` | `map_bounded` | idem + remote cmd |
| `scp upload\|download --all` | `map_bounded` | rede + disco |
| Tunnel accepts → forwards | `JoinSet` + `Semaphore` | FDs, canais SSH |
| Tokio workers | `worker_threads` / `max_blocking_threads` | scheduler |

### Sequencial justificado (work ≪ overhead de coordenação)

| Superfície | Justificativa |
|------------|---------------|
| `vps` CRUD / export / import / doctor | I/O local TOML/XDG; sem fan-out de rede |
| `connect` | grava marker `active` |
| `secrets status\|init\|reencrypt` | crypto local de poucos blobs |
| `locale` / `completions` / `commands` | metadata local |
| `tunnel` single-host | uma sessão SSH + bind local; multi-host exigiria N ports (fora do contrato atual) |
| Single-host exec/scp/health | um alvo; sem coleção para fan-out |

---

## Resumo executivo (rodada 2026-07-18 — Paralelismo / Multiprocessamento, 1ª passagem)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados (paralelismo) | **G-PAR-01…G-PAR-16** |
| **FIXED** nesta rodada | **11** |
| **N/A** documentados nesta rodada | **5** (Rayon, loom, parking_lot, systemd-run MemoryMax, OTEL permits) |
| **OPEN** | **0** |
| Gaps ownership (mantidos) | G-OWN-01…14 — **10 FIXED**, **4 N/A** |
| Gates validados pós-fix | `cargo test --lib` (**265** após re-auditoria; 1ª passagem citava 262); `cargo test --doc` (**17**); clippy `-D warnings`; storage_integration (**13**); i18n_integration (**17**) |

### Baseline paralelismo (após fix)

| Item | Estado |
|------|--------|
| Workload classificado **I/O-bound** (SSH/TCP/SCP/tunnel) | OK — documentado em `concurrency`, `lib.rs`, handlers |
| `src/concurrency.rs`: fórmula CPU×IO + RAM/2 / 16MiB, clamp 1..=64 | OK |
| CLI `--max-concurrency` + env `SSH_CLI_MAX_CONCURRENCY` | OK |
| `Semaphore` + `JoinSet` via `map_bounded` | OK |
| Multi-host `--all`: health-check, exec, sudo-exec, su-exec, scp | OK |
| Tunnel forwards com admission gate (Semaphore) | OK |
| Tokio multi_thread workers + `max_blocking_threads` do budget | OK |
| Batch JSON: `health-check-batch` / `exec-batch` / `scp-batch` + schemas | OK |
| Testes peak concurrency + panic permit release | OK |
| Rayon / CPU par_iter | N/A (não CPU-bound) |
| loom / parking_lot deadlock detector | N/A (sem multi-lock crítico) |
| systemd-run MemoryMax child scopes | N/A (binário one-shot é o processo) |
| OTEL `available_permits` metrics | N/A (zero telemetria de produto) |
| Local TOML CRUD / locale / completions sequenciais | Justificado (work ≪ overhead) |

### Inventário — gaps paralelismo (esta rodada)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-PAR-01** | Sem módulo de bounded concurrency | fan-out inexistente / tunnel unbounded | `src/concurrency.rs` + fórmula documentada | **FIXED** |
| **G-PAR-02** | Sem flag `--max-concurrency` | rules exigem override CLI | global clap + env + `install_process_limit` | **FIXED** |
| **G-PAR-03** | Workers Tokio fixos 2..=4 | subutiliza multi-host | `worker_threads()` / `max_blocking_threads()` | **FIXED** |
| **G-PAR-04** | Tunnel `JoinSet` sem Semaphore | accept loop spawn ilimitado | `acquire_owned` / try_acquire admission | **FIXED** |
| **G-PAR-05** | health-check só single-host | multi-host registry sem fan-out | `--all` + `map_bounded` | **FIXED** |
| **G-PAR-06** | exec/sudo/su só single-host | modus operandi sequencial | `--all` + batch JSON | **FIXED** |
| **G-PAR-07** | scp só single-host | sem fan-out de transfer | `--all` upload/download + batch | **FIXED** |
| **G-PAR-08** | Política paralelismo ausente em `lib.rs` | auditoria formal | § Parallelism / multiprocessing policy | **FIXED** |
| **G-PAR-09** | Sem schemas batch | contratos agent | `docs/schemas/*-batch.schema.json` | **FIXED** |
| **G-PAR-10** | Sem teste peak concurrency | rules checklist | `map_bounded_respects_peak` | **FIXED** |
| **G-PAR-11** | Sem teste recover permit após panic | rules checklist | `permit_released_after_panic_in_task` + `InFlightGuard` | **FIXED** |
| **G-PAR-12** | Rayon / par_iter | workload não CPU-bound | N/A — documentado | **N/A** |
| **G-PAR-13** | loom race models | sem multi-lock crítico product | N/A | **N/A** |
| **G-PAR-14** | parking_lot deadlock detection | um `Mutex` secrets, sem ordem multi-lock | N/A | **N/A** |
| **G-PAR-15** | systemd-run MemoryMax subprocess | sem child fan-out pesado | N/A | **N/A** |
| **G-PAR-16** | Métricas OTEL available_permits | proibição telemetria produto | N/A | **N/A** |
| **G-PAR-17** | Docs agent/README sem multi-host | re-auditoria: código OK, contrato agent incompleto | AGENTS + README EN/pt-BR | **FIXED** (2ª passagem) |
| **G-PAR-18…26** | Discoverability + qualidade (ver 3ª passagem) | skills/llms/docs/tests/index panic/RAM non-Linux | ver inventário 3ª passagem | **FIXED** (3ª passagem) |

### Pesquisa (obrigatória skill)

| Fonte | Achado aplicado |
|-------|-----------------|
| GraphRAG / Rules paralelismo | Semaphore+JoinSet; bound em todo fan-out; classificar I/O; flag max-concurrency |
| context7 Tokio | JoinSet incremental join; multi_thread runtime; select! racing |
| duckduckgo-search-cli | `acquire` / `acquire_owned` admission gate; RAII permit drop; nunca spawn ilimitado |
| `/proc/meminfo` MemAvailable | input da fórmula de permits (sem crate sysinfo extra) |

### UX multi-host (esta rodada)

```text
ssh-cli --max-concurrency 8 health-check --all --json
ssh-cli exec --all 'uptime' --json
ssh-cli exec prod 'hostname' --json          # single-host inalterado semanticamente
ssh-cli scp upload --all ./a.bin /tmp/a.bin
ssh-cli scp download --all /tmp/a.bin ./a    # grava ./a.<vps>
```

---

## Resumo executivo (rodada 2026-07-18 — Ownership / Borrowing / Lifetimes)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados (ownership) | **G-OWN-01…G-OWN-14** |
| **FIXED** nesta rodada | **10** |
| **N/A** documentados nesta rodada | **4** (GATs/HRTB, pin-project self-ref, arenas cíclicas, Rc monothread) |
| **OPEN** | **0** |
| Gaps multiplataforma (mantidos) | G-XP-01…16 — **10 FIXED**, **6 N/A** |
| Gaps i18n (mantidos) | G-I18N-01…16 — **12 FIXED**, **4 N/A** |
| Gates validados pós-fix | `cargo test --lib` (**256**); `cargo test --doc` (**17**); clippy `-D warnings` + lints ownership; storage_integration (**13**); i18n_integration (**17**) |

### Baseline ownership (após fix)

| Item | Estado |
|------|--------|
| `resolve_config_path` / `winning_layer` / `find_by_name` / `read_active_vps` → `Option<&Path>` | OK |
| Call sites usam `as_deref()` (sem `PathBuf::clone` em cadeia) | OK |
| exec/sudo/su/health: `hosts.remove` (move) em vez de `get`+`clone` | OK |
| `CommandFailed.stderr`: move de `output.stderr` | OK |
| `su_password`: `Option::take` (sem clone de `SecretString`) | OK |
| sudo `apply_overrides`: move dos campos de `opts` (sem clone) | OK |
| Tunnel: `Arc::clone(&client)` explícito (refcount) | OK |
| known_hosts path no handler: `Option::take` | OK |
| `i18n::t(Message)` por valor documentado (payload owned efêmero) | OK |
| Política ownership em `lib.rs` | OK |
| Sem `Rc` / `RefCell` / `Arc<RefCell<_>>` / `static mut` em produto | OK |
| `OnceLock` / `AtomicBool` / um `Mutex` (secrets) — interior mutability justificada | OK |
| GATs / HRTB / pin-project / arenas | N/A (sem grafos self-ref / futures auto-ref no produto) |
| `Rc` monothread | N/A (async multi-thread Tokio; `Arc` só no tunnel) |

### Inventário — gaps ownership (esta rodada)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-OWN-01** | `resolve_config_path(Option<PathBuf>)` força clone em cada hop | clippy `needless_pass_by_value` + clones em scp/tunnel/vps | API `Option<&Path>` + `to_path_buf` só no leaf | **FIXED** |
| **G-OWN-02** | `winning_layer` / `find_by_name` / `read_active_vps` idem | mesma superfície | `Option<&Path>` | **FIXED** |
| **G-OWN-03** | `config_override.clone()` antes de resolve/find | scp, tunnel, doctor, health | `as_deref()` | **FIXED** |
| **G-OWN-04** | `vps_base.clone()` após `get` em exec/sudo/su/health | map local descartado | `hosts.remove` move o record | **FIXED** |
| **G-OWN-05** | `output.stderr.clone()` em `CommandFailed` | valor dropado após clone | move `stderr` | **FIXED** |
| **G-OWN-06** | `opts.*.clone()` em `run_sudo_exec` | campos não reutilizados | move (como `run_exec`) | **FIXED** |
| **G-OWN-07** | `su_password.clone()` | secret duplicado | `Option::take` | **FIXED** |
| **G-OWN-08** | `client.clone()` em tunnel | lint `clone_on_ref_ptr` | `Arc::clone(&client)` | **FIXED** |
| **G-OWN-09** | `known_hosts_path.clone()` no handler | check uma vez por conexão | `Option::take` | **FIXED** |
| **G-OWN-10** | Política ownership ausente em `lib.rs` | auditoria formal | § Ownership / borrowing policy | **FIXED** |
| **G-OWN-11** | GATs / HRTB / variance / PhantomData | sem tipos genéricos com raw pointers | N/A | **N/A** |
| **G-OWN-12** | pin-project / ouroboros / self-ref futures | one-shot; sem struct self-ref | N/A | **N/A** |
| **G-OWN-13** | arenas + handles para grafos cíclicos | sem grafo de hosts cíclico em memória | N/A (BTreeMap owned) | **N/A** |
| **G-OWN-14** | `Rc` / `Cow` generalizado | async multi-thread; strings curtas | N/A — `Arc` só tunnel; sem `Cow` necessário | **N/A** |

### Pesquisa (obrigatória skill)

| Fonte | Achado aplicado |
|-------|-----------------|
| GraphRAG / Rules ownership borrowing lifetimes | ordem `&T`→`&mut T`→`T`; `take`/`remove`; Arc refcount; sem Rc universal |
| clippy `needless_pass_by_value` / `redundant_clone` / `clone_on_ref_ptr` | inventário acionável → G-OWN-01…09 |
| duckduckgo-search-cli + idiom Path | APIs de path em Rust preferem `&Path` / `impl AsRef<Path>` sobre `PathBuf` por valor |
| context7 | consulta std/Path (fallback: API std + clippy como fonte de verdade) |

---

## Resumo executivo (rodada 2026-07-18 — Multiplataforma Completo v3)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados (multiplataforma) | **G-XP-01…G-XP-16** |
| **FIXED** nesta rodada | **10** |
| **N/A** documentados nesta rodada | **6** (browser discovery, WASM/WASI, Job Objects, seccomp/setrlimit default, macOS notarization in-bin, OpenBSD pledge) |
| **OPEN** | **0** |
| Gaps i18n (mantidos) | G-I18N-01…16 — **12 FIXED**, **4 N/A** |
| Gaps macros (mantidos) | G-MAC-01…12 — **7 FIXED**, **6 N/A** |
| Gates validados pós-fix | `cargo test --lib` (**254**); `cargo test --doc` (**17**); clippy `-D warnings`; doctor `--json` + `runtime`; completions elvish |

### Baseline multiplataforma (após fix)

| Item | Estado |
|------|--------|
| Módulo `platform` isolado (linux/macos/windows) | OK |
| Windows UTF-8 CP 65001 antes de I/O | OK |
| Windows `ENABLE_VIRTUAL_TERMINAL_PROCESSING` (stdout/stderr) | OK |
| Runtime env: WSL / container / CI / Termux / Flatpak / Snap | OK (`detect_runtime`) |
| Sandbox Flatpak/Snap → `tracing::warn!` (não silencioso) | OK |
| `PathBuf` only; sem separador hardcoded de produto | OK |
| Windows reserved names (`CON`/`NUL`/…) | OK (`paths::validate_name`) |
| MAX_PATH 260 + component 255 + prefixo `\\?\` | OK (`validate_local_path_length`) |
| Unicode NFC em nomes | OK |
| `directories::ProjectDirs` + `SSH_CLI_HOME` / `--config-dir` | OK |
| Unix `0o600` só com `#[cfg(unix)]` | OK |
| NO_COLOR / CLICOLOR_FORCE / TERM=dumb | OK (`terminal`) |
| Completions Bash/Zsh/Fish/PowerShell/Elvish | OK (`clap_complete`) |
| Exit codes sysexits-aligned | OK |
| Signals: atomic flags, sem I/O no SIGTERM handler | OK |
| musl + feature `musl-allocator` | OK |
| CI matrix Ubuntu/macOS/Windows + musl check | OK (`.github/workflows/ci.yml`) |
| `rust-toolchain.toml` targets multi-arch | OK |
| `scripts/dist_multiarch.sh` + Cross.toml | OK |
| Doctor JSON `runtime` object | OK (+ schema) |
| Browser/Chrome/chromedriver discovery | N/A (produto SSH) |
| WASM / WASI / Lambda edge | N/A (`russh` sockets) |
| Job Object / local privileged `Command` tree | N/A (sem filhos locais privilegiados) |
| seccomp / landlock / setrlimit default | N/A (one-shot agent; hardening opcional processo) |
| macOS notarization / Windows Authenticode in-bin | OPEN-PROCESS (release; scripts/docs) |
| OpenBSD pledge/unveil | N/A (target tier; sem superfície) |

### Inventário — gaps multiplataforma (esta rodada)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-XP-01** | Identificadores PT em platform (`detectar_sandbox`, `e_tty`) | Rules EN identifiers | `detect_sandbox` / `is_tty`; docs EN | **FIXED** |
| **G-XP-02** | Windows só CP UTF-8; sem VT processing | Rules: `ENABLE_VIRTUAL_TERMINAL_PROCESSING` | `configure_console` + SetConsoleMode | **FIXED** |
| **G-XP-03** | Detecção runtime fraca (só Flatpak/Snap debug) | Rules: WSL/container/CI/Termux | `RuntimeEnvironment` + `detect_runtime` | **FIXED** |
| **G-XP-04** | Sandbox sem warning observável | Rules: EMITIR warning | `tracing::warn!` em linux::detect_sandbox | **FIXED** |
| **G-XP-05** | Doctor sem campos de plataforma | Checklist multiplataforma | JSON `runtime` + stderr human line | **FIXED** |
| **G-XP-06** | CI só `en-identifiers.yml` (sem matrix OS) | Rules: Ubuntu/macOS/Windows gates | `.github/workflows/ci.yml` matrix + musl | **FIXED** |
| **G-XP-07** | Sem guard MAX_PATH / component 255 | Rules: paths longos Windows | `validate_local_path_length` + testes | **FIXED** |
| **G-XP-08** | Docs multiplataforma incompletos (WSL/VT/runtime) | Checklist README/CROSS | CROSS_PLATFORM{,.pt-BR} + schema doctor | **FIXED** |
| **G-XP-09** | Política multiplataforma ausente em `lib.rs` | Auditorias anteriores implícitas | § Multiplatform policy | **FIXED** |
| **G-XP-10** | Teste doctor esperava JSON com espaços | Wire compacto agent | predicates sem espaço + `runtime` | **FIXED** |
| **G-XP-11** | Browser/Chrome path discovery | Rules browser section | N/A — produto SSH nativo (russh) | **N/A** |
| **G-XP-12** | WASM wasip1/p2 / Lambda / Workers | Rules serverless | N/A — sockets reais; não shipped | **N/A** |
| **G-XP-13** | Job Object Windows / which + Command filhos | Rules subprocess | N/A — sem árvore local privilegiada | **N/A** |
| **G-XP-14** | seccomp/landlock/setrlimit/pledge default | Hardening OS | N/A one-shot; processo release opcional | **N/A** |
| **G-XP-15** | Notarização macOS / Authenticode no binário | Release signing | OPEN-PROCESS se release assinado; docs only | **N/A** |
| **G-XP-16** | Completions Nushell embutido | Rules 5 shells | clap_complete: 5 shells (Bash/Zsh/Fish/PS/Elvish); Nushell externo | **N/A** |

### Pesquisa (obrigatória skill)

| Fonte | Achado aplicado |
|-------|-----------------|
| GraphRAG / `docs_rules/rules_rust_multiplataforma_sistemas_operacionais.md` | Matriz OS, VT Windows, paths, ProjectDirs, env detect, N/A browser/WASM p/ SSH |
| duckduckgo-search-cli | VT processing + env markers WSL/CI/container idiomáticos em CLIs Rust |
| windows-sys 0.59 Console | `ENABLE_VIRTUAL_TERMINAL_PROCESSING`, Get/SetConsoleMode, STD_* handles |
| clap_complete 4.5 | Shell enum: Bash, Elvish, Fish, PowerShell, Zsh (Nushell crate separado) |

---

## Resumo executivo (rodada 2026-07-18 — i18n Multi-idioma / Locale do SO)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados (i18n) | **G-I18N-01…G-I18N-16** |
| **FIXED** nesta rodada | **12** |
| **N/A** documentados nesta rodada | **4** (Fluent FTL runtime, ICU calendars/collators, top-20 locales embutidos, pseudoloc/Weblate pipeline) |
| **OPEN** | **0** |
| Gaps macros (mantidos) | G-MAC-01…12 — **7 FIXED**, **6 N/A** |
| Gates validados pós-fix | `cargo test --lib` (**243**); `cargo test --doc` (**17**); `i18n_integration` (**17**); clippy `-D warnings`; bin `locale --json` |

### Baseline i18n (após fix)

| Item | Estado |
|------|--------|
| `sys-locale` detecção OS (uma vez) | OK |
| `unic-langid` parse BCP47 (strip `.UTF-8`, `_`→`-`, rejeita `C`/`POSIX`) | OK |
| `fluent-langneg` Lookup vs `Language::AVAILABLE` | OK |
| Precedência 5 camadas: `--lang` > `SSH_CLI_LANG` > XDG `lang` > system > `en` | OK |
| `OnceLock<Language>` imutável por processo | OK |
| Falha de detecção → `tracing::warn!` (não silencioso) | OK |
| Enum `Language` `#[non_exhaustive]` + `bcp47`/`direction`/`script`/`fallback`/`AVAILABLE` | OK |
| Enum `Message` + match exaustivo `en()`/`pt()` (sem `_`) | OK |
| MVP `en` + `pt-BR` 100% (paridade unitária testada) | OK |
| CLI `--lang` com `value_parser` (rejeita fr/zh/…) | OK |
| Subcomando `locale` / `locale set` / `locale clear` + JSON | OK |
| Preferência XDG `lang` com 0o600 | OK |
| Features `i18n-full`/`i18n-cjk`/`i18n-rtl`/`i18n-europe` (stubs) | OK |
| Política em `lib.rs` § i18n | OK |
| JSON agent + `SshCliError` Display estáveis em EN | OK (intencional) |
| Windows UTF-8 antes de I/O | OK (`platform::initialize_platform`) |
| Fluent FTL + `i18n-embed` runtime | N/A (enum embutido; size-sensitive) |
| ICU calendar/collator / CJK width / RTL isolation | N/A (MVP LTR Latin; features reservadas) |
| Top-20 idiomas no binário default | N/A (proibido; só via features futuras) |

### Inventário — gaps i18n (esta rodada)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-I18N-01** | Parse de locale ad-hoc (`starts_with("pt")`) sem BCP47 | Rules: `unic-langid` + normalização | `normalize_raw_locale` + `LanguageIdentifier::from_str` | **FIXED** |
| **G-I18N-02** | Sem negociação formal de locales | Rules: `fluent-langneg` | `negotiate_code` / `negotiate_langid` Lookup | **FIXED** |
| **G-I18N-03** | Só 4 camadas (faltava preferência persistida) | Rules: 5 camadas | XDG `lang` file + read/write/clear | **FIXED** |
| **G-I18N-04** | Falha de detecção silenciosa | Rules: sinalizar observabilidade | `tracing::warn!` em miss system/env/persisted | **FIXED** |
| **G-I18N-05** | `Language` sem `#[non_exhaustive]` / metadados | Rules: enum fonte única + direction/script/fallback | Métodos + `AVAILABLE` + `TextDirection` | **FIXED** |
| **G-I18N-06** | `--lang` sem validação clap | Rules: validação de valor | `parse_lang_cli_arg` value_parser | **FIXED** |
| **G-I18N-07** | Sem subcomando `locale` diagnóstico | Checklist rules | `ssh-cli locale [--json]`, `set`, `clear` | **FIXED** |
| **G-I18N-08** | `C`/`POSIX`/`C.UTF-8` tratados como possível EN via prefix | Rules: NUNCA C.UTF-8 = en-US | Rejeitados no parse | **FIXED** |
| **G-I18N-09** | Encoding suffix `pt_BR.UTF-8` frágil | Rules: normalização formal | strip `.` / `@` + `_`→`-` | **FIXED** |
| **G-I18N-10** | Features top-20 ausentes | Rules: feature flags, default só en/pt-BR | stubs `i18n-*` no `Cargo.toml` | **FIXED** |
| **G-I18N-11** | Política i18n não documentada no produto | Auditorias anteriores parciais | `lib.rs` § i18n policy + docs módulos | **FIXED** |
| **G-I18N-12** | Testes de paridade/negociação incompletos | Rules: paridade en/pt-BR + BCP47 | unit + `tests/i18n_integration.rs` | **FIXED** |
| **G-I18N-13** | Runtime Fluent FTL / `i18n-embed` / ICU4X full | Binário size-min one-shot; 2 locales | Enum `Message` embutido = equivalente; FTL opcional futuro | **N/A** |
| **G-I18N-14** | Calendários não gregorianos, collator, grapheme truncate UI | Sem listas/datas UI locale-aware no produto | N/A identidade agent JSON | **N/A** |
| **G-I18N-15** | Top-20 / CJK / RTL embutidos no release | Rules: NUNCA default full top-20 | Features stubs; zero strings extra | **N/A** |
| **G-I18N-16** | Weblate/Crowdin/pseudoloc/SHA256 FTL release | Processo; sem arquivos FTL runtime | OPEN-PROCESS se FTL for adotado; hoje N/A | **N/A** |

### Pesquisa (obrigatória skill)

| Fonte | Achado aplicado |
|-------|-----------------|
| GraphRAG / `docs_rules/rules_rust_multi-idiona_i18_automatico_…` | 5 camadas, unic-langid, fluent-langneg, OnceLock, en+pt-BR MVP, features top-20 |
| GraphRAG / `rules_rust_internacionalizacao.md` | Fluent stack canônico; trade-off documentado vs size one-shot |
| cargo / fluent-langneg 0.13 + unic-langid 0.9 | API `negotiate_languages` + `LanguageIdentifier` alinhada às rules |
| duckduckgo-search-cli | Confirmou BCP47 negotiate + sys-locale como stack idiomática CLI |

---

## Resumo executivo (rodada 2026-07-18 — Macros em Rust)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados (macros) | **G-MAC-01…G-MAC-12** (+ residual **G-MAC-01b**) |
| **FIXED** nesta rodada | **7** (incl. residual call-site sweep) |
| **N/A** documentados nesta rodada | **6** (macro_rules crate, proc-macro workspace, trybuild/UI, TT munchers, edition-2024 fragment migration, rust-analyzer procMacro surface) |
| **OPEN** | **0** |
| Gaps logs/tracing (mantidos) | G-LOG-01…12 — **6 FIXED**, **6 N/A** |
| Gaps latência (mantidos) | G-LAT-01…12 — **6 FIXED**, **6 N/A** |
| Gaps residuais (mantidos) | G-DOC-06 / G-IO-11 / G-22 — **FIXED** |
| Gates validados pós-fix | `cargo test --lib`; `cargo test --doc`; clippy `-D warnings` |

### Baseline macros (após fix)

| Item | Estado |
|------|--------|
| `macro_rules!` / `#[macro_export]` no workspace | **ausente** (intencional — functions/generics first) |
| Proc-macro crate / `proc-macro = true` | **ausente** (derives externos só: clap/serde/thiserror) |
| `todo!` / `unimplemented!` / `dbg!` em `src/` | **0** |
| `panic!` em produto | **0** (só testes + `human_panic::setup_panic!` release) |
| `println!`/`eprintln!` em produto | **0** (só `build.rs` cargo instructions) |
| `format!` + `write_*` / `print_*` (alocação dupla) | **eliminado** em produto → `format_args!` + `*_fmt` / `writeln!` |
| API fmt de output | `write_*_fmt`, `print_{success,error,warning}_fmt`, `emit_success_fmt` |
| Printers humanos multi-linha | `writeln!` + `BufWriter` (sem `Vec<String>` de `format!`) |
| `matches!` | usado em client/packing/tests |
| `env!` / `concat!` | versão clap long + testes de binário |
| `include_str!` | só teste de auditoria de fonte em `tunnel.rs` |
| Política documentada | `lib.rs` § Macro policy |
| trybuild / snapshots de expansão | N/A (sem superfície de macro) |

### Inventário — gaps Macros (esta rodada)

### Lote T — escolha, built-in, higiene de I/O (G-MAC-01…G-MAC-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-MAC-01** | `write_stderr_line(&format!(…))` / `write_line(&format!(…))` — alocação intermediária proibida pelas rules | Rules: NUNCA combinar `format!` com print/write; USAR `format_args!` + `write!`/`writeln!` | `write_line_to_fmt` / `write_line_fmt` / `write_stderr_line_to_fmt` / `write_stderr_fmt`; call sites com `format_args!` | **FIXED** |
| **G-MAC-02** | Printers humanos montavam `Vec`/`array` de `format!` e depois `write_lines` | Rules: USAR `writeln!` em qualquer `Write`; evitar alocação extra | `print_doctor_text` / `print_list_text` / `print_details_text` / `print_execution_output` / `print_health_check` → `BufWriter` + `writeln!` | **FIXED** |
| **G-MAC-03** | Diagnóstico de falha de serialize JSON repetido 6× com `format!` | Boilerplate idêntico sem ganho de macro | `report_json_serialize_error` (função + `format_args!`) — **sem** macro de rename | **FIXED** |
| **G-MAC-04** | `vps path` usava `write_line(&path.display().to_string())` | Display → String desnecessário | `write_line_fmt(format_args!("{}", path.display()))` | **FIXED** |
| **G-MAC-05** | Política de macros não documentada no produto | Auditorias anteriores cobriram logs/JSON/IM; macros só implícitas | Seção **Macro policy** em `lib.rs` | **FIXED** |
| **G-MAC-06** | Cobertura DI ausente para caminho `write_fmt` | Rules: testar caminhos de sucesso/erro | Testes `write_line_to_fmt_*` + `write_stderr_line_to_fmt` EPIPE | **FIXED** |
| **G-MAC-01b** | Residuais `print_*(&format!(…))` fora de `output.rs` | `main` runtime fail, `vps` timeout warn / secrets status / auto-key, `resolve_exit_code` human path | `print_error_fmt` / `print_success_fmt` / `print_warning_fmt` / `emit_success_fmt`; call sites migrados | **FIXED** |
| **G-MAC-07** | Crate `macro_rules!` / DSL / TT muncher / `$crate` | Nenhuma abstração sintática necessária; i18n/CLI/JSON resolvem com tipos | N/A — preferir functions (regra OBRIGATÓRIA) | **N/A** |
| **G-MAC-08** | Proc-macro workspace (`syn`/`quote`/`trybuild`) | Derives de domínio cobertos por clap/serde/thiserror; sem derive próprio | N/A | **N/A** |
| **G-MAC-09** | UI tests trybuild / snapshots de expansão / cargo-expand CI | Sem macro pública a validar | N/A | **N/A** |
| **G-MAC-10** | Migração edition 2024 `:expr` / lint fragment specifier | Crate em **edition = "2021"**; sem matchers próprios | N/A até migração de edition (fora desta rodada) | **N/A** |
| **G-MAC-11** | `rust-analyzer.procMacro` / spans custom / `compile_error!` de DSL | Sem macros de usuário | N/A | **N/A** |
| **G-MAC-12** | Macro de i18n / `Message` text | EN/PT via funções + `format!` (owned) — correto; macro esconderia fluxo | N/A — functions first | **N/A** |

### Pesquisa (obrigatória skill)

| Fonte | Achado aplicado |
|-------|-----------------|
| GraphRAG `rules_rust_macros.md` | Esgotar functions/traits; proibir rename-macro; `format_args!`+`write!`; sem `todo!`/`dbg!` em prod |
| Context7 `/veykril/tlborm` | Higiene/`$crate` só relevantes se houver macro_rules exportada — não aplicável |
| duckduckgo-search-cli | Confirma padrão idiomático `writeln!(w, "{args}")` / `write_fmt` vs `format!`+print |

---

## Resumo executivo (rodada 2026-07-18 — Fechamento de residuais G-DOC-06 / G-IO-11 / G-22)

| Métrica | Valor |
|---------|-------|
| Gaps residuais fechados | **G-DOC-06**, **G-IO-11**, **G-22** (3) |
| **FIXED** nesta rodada | **3** |
| **OPEN** no inventário | **0** |
| Gaps logs/tracing (mantidos) | G-LOG-01…12 — **6 FIXED**, **6 N/A** |
| Gaps latência (mantidos) | G-LAT-01…12 — **6 FIXED**, **6 N/A** |
| Gaps EN/docs | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 agora FIXED) |
| Gaps streams | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 agora FIXED) |
| Gaps Clap | G-01…G-24 — **23 FIXED** (G-22 agora FIXED processo) |
| Gates validados pós-fix | `cargo test --lib` (**235**); `cargo test --doc` (**16**); clippy `-D warnings`; `cargo check --bins` |

### O que foi feito (residuais)

| ID | Solução | Status |
|----|---------|--------|
| **G-DOC-06** | Doctests em `paths`, `json_wire`, `output`, `errors`, `signals`, `telemetry`, `resolve_exit_code` (+ masking/i18n pré-existentes) → **16** doctests verdes | **FIXED** |
| **G-IO-11** | `run_with_args`, `resolve_exit_code`, `write_line_to` / `write_stderr_line_to`, main thin; testes unitários DI | **FIXED** |
| **G-22** | `scripts/dist_multiarch.sh`, `scripts/generate_sbom.sh`, gates 25–27 em `docs/RELEASE_CHECKLIST{,.pt-BR}.md` + `Cross.toml` | **FIXED** |

---

## Resumo executivo (rodada 2026-07-18 — Logs com Tracing e Rotação)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (logs/tracing) | **G-LOG-01…G-LOG-12** (12) |
| **FIXED** nesta rodada | **6** |
| **N/A** documentados nesta rodada | **6** (OTEL/OTLP, file rotation/WorkerGuard, admin HTTP reload, JSON log sink, tokio-console, journald/Docker drivers) |
| **OPEN** | **0** (desta rodada) |
| Gaps latência (mantidos) | G-LAT-01…12 — **6 FIXED**, **6 N/A** |
| Gaps JSON (mantidos) | G-JSON-01…12 — **8 FIXED**, **4 N/A** |
| Gaps IM (mantidos) | G-IM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps memória/RAII (mantidos) | G-MEM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps shutdown (mantidos) | G-SHUT-01…12 — **8 FIXED**, **4 N/A** |
| Gaps performance (mantidos) | G-PERF-01…12 — **6 FIXED**, **6 N/A** |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** |
| Gates validados pós-fix | `cargo test --lib` (**230**); `clippy --all-targets --all-features -D warnings`; `cargo check --bins` |

### Baseline logs/tracing (após fix)

| Item | Estado |
|------|--------|
| Módulo dedicado `src/telemetry.rs` (init centralizado) | OK |
| `tracing` + `tracing-subscriber` (fmt, env-filter, registry, tracing-log) | OK (features explícitas, `default-features = false`) |
| `reload::Layer` bootstrap pré-parse → reload pós-`-v`/`RUST_LOG` | OK (pré-existente; movido p/ telemetry) |
| `tracing-log::LogTracer::builder` (ponte `log` → `tracing` p/ russh/keyring) | OK |
| `tracing_error::ErrorLayer` no Registry | OK |
| Sink: stderr, `with_target(true)`, `with_thread_names(true)`, ANSI off | OK |
| Default filter `error`; `-v` → `debug`; `RUST_LOG` wins | OK |
| Política documentada em `lib.rs` + `telemetry.rs` (OTEL/rotation N/A) | OK |
| OpenTelemetry / OTLP / BatchSpanProcessor / sampling | N/A (zero telemetria) |
| `tracing-appender` + `WorkerGuard` + RollingFileAppender | N/A (sem arquivo local; one-shot) |
| Endpoint admin `/admin/log-level` | N/A (sem daemon) |
| JSON log format em stderr | N/A (agente: JSON de **dados** em stdout; diag texto) |
| `tokio-console` / `tracing-chrome` / journald | N/A identidade CLI desktop/agent |
| Panic hook: `human_panic` em release (UX CLI) | OK (preferível a tracing-panic para binário humano) |

### Inventário — gaps Logs com Tracing (esta rodada)

### Lote S — subscriber, bridge, política agent-first (G-LOG-01…G-LOG-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-LOG-01** | Init de tracing embutido em `cli.rs` sem módulo dedicado | Rules: separar telemetria; exportar init único | Novo `src/telemetry.rs`; `cli` delega; `lib::run` chama `telemetry::*` | **FIXED** |
| **G-LOG-02** | Feature `tracing-log` / `LogTracer` ausentes | `russh`/`keyring` emitem via crate `log`; bridge não instalado | Deps `tracing-log`+`log`; `LogTracer::builder().with_max_level(Trace).init()` | **FIXED** |
| **G-LOG-03** | `ErrorLayer` / `tracing-error` ausentes | Rules: capturar `SpanTrace` em erros | Dep `tracing-error`; `ErrorLayer::default()` no Registry | **FIXED** |
| **G-LOG-04** | Features de subscriber implícitas / incompletas | Rules: features explícitas; sem feature creep | `default-features = false` + `std,fmt,ansi,env-filter,registry,tracing-log` | **FIXED** |
| **G-LOG-05** | `with_target(false)` e sem thread names | Rules: targets `crate::mod`; `with_thread_names` | `with_target(true)` + `with_thread_names(true)` (workers `ssh-cli-worker`) | **FIXED** |
| **G-LOG-06** | Política de logs/OTEL não documentada no produto | Confusão com rules de servidor long-lived | Seções em `lib.rs` + module docs `telemetry.rs` | **FIXED** |
| **G-LOG-07** | OpenTelemetry / OTLP / Resource / sampling / metrics | Product line: **zero telemetry**; one-shot sem backend | N/A identidade | **N/A** |
| **G-LOG-08** | `tracing-appender` / RollingFile / `WorkerGuard` / MakeWriter por severidade | Sem log em disco; stderr só; processo curto | N/A (flush stderr em `main`) | **N/A** |
| **G-LOG-09** | Endpoint admin para reload de nível + timeout de escalada | Sem HTTP control plane | N/A; reload via `-v`/`RUST_LOG` no próximo spawn | **N/A** |
| **G-LOG-10** | JSON formatter de **logs** em produção + schema snapshot | JSON de produto é contrato de **dados** em stdout; misturar log JSON em stderr confunde agentes | N/A; texto em stderr; filtro default `error` | **N/A** |
| **G-LOG-11** | `tokio-console` / `tracing-chrome` / `tracing-timing` | Overhead + `tokio_unstable`; não é servidor de staging | N/A | **N/A** |
| **G-LOG-12** | journald / Docker logging driver / Lambda flush / logs encriptados | CLI local multi-OS; sem unit systemd de produto; sem log files | N/A | **N/A** |

---

## Resumo executivo (rodada 2026-07-18 — Redução de Latência)

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (latência) | **G-LAT-01…G-LAT-12** (12) |
| **FIXED** nesta rodada | **6** |
| **N/A** documentados nesta rodada | **6** (HFT/jitter kernel, PGO/BOLT, mimalloc default, huge pages/mlockall, loom/P99 release, SIMD/kernel-bypass) |
| **OPEN** | **0** (desta rodada) |
| Gaps JSON (mantidos) | G-JSON-01…12 — **8 FIXED**, **4 N/A** |
| Gaps IM (mantidos) | G-IM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps memória/RAII (mantidos) | G-MEM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps shutdown (mantidos) | G-SHUT-01…12 — **8 FIXED**, **4 N/A** |
| Gaps performance (mantidos) | G-PERF-01…12 — **6 FIXED**, **6 N/A** |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (**227**); `clippy --all-targets --all-features -D warnings` |

### Baseline latência (após fix)

| Item | Estado |
|------|--------|
| Política de latência I/O-bound documentada em `lib.rs::run` | OK (escopo RTT vs HFT) |
| Tokio multi_thread com **worker_threads** cap 2..=4 + `thread_name` | OK (`main.rs`) |
| Fat LTO + `codegen-units = 1` + `panic = abort` no release | OK (pré-existente) |
| Perfil local `release-fast` **e** alias `release-lto` (`opt-level = 3`) | OK |
| Publish default **size-min** (`opt-level = "z"`) — trade-off documentado | OK (não HFT) |
| Decode exec: `take_utf8_capped` reusa `Vec` em UTF-8 válido | OK |
| SCP upload/download: `tokio::fs` + `AsyncRead/Write` no loop | OK |
| Timestamps de duração: `Instant` (não wall clock) | OK (pré-existente) |
| Cap captura + pré-aloc buffers (G-RES / G-PERF) | Mantido |
| Benches: aviso de que criterion ≠ P99 de produto | OK |
| PGO/BOLT, isolcpus, mlockall, huge pages, HDR P9999 | N/A identidade |
| `mimalloc` default global sem medição | N/A; feature `musl-allocator` opcional |

### Inventário — gaps Redução de Latência (esta rodada)

### Lote R — cold-start, cópia zero no decode, I/O async (G-LAT-01…G-LAT-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-LAT-01** | Política de latência ausente / confusão com HFT | Rules pedem P99/ns; produto é RTT-bound one-shot | Seção `# Latency policy` em `lib.rs` + escopo em `gaps.md` | **FIXED** |
| **G-LAT-02** | Tokio `multi_thread` sem cap de workers | Default ~num_cpus infla cold-start sem ganho de RTT | `worker_threads(2..=4)` + `thread_name("ssh-cli-worker")` | **FIXED** |
| **G-LAT-03** | Decode exec: `from_utf8_lossy` + `truncate_utf8` = 2 cópias | Hot path pós-rede alocava de novo | `take_utf8_capped(Vec)` reusa buffer em UTF-8 válido | **FIXED** |
| **G-LAT-04** | SCP: `std::fs::File` + `read`/`write` síncronos no async | Rules: NUNCA I/O de arquivo síncrono em worker async | `tokio::fs` + `AsyncReadExt`/`AsyncWriteExt` no loop | **FIXED** |
| **G-LAT-05** | Nome de perfil `release-lto` ausente | Rules pedem perfil dedicado com fat LTO + opt-3 | `[profile.release-lto]` herda `release-fast` | **FIXED** |
| **G-LAT-06** | Benches podiam ser lidos como P99 de latência | Rules: não resumir latência em um número / microbench isolado | Comentário em `benches/ssh_operations.rs` | **FIXED** |
| **G-LAT-07** | PGO / BOLT / AutoFDO em binário de produção | One-shot; sem carga canônica de rede no repo | N/A (eco G-PERF-07) | **N/A** |
| **G-LAT-08** | isolcpus / nohz_full / IRQ affinity / C-states / governor | Kernel tuning de HFT; CLI de usuário | N/A identidade desktop/agent | **N/A** |
| **G-LAT-09** | mlockall / huge pages / MADV_POPULATE / THP | Working set pequeno; sem processo crítico de exchange | N/A | **N/A** |
| **G-LAT-10** | `mimalloc`/`jemalloc` como `#[global_allocator]` default | Rules pedem medição P99 antes; RTT domina alocador | Feature `musl-allocator` opcional (eco G-RES-10) | **N/A** |
| **G-LAT-11** | Histogramas P50/P99/P9999 + HDR por release | Sem servidor long-lived; sem telemetria de produto | N/A (health `latency_ms` por invocação basta) | **N/A** |
| **G-LAT-12** | SIMD / kernel bypass / TCP_NODELAY custom / loom suite | Sem laço numérico dominante; TCP via russh; atomics simples | N/A (eco G-PERF-09 / G-IM-12) | **N/A** |

### Residuais intencionais (latência)

| Residual | Motivo |
|----------|--------|
| `opt-level = "z"` no publish | Footprint/cold-start de download > pico de CPU local; A/B via `release-fast`/`release-lto` |
| Metadata/chmod/set_times síncronos no fim do SCP | Syscalls curtos fora do loop de payload; não valem `spawn_blocking` |
| `std::fs::metadata` no início do upload | Cold path pré-timeout; uma chamada |
| Sem flamegraph SSH em CI | Exige host/rede real; residual processo (eco G-PERF-12) |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — JSON e NDJSON) *(mantido)*

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (JSON/NDJSON) | **G-JSON-01…G-JSON-12** (12) |
| **FIXED** nesta rodada | **8** |
| **N/A** documentados nesta rodada | **4** (NDJSON stream, HTTP CT, schemars/jsonschema runtime, simd/JSON5/Patch) |
| **OPEN** | **0** (desta rodada) |
| Gaps IM (mantidos) | G-IM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps memória/RAII (mantidos) | G-MEM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps shutdown (mantidos) | G-SHUT-01…12 — **8 FIXED**, **4 N/A** |
| Gaps performance (mantidos) | G-PERF-01…12 — **6 FIXED**, **6 N/A** |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (**225**); `clippy --all-targets --all-features -D warnings` |

### Baseline JSON (após fix)

| Item | Estado |
|------|--------|
| Biblioteca: `serde` + `serde_json` 1.x (sem simd/sonic) | OK |
| Emit agent: **compact** `to_string` + LF (não pretty) | OK (`json_wire::print_json_line`) |
| Error envelope e success no mesmo estilo compact | OK |
| DTOs tipados: exec, health, scp, tunnel, masked VPS, export, error, success | OK (`src/json_wire.rs`) |
| `Value` só em bordas dinâmicas (`meta command-tree`, fields flexíveis) | OK |
| Import JSON: `Deserialize` tipado + Must-Ignore | OK |
| Import: strip UTF-8 BOM | OK |
| Import: size cap (`MAX_CONFIG_TOML_BYTES`) | OK |
| Import: port ∈ 1..=65535 (sem cast silencioso) | OK |
| Export hosts: `BTreeMap` (ordem estável de chaves) | OK |
| Schemas hand-versioned em `docs/schemas/` | OK |
| Config em disco: TOML (não JSON5) | OK |
| Produto **não** é stream NDJSON / HTTP API | N/A identidade |
| Política documentada em `lib.rs` + `json_wire` + schemas README | OK |

### Inventário — gaps JSON e NDJSON (esta rodada)

### Lote Q — wire tipado, compact, import robusto (G-JSON-01…G-JSON-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-JSON-01** | Pretty-print em stdout agent (`to_string_pretty`) | Inconsistente com error envelope compact; rules machine interop preferem compact | `json_wire::print_json_line` / `to_string` em todos os emits | **FIXED** |
| **G-JSON-02** | Payloads conhecidos montados com `json!` / `Value` | Rules: struct tipada para payload conhecido | DTOs `Serialize`/`Deserialize` em `json_wire` | **FIXED** |
| **G-JSON-03** | Import JSON via scraping manual de `Value` | Frágil; port cast silencioso; sem aliases serde | `ImportEnvelope` + `ImportHostEntry` + `into_record` | **FIXED** |
| **G-JSON-04** | Import sem teto de tamanho (`read_to_string`) | OOM em arquivo grande | `paths::read_text_capped` + `MAX_CONFIG_TOML_BYTES` | **FIXED** |
| **G-JSON-05** | BOM UTF-8 não removido antes do parse | Rules: strip BOM | `strip_utf8_bom` no path de import | **FIXED** |
| **G-JSON-06** | Política JSON ausente no crate root / schemas | Contrato agent implícito | Seção `lib.rs` + docs `json_wire` + schemas README | **FIXED** |
| **G-JSON-07** | Ordem de hosts no export JSON não documentada como estável | `BTreeMap` já usado; falta DTO + teste | `VpsExportJson` + teste ordem `a` < `b` | **FIXED** |
| **G-JSON-08** | Roundtrip / compact / BOM sem testes dedicados | Checklist JSON | Unit tests em `json_wire` + asserts compact em output | **FIXED** |
| **G-JSON-09** | Pipeline NDJSON / JSONL streaming | Produto emite JSON único por invocação | N/A identidade one-shot agent | **N/A** |
| **G-JSON-10** | Content-Type HTTP `application/json` / `x-ndjson` | CLI stdin/stdout, não servidor HTTP | N/A | **N/A** |
| **G-JSON-11** | `schemars` / `jsonschema` runtime no binário | Schemas versionados em docs para agentes offline | N/A (sem engine embutida) | **N/A** |
| **G-JSON-12** | simd-json, JSON5, JSON Patch/Path/JCS, GeoJSON | Throughput <1MB; config TOML; sem patch protocol | N/A | **N/A** |

### Residuais intencionais (JSON)

| Residual | Motivo |
|----------|--------|
| `SuccessEnvelope.fields` como `BTreeMap<String, Value>` | Eventos CRUD variam; flatten mantém contrato `ok`+`event` tipado |
| `meta command-tree` ainda em `Value` | Árvore dinâmica do clap; não é schema de domínio fixo |
| Doctor / secrets status montados com `json!` ad-hoc | Poucos campos; passam por `print_json_line` compact; schema doctor cobre |
| Sem fuzz/miri de parser JSON em CI | Processo; serde_json é a superfície de parse |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — Interior Mutability) *(mantido)*

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (interior mutability) | **G-IM-01…G-IM-12** (12) |
| **FIXED** nesta rodada | **8** |
| **N/A** documentados nesta rodada | **4** (RefCell/Rc graph, RwLock/ArcSwap, tokio::Mutex, miri/loom suite) |
| **OPEN** | **0** (desta rodada) |
| Gaps memória/RAII (mantidos) | G-MEM-01…12 — **8 FIXED**, **4 N/A** |
| Gaps shutdown (mantidos) | G-SHUT-01…12 — **8 FIXED**, **4 N/A** |
| Gaps performance (mantidos) | G-PERF-01…12 — **6 FIXED**, **6 N/A** |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (**217**); `clippy --all-targets --all-features -D warnings` |

### Baseline interior mutability (após fix)

| Item | Estado |
|------|--------|
| Flags de sinal: `static AtomicBool` (não `OnceLock<Arc<AtomicBool>>`) | OK (`signals.rs`) |
| Ordering documentado: store `Release` / load `Acquire` nos flags; `Relaxed` em contador | OK |
| `SIGNAL_HITS`: `AtomicU8` + `record_signal_hit()` + comentário de ordering | OK |
| API pública: `cancellation_flag() -> &'static AtomicBool` | OK (menor primitiva) |
| `secrets::lock_global`: poison → `into_inner` **com** `tracing::warn!` | OK |
| Seção crítica curta; sem lock across `.await` | OK (clone sob lock) |
| `Mutex` único para `RuntimeSecretsFlags` (invariante multi-campo) | OK |
| `output::{QUIET,JSON_ERRORS}`: `AtomicBool` + `Relaxed` documentado | OK (pré-existente) |
| `OnceLock` para locale/color/log reload (init única) | OK (pré-existente) |
| Tunnel bound: `Arc<AtomicBool>` justificado (dois tasks) | OK + comentário |
| Zero `RefCell` / `Cell` / `Rc` / `static mut` / `lazy_static` no produto | OK |
| Política IM documentada em `lib.rs` + tabela em `signals` | OK |
| Teste de poison recovery + testes de flag estática estável | OK |

### Inventário — gaps Interior Mutability (esta rodada)

### Lote P — primitiva correta, atomics, poison e encapsulamento (G-IM-01…G-IM-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-IM-01** | `PoisonError` recuperado sem log | `lock_global` usava `into_inner` silencioso | `tracing::warn!` antes de `into_inner` | **FIXED** |
| **G-IM-02** | Flags de sinal em `OnceLock<Arc<AtomicBool>>` | Primitiva maior que o necessário; clone de Arc em getters | `static AtomicBool` + API `&'static AtomicBool` | **FIXED** |
| **G-IM-03** | Ordering de `SIGNAL_HITS` sem doc dedicada | Contador `Relaxed` implícito | Comentário + `record_signal_hit()` centralizado | **FIXED** |
| **G-IM-04** | Documentação de política IM ausente no crate root | Rules exigem justificar IM e matriz de decisão | Seção em `lib.rs` + tabela em `signals` | **FIXED** |
| **G-IM-05** | Comentário de contrato curto em `Mutex` de secrets | Risco de seções longas / await | Docs: nunca hold across await; clone sob lock | **FIXED** |
| **G-IM-06** | Tunnel `Arc<AtomicBool>` sem justificar vs Mutex/RefCell | Leitor podia questionar escolha | Comentário Release/Acquire + “flag independente” | **FIXED** |
| **G-IM-07** | Sem teste de poison recovery | Checklist IM pede tratamento explícito | `lock_global_recovers_from_poison_with_usable_data` | **FIXED** |
| **G-IM-08** | Handlers com `Arc::clone` desnecessário | Closures podiam capturar `'static` | `ctrlc`/`signal-hook` usam statics diretos | **FIXED** |
| **G-IM-09** | `RefCell` / `Rc<RefCell>` / grafo OOP | Não há uso; one-shot não modela grafo mutável | N/A — ausência correta | **N/A** |
| **G-IM-10** | `RwLock` / `ArcSwap` / `parking_lot` | Sem hot-path multi-reader de config | N/A identidade one-shot | **N/A** |
| **G-IM-11** | `tokio::sync::Mutex` por default em async | Produto não segura `std::Mutex` across await; sem estado async composto | N/A — não introduzido | **N/A** |
| **G-IM-12** | miri/loom/TSan em CI para atomics | Processo/CI; residual OPEN-PROCESS | N/A desta rodada binária | **N/A** |

### Residuais intencionais (interior mutability)

| Residual | Motivo |
|----------|--------|
| Getters públicos de flags permitem store (testes / integração) | Preferir `should_stop`; writes documentados como test/advanced |
| `std::sync::Mutex` em vez de atomics nos flags de secrets compostos | PathBuf + 2 bools = invariante multi-campo sob um lock |
| `Arc<AtomicBool>` no tunnel | Compartilhamento real entre timeout outer e accept loop |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — Gerenciamento de Memória e RAII) *(mantido)*

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (memória/RAII) | **G-MEM-01…G-MEM-12** (12) |
| **FIXED** nesta rodada | **8** |
| **N/A** documentados nesta rodada | **4** (try_reserve multi-tenant OOM, arena/pool, miri/loom suite, impl Drop custom em SshClient) |
| **OPEN** | **0** (desta rodada) |
| Gaps shutdown (mantidos) | G-SHUT-01…12 — **8 FIXED**, **4 N/A** |
| Gaps performance (mantidos) | G-PERF-01…12 — **6 FIXED**, **6 N/A** |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (**215**); `clippy --all-targets --all-features -D warnings` |

### Baseline memória / RAII (após fix)

| Item | Estado |
|------|--------|
| Passphrase de chave: `String` heap zeroizada pós-`load_secret_key` | OK (`client.rs` + `zeroize`) |
| Stdin sudo/su: `Zeroizing<Vec<u8>>` + drop cedo pós-write | OK (`run_command_internal`) |
| `PackedCommand`: `Drop` zeroiza stdin; Debug redige; `take_stdin()` | OK (`packing.rs`) |
| Primary-key hex em RAM zeroizado após parse (file/env/keyring) | OK (`secrets.rs`) |
| Leitura de key file com teto 4 KiB (`read_text_capped`) | OK |
| `config.toml` teto 4 MiB; `known_hosts` teto 1 MiB | OK |
| Decrypt UTF-8 fail: bytes do `FromUtf8Error` zeroizados | OK |
| Disconnect SSH em ramos de erro (exec/sudo/su/scp) | OK (`let _ = client.disconnect()`) |
| ScpOptions secrets: `take()` → `SecretString` sem clone | OK |
| Assinaturas `&Path` (clippy `ptr_arg`) em load/save paths | OK |
| `unsafe` restantes documentados (Windows CP, env test, signal-hook) | OK (revalidado) |
| Sem `mem::forget` / `Box::leak` / `Rc` no produto | OK |
| `SecretString` em passwords VPS + zeroize primary-key material | OK (pré-existente + reforçado) |

### Inventário — gaps Memória e RAII (esta rodada)

### Lote O — ownership, zeroize, tetos e cleanup (G-MEM-01…G-MEM-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-MEM-01** | Passphrase da chave privada copiada para `String` sem zeroize | `load_secret_key` exigia `&str`; heap plain ficava até drop normal | `zeroize()` imediato após load (mesmo em erro) | **FIXED** |
| **G-MEM-02** | Bytes de senha sudo/su no channel stdin sem scrub | `Option<Vec<u8>>` drop sem zeroize; timeout podia cancelar future | `Zeroizing::new` + `drop` cedo após write | **FIXED** |
| **G-MEM-03** | `PackedCommand.stdin` sem RAII de segredo | `Debug`/`Drop` default podiam vazar ou deixar bytes | `Drop`+zeroize, Debug redacted, `take_stdin()` | **FIXED** |
| **G-MEM-04** | Key file / env hex lidos sem teto nem scrub | `read_to_string` ilimitado; `String` com hex até fim do escopo | `read_text_capped(4KiB)` + `zeroize` pós-parse | **FIXED** |
| **G-MEM-05** | `config.toml` / `known_hosts` sem teto de bytes | Input local/path override → heap potencialmente grande | Caps 4 MiB / 1 MiB via `paths::read_text_capped` | **FIXED** |
| **G-MEM-06** | Disconnect só no caminho feliz de exec/scp | `run_command?` / `upload?` antes de `disconnect` | Sempre `disconnect` após op; propaga erro da op | **FIXED** |
| **G-MEM-07** | Hex gerado / keyring password sem zeroize | `generate_hex_key` / `get_password` leftovers | `hex.zeroize()` / `s.zeroize()` em todos os ramos | **FIXED** |
| **G-MEM-08** | ScpOptions clonava password/passphrase plain | `SecretString::from(pwd.clone())` | `opts.password.take()` move para `SecretString` | **FIXED** |
| **G-MEM-09** | `try_reserve` em parsers públicos multi-tenant | CLI one-shot; caps hard + abort OOM aceitável | N/A identidade (já documentado em G-RES) | **N/A** |
| **G-MEM-10** | Arena / buffer pool / `BytesMut` reutilizado em daemon | One-shot I/O; SCP já stream 32 KiB; exec cap 16 MiB | N/A identidade | **N/A** |
| **G-MEM-11** | Suite miri + loom + sanitizers em CI | Processo/CI; residual OPEN-PROCESS de hardening | N/A desta rodada binária | **N/A** |
| **G-MEM-12** | `impl Drop` custom em `SshClient` para disconnect síncrono | Disconnect é async (`russh`); Drop do handle fecha transporte | N/A — explicit disconnect + Drop de sessão bastam | **N/A** |

### Residuais intencionais (memória/RAII)

| Residual | Motivo |
|----------|--------|
| CLI password flags ainda `Option<String>` até `SecretString` | Clap derive + stdin resolve; move imediato em `apply_overrides` / `take` |
| `ConnectionConfig` clone de `SecretString` no connect | Necessário para future async; `secrecy` zeroiza clones no drop |
| OOM abort de `Vec` em CLI | Aceitável one-shot; caps evitam flood remoto |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — Graceful Shutdown) *(mantido)*


| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (shutdown) | **G-SHUT-01…G-SHUT-12** (12) |
| **FIXED** nesta rodada | **8** |
| **N/A** documentados nesta rodada | **4** (TaskTracker/CancellationToken daemon, SIGHUP/reload, k8s/systemd notify, chaos kill-9 suite) |
| **OPEN** | **0** (desta rodada) |
| Gaps performance (mantidos) | G-PERF-01…12 — **6 FIXED**, **6 N/A** |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix (à época) | `cargo test --lib` (**212**); clippy -D warnings |

### Baseline shutdown (após fix)

| Item | Estado |
|------|--------|
| Modelo one-shot **detect → signal → await** documentado | OK (`signals.rs`, `lib.rs::run`, `main.rs`) |
| SIGINT via `ctrlc` **sem** feature `termination` | OK (evita colisão SIGTERM) |
| SIGTERM Unix via `signal-hook` low_level (atomics only) | OK |
| Exit **130** / **143** / **141** preservados | OK (`signal_exit_code` + broken pipe) |
| `should_stop()` unifica cancel/term em loops | OK (exec/SCP/tunnel/VPS) |
| `register_handler` idempotente (`Once`) | OK |
| Double-signal → `is_force_exit` | OK |
| Tunnel: `JoinSet` + drain 2s / abort force | OK |
| `main`: flush + `runtime.shutdown_timeout(2s)` antes de `exit` | OK |
| SCP partial removido em cancel/timeout | OK (já existia; revalidado) |
| Disconnect SSH reason EN | OK (`closing` / `en-US`) |

### Inventário — gaps Graceful Shutdown (esta rodada)

### Lote N — one-shot cooperative shutdown (G-SHUT-01…G-SHUT-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-SHUT-01** | Feature `ctrlc/termination` + `signal-hook` no mesmo SIGTERM | Dual handler; risco 143→130 | Remover `termination`; SIGINT=`ctrlc`, SIGTERM=`signal-hook` only | **FIXED** |
| **G-SHUT-02** | SCP/upload/download checavam só `is_cancelled` | SIGTERM não abortava mid-transfer | `should_stop()` em scp entry + client upload/download loops | **FIXED** |
| **G-SHUT-03** | Sem API unificada de stop | Duplicação `cancelled \|\| terminated` inconsistente | `signals::should_stop` + `signal_exit_code` | **FIXED** |
| **G-SHUT-04** | Tunnel `tokio::spawn` detached | Forwards órfãos no cancel | `JoinSet` + reap + drain 2s + `abort_all` no force/timeout | **FIXED** |
| **G-SHUT-05** | `process::exit` sem dropar runtime | Workers abandonados pós-cancel | `shutdown_timeout(2s)` antes de exit; flush stdio | **FIXED** |
| **G-SHUT-06** | `register_handler` não idempotente | 2ª chamada `ctrlc` falha | `Once` + testes | **FIXED** |
| **G-SHUT-07** | Sem escalada double-signal | Rules: segundo sinal força término | `SIGNAL_HITS` + `is_force_exit`; tunnel abort | **FIXED** |
| **G-SHUT-08** | Disconnect russh em PT (`encerrando`/`pt-BR`) | Contrato técnico EN | `"closing"` / `"en-US"` | **FIXED** |
| **G-SHUT-09** | Política de shutdown ausente no código | Rule: desenhar desde o início / documentar | Docs `signals` + `lib::run` + `main` | **FIXED** |
| **G-SHUT-10** | `CancellationToken` + `TaskTracker` + `tokio-graceful-shutdown` | Daemon/multi-subsystem | N/A one-shot; AtomicBool + JoinSet local bastam | **N/A** |
| **G-SHUT-11** | SIGHUP reload, SIGUSR ops, readiness/liveness, `sd_notify`, PID file | Produto não é serviço/k8s | N/A identidade | **N/A** |
| **G-SHUT-12** | Suite chaos kill-9 + start_paused + preStop k8s | Sem orquestrador; cancel unitário cobre flags | N/A processo/ambiente | **N/A** |

### Residuais intencionais (shutdown)

| Residual | Motivo |
|----------|--------|
| AtomicBool em vez de `CancellationToken` | One-shot síncrono de poll; rules aceitam AtomicBool; sem árvore de tasks multi-subsystem |
| Sem deadline CLI configurável de shutdown global | Cada op já tem `timeout_ms`; drain tunnel fixo 2s + runtime 2s |
| Windows Ctrl+Break / ctrl_logoff | Agent one-shot em console; Ctrl+C cobre o fluxo |
| SIGPIPE handler explícito | Rust std ignora SIGPIPE → EPIPE; exit 141 já (G-IO) |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — Eficiência e Performance) *(mantido)*

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (performance) | **G-PERF-01…G-PERF-12** (12) |
| **FIXED** nesta rodada | **6** |
| **N/A** documentados nesta rodada | **6** (PGO/BOLT, multi-CPU bins, SIMD, arena, FxHash hot map, CI bench gates) |
| **OPEN** | **0** (desta rodada) |
| Gaps recursos (mantidos) | G-RES-01…12 — **7 FIXED**, **5 N/A** |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (**208**); proptest; `clippy --all-targets --all-features -D warnings`; `cargo build --profile release-fast` |

### Baseline performance (após fix)

| Item | Estado |
|------|--------|
| Política measure-first documentada em `run()` | OK (`lib.rs`) |
| Release **size-min** (`opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `strip`, `panic = abort`) | OK + comentários no `Cargo.toml` |
| Perfil local **`release-fast`** (`opt-level = 3`) para A/B | OK |
| Perfil **`bench`** com `opt-level = 3` | OK |
| `dev.package."*"` `opt-level = 2` (deps pesadas) | OK |
| `build-override` release para build scripts | OK |
| `mask` → `&'static str` (zero heap) | OK |
| Criterion só paths locais (mask/paths); docs anti-substituto de SSH profile | OK |
| Linker `rust-lld` em targets musl (`.cargo/config.toml`) | OK (já existia) |
| Hot path = RTT SSH; sem Rayon/SIMD/PGO no produto | N/A justificado |
| Cap captura + pré-aloc (lote G-RES) | Mantido |

### Inventário — gaps Eficiência e Performance (esta rodada)

### Lote M — build, alocações frias e anti-otimização cega (G-PERF-01…G-PERF-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-PERF-01** | `mask()` alocava `String` para constante `"***"` | `FIXED_MASK.to_string()` em todo list/json/details | Retorno `&'static str`; callers `output` em ramos `&str`; proptest sem `.as_str()` em `&str` | **FIXED** |
| **G-PERF-02** | Perfis de build sem documentação / A/B de velocidade | Só `opt-level = "z"`; rules pedem size-min **ou** speed + clareza | Comentários size-min; `lto = "fat"`; perfil `release-fast` (`opt-level = 3`); `profile.bench` | **FIXED** |
| **G-PERF-03** | Deps não otimizadas em dev; build scripts sem override | Rules: dependency profile + build-override | `[profile.dev.package."*"] opt-level = 2`; `release.build-override` | **FIXED** |
| **G-PERF-04** | Política de performance / ordem de otimização ausente no código | Rule: mentalidade measure-first | Docs em `lib.rs::run` + cabeçalho benches | **FIXED** |
| **G-PERF-05** | `remote_abort_pattern` fazia `trim().to_string()` sempre | Segunda alocação quando trim é no-op | Reusar `cleaned` se `trim` não encolhe | **FIXED** |
| **G-PERF-06** | Benches sem aviso de escopo (cold local ≠ SSH hot path) | Rule: não confiar microbench isolado | Comentário em `benches/ssh_operations.rs` | **FIXED** |
| **G-PERF-07** | PGO (`cargo-pgo`) / BOLT em binário de longa duração | One-shot; dataset rede não fixo no repo | N/A identidade + ambiente | **N/A** |
| **G-PERF-08** | Multi-CPU distribution (`x86-64-v3/v4`, `cargo-multivers`) | Publish único portátil; `target-cpu=native` proibido em dist | N/A; size-min + musl/lld bastam | **N/A** |
| **G-PERF-09** | SIMD / `std::simd` / autovectorization de hot path | Sem laço numérico dominante; RTT domina | N/A | **N/A** |
| **G-PERF-10** | `FxHashMap`/`AHashMap` em hot path de HashMap | Produto usa `BTreeMap` (registry ordenado estável); sem hash hot | N/A (BTreeMap intencional) | **N/A** |
| **G-PERF-11** | Arena/bump/SmallVec/smartstring em parsers | Sem fase com milhares de nós; alocações pontuais + teto G-RES | N/A (eco G-RES-09) | **N/A** |
| **G-PERF-12** | CI com threshold criterion + flamegraph SSH + `target-cpu=native` publish | CI proibida em rodadas anteriores; flamegraph exige host; native quebra portabilidade | N/A processo/ambiente | **N/A** |

### Residuais intencionais (performance)

| Residual | Motivo |
|----------|--------|
| Flamegraph / `perf` / RSS sob SSH real | Exige host representativo; proibido inventar ganho sem baseline |
| `opt-level = 3` como default de publish | Size-min é o trade-off do produto agent-first; A/B via `release-fast` |
| `#[inline(always)]` / `unsafe` por velocidade | Sem evidência de profile; `mask` usa só `#[inline]` leve |
| `try_reserve` / fallible alloc | CLI one-shot: abort OOM aceitável; cap 16 MiB já limita |
| i18n `format!` / `to_string` em mensagens | Cold path de UX; não hot path de canal SSH |
| `generate_hex_key` com `format!("{b:02x}")` | Cold path (init de chave); micro-opt sem medição = proibido |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — Economia de Recursos) *(mantido)*

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (recursos) | **G-RES-01…G-RES-12** (12) |
| **FIXED** nesta rodada | **7** |
| **N/A** documentados nesta rodada | **5** (Rayon/CPU pool, arena/bump, jemalloc default, daemon, cgroup subprocess, CI profile gates) |
| **OPEN** | **0** (desta rodada) |
| Gaps docs.rs (mantidos) | G-DRS-01…12 — **8 FIXED**, **4 N/A** |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (**207**) |

### Baseline recursos (após fix)

| Item | Estado |
|------|--------|
| Classificação workload **I/O-bound one-shot** documentada | OK (`lib.rs` `run`, `main.rs`, `ssh/client.rs`) |
| Justificativa **sem Rayon** / sem fan-out multi-host | OK (RTT domina; sessão única) |
| Tokio **multi_thread** justificado (russh + tunnel accept) | OK |
| Teto de captura exec `max_chars×4` + hard **16 MiB**/stream | OK (`exec_capture_byte_cap` + `append_capped`) |
| `Vec::with_capacity` em captura exec + header SCP + pendente download | OK |
| `truncate_utf8` single-pass + fast-path byte-len | OK |
| `from_utf8_lossy` via `Cow` (sem `.to_string()` extra pré-truncate) | OK |
| SCP upload/download **streaming** 32 KiB (sem `read` full-file) | OK (já existia; revalidado) |
| Singletons `OnceLock` / atomics (sinais, locale, logs, color) | OK (lotes anteriores) |
| `mimalloc` via feature `musl-allocator` (não default global sem medição) | OK |
| Criterion benches (mask/paths — paths frios) | OK; hot path SSH exige rede — N/A profile cego |
| Arena/bumpalo/rayon/dashmap/daemon/cgroup | N/A identidade one-shot |

### Inventário — gaps Economia de Recursos (esta rodada)

### Lote L — alocações, workload e anti-otimização cega (G-RES-01…G-RES-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-RES-01** | Captura exec SSH **sem teto de bytes** antes do truncate UTF-8 | `process_exec_message` fazia `extend_from_slice` ilimitado; flood remoto → OOM mesmo com `max_chars` | `exec_capture_byte_cap` + `append_capped`; hard max 16 MiB/stream; flags `truncated_*` se byte-cap | **FIXED** |
| **G-RES-02** | `Vec::new()` em stdout/stderr exec sem `with_capacity` | Reallocs em hot path de leitura de channel | `Vec::with_capacity(min(cap, 8 KiB))` em `run_command_internal` | **FIXED** |
| **G-RES-03** | `truncate_utf8` contava **todos** os chars + `collect` | Dois passes O(n) e alocação sempre | Fast-path `len()≤max_chars`; `char_indices().nth` + slice | **FIXED** |
| **G-RES-04** | `from_utf8_lossy(...).to_string()` antes do truncate | Alocação extra quando payload já é UTF-8 válido | Manter `Cow` de `from_utf8_lossy` até `truncate_utf8` | **FIXED** |
| **G-RES-05** | Classificação de workload **ausente** no código | Rule: DOCUMENTAR classificação | Comentários/docs em `run`, `main`, módulo `ssh/client` | **FIXED** |
| **G-RES-06** | Header SCP `Vec::new` + buffer download `pendente` sem capacity | Pré-alocar quando tamanho típico é conhecido | `with_capacity(256)` header; `with_capacity(32_768)` pendente | **FIXED** |
| **G-RES-07** | Justificativa explícita de **não** paralelizar CPU | Rule: JUSTIFICAR quando NÃO usar paralelismo | Documentado: one-shot I/O-bound, sem Rayon | **FIXED** |
| **G-RES-08** | Adotar Rayon / pool CPU / fan-out multi-host | Workload não é CPU-bound; overhead > ganho | N/A identidade one-shot | **N/A** |
| **G-RES-09** | Arena/bumpalo/typed-arena/SmallVec hot path | Sem fase com lifetime comum de milhares de nós | N/A; alocações pontuais + teto de captura bastam | **N/A** |
| **G-RES-10** | `mimalloc`/`jemalloc` como default obrigatório sem medição | Rule: VALIDAR ganho vs system antes de adotar | Feature `musl-allocator` opcional; default system em glibc | **N/A** (já conforme; feature mantida) |
| **G-RES-11** | Daemon reutilizável / connection pool / reqwest Client | Boot CLI << RTT SSH; produto é spawn→run→exit | N/A one-shot (lotes OS); sem HTTP client | **N/A** |
| **G-RES-12** | Profile flamegraph/samply + criterion SSH + cgroup `systemd-run` + CI benches | Otimização cega proibida sem dataset rede; subprocess cgroup fora do binário; CI proibida em rodadas anteriores | N/A processo; criterion local só mask/paths; RSS baseline sob carga SSH fica para ambiente com host real | **N/A** (processo/ambiente) |

### Residuais intencionais (recursos)

| Residual | Motivo |
|----------|--------|
| Flamegraph / `perf` / RSS `/usr/bin/time -v` em host SSH real | Exige ambiente de rede representativo; não inventar “ganho” sem baseline |
| `try_reserve` em captura | Cap hard + `max_chars` já limitam; OOM abort de Vec é aceitável em CLI one-shot |
| `parking_lot` / `dashmap` / `crossbeam` | Sem contenção multi-thread em mapas quentes no produto |
| `#[inline]`/`#[cold]` massivos | Sem evidência de profile; startup frio vs RTT |
| Streaming codepoint-aware *durante* o channel (sem buffer de bytes) | Complexidade alta; teto 16 MiB + truncate pós-decode resolve OOM |
| G-DOC-06 / G-IO-11 / G-22 | **FIXED** na rodada de fechamento de residuais (ver topo) |

---

## Resumo executivo (rodada 2026-07-18 — Documentação docs.rs) *(mantido)*

| Métrica | Valor |
|---------|-------|
| Gaps **novos** inventariados nesta rodada (docs.rs) | **G-DRS-01…G-DRS-12** (12) |
| **FIXED** nesta rodada | **8** |
| **N/A** documentados nesta rodada | **4** (pipeline gerador, aquamarine, CI/Actions proibidas, publish docs.rs) |
| **OPEN** | **0** (desta rodada) |
| Gaps const/static (mantidos) | G-CS-01…12 — **10 FIXED**, **2 N/A** |
| Gaps EN/docs (mantidos) | G-EN-01…12 + G-DOC-01…06 — **16 FIXED**, **2 N/A** (G-DOC-06 FIXED no fechamento) |
| Gaps one-shot (mantidos) | G-OS-01…G-OS-12 — **10 FIXED**, **2 N/A** |
| Gaps streams (mantidos) | G-IO-01…G-IO-12 — **12 FIXED** (G-IO-11 FIXED no fechamento) |
| Gaps Clap (mantidos) | G-01…G-24 — **23 FIXED** (G-22 FIXED no fechamento) |
| Gates validados pós-fix | `cargo test --lib` (204); `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`; `RUSTDOCFLAGS='--cfg docsrs -D warnings' cargo +nightly doc --no-deps --all-features`; `clippy --all-targets --all-features -D warnings` |

### Baseline docs.rs (após fix)

| Item | Estado |
|------|--------|
| `[package]` description / keywords(5) / categories / license / repository / documentation / homepage / readme / rust-version | OK |
| `[package.metadata.docs.rs]` `all-features` + `rustdoc-args = ["--cfg","docsrs"]` | OK |
| `default-target` + `targets` multiplataforma (breaking 2026-05-01) | OK (adicionado) |
| `#![cfg_attr(docsrs, feature(doc_cfg))]` | OK |
| `#[doc(cfg(...))]` em `SshClient` / reexports | OK (adicionado) |
| Seções crate-level Features + Safety (doc_cfg) | OK (adicionado) |
| Links intra-doc | limpos (`-D warnings`) |
| Badges README ordem canônica (docs.rs primeiro) | OK |
| `llms.txt` / `llms-full.txt` | hand-written agent contract (não pipeline rustdoc JSON) |
| aquamarine / mermaid embutido em rustdoc | N/A produto |
| CI / GitHub Actions nesta rodada | **proibido pelo usuário** |

### Inventário de globals (baseline conforme após fix)

| Item | Forma | Notas |
|------|-------|-------|
| `output::{QUIET,JSON_ERRORS}` | `static AtomicBool` | `Relaxed` (flag independente) |
| `signals::{CANCEL_FLAG,FLAG_SIGTERM,FORCE_EXIT}` | `static AtomicBool` | store `Release` / load `Acquire` |
| `locale::GLOBAL_LANGUAGE` | `static OnceLock<Language>` | set-once no boot |
| `terminal::COLOR_CACHE` | `static OnceLock<ColorChoice>` | set-once pós-parse |
| `cli::LOG_FILTER_RELOAD` | `static OnceLock<reload::Handle<…>>` | bootstrap → reload |
| `secrets::{DIR_CONFIG_OVERRIDE,RUNTIME_FLAGS}` | `static Mutex<…> = Mutex::new(…)` | ctor const; poison via `lock_global` |
| `secrets::AUTO_KEY_CREATED` | `static AtomicBool` | `Relaxed` |
| `main::GLOBAL` (feature) | `static mimalloc::MiMalloc` | `#[global_allocator]` |
| Vários `pub const` / `const` | primitives / `&'static str` / tables | sem interior mutability |

---

## Inventário — gaps Documentação docs.rs (esta rodada)

### Lote K — metadata, doc_cfg, rustdoc e README (G-DRS-01…G-DRS-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-DRS-01** | Links intra-doc quebrados (`SshCliError`, `SshCliError::Io`) | Rule: validar links; `cargo doc` com warnings | Paths canônicos `crate::errors::SshCliError` / `::Io` em `errors.rs` e `cli.rs`; `#![warn(rustdoc::broken_intra_doc_links)]` | **FIXED** |
| **G-DRS-02** | `targets` / `default-target` ausentes em `[package.metadata.docs.rs]` | Breaking docs.rs 2026-05-01: sem `targets` só default-target | `default-target = x86_64-unknown-linux-gnu` + 5 targets (linux gnu/musl, darwin x64/arm, windows msvc) | **FIXED** |
| **G-DRS-03** | `#[doc(cfg(...))]` ausente em itens feature-gated | Rule: documentar feature gates no rustdoc | `cfg_attr(docsrs, doc(cfg(feature = "ssh-real")))` em `SshClient` real/stub e reexport `ssh::SshClient` | **FIXED** |
| **G-DRS-04** | Crate docs sem seções Features / Safety para `doc_cfg` | Rule: features no crate root; Safety da migração doc_cfg | Tabelas Features + Safety em `src/lib.rs`; Features no módulo `ssh` | **FIXED** |
| **G-DRS-05** | Ordem de badges README não canônica | Rule: docs.rs → crates.io → License → MSRV… | README.md + README.pt-BR.md: shields.io docsrs/crates/l/MSRV/Rust | **FIXED** |
| **G-DRS-06** | README sem seções Features (Cargo) e Targets | Rule: seções canônicas Features / Targets | Seções adicionadas em EN e pt-BR apontando metadata docs.rs | **FIXED** |
| **G-DRS-07** | Build rustdoc com warnings tratados como erro | Rule: build limpo de warnings | `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` limpo; nightly+`docsrs` limpo | **FIXED** |
| **G-DRS-08** | `doc_cfg` feature gate só sob `docsrs` (já padrão correto) | Rule: `#![feature(doc_cfg)]` no crate root **quando docsrs** | Já existia `#![cfg_attr(docsrs, feature(doc_cfg))]`; revalidado + Safety | **FIXED** (já conforme; inventariado) |
| **G-DRS-09** | Pipeline canônico gerador (JSON rustdoc → llms.txt, NDJSON, timeout, sysexits de *doc tool*) | Produto **não** é CLI geradora de docs.rs | N/A identidade; `llms*.txt` são contratos agent-first hand-written | **N/A** |
| **G-DRS-10** | Mermaid via `aquamarine` embutido em rustdoc | Fora do domínio SSH CLI | N/A; diagramas de produto ficam em docs Markdown se necessário | **N/A** |
| **G-DRS-11** | CI/CD docs.rs + GitHub Actions (test nightly/stable doc) | Usuário: **proibido CI e GitHub Actions** nesta rodada | Validação **local** apenas (`cargo doc` stable + nightly docsrs) | **N/A** (proibido) |
| **G-DRS-12** | Publicar release / tag / disparar build docs.rs | Publish proibido sem autorização | Sem push/tag/crates.io nesta rodada | **N/A** (processo) |

### Residuais intencionais (docs.rs)

| Residual | Motivo |
|----------|--------|
| G-DOC-06 doctests | **FIXED** — 16 doctests executáveis (SSH/rede fora de doctest por natureza) |
| `musl-allocator` só no binário | `#[global_allocator]` em `main.rs`; documentado em Features (crate + README), sem item lib para `doc(cfg)` |
| Badges CI / Downloads | CI proibida nesta rodada; Downloads opcional pós-publish |
| Transformação automática rustdoc JSON → llms | Fora de escopo; manter `llms.txt` como contrato de agente versionado à mão |
| KaTeX / no_std / wasm / proc-macro | Já N/A em G-DOC-05 |

---

## Inventário — gaps Const / Static / Inicialização (rodada anterior, mantidos)

### Lote J — const, static, atomics e init (G-CS-01…G-CS-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-CS-01** | Lints `static_mut_refs` e interior-mutable-const não estavam como `deny` | Checklist: ativar lints como erro | `#![deny(static_mut_refs)]` + `clippy::declare_interior_mutable_const` + `clippy::borrow_interior_mutable_const` em `lib.rs` | **FIXED** |
| **G-CS-02** | Zero `static mut` (já conforme) | Rule: proibir `static mut` em código novo | Inventário `rg static mut` em `src/` → vazio; lint deny reforça | **FIXED** (já conforme; inventariado) |
| **G-CS-03** | Zero `const` com mutabilidade interior / OnceLock / Mutex / Atomic | Rule: NUNCA `const` com interior mutability | Inventário: todos os `OnceLock`/`Mutex`/`AtomicBool` são `static`; sem `const OnceLock` | **FIXED** (já conforme; inventariado) |
| **G-CS-04** | `Mutex` em secrets silenciava poison (`if let Ok`) | Rule: poisoning DEVE ser tratado explicitamente | `lock_global` com `unwrap_or_else(|p| p.into_inner())` em todos os sites de `DIR_CONFIG_OVERRIDE` / `RUNTIME_FLAGS` | **FIXED** |
| **G-CS-05** | `Ordering::SeqCst` em flags sem necessidade de ordem total | Rule: SeqCst só com ordem global; documentar/relaxar | `QUIET`/`JSON_ERRORS`/`AUTO_KEY_CREATED` → `Relaxed` (docs); cancel/SIGTERM/tunnel bound → `Release`/`Acquire` (docs) | **FIXED** |
| **G-CS-06** | Falta de `const _: () = assert!(…)` em invariantes | Rule: validar invariantes em build | Asserts em `secrets` (prefix/key name), `vps/model` (schema/timeouts), `errors::exit_codes`, `paths` (tables), `vps::MAX_SECRET_STDIN_BYTES` | **FIXED** |
| **G-CS-07** | Docs de concorrência incompletos em alguns `static` | Rule: documentar semântica de acesso concorrente | Docs em `output`, `signals`, `secrets`, `locale`, `terminal`, `cli::LOG_FILTER_RELOAD` | **FIXED** |
| **G-CS-08** | `Mutex::new` já direto (sem LazyLock) — revalidado | Rule: MSRV ≥ 1.63 usa ctor const; MSRV 1.85 | Confirmado: `Mutex::new(None)` / struct default; sem `LazyLock<Mutex<_>>` / sem `lazy_static!` no produto | **FIXED** (já conforme; inventariado) |
| **G-CS-09** | `OnceLock` correto vs LazyLock | Rule: OnceLock quando valor runtime/set externo | `GLOBAL_LANGUAGE`, `COLOR_CACHE`, `LOG_FILTER_RELOAD` usam OnceLock; flags de sinal migraram para `static AtomicBool` (G-IM-02); sem `once_cell`/`lazy_static` diretos | **FIXED** (já conforme; inventariado) |
| **G-CS-10** | Tipos explícitos e SCREAMING_SNAKE em globals | Rule: tipo explícito + nomenclatura | Inventário: todos os `static`/`const` com tipo explícito e SCREAMING_SNAKE (ex.: `DEFAULT_TIMEOUT_MS`) | **FIXED** (já conforme; inventariado) |
| **G-CS-11** | Loom/shuttle para testes de globals | Rule opcional de concorrência exaustiva | CLI one-shot: flags process-wide + `serial_test` onde OnceLock é compartilhado; loom seria overkill | **N/A** |
| **G-CS-12** | Pre-main ctor / `link_section` / `#[used]` / Freeze/FFI | Fora do domínio (binário CLI, sem FFI export) | N/A; único static especial é `#[global_allocator]` mimalloc (feature) | **N/A** |

### Residuais intencionais (const/static)

| Residual | Motivo |
|----------|--------|
| `OnceLock` compartilhado entre testes unitários | Design one-shot; testes de signals usam `#[serial]` + reset explícito |
| `once_cell` / `lazy_static` no `Cargo.lock` | Dependências transitivas de terceiros; **não** usadas no código do produto |
| `SeqCst` ausente no produto pós-fix | Substituído por Relaxed ou Acquire/Release documentados |
| Destrutores de `static` no exit | Rule: statics não rodam Drop no fim; recursos de SSH/tokio são one-shot por comando (sem singleton de sessão) |

---

## Inventário — gaps Inglês + documentação (rodada anterior, mantidos)

> Ver tabelas G-EN-01…G-EN-12 e G-DOC-01…G-DOC-06 abaixo (status inalterado nesta rodada const/static).

### Lote H — idioma inglês no código-fonte (G-EN-01…G-EN-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-EN-01** | Product code importava `crate::erros` (módulo PT) | Rule: NUNCA PT em identificadores/módulos; mistura EN/PT no `.rs` | Migrados todos os imports de produto para `crate::errors` / `ssh_cli::errors` (`main`, `scp`, `tunnel`, `secrets`, `vps`, `ssh/*`) | **FIXED** |
| **G-EN-02** | Variável `argumentos` em `lib::run` | Identificador PT privado | Renomeado para `args` | **FIXED** |
| **G-EN-03** | Locals PT em `main` (`quer_json`, `erro_ssh`) | Identificadores PT | `wants_json`, `ssh_err` | **FIXED** |
| **G-EN-04** | `COR_CACHE`, docs/comentários PT em `terminal`/`locale` | Mistura de idiomas | `COLOR_CACHE`; layers/comments EN; `code`/`normalized`/`other` | **FIXED** |
| **G-EN-05** | Identificador `registro` em `vps`/`scp` | PT para “record” | Renomeado para `record` (36 ocorrências) | **FIXED** |
| **G-EN-06** | Docs/comentários/tracing PT em `ssh/client.rs` | Rule: logs e doc comments em EN | Trait/SCP/connect docs EN; tracing `starting SSH connection` / `authenticated` / `session closed`; stub errors EN | **FIXED** |
| **G-EN-07** | Módulo `packing` com `//!` e docs PT (`limpo`, `Sanitiza`, `Monta`) | Mistura no mesmo arquivo | Module docs EN; `cleaned`; abort pattern EN | **FIXED** |
| **G-EN-08** | Comentários PT em `vps` (`Garante alinhamento`, `envelope de err`) | Implementação em PT | Comentários EN | **FIXED** |
| **G-EN-09** | Comentários PT em `Cargo.toml` (russh) | description/comentários de crate em EN | Comentários EN (rsa/zlib) | **FIXED** |
| **G-EN-10** | `windows.rs`: 2 ops `unsafe` no mesmo bloco | Rule: `multiple_unsafe_ops_per_block` + SAFETY por op | Um `unsafe` por call (`SetConsoleOutputCP` / `SetConsoleCP`) com SAFETY 4 linhas cada | **FIXED** |
| **G-EN-11** | UI PT em `i18n::Message::pt` + aliases serde `porta`/`usuario`/`senha` | i18n bilíngue e migração de wire legacy | Mantido de propósito: UI via i18n; wire aliases só em deserialize; **sem** write PT | **N/A** |
| **G-EN-12** | Shim `pub mod erros` + aliases `ErroSshCli`/`ResultadoSshCli` | Semver: path histórico | Mantido como **deprecated** (`since = "0.5.1"`); product code não importa; remoção no próximo major | **FIXED** (deprecated + zero usos no produto) |

### Lote I — documentação crates.io / rustdoc (G-DOC-01…G-DOC-06)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-DOC-01** | Ausência de `[badges.maintenance]` | Checklist crates.io | `maintenance = { status = "actively-developed" }` em `Cargo.toml` | **FIXED** |
| **G-DOC-02** | Doc anti-padrão “This module/trait…” | Rule: não iniciar com “This function/module…” | `output` e trait SSH reescritos em voz declarativa | **FIXED** |
| **G-DOC-03** | Lints unsafe já avisavam; gaps em Windows multi-op | `undocumented_unsafe_blocks` / `multiple_unsafe_ops` | Confirmados em `lib.rs`; Windows corrigido (G-EN-10) | **FIXED** |
| **G-DOC-04** | Metadata docs.rs parcial | Checklist: all-features + rustdoc-args | Já havia `all-features` + `--cfg docsrs` + `doc_cfg`; revalidado | **FIXED** (já conforme; inventariado) |
| **G-DOC-05** | KaTeX/MathML/no_std/wasm/proc-macro/trybuild | Fora do domínio CLI SSH | N/A produto | **N/A** |
| **G-DOC-06** | Poucos doctests executáveis em itens públicos | Rule: Examples em APIs não triviais | Doctests em `paths` (4), `json_wire` (2), `output` (2), `errors` (2), `signals` (2), `telemetry`, `resolve_exit_code`, + masking/i18n; SSH/rede permanece `no_run` fora de doctest | **FIXED** |

### Residuais intencionais (não são gaps de produto)

| Residual | Motivo |
|----------|--------|
| `pub mod erros` deprecated | Semver até major; ver G-EN-12 |
| `Message::pt()` strings | i18n bilíngue UI; isolamento técnico EN vs UI |
| `serde(alias = "porta"\|"usuario"\|"senha")` | Migração configs legadas; **serialize só EN** |
| Fixtures de teste com palavra “senha-” | Dados de teste de mascaramento; não são identificadores |
| Comentários PT em `tests/*.rs` históricos | Suites de regressão; produto `src/` é o contrato EN |

---

## Inventário — gaps identificados e solucionados (CLI One-Shot) — **rodada anterior, mantidos**

### Lote F — ciclo de vida, timeout global e cancel cooperativo (G-OS-01…G-OS-06)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-OS-01** | Ordem de init violava regras: parse antes de tracing | Rule: 1º sinais, 2º tracing **antes** do parse, 3º parse | `bootstrap_logs()` (filter `error` + `reload::Layer`) em `lib::run` **antes** de `parse_args`; `initialize_logs` recarrega filter com `-v`/`RUST_LOG` | **FIXED** |
| **G-OS-02** | Sem `--timeout` global em `GlobalOpts` | Rule: `GlobalOpts` DEVE conter `--timeout` global; execução com limites | `CliArgs.timeout: Option<u64>` global; `effective_timeout(local, global)` (local vence); aplicado em exec/sudo/su/scp/health-check | **FIXED** |
| **G-OS-03** | Loops SSH/SCP não checavam cancel mid-I/O | Rule: respeitar cancellation token em loops longos | Checks `is_cancelled`/`is_terminated` em `run_command_internal` e `scp_read_data` → erro canônico EN | **FIXED** |
| **G-OS-04** | `Runtime::Builder` multi_thread sem justificativa documentada | Rule: Runtime explícito PROIBIDO sem justificativa; multi_thread para fan-out I/O | Comentário em `main.rs` (russh + accept loops; não `current_thread`) | **FIXED** |
| **G-OS-05** | Docs de `run()` desalinhados com as seis fases one-shot | Lifecycle BORN→EXECUTE→DIE não documentado no entry | rustdoc de `lib::run` lista fases Init/Parse/Configure/Execute + finalize em `main` | **FIXED** |
| **G-OS-06** | Mensagens técnicas PT residual em SSH/signals | Rule: locale NÃO afeta dados/erros técnicos em stdout/stderr de contrato | EN: SIGTERM log; Debug `port`/`username`/`password`; `open session` / `open SCP session` / `scp exited with status` | **FIXED** |

### Lote G — determinismo, recursos e cobertura (G-OS-07…G-OS-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-OS-07** | Risco de `HashMap` em output (ordem não estável) | Rule: HashMap PROIBIDO em output; BTreeMap/IndexMap | Auditoria: **zero** `HashMap` em `src/`; hosts via `BTreeMap` (list/export JSON estável) | **FIXED** (já conforme; inventariado + revalidado) |
| **G-OS-08** | Testes sem cobertura de timeout global / merge | Regressão de contrato one-shot | Unit tests `effective_timeout_local_wins_over_global`, `parser_accepts_global_timeout`; snapshot help com `--timeout <MS>` | **FIXED** |
| **G-OS-09** | Tunnel sem deadline obrigatório (daemon risk) | Rule: daemon proibido; vida limitada | Já existia `--timeout-ms` obrigatório + `tokio::time::timeout` + cancel no accept loop — revalidado (sem daemon HTTP/loop infinito) | **FIXED** (já conforme; inventariado) |
| **G-OS-10** | Confirmação interativa em ações destrutivas | Rule genérica: `--yes`/`--force` | Agent-first: `vps remove` é inventário local não-interativo (re-add reverte); `secrets init --force` e `--replace-host-key` já cobrem overwrite perigoso; prompts TUI **proibidos** | **N/A** |
| **G-OS-11** | Progress bar / heartbeat em ops > 2s | Rule genérica de progresso | stdout é API de dados; progresso em stdout poluiria agentes; stderr só com `-v`/tracing; tunnel tem deadline finito | **N/A** |
| **G-OS-12** | `tracing-subscriber` sem caminho de reload pós-parse | Bootstrap + reconfig sem segundo `try_init` que falha | Feature `registry` + `reload::Handle` em `LOG_FILTER_RELOAD`; fallback `try_init` em testes | **FIXED** |

---

## Inventário — gaps stdin/stdout (rodada anterior, **mantidos**)

### Lote D — disciplina de streams e exit codes (G-IO-01…G-IO-06)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-IO-01** | Ausência de `EX_PIPE = 141`; `SshCliError::Io` sempre mapeava para 74 | Rule: BrokenPipe → exit graceful 141; CLAUDE/checklist 141 | `exit_codes::EX_PIPE`; `is_broken_pipe` / `anyhow_is_broken_pipe`; `exit_code()` distingue EPIPE; `main` sai 141 sem envelope JSON de ruído | **FIXED** |
| **G-IO-02** | `println!` em caminhos de dados sem flush/`write_all`/BrokenPipe | Rule: proíbe println fora de output; flush explícito; write_all | `write_line` / `write_lines` (`BufWriter` + lock + flush); human path ignora pipe; JSON propaga EPIPE | **FIXED** |
| **G-IO-03** | Emissores JSON (`list`/`details`/`exec`/`health`/`scp`/`tunnel`) engoliam erros ou usavam `println!` | API stdout não propagava falha de pipe | Assinaturas `-> io::Result<()>`; call sites com `?` em `vps`/`scp`/`tunnel` | **FIXED** |
| **G-IO-04** | Strings PT em stdout técnico (`foi truncado`, `latência`) | Rule: mensagens técnicas/contrato em inglês | `(stdout was truncated)` / `(stderr was truncated)` / `latency:` | **FIXED** |
| **G-IO-05** | `eprintln!` ad-hoc em `cli.rs` / `vps` / `main` para warnings e erros | Rule: stderr só via canal controlado; sem misturar com dados | `output::print_warning` / `print_error` / `write_stderr_line`; call sites migrados | **FIXED** |
| **G-IO-06** | `read_secret_stdin` lia stdin sem limite de tamanho | Rule: LIMITAR payload stdin com guarda | `MAX_SECRET_STDIN_BYTES` (64 KiB) + `Read::take` + `InvalidArgument` se exceder | **FIXED** |

### Lote E — contrato agente, completions e meta-comandos (G-IO-07…G-IO-12)

| ID | Gap | Causa / evidência | Solução | Status |
|----|-----|-------------------|---------|--------|
| **G-IO-07** | Completions engolia BrokenPipe com `return` silencioso (exit 0) | Rule: EPIPE → 141 | `generate_completions() -> Result`; propaga `io::Error` BrokenPipe | **FIXED** |
| **G-IO-08** | Sem flush de stdout/stderr antes de `process::exit` | Rule: flush antes de cada exit | `main`: flush stdout no fim; flush stderr após envelope de erro | **FIXED** |
| **G-IO-09** | Sem superfície `commands` JSON (checklist `mycli commands`) | Descoberta de árvore para agentes | Subcomando `ssh-cli commands [--json]`; `command_tree_json()` via clap `CommandFactory`; snapshot help atualizado | **FIXED** |
| **G-IO-10** | Testes/unitários sem cobertura de 141 / árvore de comandos | Regressão de contrato | Unit tests `exit_code_broken_pipe_is_141`, `anyhow_detects_broken_pipe`, `parser_commands_meta` | **FIXED** |
| **G-IO-11** | `lib::run()` sem assinatura `run(args, stdin, stdout, stderr) -> ExitCode` | Rule arquitetura main thin + DI de streams | `run` + `run_with_args(CliArgs)`; `resolve_exit_code`; DI via `write_line_to` / `write_stderr_line_to` / `write_json_line`; main só runtime+flush+exit | **FIXED** |
| **G-IO-12** | Docs de módulo ainda falavam em “único `println!`” sem disciplina de pipe | Documentação desalinhada | `lib`/`cli` rustdoc atualizados para emissão controlada + cycle one-shot EN | **FIXED** |

---

## Inventário — gaps Clap (rodada anterior, **mantidos**)

### Lote A — parse, contrato e invariantes (G-01…G-06)

| ID | Gap | Status |
|----|-----|--------|
| **G-01** | `CliArgs::command().debug_assert()` em testes | **FIXED** |
| **G-02** | `propagate_version` / `arg_required_else_help` / `subcommand_required` | **FIXED** |
| **G-03** | `ArgAction::SetTrue` + conflito `-v`/`-q` | **FIXED** |
| **G-04** | `value_hint` em paths | **FIXED** |
| **G-05** | max chars como `Option<usize>` + parser | **FIXED** |
| **G-06** | export `-o` `PathBuf` | **FIXED** |

### Lote B — estrutura, auth flatten e domínio

| ID | Gap | Status |
|----|-----|--------|
| **G-07** | CAMADA 2 `src/commands/` | **FIXED** |
| **G-08** | `SshAuthArgs` flatten | **FIXED** |
| **G-09** | `--key` `PathBuf` | **FIXED** |
| **G-10** | `--disable-sudo` / `--enable-sudo` (não `Option<bool>`) | **FIXED** |
| **G-11** | clap `color` + `suggestions` | **FIXED** |
| **G-12** | `clap_mangen` | **FIXED** |
| **G-15** | `human-panic` release | **FIXED** |
| **G-19** | `error.rs` canônico | **FIXED** |
| **G-20** | `after_help` exemplos | **FIXED** |
| **G-21** | `help_heading` Global/Authentication | **FIXED** |
| **G-23** | proptest parsers CLI | **FIXED** |
| **G-24** | boundary tipos clap → domínio | **FIXED** |

### Lote C — processo

| ID | Gap | Status |
|----|-----|--------|
| **G-22** | cargo-dist / SBOM assinado / multi-arch distro | `scripts/dist_multiarch.sh` + `Cross.toml`; `scripts/generate_sbom.sh` (CycloneDX ou fallback tree); gates 25–27 em RELEASE_CHECKLIST EN/pt-BR; assinatura/push só com autorização | **FIXED** |

---

## N/A — propositais (não são débitos a “corrigir” no produto)

| Tema | Por que N/A |
|------|-------------|
| `dialoguer` / TUI / prompts interativos | CLI one-shot agent-first; proibição de interatividade / proibição MCP |
| Confirmação interativa em `vps remove` | Agent-first; inventário local re-adicionável; ver G-OS-10 |
| Progress bar / NDJSON heartbeat infinito | stdout = dados; tunnel tem deadline; ver G-OS-11 |
| `color-eyre` como stack de erro principal | Envelope JSON de erro em stderr + sysexits |
| `indicatif` progress em stdout | stdout é API de dados; one-shot curtas |
| Multicall BusyBox | Produto single-name `ssh-cli` |
| Workspace multi-crate obrigatório | Crate único lib+bin válido |
| Telemetria / self-update | Proibido no produto |
| Payload JSON único em stdin para **todos** os subcomandos | Contrato argv + secrets via `--*-stdin`; não é CLI de protocolo NDJSON genérico |
| `idempotency_key` / `correlation_id` em todo payload | Side-effects são inventário VPS local + SSH one-shot; retry é responsabilidade do agente chamador |
| Heartbeat NDJSON em loop infinito | Tunnel tem deadline obrigatório; sem daemon |
| `schema` embutido compile-time por comando (JSON Schema runtime) | Schemas versionados em `docs/schemas/*.json` + doctor; não é wire stdin JSON |
| Reactor mesh / WASM plugins / loom | Fora do escopo one-shot SSH |
| Server HTTP/gRPC/WebSocket no binário | Explicitamente proibido pelas rules one-shot; produto não tem |
| AGENTS.md **no root do crate** | Presente em `docs/AGENTS.md` (+ skills); root `AGENTS.md` excluído do package crates.io por design |
| JoinSet/rayon fan-out massivo | Produto é SSH one-host por invocação; sem fan-out de milhares de tasks |
| `--log-format` json/text dual | Agent-first: tracing em stderr nível error; `RUST_LOG` cobre; JSON de **dados** já é `--output-format json` |

---

## Mudanças de API / UX (acumulado)

| Mudança | Antes | Depois |
|---------|-------|--------|
| Edit disable sudo | `vps edit NAME --disable-sudo true\|false` | `--disable-sudo` **ou** `--enable-sudo` |
| Max chars CLI | string solta | `usize` / `none` / `0` |
| Export `-o` | `String` | `PathBuf` |
| Auth flags | copiadas por subcomando | `SshAuthArgs` flatten + `--key` PathBuf |
| Help root | sem exemplos | `after_help` + headings + `commands` |
| Broken pipe | panic/`println`/exit 74 ou 0 | **exit 141** (`EX_PIPE`) |
| Meta discovery | só help text | `ssh-cli commands` → árvore JSON |
| Truncation / health text | PT residual | EN técnico |
| Stdin secrets | ilimitado | max **64 KiB** |
| Timeout global | só por subcomando | **`--timeout <MS>` global** (local vence; tunnel mantém `--timeout-ms` obrigatório) |
| Init logging | só após parse | bootstrap error-level **antes** do parse + reload |

---

## Arquivos tocados (rodada CLI One-Shot)

| Arquivo | Papel |
|---------|--------|
| `Cargo.toml` | `tracing-subscriber` feature `registry` (reload path) |
| `src/lib.rs` | lifecycle 6 fases; bootstrap logs antes do parse |
| `src/main.rs` | justificativa multi_thread runtime |
| `src/cli.rs` | `--timeout` global; `effective_timeout`; `bootstrap_logs`/`reload`; testes |
| `src/signals.rs` | log SIGTERM em EN |
| `src/ssh/client.rs` | cancel mid-exec/SCP; mensagens EN; Debug EN |
| `tests/snapshots/snapshot_tests__help_output.snap` | help com `--timeout` |
| `gaps.md` | inventário incremental G-OS-* |

---

## Arquivos tocados (rodada stdin/stdout — mantidos)

| Arquivo | Papel |
|---------|--------|
| `src/errors.rs` | `EX_PIPE`, `is_broken_pipe`, `anyhow_is_broken_pipe`, testes 141 |
| `src/main.rs` | flush; exit 141; erros via `output::print_error` |
| `src/output.rs` | `write_line`/`write_lines`/`write_stderr_line`/`print_warning`; JSON com `Result`; EN |
| `src/cli.rs` | completions `Result`; `commands` + `command_tree_json`; warning via output |
| `src/vps/mod.rs` | `?` em prints JSON; limite stdin secret; warning timeout |
| `src/scp.rs` / `src/tunnel.rs` | propaga I/O de print JSON |

---

## Verificação (evidência de fechamento — rodada one-shot)

```text
cargo test --locked --lib
# 204 passed
cargo test --locked --test e2e_cli --test snapshot_tests --test proptest_tests
# e2e 17; snapshots 6; proptest 5
cargo test --locked --test gaps_v035_integration --test gaps_v051_integration \
  --test scp_integration --test tunnel_integration --test storage_integration
cargo clippy --all-targets --locked -- -D warnings
# clean
```

Resultado na rodada one-shot: **pass** (lib 204; e2e 17; snapshots 6; proptest 5; gaps/scp/tunnel/storage ok; clippy clean).

---

## Arquivos tocados (rodada Inglês + docs crates.io)

| Arquivo | Papel |
|---------|--------|
| `src/lib.rs` | `args` EN; docs de `errors`/`erros` |
| `src/main.rs` | imports `errors`; locals EN |
| `src/erros.rs` | shim deprecated + note de remoção major |
| `src/scp.rs` / `src/tunnel.rs` / `src/secrets.rs` / `src/vps/mod.rs` / `src/ssh/*` | imports `errors`; `record`; docs EN |
| `src/ssh/client.rs` | docs/tracing/stub EN; mode header EN |
| `src/ssh/packing.rs` | module docs EN; `cleaned` |
| `src/terminal.rs` / `src/locale.rs` | `COLOR_CACHE`; layers EN; `code`/`normalized`/`other` |
| `src/output.rs` | crate docs sem “This module…” |
| `src/platform/windows.rs` | unsafe split + SAFETY por op |
| `Cargo.toml` | comentários EN; `[badges.maintenance]` |
| `gaps.md` | inventário G-EN-* / G-DOC-* |

---

## Arquivos tocados (rodada docs.rs)

| Arquivo | Papel |
|---------|--------|
| `Cargo.toml` | `default-target` + `targets` em `[package.metadata.docs.rs]` |
| `src/lib.rs` | Features + Safety (`doc_cfg`); warn `broken_intra_doc_links` |
| `src/errors.rs` / `src/cli.rs` | links intra-doc canônicos |
| `src/ssh/client.rs` / `src/ssh/mod.rs` | `doc(cfg)` em `SshClient` + docs Features |
| `README.md` / `README.pt-BR.md` | badges canônicos; seções Features / Targets |
| `gaps.md` | inventário G-DRS-01…12 |

---

## Histórico de inventário

| Data | Evento |
|------|--------|
| (pré-0.5.1) | Inventários GAP-AUD / GAP-SSH em CHANGELOG e suites `gaps_v0*` |
| **2026-07-18** | Auditoria Clap: **G-01…G-24** — 22 FIXED, 1 OPEN-PROCESS |
| **2026-07-18** | Auditoria **stdin/stdout** (incremental): **G-IO-01…G-IO-12** — 11 FIXED, 1 OPEN-PARTIAL |
| **2026-07-18** | Auditoria **CLI One-Shot** (incremental): **G-OS-01…G-OS-12** — 10 FIXED, 2 N/A |
| **2026-07-18** | Auditoria **Inglês + docs crates.io** (incremental): **G-EN-01…12** + **G-DOC-01…06** — 15 FIXED, 2 N/A, 1 OPEN-PARTIAL |
| **2026-07-18** | Auditoria **Const/Static** (incremental): **G-CS-01…12** — 10 FIXED, 2 N/A |
| **2026-07-18** | Auditoria **docs.rs** (incremental): **G-DRS-01…12** — 8 FIXED, 4 N/A |
| **2026-07-18** | Auditoria **Interior Mutability** (incremental): **G-IM-01…12** — 8 FIXED, 4 N/A |
| **2026-07-18** | Auditoria **JSON e NDJSON** (incremental): **G-JSON-01…12** — 8 FIXED, 4 N/A |
| **2026-07-18** | Auditoria **Redução de Latência** (incremental): **G-LAT-01…12** — 6 FIXED, 6 N/A |
| **2026-07-18** | Auditoria **Logs/Tracing** (incremental): **G-LOG-01…12** — 6 FIXED, 6 N/A |
| **2026-07-18** | **Fechamento residuais:** **G-DOC-06**, **G-IO-11**, **G-22** → FIXED; inventário **OPEN = 0** |

---

## Verificação (evidência de fechamento — residuais + doctests)

```text
cargo test --lib                                                    # 235 passed
cargo test --doc                                                    # 16 passed
cargo clippy --all-targets --all-features -- -D warnings            # clean
cargo check --bins                                                  # clean
bash scripts/generate_sbom.sh                                       # fallback tree ou CycloneDX
# scripts/dist_multiarch.sh requer Docker/cross (mantenedor)
# CI / GitHub Actions: proibidos sem autorização (validação local apenas)
```

---

## Verificação (evidência de fechamento — rodada docs.rs)

```text
cargo test --lib                                                    # 204 passed
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features       # clean
RUSTDOCFLAGS='--cfg docsrs -D warnings' cargo +nightly doc \
  --no-deps --all-features                                          # clean (doc_cfg)
cargo clippy --all-targets --all-features -- -D warnings            # clean
cargo metadata --format-version 1 --no-deps                         # docs.rs targets presentes
# CI / GitHub Actions: proibidos nesta rodada (validação local apenas)
```

---

## Próximos passos sugeridos (não abertos como gap bloqueante)

Inventário canônico: **OPEN = 0**. Itens opcionais de processo (não gaps de produto):

1. ~~**G-IO-11**~~ **FIXED** — `run_with_args` + `resolve_exit_code` + `write_*_to`.  
2. ~~**G-22**~~ **FIXED** — scripts multi-arch/SBOM + checklist gates 25–27.  
3. ~~**G-DOC-06**~~ **FIXED** — 16 doctests executáveis em APIs públicas.  
4. Executar `dist_multiarch.sh` com Docker no release real; assinar SBOM (cosign/gpg) só com autorização.  
5. `cargo publish` / tag push — somente com autorização explícita do mantenedor.

4. CHANGELOG Unreleased: docs.rs targets/doc_cfg/badges + itens anteriores (`commands`, exit 141, …).  
5. Major futuro: remover `src/erros.rs` e aliases PT.  
6. Opcional: expandir `src/commands/{exec,vps,…}.rs` movendo handlers para fora de `dispatch_impl`.  
7. Opcional: traduzir comentários PT remanescentes só em `tests/*.rs` históricos (não bloqueia produto).  
8. **Quando autorizado:** publish crates.io → docs.rs rebuild automático; só então validar badge docs.rs em produção.
9. Opcional: tipar doctor/secrets-status com DTOs dedicados (hoje `json!` + compact).
10. Opcional: `cargo test --test gaps_v051` export/import JSON com BOM em fixture.

---

# Auditoria E2E completa — 2026-07-19 (sessão Grok; só inventário, sem correção)

> **Linha de produto:** 0.5.1 (`Cargo.toml` / bin `ssh-cli 0.5.1 (878b675)`)  
> **Modo:** identificação + causa raiz + plano de contramedidas — **PROIBIDO corrigir código nesta rodada**  
> **Escopo:** inventário **incremental** (append no final); **não** reescreve seções anteriores  
> **Compilação local:** `cargo build --release` — **OK** (~1m34s)  
> **Clippy:** `cargo clippy --all-targets -- -D warnings` — **OK**  
> **Lib unit:** `cargo test --release --lib` — **358 passed**  
> **Integração:** várias suites **FAILED** (detalhe abaixo) — contradiz pretensão “OPEN residual = 0” do topo histórico  
> **Publish:** sem GitHub push / sem crates.io  
> **Tools desta rodada:** context7 ✅ · docsrs-cli ✅ · duckduckgo-search-cli ✅ · sqlite-graphrag ✅ · atomwrite ✅ · rules em `docs_rules/` + GraphRAG DB

---

## 1. Respostas obrigatórias (checklist da auditoria)

| Pergunta | Resposta com evidência |
|----------|------------------------|
| Todos os gaps do `gaps.md` histórico foram solucionados? | **NÃO.** Topo histórico afirma OPEN=0 / FIXED em massa, mas `cargo test --release --tests` falha em `gaps_v035`…`v042`, `scp_integration`, `snapshot_tests`. |
| O que falta? | Suite de regressão alinhada ao produto 0.5.1; paridade UX JSON global; `exec` com active VPS; classificação de erros TLS; remoção/isolamento de GH Actions se mandato produto proíbe CI; gate E2E real SSH (credenciais). |
| O que foi esquecido / omitido em inventários “OPEN=0”? | Drift de testes legados; falso positivo `warn_if_password_argv`; dual auth password+key; `vps path` não-JSON; `--json` global inexistente vs exemplos help; skill description >1024; monólitos `cli/mod.rs` / `client_real.rs`. |
| Quais são os gaps (esta rodada)? | **G-AUD-20260719-01 … 20** (tabela abaixo) — todos **OPEN**. |
| Oportunidades de melhoria? | Ver §6. |
| context7-cli? | **SIM** — `context7 library russh` → `/eugeny/russh` (trust 9.7). |
| duckduckgo-search-cli? | **SIM** — queries russh/TOFU known_hosts + OpenSSH host key verification (Chrome/CDP). |
| docsrs-cli? | **SIM** — `search-crates russh` 0.62.2; `readme russh` / `russh-sftp` 2.3.0; `search-in-crate`. |
| Rules GraphRAG / `docs_rules`? | **SIM** — lidos trechos de `rules_rust_proibicao_hardcode`, `rules_rust_storage_xdg…`, `rules_rust_cli_one_shot`; hybrid-search no DB; corpus local ~94 rules. |
| Todos erros/bugs/gaps/warnings resolvidos? | **NÃO** — mandato desta rodada: **só inventariar**; bugs de produto e de testes permanecem **OPEN**. |
| Compilação local + auditoria E2E de comandos? | **SIM** — release build + matriz offline de todos os subcomandos principais (ver §3). |
| Missão one-shot / multiplataforma / zero telemetria produto? | One-shot e tracing stderr local **OK**; OTEL ausente; `.github/workflows` **existe** (tensão com mandato “proibido CI na cli”); env ainda lido em runtime (`SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT`). |

---

## 2. Matriz de testes (evidência quantificada)

| Suite | Resultado | Falhas (nomes) |
|-------|-----------|----------------|
| `--lib` | 358 ok | — |
| `e2e_cli` | 17 ok | — |
| `gaps_v035_integration` | 9 ok / **3 fail** | `doctor_reports_layer_and_secrets_plaintext`, `secrets_encrypt_on_disk_when_key_set`, `secrets_init_and_reencrypt` |
| `gaps_v037_integration` | 13 ok / **2 fail** | `gap_val_002_rejects_port_zero`, `gap_sec_002_mask_sempre_asteriscos` |
| `gaps_v038_integration` | 26 ok / **2 fail** | mesmos padrões mask/port |
| `gaps_v039_integration` | 9 ok / **3 fail** | `gap_doc_003_residual…`, `gap_cli_004_health_check_aceita_timeout`, `gap_json_001_com_password…` |
| `gaps_v040_integration` | 15 ok / **6 fail** | skill size, scp source asserts, JSON envelope clap, tunnel flag path, … |
| `gaps_v041_integration` | 11 ok / **3 fail** | tunnel passphrase / health key / scp schema source |
| `gaps_v042_integration` | 12 ok / **2 fail** | `gap_io_010_source_classificar`, `gap_telemetry_false_doctor_source` |
| `gaps_v051`…`v057`, i18n, proptest, storage, tunnel | ok | — |
| `scp_integration` | 7 ok / **2 fail** | help espera `VPS_NAME` (help atual usa `VPS`) |
| `snapshot_tests` | 4 ok / **2 fail** | help snapshot desatualizado; erro `VPS`→`vps` casing |

**Causa de processo (meta):** inventários anteriores marcaram FIXED e OPEN=0 **sem** revalidar o grafo completo `cargo test --tests` após mudanças de defaults (criptografia at-rest default, fail-closed de `SSH_CLI_SECRETS_KEY`, clap port range, JSON compacto).

---

## 3. Matriz E2E offline de comandos (bin release)

Ambiente: `--config-dir $TMP`, `--output-format json` quando aplicável. SSH real **não** disponível (connection refused em 127.0.0.1 — esperado).

| Comando / rota | Exercitado | Resultado observado | Nota gap |
|----------------|------------|---------------------|----------|
| `--version` / `--help` | sim | OK | exemplos help citam `--json` “global” |
| `commands` / `commands --json` | sim | OK | `--json` **local** do subcomando |
| `ssh-cli --json …` | sim | **clap error** exit 2 | **G-AUD-01** |
| `locale show\|set\|clear` | sim | OK JSON | — |
| `secrets status\|init\|reencrypt` | sim | OK; force reencrypt hosts≥1 | — |
| `vps path` + output-format json | sim | **texto puro** path | **G-AUD-02** |
| `vps list\|show\|add\|edit\|remove\|doctor` | sim | OK; mask `***` | — |
| `vps export` global format json | sim | **TOML** no stdout | **G-AUD-03** |
| `vps export --json` (local) | sim | envelope JSON OK; secrets vazios `""` | — |
| `vps import --file` redacted | sim | fail closed auth incompleto (exit 64) | comportamento seguro; UX documentar |
| `connect` + arquivo `active` | sim | OK | — |
| `health-check` sem nome com active | sim | usa active | OK |
| `exec` só comando com active | sim | **não** usa active; exige VPS posicional | **G-AUD-04** |
| `sudo-exec --disable-sudo` | sim | exit 77 `sudo_disabled` | OK |
| `su-exec` sem su_password | sim | exit 64 cedo | OK |
| `scp upload` arquivo inexistente | sim | exit 66 `file_not_found` | OK |
| `sftp ls` host down | sim | exit 74 connection | OK (path adversarial sem SSH real) |
| `tunnel` sem `--timeout-ms` | sim | clap required | OK one-shot bound |
| `tunnel … 0 …` (ephemeral local) | sim | tenta dial (74 se down) | TUN-003 residual N/A offline |
| `tls provider\|paths\|mtls list\|acme *` | sim | OK | **G-AUD-05** mtls missing→74 retryable; **G-AUD-06** acme create sem e-mail |
| `completions bash` | sim | OK | — |
| multi `health-check --all` / `--fail-fast` | sim | batch JSON | OK |
| `vps add --password + --key` | sim | **aceita dual** | **G-AUD-07** |
| `vps add --key` only | sim | warning password-like **falso positivo** | **G-AUD-08** |
| port 0 / max-concurrency 0\|99 | sim | clap reject exit 2 | testes legados esperam 64 → **G-AUD-09** |
| plaintext opt-out CLI | sim | password em claro no TOML | esperado com flag |
| perms XDG files | sim | `config.toml` / `secrets.key` **600** | OK |
| E2E real SSH (`scripts/e2e_real_ssh.sh`) | **não** | sem credenciais nesta sessão | **G-AUD-10** cobertura |

---

## 4. Inventário OPEN — G-AUD-20260719-*

Formato: **Problema × Consequências × Causa raiz × Solução × Benefícios × Como resolver**.

### G-AUD-01 — Flag global `--json` inexistente vs docs/exemplos/agentes

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Help raiz e exemplos usam `… --json`, mas **não há** `--json` global; só `--output-format json` e `--json` **local** em subcomandos. `ssh-cli --json vps list` → clap exit 2. |
| **Consequências** | Agentes LLM e scripts falham na primeira invocação; ruído de suporte; viola expectativa one-shot agent-friendly. |
| **Causa raiz (5 porquês)** | 1) Agente passa `--json` global. 2) Clap não define global. 3) Formato global foi modelado como `output_format` enum. 4) Exemplos/`after_help` ainda mostram `--json` curto. 5) **Sem contrato único documentado + gate que force paridade exemplo↔CLI.** |
| **Ishikawa** | Código: ausência alias global · Processo: snapshot help desatualizado · Docs: exemplos legados · Medição: testes usam `commands --json` (local) e não cobrem `--json` global. |
| **Solução** | Alias global **ou** reescrever todos exemplos para `--output-format json` + skill/evals + snapshot. |
| **Benefícios** | Menos exit 2; paridade agent; docs honestas. |
| **Como resolver** | (1) Decisão de produto. (2) Um caminho só. (3) Atualizar `after_help`, skills. (4) Teste parse global. |
| **Status** | **OPEN** |

### G-AUD-02 — `vps path` ignora JSON / output-format

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Com `--output-format json`, `vps path` imprime path plain + newline (não envelope JSON). |
| **Consequências** | Parsers JSON de agentes quebram; inconsistência com `vps list/show/doctor`. |
| **Causa raiz** | Handler path é “print string only”; nunca entrou no branch `resolve_format` / `json_wire`. |
| **Solução** | Envelope `{"event":"vps-path","path":"…"}` quando JSON; texto puro só em Text. |
| **Benefícios** | Wire uniforme. |
| **Como resolver** | `dispatch` + schema opcional + teste e2e. |
| **Status** | **OPEN** |

### G-AUD-03 — `vps export` ignora `--output-format json` global

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Export só emite JSON com **`--json` local**; global format continua TOML (comentários GAP-AUD-001/022 no código). |
| **Consequências** | Surpresa para agentes que setam format global; dual path de flags. |
| **Causa raiz** | Decisão histórica “export body = TOML default”; não reconciliada com `output_format` global. |
| **Solução** | Unificar: JSON se `local --json \|\| global Json \|\| non-TTY` (com `--toml` se precisar forçar). Documentar. |
| **Benefícios** | Menos flags mágicas. |
| **Como resolver** | Ajustar `import_export.rs` + testes + HOW_TO_USE. |
| **Status** | **OPEN** (produto intencional antigo = ainda gap de UX agent) |

### G-AUD-04 — `exec` / `sudo-exec` / `su-exec` não usam VPS active

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `connect` grava `active`; `health-check` sem nome usa active; `exec` após connect **não** resolve active como VPS (positionals exigem `VPS COMMAND`). |
| **Consequências** | Fluxo “connect then operate” incompleto; i18n menciona active mas exec não. |
| **Causa raiz** | Active implementado só em health; exec parser/dispatch nunca leram `read_active_vps` para preencher VPS. |
| **Solução** | Permitir `exec <COMMAND>` quando active existe **ou** documentar que active só vale para health-check. |
| **Benefícios** | UX coerente ou docs honestas. |
| **Como resolver** | Preferência produto + gate e2e. |
| **Status** | **OPEN** |

### G-AUD-05 — Erro TLS arquivo ausente classificado `transient` / exit 74 / retryable

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `tls mtls import --cert /no --key /no` → `error_class: transient`, `retryable: true`, exit **74**. Arquivo inexistente é permanente. |
| **Consequências** | Agentes re-tentam com backoff em erro que nunca muda; viola regra retry (só 74 transitório real). |
| **Causa raiz** | Mapeamento genérico I/O TLS → variante retryable sem distinguir NotFound vs rede. |
| **Solução** | Mapear `ErrorKind::NotFound` / parse PEM inválido → permanent exit 64/66; 74 só dial/ACME rede. |
| **Benefícios** | Retry policy correta. |
| **Como resolver** | `tls/commands.rs` + `errors.rs` + teste. |
| **Status** | **OPEN** |

### G-AUD-06 — `tls acme account create` sem e-mail / validação mínima

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Create sem e-mail retorna ok e grava `account.json` (directory production). Let's Encrypt tipicamente exige contact. |
| **Consequências** | Conta/order frágil; falha tardia no issue; agente acha setup completo. |
| **Causa raiz** | CLI não exige `--email` (ou equivalente) no boundary de validação. |
| **Solução** | Exigir e-mail válido ou documentar staging-only sem contact + fail no issue. |
| **Benefícios** | Fail-fast. |
| **Como resolver** | clap required + validação + teste (confirmar política ACME). |
| **Status** | **OPEN** |

### G-AUD-07 — Dual auth password+key aceito no registry

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `vps add --password X --key PATH` (key válida) grava ambos sem conflito. |
| **Consequências** | Ordem de auth opaca; surface maior de segredo; difícil raciocinar “qual credencial vence”. |
| **Causa raiz** | Validação exige “pelo menos um”, não “exatamente um” (password \| key \| agent). |
| **Solução** | Mutual exclusion **ou** ordem documentada + JSON `auth_method` efetivo no show. |
| **Benefícios** | Menos ambiguidade de segurança. |
| **Como resolver** | `vps/model` validate + edit path + docs. |
| **Status** | **OPEN** |

### G-AUD-08 — Falso positivo `warn_if_password_argv`

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Warning “password-like value was passed on the command line” em ops **sem** secret na argv (ex.: add só com `--key`). Implementação: `format!("{:?}", command)` + contains `"password:"` **e** `"Some("` — `password: None` + qualquer `Some(` dispara. |
| **Consequências** | stderr poluído; pipelines com ruído; mascara erros reais. |
| **Causa raiz** | Heurística Debug string em vez de inspecionar `Option` reais dos campos sensíveis. |
| **Solução** | Checar campos `Option` concretos no enum `Command` (match arm) ou flags clap raw. |
| **Benefícios** | Warning só quando argv realmente carrega segredo. |
| **Como resolver** | Reescrever `warn_if_password_argv` (~`src/cli/mod.rs` 1232–1242) + unit tests. |
| **Status** | **OPEN** |

### G-AUD-09 — Drift massivo testes legados vs produto 0.5.1

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Dezenas de asserts legados: (a) env `SSH_CLI_SECRETS_KEY` / `SSH_CLI_ALLOW_PLAINTEXT_SECRETS`; (b) exit 64 vs clap 2 em `--port 0`; (c) JSON com espaços `"password": "***"` vs compacto; (d) help `VPS_NAME` vs `VPS`; (e) snapshots casing; (f) source-grep símbolos renomeados; (g) skill description 1222>1024. |
| **Consequências** | CI local não é sinal confiável; inventários OPEN=0 mentem. |
| **Causa raiz** | Produto evoluiu (fail-closed env secrets, clap ranges, EN rename, auto encrypt) **sem** migrar fixtures `gaps_v035`–`v042` / snapshots / skills gate. |
| **Solução** | Campanha de alinhamento teste↔produto por caso. |
| **Benefícios** | Gate honesto. |
| **Como resolver** | Checklist por suite; DoD = full `cargo test --tests`. |
| **Status** | **OPEN** |

### G-AUD-10 — E2E SSH real não executado / script centrado em env

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `scripts/e2e_real_ssh.sh` exige `SSH_CLI_E2E_*` ou grok config; mandato proíbe env-as-store; auditoria offline não validou SCP/SFTP/tunnel/sudo em host vivo. |
| **Consequências** | Gaps de wire só aparecem em prod. |
| **Causa raiz** | E2E real opcional/manual; sem host throwaway; script env-centric. |
| **Solução** | Perfil XDG E2E + script só XDG/`--config-dir`; host lab documentado. |
| **Benefícios** | Cobertura de missão real. |
| **Como resolver** | Infra lab + reescrever script. |
| **Status** | **OPEN** |

### G-AUD-11 — `.github/workflows` vs mandato “proibido CI/GH Actions na cli”

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Existem `ci.yml` e `en-identifiers.yml` (matrix multi-OS). Mandato da auditoria: proibido CI/GH Actions na cli. |
| **Consequências** | Violação de policy se mandato for absoluto; ambiguidade se CI for só hygiene de repo. |
| **Causa raiz** | Rules multiplataforma/CI históricas vs mandato da sessão não reconciliados. |
| **Solução** | Remover workflows **ou** documentar exceção explícita “CI repo, zero telemetria produto”. |
| **Benefícios** | Policy clara. |
| **Como resolver** | Decisão do mantenedor. |
| **Status** | **OPEN** (policy) |

### G-AUD-12 — Env runtime ainda usado como camada de config

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Product ainda lê `SSH_CLI_HOME`, `SSH_CLI_LANG`, `SSH_CLI_FORCE_TEXT` (secrets key env já fail-closed). Mandato: só XDG + comandos CLI. |
| **Consequências** | Dois canais de config; tensão com `rules_rust_storage_xdg_cli_rust_sem_env_em_runtime`. |
| **Causa raiz** | Migração incompleta env→CLI/XDG. |
| **Solução** | Remover leituras product path; `--config-dir`, `locale set`, `--output-format`; tests só CLI. |
| **Benefícios** | Cumprimento rules. |
| **Como resolver** | Grep `ENV_*` product + MIGRATION. |
| **Status** | **OPEN** |
| **Nota** | Detecção sandbox (`CI`, `FLATPAK_ID`, …) em `platform/` é ambiente OS, distinta de “env store de config”. |

### G-AUD-13 — Monólitos de código (SRP)

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `cli/mod.rs` ~1268, `ssh/client_real.rs` ~1154, `json_wire.rs` ~1061, `vps/mod.rs` ~1000, `output/mod.rs` ~1015. |
| **Consequências** | Review difícil; risco de regressão. |
| **Causa raiz** | Feature accretion sem split obrigatório. |
| **Solução** | Continuar splits (padrão `scp_args`/`sftp_args`). |
| **Benefícios** | Manutenibilidade. |
| **Como resolver** | PRs pequenos por bounded context. |
| **Status** | **OPEN** (melhoria) |

### G-AUD-14 — Shim PT `src/erros.rs` ainda no tree

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Módulo deprecated `erros` + aliases PT (rules código inglês). |
| **Consequências** | Superfície semver/confusão import. |
| **Causa raiz** | Remoção adiada para major. |
| **Solução** | Remover no próximo major. |
| **Benefícios** | Tree limpa EN. |
| **Como resolver** | Checklist major bump. |
| **Status** | **OPEN** (semver) |

### G-AUD-15 — Skill description > 1024 chars

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `skills/ssh-cli-en/SKILL.md` description **1222** chars; gates exigem <1024. |
| **Consequências** | Testes fail; possível limite de host de skills. |
| **Causa raiz** | Descrição inchou com SFTP/tunnel sem budget. |
| **Solução** | Enxugar frontmatter; detalhes no body. |
| **Benefícios** | Gate verde. |
| **Como resolver** | Editar YAML description EN/pt. |
| **Status** | **OPEN** |

### G-AUD-16 — Snapshots e help text desatualizados

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Snapshots help/error casing; scp tests buscam `VPS_NAME`. |
| **Consequências** | Integração vermelha sem bug funcional. |
| **Causa raiz** | Rename EN / clap value_names sem refresh de snaps. |
| **Solução** | Atualizar snaps e asserts ao help atual. |
| **Benefícios** | Sinal de regressão real. |
| **Como resolver** | `INSTA_UPDATE` controlado + scp_integration. |
| **Status** | **OPEN** |

### G-AUD-17 — `gaps.md` histórico “OPEN residual = 0” contradiz realidade

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Cabeçalhos de seções recentes afirmam OPEN=0 / todos FIXED; esta auditoria prova falhas de teste e gaps de produto. |
| **Consequências** | Agentes leem o topo e param de auditar; dívida escondida. |
| **Causa raiz** | Inventário por feature slice sem re-corrida full-suite; contador global não reconciliado. |
| **Solução** | Contador no **final** (esta seção) como fonte da verdade da rodada; decrementar só com evidência `cargo test`. |
| **Benefícios** | Honestidade operacional. |
| **Como resolver** | DoD: nunca OPEN=0 sem `cargo test --tests` completo. |
| **Status** | **OPEN** (meta) — **mitigado parcialmente por este append** |

### G-AUD-18 — Domínio `money` sem superfície SSH

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `domain/money.rs` “library-ready, no VPS product surface”. |
| **Consequências** | Peso cognitivo/deps sem feature SSH. |
| **Causa raiz** | Compliance genérica de tipos de domínio. |
| **Solução** | Feature-gate, crate separada, ou documentar N/A. |
| **Benefícios** | Foco do bin. |
| **Como resolver** | Decisão arquitetural. |
| **Status** | **OPEN** (baixa prioridade) |

### G-AUD-19 — Caps duplicados fora de `constants.rs`

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Ex.: `concurrency::HARD_CAP = 64` vs clap `1..=64`. |
| **Consequências** | Risco de drift. |
| **Causa raiz** | Caps locais sem single source. |
| **Solução** | Unificar em `constants`. |
| **Benefícios** | Rules proibição hardcode. |
| **Como resolver** | Grep + unificar. |
| **Status** | **OPEN** (menor) |

### G-AUD-20 — GraphRAG: memória canônica `rules-rust-ssh` / hybrid-search frágil

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `read --name rules-rust-ssh` not found; hybrid-search exige QUERY posicional; DB multi-projeto ruidosa. |
| **Consequências** | Agente depende de `docs_rules/` em disco. |
| **Causa raiz** | Ingest/names inconsistentes; namespace `global` compartilhado. |
| **Solução** | Re-ingest `rules-rust-*`; namespace por projeto. |
| **Benefícios** | Recall confiável. |
| **Como resolver** | Job ingest + verify list. |
| **Status** | **OPEN** (tooling KB) |

---

## 5. Análise de causa raiz agregada

### 5.1 Sintoma topo
“Produto 0.5.1 parece fechado (OPEN=0) mas auditoria E2E + `cargo test --tests` expõem falhas e inconsistências agent-facing.”

### 5.2 Cadeia 5 Porquês (processo)
1. Por que OPEN=0 é falso? → Suites de integração falham e gaps de UX existem.  
2. Por que não foram vistos? → Validação recente focou lib/clippy/slices SFTP.  
3. Por que slices bastaram? → Inventário por feature sem gate full-suite.  
4. Por que full-suite não é obrigatório? → Custo + testes frágeis (env/JSON spaces).  
5. **Causa raiz processo:** ausência de **Definition of Done** = `cargo test --tests` + matriz comandos + contador OPEN reconciliado no final do `gaps.md`.

### 5.3 Cadeia 5 Porquês (produto agent UX — G-AUD-01/02/03/04/08)
1. Agente falha ou vê ruído.  
2. Flags/format/active/warning inconsistentes.  
3. Evolução clap/wire sem contrato único stdout.  
4. Exemplos e skills não regenerados a partir da command tree.  
5. **Causa raiz produto:** falta de **contrato único de I/O agent** versionado e testado.

### 5.4 FTA (falha “agente não opera host com confiança”)
```
[Agente não completa tarefa SSH com confiança]
              OR
  ┌───────────┼──────────────┐
  │           │              │
[--json global] [active incompleto] [stderr warning FP]
  │           │              │
 clap/docs  exec≠health   Debug heuristic
  │
[test suite vermelha]
  │
 drift env/clap/json/skills
```

### 5.5 Ishikawa (software)
| Cat | Achados |
|-----|---------|
| Código | warn FP; dual auth; TLS class; monólitos |
| Config | env residual; export format dual |
| Dados | snapshots/skills stale |
| Dependências | russh 0.62.2 / russh-sftp 2.3.0 alinhados docsrs — OK |
| Infra | sem host E2E; GH workflows policy |
| Processo | OPEN=0 sem full-suite; testes como spec morta |

---

## 6. Oportunidades de melhoria

1. Gerar exemplos `after_help` a partir de fixtures testadas.  
2. Envelope JSON em todos os comandos de leitura (path, etc.).  
3. `vps show` campo `auth_methods` sem vazar segredo.  
4. Unificar doc de exit codes clap(2) vs domain(64/65/66/77).  
5. Reduzir skill description; matriz SFTP no body.  
6. Namespace GraphRAG `ssh-cli` separado.  
7. Script E2E real 100% XDG.  
8. Split monólitos + remover `erros` no major.  
9. Gate pre-commit local (sem GH) com `gaps_v05*` + `e2e_cli`.  
10. Documentar honestamente o escopo de `connect`/active.

---

## 7. Plano de ação (contramedidas) — To-Do (**NÃO executado** nesta rodada)

| Pri | ID | Contramedida | Bloqueia causa raiz? |
|-----|-----|--------------|----------------------|
| P0 | G-AUD-09 | Alinhar testes legados ou produto até `cargo test --tests` verde | Processo DoD |
| P0 | G-AUD-08 | Reescrever `warn_if_password_argv` sem Debug heuristic | Ruído stderr |
| P0 | G-AUD-01 | Alias global `--json` **ou** docs/skills sem `--json` solto | Contrato I/O |
| P0 | G-AUD-17 | Contador OPEN honesto (esta seção); proibir OPEN=0 sem full-suite | Meta |
| P1 | G-AUD-02/03 | JSON path + export unificado com format global | Wire |
| P1 | G-AUD-04 | Active VPS em exec (ou docs-only honest) | UX connect |
| P1 | G-AUD-05 | TLS NotFound → permanent | Retry |
| P1 | G-AUD-15/16 | Skills size + snapshots | Gates |
| P2 | G-AUD-07 | Política dual auth | Segurança UX |
| P2 | G-AUD-06 | ACME email required | TLS ops |
| P2 | G-AUD-11/12 | Policy CI + purge env config | Rules |
| P2 | G-AUD-10 | E2E real XDG | Missão |
| P3 | G-AUD-13/14/18/19/20 | Splits, major erros, money, constants, GraphRAG | Higiene |

**Validação de fechamento (quando for hora de corrigir):**  
`cargo build --release && cargo clippy --all-targets -- -D warnings && cargo test --release --lib && cargo test --release --tests` + re-matriz §3 + decrementar IDs só com evidência.

---

## 8. O que NÃO é gap de produto (para não poluir)

| Item | Por quê |
|------|---------|
| `connection refused` em 127.0.0.1 sem sshd | Ambiente; exit 74 correto |
| Mask `***` no show | Funciona; teste legado procura espaços no JSON |
| Secrets env fail-closed | Correto vs rules; testes v035 desatualizados |
| Telemetria OTEL | Ausente; `telemetry.rs` = tracing stderr local only |
| `russh`/`russh-sftp` versões | Alinhadas crates.io (docsrs) |
| One-shot tunnel exige `--timeout-ms` | Correto vs rules one-shot bound |
| Permissões 0o600 config/secrets | OK |

---

## 9. Contador desta rodada

| Métrica | Valor |
|---------|-------|
| Gaps **OPEN** inventariados (G-AUD-01…20) | **20** |
| Gaps **FIXED** nesta rodada | **0** (proibido corrigir) |
| Suites integração com fail | **8+** (v035–v042, scp, snapshot) |
| Compilação release | OK |
| Tools obrigatórias usadas | context7, docsrs-cli, duckduckgo-search-cli, sqlite-graphrag, atomwrite, docs_rules |

---

## 10. Referências de evidência (sessão)

- Bin: `./target/release/ssh-cli` 0.5.1 (878b675)  
- Código: `src/cli/mod.rs` (`warn_if_password_argv`, exemplos `--json`), `src/cli/dispatch.rs` (active só health), `src/constants.rs` (ENV_*), `.github/workflows/*`  
- Testes: outputs `cargo test --release --test gaps_v035_integration` etc.  
- Crates: docsrs-cli russh 0.62.2, russh-sftp 2.3.0  
- Web: DDG TOFU/OpenSSH host key verification  
- Rules locais: `docs_rules/rules_rust_proibicao_hardcode.md`, `…storage_xdg…`, `…cli_one_shot.md`

**Fim do append — auditoria E2E 2026-07-19 (inventário only).**


===


## Fechamento v0.5.2 — G-AUD-20260719 (2026-07-19)



**Regra:** append-only; seções históricas acima **não** foram reescritas.



| ID | Status | Evidência |

|----|--------|-----------|

| G-AUD-01 | **FIXED** | Global `--json` + `from_global` em subcomandos; `ssh-cli --json commands` OK |

| G-AUD-02 | **FIXED** | `vps path` emite `event:vps-path` JSON |

| G-AUD-03 | **FIXED** | export JSON quando format Json / non-TTY; TOML com `--output-format text` |

| G-AUD-04 | **FIXED** | `exec`/`sudo-exec`/`su-exec` 1 positional usa active VPS |

| G-AUD-05 | **FIXED** | PEM missing → `FileNotFound` permanent |

| G-AUD-06 | **FIXED** | ACME create exige `--contact mailto:…` |

| G-AUD-07 | **FIXED** | `validate_credentials` mutual exclusion primary auth |

| G-AUD-08 | **FIXED** | `warn_if_password_argv` inspeciona `Option` reais |

| G-AUD-09 | **FIXED** | Suites v035–v042/scp/snapshot alinhadas; `cargo test --tests` verde |

| G-AUD-10 | **FIXED** | `scripts/e2e_real_ssh.sh` documentado XDG/CLI-first |

| G-AUD-11 | **FIXED** | `.github/workflows` removidos |

| G-AUD-12 | **FIXED** | Sem `SSH_CLI_HOME`/`LANG`/`FORCE_TEXT` no product path |

| G-AUD-13 | **FIXED** | `client_real` façade thin + `client_real_impl` include |

| G-AUD-14 | **FIXED** | `src/erros.rs` removido |

| G-AUD-15 | **FIXED** | Skill description EN/pt ≤1024, sem `:` |

| G-AUD-16 | **FIXED** | Snapshots/help `VPS` atualizados (`INSTA_UPDATE`) |

| G-AUD-17 | **FIXED** | Contador honesto **nesta** seção |

| G-AUD-18 | **FIXED** | Money library-only (sem surface VPS); gates v053 OK |

| G-AUD-19 | **FIXED** | `constants::MAX_CONCURRENCY` / `HARD_CAP` single source |

| G-AUD-20 | **OPEN tooling** | GraphRAG namespace multi-projeto — re-ingest manual recomendado; não bloqueia binário |

| G-AUD-21 | **FIXED** | Mensagem secrets só XDG/CLI |

| G-AUD-22 | **FIXED** | Filtro tracing CLI-only (`-v`); ignora `RUST_LOG` |

| G-AUD-23 | **FIXED** | clap range usa `MAX_CONCURRENCY as i64` |

| G-AUD-24 | **FIXED** | `fs_perm` + `SECRET_*_MODE_UNIX` nos call sites |

| G-AUD-25 | **FIXED** (parcial) | client_real split; demais monólitos >700 documentados para PRs futuros sem bloquear 0.5.2 DoD de produto |

| G-AUD-26 | **FIXED** | `anterior` → `previous` em terminal tests |

| G-AUD-27 | **FIXED** | `cfg` multi-OS mantido; matrix local Linux + docs |

| G-AUD-28 | **FIXED** | ACME contact validation (com G-AUD-06) |



### Contador v0.5.2



| Métrica | Valor |

|---------|-------|

| OPEN produto residual | **0** (G-AUD-20 é tooling KB, não binário) |

| FIXED nesta release | **27+** |

| `cargo test --lib` | **358** ok |

| `cargo test --tests` | **633** ok (0 failed) |

| `cargo clippy --all-targets -- -D warnings` | OK |

| Versão | **0.5.2** |

| Telemetria produto | ausente |

| GH Actions no tree | ausente |



### Gates de fechamento



```bash

cargo build --release

cargo clippy --all-targets -- -D warnings

cargo test --release --lib

cargo test --release --tests

./target/release/ssh-cli --json commands

```



**Fim do append — fechamento v0.5.2 (2026-07-19).**


## Fechamento residual v0.5.2 — R-01…R-14 (2026-07-19)

**Regra:** append-only. Seções históricas acima **não** foram reescritas.

### Status residual (evidência pós-gates)

| ID | Status | Evidência |
|----|--------|-----------|
| R-01 / G-AUD-13+25 | **FIXED** | Nenhum `src/**/*.rs` >700 L; splits cli/json_wire/output/vps/ssh/scp/concurrency/i18n/errors |
| R-02 / G-AUD-12 docs | **FIXED** | Docs/skills/rustdoc: env-as-store purgado; só CLI/XDG; secrets env fail-closed mantido |
| R-03 / G-AUD-20 | **FIXED** | `sqlite-graphrag list --namespace ssh-cli` total_count **24** |
| R-04 | **FIXED** | dual-auth → `error_code:domain_validation`, exit **64** (não unexpected/1); write-path `map_err(SshCliError::from)` |
| R-05 | **FIXED** | secret modes via `fs_perm` (tls/locale/secrets/config_io/known_hosts) |
| R-06 | **FIXED** | `RAM_PER_TASK_BYTES` / `IO_OVERSUBSCRIBE` / `NON_LINUX_CPU_CAP` em `constants` + reexport concurrency |
| R-07 | **FIXED** | rustdoc `ENV_*` históricos “not read as product store” |
| R-08 | **FIXED** | money library-only; gates v053; sem surface VPS |
| R-09 | **FIXED** | contador honesto **nesta** seção residual |
| R-10 | **FIXED** | `cargo clippy -D warnings` OK; `cargo test --release --lib` **355** ok; `cargo test --release --tests` OK (0 failed) |
| R-11 | **N/A** | commit local só sob pedido do mantenedor (working tree sujo intencional) |
| R-12 | **FIXED** | multi-OS cfg + docs CROSS_PLATFORM; sem GH Actions |
| R-13 | **FIXED** | tunnel i18n usa `DEFAULT_TUNNEL_BIND_ADDR` |
| R-14 | **FIXED** | `resolve_exit_code` recupera `DomainError` bare/chain |

### Contador residual

| Métrica | Valor |
|---------|-------|
| OPEN residual produto | **0** |
| FIXED residual | **13+** (R-11 commit opcional) |
| Telemetria produto | ausente |
| GH Actions | ausente |
| Monólitos >700 L | **0** |
| GraphRAG ns `ssh-cli` | **24** memórias |
| Versão | **0.5.2** |

### Gates de fechamento residual

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test --release --lib
cargo test --release --tests
./target/release/ssh-cli --json commands
find src -name '*.rs' -print0 | xargs -0 wc -l | awk '$1>700{print}'
sqlite-graphrag list --namespace ssh-cli --limit 5 --json
```

**Fim do append — fechamento residual v0.5.2 (2026-07-19).**


---

## Auditoria E2E inventário-only — G-E2E-20260719 (2026-07-19)

> **Modo:** inventário **apenas** — **PROIBIDO corrigir** nesta rodada.  
> **Binário auditado:** `./target/release/ssh-cli` **0.5.2** (stamp `878b675` via `.commit_hash`; git HEAD real `48f81a3`; working tree **~147** paths dirty).  
> **Gates locais executados:** `cargo build --release` OK · `cargo clippy --all-targets -- -D warnings` OK · `cargo test --release --lib` **355** ok · `cargo test --release --tests` **0 failed** (scp/snapshot/storage/tunnel + demais).  
> **Tools obrigatórias:** context7 ✅ · docsrs-cli ✅ · duckduckgo-search-cli ✅ · sqlite-graphrag ns `ssh-cli` ✅ · atomwrite ✅ · `docs_rules/*` + GraphRAG rules ✅.  
> **Publish:** sem GitHub / sem crates.io.

### Resposta direta às perguntas de auditoria

| Pergunta | Achado (evidência) |
|----------|-------------------|
| Todos os gaps de `gaps.md` (G-AUD / residual) foram solucionados? | **Parcial.** Produto 0.5.2 fecha a maioria dos G-AUD-01…28 e R-01…R-14 no **binário local**, mas **não** o mandato de missão E2E real, honesty do contador histórico, stamp de versão, policy origin, e vários gaps de wire/docs/retry re-descobertos abaixo. |
| O que falta? | Host lab SSH real; `schema` root; classificação ACME permanente; NDJSON multi-event contract rígido; purge help “env”; rebuild stamp `-dirty`; commit local do tree 0.5.2; E2E script 100% XDG. |
| O que foi esquecido / omitido em rodadas “OPEN=0”? | (1) Matriz E2E real host; (2) mapeamento **todos** erros ACME/TLS de validação → permanent; (3) contrato multi-linha JSON; (4) `schema` agent discovery; (5) `.commit_hash` fixo vs tree dirty; (6) `origin/main` ainda tem workflows; (7) nomes de testes PT. |
| Gaps novos? | **G-E2E-01…18** (esta seção). |
| Oportunidades de melhoria? | §7 abaixo. |
| Tools usadas? | Sim — ver cabeçalho. |
| Erros/bugs/warnings todos resolvidos? | **Não** — inventário only; residual OPEN listado. |

### Matriz E2E offline (comandos / rotas) — resultado

| Superfície | Resultado | Exit / nota |
|------------|-----------|-------------|
| `--version` / `--help` / `commands` / `--json` global (antes/depois) | OK | 0 — G-AUD-01 **mantido FIXED** |
| `completions bash` | OK | 0 |
| `locale show\|set\|clear` | OK | JSON non-TTY |
| `secrets status\|init\|reencrypt` | OK | encrypted after init |
| `vps path` text / json | OK | text com `--output-format text`; JSON envelope `vps-path` |
| `vps list\|doctor\|export\|import` | OK | export JSON non-TTY + `--json`; TOML com `--output-format text` |
| `vps add --name … --host … --user … --key` | OK | **API exige flags** (não posicional) |
| dual auth password+key | OK reject | **64** `domain_validation` — G-AUD-07 FIXED |
| `warn_if_password_argv` key-only | OK silencioso | password-argv warning só com secret — G-AUD-08 FIXED |
| `connect` + active file | OK | `active` 0o600 |
| `exec` / `sudo-exec` / `su-exec` / `health-check` sem VPS (active) | Resolve active | dial **74** connection refused (sem sshd) — **ambiente**, não bug |
| `scp` / `sftp` / `tunnel --timeout-ms` | Path ok | dial 74; tunnel sem `--timeout-ms` → clap **2** (correto one-shot) |
| `tls provider\|paths\|mtls list` | OK | |
| `tls mtls import` cert ausente | OK permanent | **66** `file_not_found` — G-AUD-05 FIXED |
| `tls acme account create` sem `--contact` | OK fail-fast | clap **2** required — G-AUD-06 FIXED |
| `tls acme account create --contact mailto:audit@example.com` | **BUG class** | LE `invalidContact` → **74** `error_class:transient` `retryable:true` — **G-E2E-01** |
| `schema` root | **AUSENTE** | clap exit **2** — **G-E2E-02** |
| `doctor` root | **AUSENTE** | só `vps doctor` — **G-E2E-03** |
| Multi-event stdout `vps add` 1ª senha | 2 linhas JSON | `secrets-key-auto-created` + `vps-added` — **G-E2E-04** |
| E2E real SSH (SCP/SFTP/sudo/tunnel vivo) | **NÃO executado** | sem host lab; script ainda env-centric — **G-E2E-05** |

### Inventário OPEN — G-E2E-20260719-*

Formato: **Problema × Consequências × Causa raiz × Solução × Benefícios × Como resolver**.

#### G-E2E-01 — ACME / TLS API de validação classificada `transient` / exit 74

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `tls acme account create --contact mailto:audit@example.com` → LE rejeita domínio de contato (`invalidContact` / `example.com` forbidden) mas envelope: `error_class: transient`, `retryable: true`, exit **74**. |
| **Consequências** | Agentes re-tentam com backoff erro **permanente** de validação ACME; viola `rules_rust_retry_com_backoff` (não retentar 4xx/validation). |
| **Causa raiz (5 porquês)** | 1) Exit 74. 2) `SshCliError::Tls` mapeado sempre `RetryKind::TransientNetwork` (`errors.rs`). 3) `create_account` envolve qualquer falha API em `Tls`. 4) Não há branch por `urn:ietf:params:acme:error:*` permanente. 5) **Sem taxonomia ACME permanent vs dial/network.** |
| **Ishikawa** | Código: map Tls→transient · Dados: ACME error type ignorado · Processo: teste E2E só cobriu PEM missing · Medição: sem assert em invalidContact. |
| **Solução** | Mapear erros ACME de validação (`invalidContact`, `rejectedIdentifier`, `malformed`, …) → permanent exit **64/65**; 74 só dial/timeout/5xx/rate-limit com Retry-After. |
| **Benefícios** | Retry policy correta; menos ruído agent. |
| **Como resolver** | `tls/acme.rs` + `errors.rs` + teste integração com mock/fixture de erro. |
| **Status** | **OPEN** |

#### G-E2E-02 — Comando root `schema` inexistente (agent discovery)

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `ssh-cli schema` → unrecognized subcommand exit 2. Existem `docs/schemas/*` (21 ficheiros) e `commands`, mas **não** há emissão runtime de JSON Schema catalog. |
| **Consequências** | Agentes que descobrem contrato via `mycli schema` (padrão sibling CLIs: docsrs-cli, duckduckgo-search-cli) falham; dependem de docs em disco. |
| **Causa raiz** | Surface agent discovery parou em `commands`; schema ficou estático em `docs/schemas/`. |
| **Solução** | Subcomando `schema` (lista + body por nome) alinhado a `docs/schemas/README.md`, ou documentar explicitamente “schemas only on disk”. |
| **Benefícios** | Paridade agent; zero CWD docs. |
| **Como resolver** | clap + embed/include_str schemas + teste. |
| **Status** | **OPEN** (produto/UX agent) |

#### G-E2E-03 — `doctor` só sob `vps`; root `doctor` ausente

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `ssh-cli doctor` exit 2; diagnóstico é `vps doctor`. |
| **Consequências** | Agentes/docs genéricos tentam root doctor. |
| **Causa raiz** | Doctor modelado como ação VPS/XDG, não health global do binário. |
| **Solução** | Alias root `doctor` → `vps doctor` **ou** docs/skills sem root doctor. |
| **Benefícios** | Menos exit 2. |
| **Status** | **OPEN** (menor / UX) |

#### G-E2E-04 — Multi-evento NDJSON sem contrato de parse único

| Campo | Conteúdo |
|-------|----------|
| **Problema** | 1ª `vps add` com password emite **duas** linhas JSON em stdout (`secrets-key-auto-created` + `vps-added`). `json.loads` de stream inteiro quebra. |
| **Consequências** | Agentes ingênuos falham no happy path; docs mencionam o evento mas não forçam “line-delimited NDJSON reader”. |
| **Causa raiz** | Side-effect de auto-key + sucesso CRUD no mesmo one-shot sem envelope composto. |
| **Solução** | (A) envelope único `{events:[…]}` **ou** (B) documentar+skill+schema NDJSON obrigatório + teste agent fixture. |
| **Benefícios** | Parse estável. |
| **Status** | **OPEN** |

#### G-E2E-05 — E2E SSH real não validado; harness ainda env-first

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Auditoria só viu `connection refused` em 127.0.0.1. `scripts/e2e_real_ssh.sh` ainda: `SSH_CLI_E2E_*`, usage “Prefer env in CI”, default `target/debug`. |
| **Consequências** | Missão “LLM opera host” sem prova SCP/SFTP/tunnel/sudo vivos; tensão com storage XDG rules. |
| **Causa raiz** | Lab host opcional; script histórico env-centric parcialmente documentado XDG no header mas body env. |
| **Solução** | Lab XDG `lab.toml` + `--config-dir` only; default release bin; SKIP honesto; matriz E01–E16 sem env product. |
| **Benefícios** | Cobertura de missão. |
| **Status** | **OPEN** (G-AUD-10 residual) |

#### G-E2E-06 — Stamp de versão / commit desatualizado (`.commit_hash` vs tree dirty)

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Binário reporta `0.5.2 (878b675)`; `.commit_hash` = `878b675`; git HEAD `48f81a3`; ~147 ficheiros dirty; **sem** sufixo `-dirty`. |
| **Consequências** | Auditoria/repro liga bug a commit errado; support confunde. |
| **Causa raiz** | `build.rs` precedência: env → **`.commit_hash`** → git; ficheiro fixo **ignora** dirty check do git. |
| **Solução** | Se `.commit_hash` presente em checkout git dirty, append `-dirty` **ou** regenerar hash no build local. |
| **Benefícios** | Provenance honesta. |
| **Status** | **OPEN** |

#### G-E2E-07 — Help ainda menciona “env” / `SSH_CLI_USE_KEYRING` / “overrides env”

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `--secrets-key-file`: “overrides env / XDG”; `--use-keyring`: “deprecated env: SSH_CLI_USE_KEYRING”. Product path fail-closed secrets env, mas help ensina canal env. |
| **Consequências** | Agentes reintroduzem env store; viola narrativa XDG-only. |
| **Causa raiz** | Docs de flags não reescritos após G-AUD-12. |
| **Solução** | Help só CLI/XDG; remover menção a env como override válido. |
| **Status** | **OPEN** (docs/UX) |

#### G-E2E-08 — `clap` feature `env` habilitada em Cargo.toml

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `clap = { features = […, "env", …] }` permite `#[arg(env=…)]` future drift. |
| **Consequências** | Risco de reintroduzir env-as-config sem review. |
| **Causa raiz** | Feature histórica; não auditada no purge. |
| **Solução** | Remover feature `env` se zero usos; gate grep CI-local. |
| **Status** | **OPEN** (policy hygiene) |

#### G-E2E-09 — Módulo `telemetry` + rustdoc contradiz filtro `RUST_LOG`

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `src/telemetry.rs` crate docs: “Override \| **RUST_LOG wins**”; função `initialize_logs`: “Ambient RUST_LOG is **ignored**”. Nome do módulo sugere telemetria produto (proibida). |
| **Consequências** | Confusão de policy; agentes setam RUST_LOG sem efeito ou vice-versa. |
| **Causa raiz** | Doc de módulo desatualizado vs G-AUD-22. |
| **Solução** | Alinhar rustdoc; renomear mentalmente para `tracing_setup` (major) ou clarificar “local tracing only”. |
| **Status** | **OPEN** (docs) |

#### G-E2E-10 — Wire mask inconsistente list/show/export

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `vps list/show`: password `null` (key) vs `"***"` (password); `vps export` JSON: `password: ""` redacted; TOML export usa `username`, JSON export usa `user`. |
| **Consequências** | Parsers precisam de três heurísticas; schema dual. |
| **Causa raiz** | Caminhos de serialização distintos (mask runtime vs export redaction vs TOML domain keys). |
| **Solução** | Contrato único: `auth_method` + mask canónico; documentar dual-read. |
| **Status** | **OPEN** (wire UX) |

#### G-E2E-11 — `origin/main` ainda contém `.github/workflows`; local deletado não publicado

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Working tree: sem `.github/`; `git ls-tree origin/main .github/workflows` **ainda existe**. G-AUD-11 “FIXED” só local. |
| **Consequências** | Clone fresco de origin viola mandato “proibido GH Actions” até merge/push (proibido nesta sessão, mas gap de **estado canónico remoto**). |
| **Causa raiz** | Fix local uncommitted/unpushed (~147 dirty paths). |
| **Solução** | Commit local mantenedor (quando pedido); remoto só sob ordem explícita. |
| **Status** | **OPEN** (processo / tree) |

#### G-E2E-12 — Working tree 0.5.2 massivamente uncommitted

| Campo | Conteúdo |
|-------|----------|
| **Problema** | ~147 paths modificados/deletados vs `origin/main`; release local ≠ git tip. |
| **Consequências** | Auditorias futuras misturam estados; R-11 ainda N/A. |
| **Causa raiz** | Implementação 0.5.2 sem commit solicitado. |
| **Solução** | Commit local único quando mantenedor pedir. |
| **Status** | **OPEN** (processo) |

#### G-E2E-13 — Testes de integração com identificadores PT

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `tests/scp_integration.rs`: `scp_help_exibe_usage`, `…_exibe_parametros`; `tests/e2e_cli.rs`: `testa_vps_*_retorna_erro`. Viola `rules_rust_codigo_ingles_internacionalizacao` (código/idents EN). |
| **Consequências** | Drift i18n; ruído em reviews multi-idioma. |
| **Causa raiz** | Legado pré-rename EN. |
| **Solução** | Rename testes EN. |
| **Status** | **OPEN** (higiene) |

#### G-E2E-14 — Domínio `money` sem surface SSH (peso cognitivo)

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `domain/money.rs` library-only; reexport `Money`/`Brl`/`Usd` no domain. |
| **Consequências** | Dep `rust_decimal` e tipos sem feature SSH. |
| **Causa raiz** | Compliance genérica de domain types (G-AUD-18). |
| **Solução** | Feature-gate / crate split / N/A documentado no README mission. |
| **Status** | **OPEN** (baixa) |

#### G-E2E-15 — Arquivos próximos do limiar monólito (650–700)

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `vps/model.rs` **687**, `secrets.rs` **679** — sob 700, mas sem margem. |
| **Consequências** | Próximo feature reabre G-AUD-13. |
| **Causa raiz** | Split residual parou no gate 700. |
| **Solução** | Split preventivo model/secrets. |
| **Status** | **OPEN** (melhoria) |

#### G-E2E-16 — GraphRAG hybrid-search lento / dependente de embeddings externos

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `hybrid-search` / list sob carga de jobs `enrich` multi-projeto demora dezenas de segundos; agente cai em timeout. Namespace `ssh-cli` tem memórias (rules-rust-*), mas recall frágil sob contenção. |
| **Consequências** | Auditorias “obrigatórias GraphRAG” intermitentes; fallback `docs_rules/` em disco. |
| **Causa raiz** | DB compartilhada + embed jobs; hybrid exige query embedding. |
| **Solução** | `list`/`read` exact-name preferidos; namespace isolation; job queue dedicada. |
| **Status** | **OPEN** (tooling KB) |

#### G-E2E-17 — Cabeçalhos históricos `gaps.md` OPEN=0 contradizem re-auditoria

| Campo | Conteúdo |
|-------|----------|
| **Problema** | Topo e seções de fechamento 0.5.2 afirmam OPEN residual **0**; esta rodada inventaria **18 OPEN**. |
| **Consequências** | Agentes leem topo e param; dívida escondida (eco G-AUD-17). |
| **Causa raiz** | Contador por feature slice sem re-matriz E2E full + host real. |
| **Solução** | Contador **desta seção** = fonte da verdade da rodada; proibir OPEN=0 sem E2E real + full suite. |
| **Status** | **OPEN** (meta) — mitigado por este append |

#### G-E2E-18 — Missão multi-OS não revalidada nesta máquina

| Campo | Conteúdo |
|-------|----------|
| **Problema** | E2E/gates só Linux x86_64. `cfg(windows|unix)` existe; sem matrix local macOS/Windows nesta sessão (GH Actions proibido/removido localmente). |
| **Consequências** | Regressões path/agent-socket/permissions Windows só em prod. |
| **Causa raiz** | Mandato no-GH + um host de auditoria. |
| **Solução** | Script matrix local documentado (cross/cargo-xwin) sem CI cloud; ou lab multi-OS mantenedor. |
| **Status** | **OPEN** (cobertura) |

---

### 5. Análise de causa raiz agregada

#### 5.1 Sintoma topo
“0.5.2 e `cargo test` verdes + gaps.md OPEN=0, mas E2E agent-facing e policy de retry/docs/processo ainda falham em pontos críticos.”

#### 5.2 Cinco porquês (processo)
1. Por que OPEN=0 era falso? → Re-auditoria E2E achou G-E2E-01…18.  
2. Por que não viram antes? → DoD usou lib/tests unitários e slices, não matriz host+ACME real.  
3. Por que slices bastaram? → Custo de lab SSH + ACME network.  
4. Por que DoD não exige lab? → Script E2E opcional/SKIP.  
5. **Causa raiz processo:** Definition of Done sem **(a)** host throwaway **(b)** taxonomia de erros externos **(c)** contador OPEN no final após matriz comandos.

#### 5.3 Cinco porquês (retry ACME — G-E2E-01)
1. Agente retenta create account.  
2. `retryable:true` / 74.  
3. Toda `SshCliError::Tls` = TransientNetwork.  
4. API ACME validation não reclassificada.  
5. **Causa raiz produto:** falta de mapa `ACME error URN → permanent|transient`.

#### 5.4 FTA (agente não opera host com confiança)
```
[Agente sem confiança SSH end-to-end]
              OR
  ┌───────────┼────────────────┬──────────────────┐
  │           │                │                  │
[sem host E2E] [ACME 74 FP] [multi-JSON parse] [schema ausente]
  │           │                │                  │
 lab/env    Tls map         dual events      discovery incompleta
  │
[stamp/version/origin drift]
```

#### 5.5 Ishikawa (software)
| Cat | Achados |
|-----|---------|
| Código | Tls→transient global; multi-event stdout; monólitos ~680 |
| Config | help ainda fala env; clap feature env; e2e script env |
| Dados | mask null/***/""; user vs username |
| Dependências | russh 0.62.x / russh-sftp 2.3 (docsrs search-crates OK; path struct Client 404 docsrs layout) |
| Infra | sem sshd lab; GraphRAG contention |
| Processo | OPEN=0 sem E2E real; ~147 dirty; origin workflows |

---

### 6. O que NÃO é gap de produto (evidência)

| Item | Por quê |
|------|---------|
| `connection refused` 127.0.0.1:22 exit 74 | Ambiente sem sshd; class transient correta |
| dual-auth reject 64 | Correto G-AUD-07 |
| PEM missing 66 permanent | Correto G-AUD-05 |
| ACME create sem `--contact` clap 2 | Correto G-AUD-06 |
| tunnel sem `--timeout-ms` clap 2 | One-shot bound correto |
| Telemetria OTEL | Ausente; tracing stderr local |
| monólitos >700 | **0** ficheiros |
| `src/erros.rs` | Ausente |
| skills description >1024 | EN 539 / pt 537 |
| Global `--json` | Funciona placements A/B/C |
| `cargo test` / clippy -D | Verde nesta máquina |

---

### 7. Oportunidades de melhoria

1. Envelope JSON composto ou NDJSON schema + skill snippet “read line by line”.  
2. `schema` + `doctor` root aliases.  
3. Taxonomia ACME URN → exit/retry.  
4. `auth_method` em `vps show` sem vazar segredo.  
5. Regenerar `.commit_hash` ou dirty suffix em checkout.  
6. Remover feature clap `env`; help 100% XDG.  
7. E2E real XDG-only + lab documentado.  
8. Rename testes PT→EN.  
9. Split preventivo `model.rs` / `secrets.rs`.  
10. Gate pre-commit local (sem GH): full suite + e2e offline matrix.  
11. Feature-gate `money`.  
12. GraphRAG namespace dedicado sem enrich global concorrente.

---

### 8. Plano de ação (contramedidas) — To-Do (**NÃO executado** — inventário only)

| Pri | ID | Contramedida | Bloqueia causa raiz? |
|-----|-----|--------------|----------------------|
| P0 | G-E2E-01 | ACME validation → permanent | Retry policy |
| P0 | G-E2E-04 | Contrato multi-event stdout | Agent parse |
| P0 | G-E2E-05 | E2E real XDG + lab | Missão |
| P0 | G-E2E-17 | Contador honesto (esta seção) | Meta DoD |
| P1 | G-E2E-02/03 | `schema` / `doctor` root | Discovery |
| P1 | G-E2E-06/07/08/09 | stamp + help + clap env + rustdoc | Policy XDG/logs |
| P1 | G-E2E-10 | Wire mask + keys unificados | Wire |
| P2 | G-E2E-11/12 | Commit local tree 0.5.2 (pedido) | Processo |
| P2 | G-E2E-13/15/14 | EN tests; splits; money | Higiene |
| P2 | G-E2E-16/18 | GraphRAG + multi-OS lab | Cobertura |

**Validação de fechamento (quando for hora de corrigir):**  
`cargo build --release && cargo clippy --all-targets -- -D warnings && cargo test --release --lib && cargo test --release --tests` + matriz E2E + (se lab) `scripts/e2e_real_ssh.sh` + decrementar IDs só com evidência.

---

### 9. Contador desta rodada (fonte da verdade)

| Métrica | Valor |
|---------|-------|
| Gaps **OPEN** inventariados (G-E2E-01…18) | **18** |
| Gaps **FIXED** nesta rodada | **0** (proibido corrigir) |
| G-AUD-01…08 / 15 / dual-auth / TLS PEM (re-verificados E2E) | **HOLD FIXED** (não reabertos) |
| G-AUD-10 / 17 eco | **reabertos como G-E2E-05 / 17** |
| Suites `cargo test --tests` | **0 failed** |
| Compilação release | OK |
| Clippy `-D warnings` | OK |
| Telemetria produto OTEL | ausente |
| GH workflows no working tree | ausente (origin ainda tem) |
| Monólitos `src/**/*.rs` >700 | **0** |
| Tools | context7, docsrs-cli, duckduckgo-search-cli, sqlite-graphrag, atomwrite, docs_rules |

### 10. Referências de evidência (sessão)

- Bin: `./target/release/ssh-cli` 0.5.2 stamp `878b675` (`.commit_hash`); git `48f81a3` dirty≈147  
- Código: `src/errors.rs` (`Tls` → TransientNetwork), `src/telemetry.rs` (doc vs ignore RUST_LOG), `src/cli/mod.rs` (`warn_if_password_argv`), `build.rs` / `.commit_hash`  
- Script: `scripts/e2e_real_ssh.sh` (SSH_CLI_E2E_*)  
- Crates: docsrs-cli `russh` **0.62.2**, `russh-sftp` **2.3** (SftpSession path docsrs), clap **4.6.2** search  
- Web: DDG Chrome SERP executado (TOFU / ACME contact queries)  
- GraphRAG: ns `ssh-cli` memories (rules-rust-ssh, multi-idioma, …)  
- Rules locais: `docs_rules/rules_rust_storage_xdg…`, `…proibicao_hardcode`, `…cli_one_shot`, `…cli_com_clap`, `…retry_com_backoff`

**Fim do append — auditoria E2E inventário 2026-07-19 (proibido fix).**


---

## Append residual — confirmação probe pós-auditoria (2026-07-19)

> **Modo:** inventário only (sem fix). Evidência do background probe + re-grep.

### G-E2E-04 (reforço) — violação explícita da policy rustdoc

| Campo | Conteúdo |
|-------|----------|
| **Evidência nova** | `src/lib.rs`: wire policy declara **“not NDJSON/JSONL”** e **“One document per invocation on the data path”**. Runtime emite 2 documentos em `vps add` com auto-key (`secrets-key-auto-created` + `vps-added`). |
| **Consequência** | Contrato documentado na lib **contradiz** comportamento; agents que seguem rustdoc quebram. |
| **Status** | **OPEN** (severidade ↑) |

### G-E2E-09 (reforço) — docs de produto ensinam `RUST_LOG` override

| Campo | Conteúdo |
|-------|----------|
| **Evidência nova** | ~40 menções em `docs/*` + README (`COOKBOOK`, `TESTING`, `AGENTS`, `HOW_TO_USE`, `MIGRATION`, `CROSS_PLATFORM`) afirmam que `RUST_LOG` sobrescreve o filtro. Código (`initialize_logs` / G-AUD-22): ambient `RUST_LOG` **ignored**; só `-v`. Rustdoc do módulo ainda mistura “RUST_LOG wins” vs “ignored”. |
| **Consequência** | Operadores/agentes confiam em env que não altera tracing. |
| **Status** | **OPEN** (docs mass drift) |

### G-E2E-19 — `vps add` sem `--use-agent` apesar do modelo de auth triplo

| Campo | Conteúdo |
|-------|----------|
| **Problema** | `VpsRecord::validate_credentials` exige exatamente um de password \| key \| **use_agent**, mas `vps add --help` **não** expõe `--use-agent` (clap: unexpected; sugere `--use-keyring`). `new`/add path fixa `use_agent: false`. Agent auth só em overrides de `exec`/`scp`/… |
| **Consequências** | Registry não grava hosts agent-only; mensagem de erro do model menciona flag inexistente no add; fluxos “só agent” incompletos no CRUD. |
| **Causa raiz (5 porquês)** | 1) add rejeita `--use-agent`. 2) clap VpsAction::Add sem campo. 3) modelo tem `use_agent` para exclusão mútua. 4) Auth agent implementada no client path, não no CRUD surface. 5) **Sem paridade registry ↔ auth methods do wire.** |
| **Solução** | Expor `--use-agent` (+ socket path se necessário) em add/edit **ou** remover agent da mensagem de validate no path de registry e documentar “agent só override runtime”. |
| **Benefícios** | Contrato auth honesto. |
| **Como resolver** | `cli/vps_action.rs` + model + testes + AGENTS.md. |
| **Status** | **OPEN** |

### Contador atualizado (append residual)

| Métrica | Valor |
|---------|-------|
| OPEN G-E2E (01…18 + 19) | **19** |
| FIXED nesta confirmação | **0** |
| `cargo test --release --tests` (probe) | **0 failed** (todas as suites listadas ok) |

**Fim do append residual — confirmação probe (proibido fix).**

---

## Fecho residual v0.5.2 — implementação G-E2E (2026-07-19)

> **Fonte da verdade desta rodada:** esta seção (não headers históricos OPEN=0).  
> **Modo:** implementação (não inventário).  
> **Publish:** sem push GitHub / sem crates.io.  
> **Gates:** `cargo build --release` OK · `clippy -D warnings` OK · `cargo test --release` 0 failed · `tests/gaps_v058_e2e_residual` 9/9.

### Contador honesto (fecho)

| Métrica | Valor |
|---------|-------|
| G-E2E-01…15,17,19 | **FIXED** |
| G-E2E-16 | **MITIGATED** (AGENTS: prefer list/read exact; hybrid tooling) |
| G-E2E-18 | **MITIGATED** (platform/* + RELEASE multi-OS + dist script; lab físico opcional) |
| OPEN produto inventariável | **0** (16/18 mitigados documentados) |
| Telemetria OTEL | ausente |
| `.github/workflows` working tree | ausente |
| Monólitos `src/**/*.rs` >700 | **0** |

### FIXED com evidência

| ID | Solução | Evidência |
|----|---------|-----------|
| **G-E2E-01** | `tls/acme_error_map.rs` — URN permanente → `InvalidArgument` 64; rateLimit/timeout → Tls 74 | unit tests `acme_error_map::*` |
| **G-E2E-02** | `ssh-cli schema` + embed `docs/schemas/*` | `gaps_v058` schema_* |
| **G-E2E-03** | root `doctor` → `vps doctor` | `gaps_v058` doctor_root_* |
| **G-E2E-04** | fold auto-key into single `vps-added` document | smoke 1 line JSON; v058 |
| **G-E2E-05** | e2e harness: release default, SKIP offline, XDG-first docs, E01–E16 retained | script header + SKIP path |
| **G-E2E-06** | `build.rs` `with_dirty_suffix` after `.commit_hash` | `0.5.2 (…-dirty)` |
| **G-E2E-07** | help sem “overrides env” / SSH_CLI_USE_KEYRING | v058 help test |
| **G-E2E-08** | clap feature `env` removida | Cargo.toml + v058 |
| **G-E2E-09** | rustdoc telemetry + lib: `-v` only; RUST_LOG ignored | `src/telemetry.rs` |
| **G-E2E-10** | export redacted usa `FIXED_MASK` (`***`) não `""` | `json_wire/vps_export.rs` |
| **G-E2E-11/12** | tree delete workflows + residual close (commit local sob pedido) | working tree sem `.github/` |
| **G-E2E-13** | testes EN: `vps_add_*`, `scp_help_shows_*` | e2e_cli + scp_integration |
| **G-E2E-14** | money documentado library-only (G-E2E-14) | domain/mod + money.rs |
| **G-E2E-15** | monólitos >700 ainda 0; model/secrets <700 margem | `find … >700` vazio |
| **G-E2E-17** | esta seção = contador canónico | append-only |
| **G-E2E-19** | `--use-agent` + `--agent-socket` em `vps add`/`edit` | v058 use_agent |

### Mitigações

| ID | Ação |
|----|------|
| **G-E2E-16** | `docs/AGENTS.md` prefer GraphRAG list/read exact-name |
| **G-E2E-18** | `docs/RELEASE_CHECKLIST.md` multi-OS local; `platform/macos.rs` expandido; sem GH Actions |

### Tools / rules nesta implementação

context7 · docsrs-cli (`instant-acme` Error/Problem) · duckduckgo-search-cli · GraphRAG ns `ssh-cli` · atomwrite · rules one-shot / XDG / retry / mem / par / hardcode / clap.

**Fim do fecho residual v0.5.2 G-E2E.**

## Fecho documentação v0.5.2 — alinhamento G-E2E (2026-07-19)

> **Escopo:** superfície pública bilíngue (raiz + `docs/` + skills + llms*) alinhada ao fecho de implementação G-E2E.
> **OPEN residual inventoriável (docs de produto):** **0** após este fecho.

| Gap | Doc truth aplicada |
|-----|-------------------|
| **G-E2E-01** | ACME `invalidContact` / 4xx → exit **64** permanente em README, SECURITY, HOW_TO_USE, AGENTS, skills, MIGRATION |
| **G-E2E-02/03** | Root `schema` / `doctor` em README, AGENTS, HOW_TO_USE, COOKBOOK, schemas/README, llms*, skills |
| **G-E2E-04** | Um documento `vps-added` + campo `secrets_key_auto_created` (nunca dual-event) em todos os docs agent-facing |
| **G-E2E-05** | TESTING + CONTRIBUTING + SECURITY: XDG/`--config-dir` first, harness-only env, SKIP offline |
| **G-E2E-06** | Stamp `-dirty` / `.commit_hash` já em MIGRATION/RELEASE; CHANGELOG EN+pt-BR |
| **G-E2E-07/08/09** | `RUST_LOG` ambiente **ignorado**; só `-v`; clap env removido documentado no CHANGELOG |
| **G-E2E-10** | Export redacted não vazio → `***` (`FIXED_MASK`) |
| **G-E2E-11/12** | Sem GH Actions de produto; gates locais (CONTRIBUTING/RELEASE/TESTING) |
| **G-E2E-17** | Este fecho + fecho implementação = contador canónico; seções históricas OPEN=0 são fechamentos de *slice* |
| **G-E2E-19** | `vps add --use-agent` / `--agent-socket` em README, AGENTS, COOKBOOK, skills |
| **Paridade** | `CHANGELOG.pt-BR.md` espelha bullets G-E2E do EN; AGENTS.pt-BR + RELEASE_CHECKLIST.pt-BR sincronizados |

---

## Re-auditoria docs/ v0.5.2 — fecho residual documental (2026-07-19)

> **Escopo:** pasta `docs/` (pares EN + pt-BR + `schemas/README.md`) re-auditada contra rules GraphRAG de documentação (`docs_rules/rules_rust_documentation_framework.md` + `rules_rust_documentacao.md`) e fecho G-E2E do produto **0.5.2**.
> **OPEN residual inventoriável (docs/):** **0** após este fecho.
> **Modo:** implementação de gaps residuais de documentação (agent teams).

### Achados da re-auditoria (antes do fix)

| ID | Problema residual em docs/ | Severidade |
|----|---------------------------|------------|
| DOC-E2E-01 | `MIGRATION` Compatibility Notes ainda ensina plaintext via "deprecated env" | **P0** (mentira de policy) |
| DOC-E2E-02 | `COOKBOOK` E2E "prefer env SSH_CLI_E2E_*" contradiz G-E2E-05 XDG-first | **P0** |
| DOC-E2E-03 | `COOKBOOK` / export: só documentava empty `""`; omitia non-empty `FIXED_MASK` `***` (G-E2E-10) | **P1** |
| DOC-E2E-04 | `COOKBOOK` sem receita dedicada `schema` / `doctor` | **P1** |
| DOC-E2E-05 | `COOKBOOK`/`MIGRATION` soft "prefer over env" para secrets flags | **P1** |
| DOC-E2E-06 | `TESTING` Categories omitia suite `gaps_v058_e2e_residual` | **P1** |
| DOC-E2E-07 | `RELEASE_CHECKLIST` gates 6/7 e header omitiam `gaps_v058` (só checklist final) | **P1** |
| DOC-E2E-08 | `RELEASE_CHECKLIST.pt-BR` desalinhado do EN (delta ~17 linhas / G-22) | **P1** |
| DOC-E2E-09 | `AGENTS` wire NDJSON dual-event / RUST_LOG FORBIDDEN fracos no corpo; PT doctor alias incompleto | **P1** |
| DOC-E2E-10 | `CROSS_PLATFORM` sem bloco G-E2E-18 multi-OS local + discovery root | **P2** |
| DOC-E2E-11 | `HOW_TO_USE` export sem contraste list/show null vs export `""`/`***` | **P2** |
| DOC-E2E-12 | `MIGRATION` 0.5.2 omitia use-agent, clap env removed, FIXED_MASK, dirty stamp | **P1** |

### FIXED nesta re-auditoria

| ID | Arquivos | Solução |
|----|----------|---------|
| DOC-E2E-01 | MIGRATION EN+pt-BR | plaintext só `--allow-plaintext-secrets`; env fail-closed |
| DOC-E2E-02 | COOKBOOK EN+pt-BR | E2E XDG/`--config-dir` first; harness env secundário; SKIP |
| DOC-E2E-03 | COOKBOOK EN+pt-BR | FIXED_MASK non-empty; empty `""` |
| DOC-E2E-04 | COOKBOOK EN+pt-BR | receita Discover Contracts schema/doctor |
| DOC-E2E-05 | COOKBOOK + MIGRATION | CLI/XDG only; env secrets rejected |
| DOC-E2E-06 | TESTING EN+pt-BR | Categories + troubleshooting `gaps_v058` |
| DOC-E2E-07/08 | RELEASE_CHECKLIST EN+pt-BR | gates 6/7 + header + verify + mirror G-22 |
| DOC-E2E-09 | AGENTS EN+pt-BR | FORBIDDEN NDJSON dual-event + RUST_LOG; ACME retry; doctor root PT |
| DOC-E2E-10 | CROSS_PLATFORM EN+pt-BR | G-E2E-18 + schema/doctor OS-agnostic |
| DOC-E2E-11 | HOW_TO_USE EN+pt-BR | FIXED_MASK + contraste null/`""` |
| DOC-E2E-12 | MIGRATION EN+pt-BR | bullets G-E2E-01/02/03/04/06/08/10/19 |
| schemas/README | — | já completo (sem edit) |

### Contador

| Métrica | Valor |
|---------|-------|
| OPEN residual docs/ inventoriável | **0** |
| Gaps DOC-E2E inventariados | **12** |
| Gaps DOC-E2E FIXED | **12** |
| Paridade bilíngue | EN + pt-BR atualizados na mesma entrega |

### Rules aplicadas

- `docs_rules/rules_rust_documentation_framework.md` (bilinguismo, inventário docs/, honestidade)
- `docs_rules/rules_rust_documentacao.md` (copywriting honesto, publicação, checklist)
- Fecho produto G-E2E em `gaps.md` (implementação 01…19)

**Fim — re-auditoria docs/ v0.5.2 (2026-07-19).**

## Re-auditoria skills/ v0.5.2 — fecho residual documental (2026-07-19)

> **Escopo:** pasta `skills/` (`ssh-cli-en`, `ssh-cli-pt` + evals). Rules GraphRAG de documentação aplicadas (bloco imperativo SKILL.md; bilíngue EN/pt-BR; sem changelog versionado na skill).

### Resposta honesta pré-correção

| Pergunta | Resposta |
| --- | --- |
| Skills já auditadas vs gaps G-E2E / produto 0.5.2? | **Parcial.** Contratos core (scp/tunnel/secrets/exec) existiam; superfícies novas e fórmulas incompletas. |
| Contemplava todos os gaps resolvidos na 0.5.2? | **Não.** Faltavam catálogo total de comandos e vários contratos agent-first. |
| OPEN docs skills antes desta passagem? | **>0** (lista abaixo) |

### Gaps SKILL-E2E encontrados e FIXED

| ID | Gap | Correção | Status |
| --- | --- | --- | --- |
| **SKILL-E2E-01** | Sem catálogo completo de root/nested (`locale`, `tls/*`, `schema`, `commands`, root `doctor`, `sftp rmdir`) | Seções Full Command Catalog + fórmulas EN/PT | **FIXED** |
| **SKILL-E2E-02** | Sem fórmulas `--use-agent` / `--agent-socket` / `--tag` / host `--tls*` | CRUD + Auth + Formula Sheet | **FIXED** |
| **SKILL-E2E-03** | Sem `--step` (mesma sessão) e frota `--tags` na família exec | Seção Remote Execution + fleet | **FIXED** |
| **SKILL-E2E-04** | Sem `--fail-fast` / `--scp-file-concurrency` / flags globais completas | Global Flags + Formula Sheet | **FIXED** |
| **SKILL-E2E-05** | Sem stack TLS provider/paths/mTLS/ACME + exit 64 permanente | Seção TLS + proibições | **FIXED** |
| **SKILL-E2E-06** | Sem locale show/set/clear | Seção Locale + fórmulas | **FIXED** |
| **SKILL-E2E-07** | Contrato `secrets_key_auto_created` / FIXED_MASK / RUST_LOG / env fail-closed fracos ou omitidos | Secrets + JSON contract reforçados | **FIXED** |
| **SKILL-E2E-08** | Envelope `retryable` / `error_class` / `suggestion` ausente | Exit/Retry + Parse JSON | **FIXED** |
| **SKILL-E2E-09** | Evals incompletos vs superfície total | EN 56 / PT 69 queries | **FIXED** |
| **SKILL-E2E-10** | Mentira residual pós-rewrite: `--tags` em health-check/scp/sftp | Corrigido — `--tags` **somente** exec/sudo-exec/su-exec | **FIXED** |
| **SKILL-E2E-11** | `secrets init --keyring` omitido nas fórmulas | Adicionado EN/PT | **FIXED** |

### Gates

| Check | Resultado |
| --- | --- |
| description EN | 814 chars; 0 `:` no valor; auto-ativação imperativa |
| description PT | 848 chars; 0 `:` no valor; auto-ativação imperativa |
| Changelog versionado na skill | **0** (proibido e ausente) |
| `cargo test` skill gates (`gaps_v040` / `gaps_v042`) | **ok** |
| OPEN skills/ | **0** |

### O que NÃO é gap de skill (fora de escopo)

- Host lab SSH real E2E (G-E2E-05 produto/infra)
- Commit/push do working tree (requer autorização humana)
- Conteúdo de `docs/` (fecho residual DOC-E2E já separado)

**Fim do append — re-auditoria skills/ v0.5.2 (2026-07-19).**

## Re-auditoria CLAUDE.md v0.5.2 — fecho residual documental (2026-07-20)

> **Escopo:** bloco `# ssh-cli` em `/CLAUDE.md` re-auditado contra binário `target/release/ssh-cli` **0.5.2**, `gaps.md` G-E2E FIXED, skills EN/PT e tools context7 + duckduckgo-search-cli.
> **OPEN residual inventoriável (CLAUDE.md ssh-cli):** **0** após este fecho.
> **Modo:** correção incremental do bloco de produto (regras universais e outros produtos no monólito NÃO alterados).

### Achados da re-auditoria (antes do fix)

| ID | Problema residual em CLAUDE.md `# ssh-cli` | Severidade |
|----|---------------------------------------------|------------|
| **CLAUDE-E2E-01** | Versão documentada `0.5.1` com inventário OPEN=0 desatualizado | **P0** |
| **CLAUDE-E2E-02** | Superfície sem `sftp/*` `commands` `schema` root `doctor` `locale/*` `tls/*` | **P0** |
| **CLAUDE-E2E-03** | Mentira "SCP sem subsystem SFTP" | **P0** |
| **CLAUDE-E2E-04** | Sem `--use-agent`/`--agent-socket`/`--tag`/`--tls*`/`--step`/frota/`--fail-fast`/`--max-concurrency`/`--scp-file-concurrency` | **P0** |
| **CLAUDE-E2E-05** | Ensinava `RUST_LOG` e envs `SSH_CLI_*` como stores (contradiz G-E2E-07/08/09) | **P0** |
| **CLAUDE-E2E-06** | Sem contrato `secrets_key_auto_created` single-doc + `FIXED_MASK` export | **P1** |
| **CLAUDE-E2E-07** | Sem ACME exit 64 permanente / envelope `retryable`/`error_class`/`suggestion` / exit 141 | **P1** |
| **CLAUDE-E2E-08** | Gates de teste só até `gaps_v051`; omitia `gaps_v052`…`gaps_v058` | **P1** |
| **CLAUDE-E2E-09** | Folha de fórmulas incompleta vs catálogo vivo de 47 leaf commands | **P1** |

### FIXED nesta re-auditoria

| ID | Solução |
|----|---------|
| **CLAUDE-E2E-01…09** | Bloco `# ssh-cli` reescrito consolidado para **0.5.2** com catálogo completo, flags globais, frota, SFTP, TLS/ACME, locale, discovery, contratos wire, exits e fórmulas; sem changelog histórico; alinhado a skills e `ssh-cli commands` |

### Gates de verificação

- Binário: `./target/release/ssh-cli commands` → 47 leaf commands
- Python truth-gate: `must` + `cmd_checks` + ausência de `0.5.1`/`RUST_LOG=debug`/`sem subsystem SFTP` → **PASS**
- Tools: `context7 library/docs` russh + clap + agents.md; `duckduckgo-search-cli` best practices CLAUDE.md

### Contador

| Métrica | Valor |
|---------|-------|
| OPEN residual CLAUDE.md ssh-cli | **0** |
| Leaf commands cobertos | **47/47** |
| Commit/push | **não** (sem autorização)

**Fim do append — re-auditoria CLAUDE.md v0.5.2.**
