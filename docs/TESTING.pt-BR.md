# Guia de testes

> **0.5.4** — release de segurança e agent-native. Corrige DoS remoto pré-auth no banner SSH (A1), impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3), fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2) e adiciona flags de redução de payload (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) aplicadas antes da serialização. BREAKING: falha parcial multi-host agora sai com exit **1** (era 65); `--bind` fora do loopback exige `--i-accept-network-exposure`. Novo evento `tunnel_closed`.


> Rode o perfil certo de testes do ssh-cli sem travar em redes remotas.

- Leia este documento em [inglês](TESTING.md).
- Linha de produto: **0.5.3** (suites históricas residuais incluem **0.4.1** AUD-POST / `gaps_v041` e wire **0.5.2** / `gaps_v051`).


## Por que testes categorizados
- Unit tests protegem packing, schema, secrets AEAD e lógica pura sem servidores SSH.
- Integration tests protegem contratos da CLI, storage e snapshots.
- Fixtures opcionais `ssh-keygen` (G-PROC-02) geram chaves OpenSSH reais para testes
  de key-path; binário ausente faz skip — o produto em runtime nunca o spawna.
- Testes live remotos são opcionais e devem sempre usar timeouts rígidos e nunca logar credenciais.
- Gates de install resolve protegem o onboarding no crates.io (GAP-014).
- Suites residuais de gaps travam I/O de agente, exit codes, supply chain, mascaramento, wire SCP/SFTP e honestidade de docs.
- O `gaps.md` local é arquivo de auditoria do mantenedor **gitignored** (também excluído do cargo) — testes **não** devem assertar seu texto FIXED (G13/G15).
- **G6:** testes que tocam estado global de signal/cancel (`CANCEL_FLAG`) usam `#[serial_test::serial]` (dev-dep `serial_test`) para a suite ser determinística; **não** remova marcadores serial de testes de concorrência/sinal.
- **G11:** a baseline deve ficar verde na primeira execução; re-rodar até passar é **proibido** como estratégia de gate.


## Categorias de teste
- Unit tests dentro de módulos `src/**` (inclui cifragem padrão de `secrets`)
- CLI e2e em `tests/e2e_cli.rs`
- Gap/residual integration em `tests/gaps_v035_integration.rs` (só secrets fake)
- Suite residual de I/O de agente em `tests/gaps_v037_integration.rs`
- Suite residual pós-0.3.7 em `tests/gaps_v038_integration.rs`
- Suite residual pós-0.3.8 em `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC/DENY/CHG)
- Suite residual pós-0.3.9 / **0.4.0** em `tests/gaps_v040_integration.rs`
- Suite AUD-POST em `tests/gaps_v041_integration.rs` (EXP-001 export empty, TUN-002 exit 0 pós-bind, CLI-005/006 paridade auth, IO-009 `event: scp-transfer`, REL-006, DOC-041 honesty)
- Suite AUD-E2E em `tests/gaps_v042_integration.rs` (TUN-003, IO-010, UX-001, REL-007, ENV-001, DOC-042, SCP-024)
- Suite **0.5.2** em `tests/gaps_v051_integration.rs` (roundtrip de redaction do export, JSON `vps-export`, dual-read schema v3, evento secrets-init, guarda include-secrets, CRUD `vps-added`, empty command, import exit 65)
- Suite residual G-TLS em `tests/gaps_v052_tls_policy.rs`
- Suite de tipos de domínio em `tests/gaps_v053_domain_types.rs`
- Suite de tratamento de erros em `tests/gaps_v054_error_handling.rs`
- Suite residual unsafe/FFI em `tests/gaps_v055_unsafe_ffi.rs`
- Suite residual G-SSH em `tests/gaps_v056_ssh.rs`
- Suite residual G-SFTP em `tests/gaps_v057_sftp.rs` (superfície SFTP; preferir prova de efeito no destino a auto-certificação de inventário)
- Suite residual G-E2E em `tests/gaps_v058_e2e_residual.rs` (root `schema` / `doctor`, um único `vps-added` com `secrets_key_auto_created`, `--use-agent`, purge de env help/clap, `RUST_LOG` ambiente ignorado, export FIXED_MASK `***`, ACME exit 64, etc.)
- Storage integration em `tests/storage_integration.rs`
- Snapshot tests em `tests/snapshot_tests.rs`
- Superfície SCP em `tests/scp_integration.rs`
- Superfície tunnel em `tests/tunnel_integration.rs`
- Superfície dos modos de tunnel em `tests/gaps_v060_tunnel_modes.rs` — os modos `--reverse`, `--socks5` e `--remote-socket` da 0.5.4, a exclusão mútua entre eles, o rótulo `mode` no wire (`local` / `reverse` / `socks5` / `streamlocal`), o guard `--i-accept-network-exposure` nas duas pontas e o evento `tunnel_closed`
- Property tests em `tests/proptest_tests.rs`
- i18n integration em `tests/i18n_integration.rs`
- Runner da bateria de gates `scripts/check_all_gates.sh` — roda os dez gates obrigatórios numa invocação
- Gate de frescor de advisories `scripts/check_advisory_freshness.sh` — alcançável somente pelo runner da bateria
- Contrato de cobertura da bateria `tests/gaps_v064_gate_runner.rs`
- Script de install resolve `scripts/verify_install_resolve.sh`
- Gate de identificadores em inglês `scripts/check_en_identifiers.sh`
- E2E SSH real (opcional, local da máquina): `scripts/e2e_real_ssh.sh` — matriz oficial **E01–E18** (E10–E14 cobrem SCP upload/download/cmp/missing/preserve; **E17/E18** cobrem SFTP checksum + árvore recursiva — G7)
- Benchmarks em `benches/` (manual)


## Como rodar
### Bateria completa (obrigatória antes de declarar gate verde)
- Uma invocação roda todo gate obrigatório e reporta todo resultado.

```bash
bash scripts/check_all_gates.sh
```

- Acrescente `--json` para NDJSON, `--only ID[,ID...]` para subconjunto e `--list` para ver os ids.
- `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila, então um único arquivo de teste quebrado esconde todo gate atrás dele.
- A execução sempre reporta quantos gates pulou, então rodada parcial não pode ler como completa.
- A bateria é sequencial de propósito: os gates de cargo disputam um lock de `target/`, então rodá-los em paralelo não ganha nada.

### Loop local do desenvolvedor

```bash
cargo test --locked --all-targets
cargo clippy --all-targets --locked -- -D warnings
bash scripts/check_en_identifiers.sh
cargo build --release
bash scripts/verify_install_resolve.sh
```

### Perfis focados

```bash
cargo test --locked --test e2e_cli
cargo test --locked --test gaps_v035_integration
cargo test --locked --test gaps_v037_integration
cargo test --locked --test gaps_v038_integration
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v052_tls_policy
cargo test --locked --test gaps_v056_ssh
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo test --locked --test storage_integration
cargo test --locked --test snapshot_tests
cargo test --locked packing
cargo test --locked secrets::
cargo fmt --check
```

### E2E SSH real (nunca imprimir segredos) — G-E2E-05

```bash
# Preferido (XDG / CLI primeiro): config-dir isolado com hosts já cadastrados
ssh-cli --config-dir /tmp/ssh-cli-e2e-lab vps add --name e2e --host … --user … --password-stdin
bash scripts/e2e_real_ssh.sh --config-dir /tmp/ssh-cli-e2e-lab

# Env harness-only (NÃO é store de runtime do produto) — nunca commite esses valores
export SSH_CLI_E2E_HOST=… SSH_CLI_E2E_USER=… SSH_CLI_E2E_PASSWORD=…
bash scripts/e2e_real_ssh.sh

# Só do mantenedor local: parse de $HOME/.grok/config.toml
# Esse arquivo deve ficar em $HOME — nunca copie para este repositório.
bash scripts/e2e_real_ssh.sh --from-grok-config
```

- Binário default: `target/release/ssh-cli` (override só com harness `SSH_CLI_E2E_BIN`).
- Sem host lab / credenciais, o script sai **0** com **SKIP** (offline-safe; não trate SKIP como gate vermelho).
- Matriz oficial **E01–E18**; **E10–E14** = SCP upload, download, integridade (`cmp`), remoto ausente, preserve mode+mtime (SCP-023); **E17** = SFTP upload/download checksum; **E18** = SFTP árvore recursiva (G7).
- O script imprime só rótulos PASS/FAIL/SKIP — nunca host, user ou password.
- Suite residual: `cargo test --locked --test gaps_v058_e2e_residual`.
- **Política GAP-014 / fail2ban:** prefira `sshd` local ou VPS throwaway. **PROIBIDO:** tempestades de falha de autenticação em hosts de produção (ban do fail2ban). E2E em VPS de produção só com cuidado, whitelist de IP / `ignoreip`, e **sem** senhas erradas intencionais.


## Perfis de CI
- Este repositório atualmente embarca sem workflows de GitHub Actions.
- Mantenedores rodam o loop local do desenvolvedor antes de cada publish.
- Gates de publish incluem package dry-run, verificação de install resolve, paridade bilíngue de docs, checagem de identificadores em inglês (`bash scripts/check_en_identifiers.sh`), suites residuais `gaps_v040` + `gaps_v041` + `gaps_v042` + **`gaps_v051`** + **`gaps_v056`** + **`gaps_v057`** + **`gaps_v058`**, `cargo fmt --check` (G10), mais o loop canônico: `cargo test --locked --all-targets`, clippy `-D warnings` e `cargo build --release`.


## Variáveis de ambiente
- Use `--config-dir` nas invocações CLI para isolar config em testes (o produto não lê `SSH_CLI_HOME`).
- `--allow-plaintext-secrets` faz opt-out da cifragem padrão em testes que assertam TOML plaintext.
- Sem esse opt-out, a primeira gravação de segredo auto-cria `secrets.key` e cifra campos.
- Nível de tracing padrão é error; não espere prosa INFO em stderr por omissão.
- `RUST_LOG` ambiente é ignorado; use `-v`/`-vv`/`-vvv` ao diagnosticar falhas (G2/G14 com escopo na crate).
- `-v` → info, `-vv` → debug, `-vvv` → trace (filtro de log só via CLI).
- `NO_COLOR=1` estabiliza saída sensível a snapshot quando necessário.
- Nunca coloque senhas de hosts live em env vars que os testes imprimem.


## Troubleshooting
- Drift de snapshot: revise `tests/snapshots/` e atualize só mudanças intencionais de UI (incluindo strings de versão).
- Falhas de resolve de crypto: recheque pins e rode de novo o script de install sem ignorar a política do lock.
- Testes de timeout flaky: garanta que nenhum host remoto real seja necessário salvo configuração explícita.
- Falhas de permissão: confirme que dirs temporários são graváveis e que asserts de mode batem com o SO.
- Surpresas de fixture cifrada: passe `--allow-plaintext-secrets` ou forneça primary-key de teste via `--secrets-key-file` / XDG `secrets.key`.
- Stderr quiet inesperado: o padrão é tracing error; passe `-v`/`-vv`/`-vvv` se precisar de mais linhas (`RUST_LOG` ambiente é ignorado).
- Falhas intermitentes de cancel/signal: confirme que `#[serial_test::serial]` permanece nos testes que tocam estado global de cancel (G6); não paralelize esses casos.
- Baseline vermelha sem mudança de código: corrija a suite — loteria de re-execução não é gate (G11). Gates de publish incluem `cargo fmt --check` (G10).
- Falhas residuais de SCP / AUD-POST / 0.5.2 / SFTP / G-E2E: rode `cargo test --locked --test gaps_v040_integration`, `gaps_v041_integration`, `gaps_v042_integration`, `gaps_v051_integration`, `gaps_v056_ssh`, `gaps_v057_sftp` e `gaps_v058_e2e_residual`. O `gaps.md` local pode ter notas do mantenedor, mas **não** é artefato de gate publicado e testes não devem assertar seu texto (G13/G15).
