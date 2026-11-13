# Checklist de release: ssh-cli


> Gates obrigatórios antes de marcar release. O registro de auditoria é local e só do mantenedor (não publicado).

- Leia este documento em [Inglês (en)](RELEASE_CHECKLIST.md)
- Alvo de release / linha de produto: 0.5.6
- Os comandos `git` deste checklist são passos do mantenedor, rodados à mão no clone dele, porque a pasta de fontes usada para editar este documento não tem repositório git
- Gate histórico: 0.4.1 DOC-041 / honestidade AUD-POST (export vazio, tunnel exit 0, paridade de auth, evento scp-transfer)
- O inventário de auditoria é local e não publicado, porque é ignorado pelo git e excluído do cargo
- Mantenedores podem manter localmente
- Testes não devem afirmar seu texto FIXED (G13/G15)
- Suítes residuais: `tests/gaps_v039_integration.rs` (LOG-001, JSON-001, CLI-004, DOC-003, DENY-002, REL/CHG)
- `tests/gaps_v040_integration.rs` (protocolo de cópia segura (SCP) 0.4.0)
- `tests/gaps_v041_integration.rs` (EXP-001, TUN-002, CLI-005/006, IO-009, REL-006)
- `tests/gaps_v042_integration.rs` (AUD-E2E)
- `tests/gaps_v051_integration.rs` (wire/export/secrets 0.5.2)
- `tests/gaps_v057_sftp.rs` (G-SFTP)
- `tests/gaps_v058_e2e_residual.rs` (residual G-E2E: schema/doctor root, um único `vps-added`, `--use-agent`, purge de env help/clap, ACME 64)
- `tests/gaps_v065_exec_target_designation.rs` (GAP-SSH-EXEC-ARGC-001: a família exec e o `ssh-cli health-check` precisam receber o alvo no argv)
- `tests/gaps_v066_transfer_target_designation.rs` (a mesma regra estendida a `scp` e `sftp`)
- `tests/gaps_v067_packaging_reach.rs` (todo arquivo-fonte deve sobreviver ao `cargo package`, porque um padrão de `.gitignore` que engole diretório de código-fonte é invisível a todo build local)
- `tests/gaps_v068_source_language.rs` (prosa não-ASCII fora de citação, a metade que `scripts/check_en_identifiers.py` não consegue ler)
- `tests/gaps_v069_markdown_tables.rs` (linha de tabela cujos pipes não escapados discordam da linha delimitadora renderiza com colunas fantasma, então nenhum gate de prosa a enxergava)
- `tests/gaps_v070_doc_reference_reach.rs` (documento que nomeia uma suíte que o repositório nunca teve, o que a varredura direta não enxerga)
- `tests/gaps_v071_vps_edit_rename.rs` (rename nativo, oito flags de limpeza, dry-run, espera do lock)
- `tests/gaps_v072_edit_truthfulness.rs`, `tests/gaps_v072_import_roundtrip.rs`, `tests/gaps_v072_lock_contention.rs`, `tests/gaps_v072_schema_conformance.rs` e `tests/gaps_v072_storage_hardening.rs` (`updated` verdadeiro, round trip de import, exit 69 do lock, envelopes reais contra os schemas, modos de arquivo)
- `tests/gaps_v073_vps_auth_switch.rs`, `tests/gaps_v073_host_name_charset.rs`, `tests/gaps_v073_terminal_escape.rs` e `tests/gaps_v073_config_dir_mode.rs` (as quatro suítes da 0.5.6: troca de auth, nomes de host estritos, escape de terminal, modo do diretório de config)
- `tests/gaps_v074_flag_reach.rs` e `tests/gaps_v075_markdown_format_gate.rs` (cada flag documentada provada por comportamento, e o gate de formatação mantido na bateria)


## Propósito
- Impedir ship com gaps abertos, docs de product line defasados ou waivers de supply-chain
- Manter evidência de release honesta (notas pré/pós-fix locais, sem segredos em logs)
- Alinhar versão Cargo, `--version`, product line de docs, tags e âncoras do CHANGELOG


## Gates (obrigatórios)
- Gate 0: bateria completa de gates com `python3 scripts/check_all_gates.py`, que sai 0 com os catorze gates verdes e zero pulados
- `--list` imprime os ids dos gates, então conte ali em vez de confiar nesta frase
- Doze dos catorze ids mapeiam para os itens abaixo: `fmt` é o Gate 6b, `build-release` é o Gate 1, `build-no-default` é o Gate 1b, `clippy` é o Gate 2, `test` é o Gate 6, `exec-target` é a suíte de argv perigoso do Gate 7, `deny` é o Gate 4, `cross-targets` é o Gate 4c, `advisory-freshness` é o Gate 4d, `advisory-hits` é o Gate 4e, `en-identifiers` é o Gate 3 e `install-resolve` é o Gate 5, enquanto `doc` e `markdown-format` rodam só dentro da bateria
- O gate `doc` roda `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features`, então um aviso de rustdoc ou um link intra-doc quebrado reprova a bateria
- O gate `test` dela roda `cargo test --locked --all-features --no-fail-fast`, então uma rodada vermelha lista todos os testes que falham em vez de parar no primeiro
- O stdout dela traz uma linha por gate separada por tabs, com o id, `pass` ou `fail`, o exit code e os milissegundos, depois uma linha `_summary` com `total=`, `failed=` e `skipped=`
- O exit dela é 0 quando todo gate escolhido passa, 1 quando algum falha e 2 em erro de uso, como um id de gate desconhecido
- Rode isso antes de marcar qualquer item abaixo, porque `cargo clippy` e `cargo test` abortam no primeiro alvo que não compila e um único arquivo de teste quebrado esconde o estado de todo gate atrás dele
- `scripts/check_advisory_freshness.py` não tem outro chamador, então pular a bateria pula esse gate inteiro
- Gate 1: build de release com `cargo build --release` sai 0
- `cargo build --locked --release` e `cargo build --locked --all-features` não imprimem linha `warning:`, porque o manifesto não carrega mais `package.homepage`, o `serde_with` sem uso nem um cfg redundante do `human-panic`
- Gate 1b: build sem features default com `cargo build --no-default-features` sai 0 (o caminho Windows, A8)
- Gate 2: clippy limpo com `cargo clippy --all-targets --all-features -- -D warnings -W clippy::doc_markdown` sai 0
- Gate 3: identificadores em inglês com `python3 scripts/check_en_identifiers.py` sai 0
- Gate 4: supply chain deny (DENY-002) com `cargo deny check` sai 0, sem `ignore` de CVE russh, `yanked = "deny"` e `ignore = []` vazio
- `cargo audit` sai 0 quando o `cargo-audit` está instalado, e `rg -c 'rust_decimal|rkyv' Cargo.lock Cargo.toml` não acha nada (o RUSTSEC-2026-0235 saiu junto com o `rust_decimal` removido)
- Gate 4b: política crypto G-TLS, só com gates locais
  - `deny.toml` bane `openssl`, `openssl-sys`, `native-tls`, `libssh2-sys`, `ring`, `ssh2`, `thrussh` e `libssh-rs`
  - `rustls` é permitido de propósito, porque a feature `tls` padrão (SSH-over-TLS, mTLS, ACME) o usa só com o provider `aws_lc_rs`, o manifesto pede `rustls` 0.23.25 e o `ssh-cli tls provider` imprime o piso de segurança `min_rustls` 0.23.18
  - `cargo tree -i openssl`, `-i native-tls`, `-i ring` e `-i flate2` não reportam package
  - Compressão SSH é só `none`, e as features do russh não habilitam `flate2`
  - SECURITY.md / SECURITY.pt-BR.md com a política de transporte e criptografia (G-TLS), onde SSH não é TLS e o backend de criptografia é aws-lc-rs
  - Toda máquina de build precisa de compilador C, porque o `aws-lc-sys` compila C, e a migração para criptografia em Rust puro foi tentada e revertida, então não sobra `[patch.crates-io]` nem pasta `vendor-patches/`
- Gate 4c: build cross-target (B1, obrigatório) com `python3 scripts/check_cross_targets.py` sai 0 (`--timeout-secs N` limita cada chamada do cargo)
  - Type-check de `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` e `x86_64-apple-darwin`
  - Todo outro gate desta lista roda somente no triple do host
  - Código sob `#[cfg(target_os = ...)]` alheio é descartado antes do type-check e fica invisível para eles
  - O alvo Windows já falhou com seis erros com esta checklist inteira verde
  - Windows usa `--no-default-features`, porque a pilha TLS padrão puxa `aws-lc-sys`, que compila C (A8)
  - `docs/CROSS_PLATFORM.pt-BR.md` não pode declarar alvo que este script não verifica, e lista o `aarch64-apple-darwin` como não coberto
- Gate 4d: frescor dos advisories com `python3 scripts/check_advisory_freshness.py` sai 0
  - Ele falha quando o clone do banco RustSec é mais velho que a idade aceita, 7 dias por padrão
  - `--db-root` aponta para a pasta que guarda os clones do cargo-deny
- Gate 4e: hits de advisory com `python3 scripts/check_advisory_hits.py` sai 0
  - Ele roda `cargo audit --json --no-fetch` contra o `Cargo.lock` e falha em qualquer advisory, nomeando pacote, versão, id RUSTSEC e a versão corrigida
  - Ele se recusa a rodar sobre banco com mais de 7 dias, então um resultado verde sempre significa interseção fresca e nunca velha
  - A correção que ele espera é pin elevado no `Cargo.toml`, nunca entrada em `deny.toml` `[advisories] ignore`
- Gate 5: install resolve com `python3 scripts/verify_install_resolve.py` sai 0, com russh no piso de segurança (0.65.0 ou posterior, pela RUSTSEC-2026-0492 e pela RUSTSEC-2026-0495) e o `Cargo.lock` na 0.65.1
- Gate 6: testes completos com `cargo test --locked --all-features --no-fail-fast` verde: a lib mais todo arquivo de `tests/`
- Gate 6b: formatação (G10) com `cargo fmt --check` sai 0
- Gate 7: suítes residuais de gaps verdes, ou seja, todos os testes em `tests/gaps_v038_integration.rs`, `tests/gaps_v039_integration.rs`, `tests/gaps_v040_integration.rs`, `tests/gaps_v041_integration.rs`, `tests/gaps_v042_integration.rs`, `tests/gaps_v051_integration.rs`, `tests/gaps_v057_sftp.rs`, `tests/gaps_v058_e2e_residual.rs`, `tests/gaps_v064_gate_runner.rs`, `tests/gaps_v065_exec_target_designation.rs`, `tests/gaps_v066_transfer_target_designation.rs`, `tests/gaps_v067_packaging_reach.rs`, `tests/gaps_v068_source_language.rs`, `tests/gaps_v069_markdown_tables.rs`, `tests/gaps_v070_doc_reference_reach.rs`, `tests/gaps_v071_vps_edit_rename.rs`, `tests/gaps_v072_edit_truthfulness.rs`, `tests/gaps_v072_import_roundtrip.rs`, `tests/gaps_v072_lock_contention.rs`, `tests/gaps_v072_schema_conformance.rs`, `tests/gaps_v072_storage_hardening.rs`, `tests/gaps_v073_config_dir_mode.rs`, `tests/gaps_v073_host_name_charset.rs`, `tests/gaps_v073_terminal_escape.rs`, `tests/gaps_v073_vps_auth_switch.rs`, `tests/gaps_v074_flag_reach.rs`, `tests/gaps_v075_markdown_format_gate.rs` e `tests/docs_conformance.rs`
- Isso inclui testes que não se chamam `gap_*`
- Esta lista é o mínimo, porque o Gate 6 roda também todo outro arquivo de `tests/`
- `the_test_gate_stays_locked` em `tests/gaps_v064_gate_runner.rs` falha quando o gate `test` perde `--locked` ou `--no-fail-fast`
- Gate 8: e2e local sem VPS real: help, CRUD fake de VPS e completions se comportam como documentado
- Gate 9: teste rápido em VPS real quando disponível: `ssh-cli health-check` / `ssh-cli exec` mais matriz SCP E10–E14 e SFTP E17/E18 (matriz completa E01–E21) via `python3 scripts/e2e_real_ssh.py`
  - Use `--host --port --user --password-file`, ou `--local-sshd` só no Unix, quando houver credenciais
  - Prefira sshd local ou VPS descartável, sem tempestade de falha de autenticação em produção
  - Registre o resultado em notas locais do mantenedor sem segredos (não publique segredos, e o registro de auditoria é só local)
- Gate 10: inventário não publicado
  - O registro de auditoria NÃO DEVE ser publicado, porque é ignorado pelo git e excluído do cargo (uma regra no `.gitignore` e outra no exclude do `Cargo.toml`)
  - `rg -n -F '/gaps.md' .gitignore` deve achar a regra (exit 0)
  - PROIBIDO: testes circulares que afirmam texto FIXED dentro do registro de auditoria (G13/G15)
- Gate 11: evidência pré/pós-fix honesta para mantenedores (DOC-002 / integridade do inventário), com notas só locais e sem segredos embarcados
- Gate 12: string de versão (REL-002), em que `ssh-cli --version` bate com a versão Cargo mais o hash git e reporta `-dirty` quando a tree está suja
  - O hash resolve env, depois git, depois `.commit_hash`, depois `unknown`
  - Sem env e sem git o arquivo só vale se a versão for igual à do pacote
- Gate 13: commit e tag locais de release (REL-003), um passo do mantenedor no clone dele
  - `git status` limpo no commit de release
  - Mensagem HEAD de Release, e a tag local é `vX.Y.Z` (para 0.5.6: `v0.5.6`)
  - Sem push remoto sem autorização
- Gate 13b: refresh do `.commit_hash` (obrigatório depois do commit de release), porque o git fica fora deste diretório e o arquivo só pode guardar o commit de release depois que ele existe
  - Depois do commit de release, grave `<short hash> <version>` (para 0.5.6: o hash curto do commit de release, um espaço, `0.5.6`) em `.commit_hash`
  - Recompile com `cargo build --release` e confirme que `ssh-cli --version` mostra esse hash
  - Até esse passo rodar, um build sem git imprime `unknown`, porque o arquivo só vale se a versão for igual à do pacote
- Gate 14: sem telemetria, em que `ssh-cli vps doctor --json` reporta `"telemetry": false` sob `local` e não há SDKs de métricas ou telemetria na tree
- Gate 15: probes temporários removidos, sem artefatos `_probe_*` sobrando na tree
- Gate 16: nível padrão de tracing em error (LOG-001)
  - O nível default é error (não info), e o stderr em modo tunnel/JSON é só envelope (sem banners INFO como "Tunnel SSH:" / "iniciando tunnel")
  - Verbosidade graduada `-v`/`-vv`/`-vvv` com escopo na crate (G2/G14), e `RUST_LOG` ambiente ignorado
- Gate 17: docs de product line = versão Cargo (DOC-003), então toda superfície de product line declara 0.5.6
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
- Gate 18: senha vazia em JSON é null (JSON-001)
  - Runtime: `ssh-cli vps show --json` e `ssh-cli vps list --json` em host só-chave emitem `"password": null` (não a string da máscara), e senha não vazia permanece mascarada
  - Schema: `docs/schemas/vps-show.schema.json` (e list via `$ref`) declara `password` como `string` ou `null`
- Gate 19: timeout de health-check (CLI-004), em que `ssh-cli health-check --timeout <ms>` é aceito (parse clap), alinhado a overrides do exec, e coberto por gaps_v039
- Gate 20: âncoras do CHANGELOG (CHG-001)
  - `CHANGELOG.md` tem a seção `## [0.5.6]` e as âncoras de compare/rodapé para 0.5.6
  - As âncoras anteriores 0.5.5 / 0.5.4 / 0.5.3 / 0.5.2 / 0.4.x / 0.3.9 ficam conforme necessário
- Gate 21: package dry-run com `cargo package --allow-dirty --list` ok, nunca auto-publish
  - A listagem não tem arquivo `*.sh`, nem arquivo chamado `json`, nem `__pycache__`, nem `*.snap.new`
- Gate 22: DOC-004 / honestidade SCP (0.4.0+), em que as superfícies de product line documentam:
  - SCP somente arquivos regulares (sem diretórios / sem `-r`)
  - Árvores via `sftp --recursive`
  - crates.io 0.3.9 anunciava SCP mas o wire estava quebrado
  - Não prometa SCP funcional em 0.3.9
  - `docs/schemas/scp-transfer.schema.json` existe e está indexado (`docs/schemas/README.md`, `llms-full.txt`)
  - sufixo partial de download `.ssh-cli.partial`
  - `ssh-cli tunnel --json` / `tunnel_listening` e/ou superfície JSON de scp em README/INTEGRATIONS/AGENTS
  - skills bilíngues `skills/ssh-cli-en` e `skills/ssh-cli-pt` ensinam scp-transfer, tunnel_listening, somente arquivos, partial, 32 KiB e a matriz de timeout (DOC-004d)
  - SECURITY Supported Versions marca 0.5.x como linha atual (não 0.3.x)
  - `cargo test --locked --test gaps_v040_integration` + `gaps_v041_integration` verde
- Gate 23: DOC-041 / honestidade AUD-POST (0.4.x histórico), em que superfícies de product line e agentes documentam:
  - `ssh-cli vps export` redacted nunca documenta nem espera `sshcli-enc:` para segredos vazios
  - deadline pós-bind do tunnel sai com exit 0 após `tunnel_listening` (sucesso one-shot, não 74)
  - paridade de flags de auth de `ssh-cli tunnel` / `ssh-cli health-check` (`--password-stdin`, overrides de chave / passphrase conforme aplicável)
  - schema `scp-transfer` exige `event: "scp-transfer"`
  - `cargo test --locked --test gaps_v041_integration` verde
- Gate 24: DOC-051 / honestidade 0.5.2 (histórico), em que superfícies de product line documentam:
  - o corpo de `ssh-cli vps export` segue o formato resolvido: JSON `event: "vps-export"` em qualquer stdout non-TTY
  - TOML somente com `--output-format text`
  - wire schema v3 dual-read (serialize EN / aliases PT no load)
  - schemas de secrets `secrets-init.schema.json` / `secrets-reencrypt.schema.json` indexados
  - tunnel `--bind` padrão `127.0.0.1`
  - exit 77 para auth
  - exit 65 para `TomlDe` / import ruim
  - flags de secrets `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` preferidas ao env
  - `--include-secrets` exige `-o` ou `--i-understand-secrets-on-stdout`
  - `cargo test --locked --test gaps_v042_integration` + `gaps_v051_integration` verde
- Gate 25: DOC-053 / honestidade 0.5.3, em que superfícies de product line documentam:
  - integridade de upload SFTP corrigida (G1)
  - Prefira 0.5.3+
  - Verifique com `sha256` no destino
  - SETSTAT atime+mtime (G3), set_metadata fail-closed (G4), máscara de perms (G12)
  - verbosidade graduada `-v`/`-vv`/`-vvv` com escopo na crate (G2/G14)
  - `RUST_LOG` ignorado
  - cardinalidade de cancel em batch (G5/G17)
  - `ssh-cli exec --json` um único objeto (G8)
  - SCP `sync_data` (G9)
  - E2E E17/E18 SFTP checksum/árvore (G7)
  - sem testes circulares sobre o registro de auditoria (G13/G15)
  - registro de auditoria ignorado pelo git
  - `cargo fmt --check` verde (G10)
  - `cargo test --locked --test gaps_v057_sftp` + `gaps_v058_e2e_residual` verde
  - árvore completa de comandos da CLI documentada em HOW_TO_USE + AGENTS + COOKBOOK (incluindo `ssh-cli locale show`, `ssh-cli locale set` e `ssh-cli locale clear` + `tls mtls/*` / `tls acme/*` completos)
  - G6 `serial_test` / isolamento determinístico da suíte residual
  - G11 baseline verde (`cargo test --locked --all-features --no-fail-fast` + suítes residuais)
  - G16 identificadores SCP em inglês (wire/schema/docs usam nomes EN)
  - G18 `set_permissions` fail-closed no download SFTP
  - G19 constante nomeada `SFTP_PERM_MASK` (`0o7777`, saída/upload) e `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`, entrada/download, A3)
  - descoberta `ssh-cli commands` documentada (superfície de inventário para agentes)
  - docs de product line listam `locale` + árvore `tls` completa (mtls + acme account/issue/complete/status/list)
- Gate 26: DOC-054 / honestidade 0.5.4, em que superfícies de product line documentam:
  - os quatro modos de `ssh-cli tunnel`: forward local padrão, `--reverse` (G-TUN-R01), `--socks5` (G-TUN-R02), `--remote-socket <PATH>` (G-TUN-R03)
  - As três flags são mutuamente exclusivas
  - o rótulo JSON `mode` é um de `local` / `reverse` / `socks5` / `streamlocal`
  - `--reverse` com `REMOTE_PORT 0` deixa o servidor alocar e informar a porta
  - Um forward local não aceita 0
  - `--socks5` e `--remote-socket` omitem `REMOTE_HOST`/`REMOTE_PORT` (passá-los é exit 64)
  - `--remote-socket` exige caminho absoluto
  - um `--bind` roteável exige `--i-accept-network-exposure` (G-TUN-R13), inclusive em forward local e não só em `--reverse`
  - sob `--reverse` o guard cobre o bind do servidor (o posicional `<remote_host>`), que é comparado como texto e não parseado como IP (typo dá exit 64, não exit 2)
  - O próprio `--bind` é aceito e descartado em silêncio
  - evento `tunnel_closed` com `reason`, `forwards_served`, `capacity_waits` é o único discriminador entre os três finais com exit 0
  - redução de payload aplicada antes da serialização: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
  - `--dry-run` é honrado por `ssh-cli vps remove`, `ssh-cli vps import`, `ssh-cli vps edit`, `ssh-cli vps rename`, `ssh-cli sftp rm`, `ssh-cli sftp rmdir`, `ssh-cli secrets init`, `ssh-cli secrets reencrypt` e somente por esses
  - Nos demais é exit 64
  - seletor de frota `--tags` só em `ssh-cli exec` / `ssh-cli sudo-exec` / `ssh-cli su-exec`, mutuamente exclusivo com `--all` e `--hosts`
  - BREAKING: falha parcial multi-host sai com exit 1 (era 65, G-ERR-R02)
  - A1 aplica o teto de log de 512 caracteres do banner, que já existia, em fronteira de caractere (remove um abort disparável remotamente sob `panic = "abort"`)
  - A2 cria chaves ACME/mTLS em `0600`
  - A3 mascara os modos de entrada com `SFTP_PERM_MASK_UNTRUSTED`
  - `cargo test --locked --test docs_conformance` verde (incluindo os gates de superfície 0.5.4, seletor de frota e máscara direcional)
- Gate 27: DOC-056 / honestidade 0.5.6, em que superfícies de product line documentam:
  - `ssh-cli vps rename <NAME> <NEW_NAME>` e `ssh-cli vps edit <NAME> --name <NEW_NAME>` existem, mantêm toda credencial legível sob o nome novo e nunca redigitam segredo
  - o desvio export/edit/import é obsoleto: o export redige segredo e não carregava `use_agent`/`agent_socket`, então host autenticado por agente voltava sem autenticação
  - oito flags de limpeza: `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key`, `--no-key` (caminho da chave mais passphrase), `--no-agent-socket`, cada uma conflitando com a flag de valor do próprio campo
  - `ssh-cli vps edit <NAME> --name <NEW_NAME>` e `ssh-cli vps rename` emitem `vps-renamed` (`name`, `previous_name`, `active_rewritten`, `updated`, `target_resolved`, `target_source`)
  - `vps-edited` é emitido só quando nada foi renomeado e não traz `previous_name`
  - Os dois publicam `target_resolved`/`target_source` só depois da busca
  - `updated` está sempre presente: campos cujo valor guardado mudou, na ordem de aplicação, cada um no máximo uma vez, tirados de 19 nomes de campo
  - Vazio quando todo valor pedido já coincidia ou quando a invocação só renomeou o host
  - Uma invocação sem flag de campo, rename ou limpeza sai 64
  - uma edição que não muda nada sai 0 com `vps-edited` e `updated: []` e não regrava o `config.toml`
  - `ssh-cli vps rename` não é idempotente (repetir depois de sucesso sai 66, e `ssh-cli vps show <NEW_NAME>` confirma)
  - `ssh-cli vps edit` troca o método de auth: `--key` limpa a senha e o agente, `--password` e `--password-stdin` limpam a chave, a passphrase dela e o agente, `--use-agent` limpa a senha e a chave, e `updated` lista o que foi limpo
  - uma `--key` diferente descarta a passphrase antiga, `--agent-socket` sozinho implica `--use-agent`, e passphrase sem chave sai 64
  - duas flags de auth primária na mesma chamada saem 2 no parse
  - nomes de host são estritos: ASCII `[A-Za-z0-9._-]`, começando por alfanumérico, de 1 a 255 bytes, em `ssh-cli vps add`, `ssh-cli vps rename`, `ssh-cli vps edit --name` e `ssh-cli vps import`, enquanto nomes antigos ainda carregam
  - `ssh-cli vps import` lista todos os nomes ruins de uma vez com exit 64 e não grava nada, também sob `--dry-run`
  - buscas que viram alvo (`ssh-cli exec`, `ssh-cli health-check`, `--hosts`, `ssh-cli connect`) recusam caracteres de controle e bidi com exit 64, e `ssh-cli tls mtls import --name` faz o mesmo sem criar diretório
  - o modo texto escapa caracteres de controle e bidi como `\u{1b}` (inclusive nomes remotos de `ssh-cli sftp ls` e `ssh-cli sftp stat`), enquanto o JSON e a saída remota do `ssh-cli exec` seguem crus
  - o diretório de config nasce `0700`, um diretório existente fora do padrão nunca recebe chmod, e o `ssh-cli doctor --json` reporta `local.config_dir_permissions`, `local.config_dir_exposed` e `local.invalid_host_names`
  - `--config-dir something.toml` faz da pasta pai o diretório de config, e `ssh-cli vps export -o new/deep/e.json` cria as pastas que faltam `0700`
  - duas tabelas de config que normalizam para o mesmo nome saem 65 com `config has two hosts named ...`
  - os planos de `--dry-run` de `ssh-cli vps edit`, `ssh-cli vps rename` e `ssh-cli vps remove` ecoam `target_resolved`, `target_source`, `host_resolved` e `host_source`
  - os nove `scripts/*.sh` agora são `scripts/*.py` (Python 3.9+, só stdlib, Windows `py -3 scripts/<name>.py`) com flags no lugar de variáveis de ambiente
  - `SshCliError::Unavailable` carrega `UnavailableService`, `paths::validate_and_normalize` devolve `String`, e `Money`, `Brl`, `Usd`, `Currency` e `rust_decimal` saíram (QUEBRA só para quem usa a biblioteca, e o texto no formato de rede não muda)
  - `--dry-run` cobre as oito operações, e `--lock-wait-ms` limita o lock do config em exit 69 `unavailable` em vez de pendurar indefinidamente
  - `ssh-cli commands --json` publica todo argumento com `id`, `long`, `short`, `value_name`, `required`, `repeatable`, `takes_value` e `hidden`
  - o `ssh-cli --help` tem uma seção `Exit status:` (0, 1, 2, 64, 65, 66, 69, 70, 73, 74, 77, 130, 141 e 143), e as 48 folhas de comando mostram `Examples:` no `--help` enquanto o `-h` continua curto
  - as duas skills ensinam as 48 folhas de comando, `ssh-cli vps rename`, a troca de auth, o passo SANITIZE, a regra do `sudo` cru, a regra do TOFU e a forma persistente do `--disable-sudo`, dentro do orçamento de 7000 palavras
  - a versão mínima suportada de Rust (MSRV) é 1.89 porque o `russh` 0.65 a declara, e toda máquina de build precisa de compilador C para o `aws-lc-sys`
  - o `secrets init --keyring` grava no cofre do SO pela `keyring` 4.2.0 com a feature `v1`, sai 69 quando nenhum cofre responde, é escopado por diretório de config, e sai 64 em vez de cunhar uma segunda chave primária num perfil que já tem uma
  - o `--use-keyring` global chega ao `secrets init`, e a chave primária resolve `--secrets-key-file`, depois o keyring com `--use-keyring`, depois o `secrets.key` do XDG, depois a entrada do keyring de um `secrets init --keyring` anterior
  - `--select` e `--truncate-content` reduzem um registro único, as flags de coleção `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only` e `--max-output-bytes` são nomeadas em `agent_shape.ignored` ali, e um array nu reduzido escreve uma linha `agent-shape` em stderr que o `--quiet` suprime
  - os itens de `ssh-cli vps list --select` satisfazem o `vps-list.schema.json`, o `ssh-cli vps list` em texto dimensiona cada coluna pela célula mais larga e mostra `(not set)` na senha de host só com chave, e o `ssh-cli vps show` em texto alinha todos os rótulos
  - `ssh-cli vps import --file` com arquivo ausente sai 66 `file_not_found` como erro permanente, e um `ssh-cli tunnel` parado por SIGINT ou SIGTERM sai 130 ou 143 com `reason: "signal"`
  - só `SSH_CLI_SECRETS_KEY` e `SSH_CLI_SECRETS_KEY_FILE` são recusadas, e o `--disable-sudo` global posto antes de `ssh-cli vps add` ou `ssh-cli vps edit` grava `disable_sudo` no registro
  - `cargo test --locked --test gaps_v071_vps_edit_rename` e `cargo test --locked --test docs_conformance` verdes


## Como verificar os resíduos rápido

```bash
cargo test --locked --test gaps_v039_integration
cargo test --locked --test gaps_v040_integration
cargo test --locked --test gaps_v041_integration
cargo test --locked --test gaps_v042_integration
cargo test --locked --test gaps_v051_integration
cargo test --locked --test gaps_v057_sftp
cargo test --locked --test gaps_v058_e2e_residual
cargo test --locked --test gaps_v065_exec_target_designation
cargo test --locked --test gaps_v066_transfer_target_designation
cargo test --locked --test gaps_v067_packaging_reach
cargo test --locked --test gaps_v068_source_language
cargo test --locked --test gaps_v069_markdown_tables
cargo test --locked --test gaps_v070_doc_reference_reach
cargo test --locked --test gaps_v071_vps_edit_rename
cargo test --locked --test gaps_v072_edit_truthfulness
cargo test --locked --test gaps_v072_import_roundtrip
cargo test --locked --test gaps_v072_lock_contention
cargo test --locked --test gaps_v072_schema_conformance
cargo test --locked --test gaps_v072_storage_hardening
cargo test --locked --test gaps_v073_vps_auth_switch
cargo test --locked --test gaps_v074_flag_reach
cargo test --locked --test gaps_v075_markdown_format_gate
cargo test --locked --test gaps_v073_host_name_charset
cargo test --locked --test gaps_v073_terminal_escape
cargo test --locked --test gaps_v073_config_dir_mode
cargo test --locked --test docs_conformance
cargo fmt --check
cargo deny check
python3 scripts/check_en_identifiers.py
python3 scripts/check_markdown_format.py
python3 scripts/verify_install_resolve.py
ssh-cli --version
rg -n -F '/gaps.md' .gitignore   # must find the rule (exit 0)
```

- LOG-001: tunnel com `--output-format json` falha sem conectar
- Stderr tem envelope JSON e sem prosa INFO
- JSON-001: show JSON de host só-chave contém `"password": null`
- Arquivo de schema contém null no tipo de password
- CLI-004: `ssh-cli health-check --timeout 50` não é "unexpected argument"
- DOC-003: arquivos de product line (incluindo este par de checklists) contêm `0.5.6`
- DOC-004: README/INTEGRATIONS/AGENTS/HOW_TO_USE/COOKBOOK/MIGRATION mencionam SCP somente de arquivos e aviso de wire 0.3.9
- Schema scp-transfer presente
- DOC-004d: `skills/ssh-cli-en` e `skills/ssh-cli-pt` ensinam scp-transfer, tunnel_listening, somente arquivos, partial, stream 32 KiB e matriz de timeout
- Evals cobrem a superfície
- DOC-041: export redigido de segredos vazios sem `sshcli-enc:`
- Prazo pós-bind do tunnel exit 0 após `tunnel_listening`
- Paridade de flags de auth tunnel/health documentada
- Schema scp-transfer exige `event`
- Gaps_v041 verde
- DOC-051: redaction do export
- Formato segue --output-format
- Schema v3
- Schemas secrets
- `--bind` loopback
- Exit 77
- Flags secrets
- Include-secrets
- Gaps_v042 + gaps_v051 verdes
- DOC-053 / 0.5.3: integridade SFTP G1
- Verbosidade G2/G14
- E17/E18
- Registro de auditoria local ignorado pelo git
- `cargo fmt --check`
- Gaps_v057 verde
- Árvore CLI completa (locale + tls mtls/acme) em HOW_TO_USE/AGENTS/COOKBOOK
- G6 serial_test
- G11 baseline
- G16 ids SCP EN
- G18 set_permissions
- G19 `SFTP_PERM_MASK`
- Descoberta `ssh-cli commands`
- DENY-002: `deny.toml` tem `yanked = "deny"`, `ignore = []`, política multiple-versions documentada
- CHG-001 / REL: seção `## [0.5.6]` no CHANGELOG + tag local `v0.5.6` sem push não autorizado
- TEST-004 / SCP: gaps_v040 cobre wire, schema, path partial, preserve, script e2e E10–E14
- AUD-POST / gaps_v041: suíte residual EXP-001, TUN-002, CLI-005/006, IO-009, REL-006 verde
- 0.5.2 / gaps_v051: roundtrip de redaction do export, JSON `vps-export`, dual-read, evento secrets-init, guarda include-secrets, CRUD `vps-added`, empty command, import exit 65
- G-E2E / gaps_v058: root `ssh-cli schema` / `ssh-cli doctor`, um único `vps-added` + `secrets_key_auto_created`, `--use-agent`, `RUST_LOG` ambiente ignorado, ACME exit 64
- 0.5.6 / `gaps_v071_vps_edit_rename`, `gaps_v072_edit_truthfulness`, `gaps_v072_import_roundtrip`, `gaps_v072_lock_contention`, `gaps_v072_schema_conformance`, `gaps_v072_storage_hardening`, `gaps_v073_config_dir_mode`, `gaps_v073_host_name_charset`, `gaps_v073_terminal_escape` e `gaps_v073_vps_auth_switch`: rename, troca de auth, nomes estritos, escape de terminal, modo do diretório de config, conformidade de schema
- G10: `cargo fmt --check` verde
- Identificadores EN: `scripts/check_en_identifiers.py` exit 0
- Formatação Markdown: `scripts/check_markdown_format.py` exit 0, fechando o que os Gaps 25 e 37 nomeavam como critério de aceitação


## Inventário de suítes
- O Gate 6 roda a lib e estes 50 arquivos de `tests/`, um nome por linha para que nenhuma faixa fique elidida

```text
docs_conformance
e2e_cli
gaps_v035_integration
gaps_v037_integration
gaps_v038_integration
gaps_v039_integration
gaps_v040_integration
gaps_v041_integration
gaps_v042_integration
gaps_v051_integration
gaps_v053_domain_types
gaps_v054_error_handling
gaps_v055_unsafe_ffi
gaps_v057_sftp
gaps_v058_e2e_residual
gaps_v059_agent_native
gaps_v060_tunnel_modes
gaps_v061_error_taxonomy
gaps_v061_scp_durability
gaps_v062_component_budget
gaps_v062_i18n_reachability
gaps_v063_e2e_harness
gaps_v063_locale_features
gaps_v063_secret_stdin
gaps_v064_gate_runner
gaps_v065_exec_target_designation
gaps_v066_transfer_target_designation
gaps_v067_packaging_reach
gaps_v068_source_language
gaps_v069_markdown_tables
gaps_v070_doc_reference_reach
gaps_v071_vps_edit_rename
gaps_v072_edit_truthfulness
gaps_v072_import_roundtrip
gaps_v072_lock_contention
gaps_v072_schema_conformance
gaps_v072_storage_hardening
gaps_v073_config_dir_mode
gaps_v073_host_name_charset
gaps_v073_terminal_escape
gaps_v073_vps_auth_switch
gaps_v074_flag_reach
gaps_v075_markdown_format_gate
i18n_integration
proptest_tests
scp_integration
snapshot_tests
storage_integration
test_quality
tunnel_integration
```


## Política
- PROIBIDO: declarar inventário Fechado enquanto qualquer gap permanecer Aberto (notas locais do mantenedor)
- PROIBIDO: waiver eterno de RUSTSEC / CVE sem tracking fechado na mesma release
- PROIBIDO: `git push` ou publicação no crates.io sem autorização explícita do mantenedor
- PROIBIDO: logar ou colar segredos reais em notas do inventário, checklist ou logs de gates
- PROIBIDO: publicar o registro de auditoria ou afirmar seu texto FIXED em testes de produto (G13/G15)
- OBRIGATÓRIO: escritas multi-linha de inventário local / CHANGELOG usam atomwrite (ou escrita atômica equivalente)
- OBRIGATÓRIO: Status Resolvido só com código + teste + nota de versão (evidência de destino, não texto de inventário)


## G-22: Distribuição, SBOM, multi-arch (processo)
- A identidade do produto permanece CLI one-shot (crates.io + tag local)
- Multi-arch binário e SBOM assinado são gates de processo de release, não features de runtime
- Os scripts ficam em `scripts/` e nunca fazem push ou publish

### 28. Binários multi-arch de release (G-22)
- Config: [`Cross.toml`](../Cross.toml) na raiz, com os alvos:
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`
  - `aarch64-unknown-linux-gnu`
- Ferramentas: instale [`cross`](https://github.com/cross-rs/cross) e o motor de contêiner que ele exige
- Matriz de build (local, sem push):

```bash
python3 scripts/dist_multiarch.py
# or a single target:
python3 scripts/dist_multiarch.py --targets x86_64-unknown-linux-musl
```

- Artefatos: `target/dist/ssh-cli-<triple>` + sidecars `.sha256`
- `--targets` aceita lista separada por espaço ou vírgula, `--jobs` limita os builds em paralelo (padrão 2) e `--timeout-secs` limita cada alvo (padrão 3600, `0` para sem limite)
- O stdout traz uma linha por alvo separada por tabs com o alvo, `ok` ou `fail` e o caminho do artefato, e o exit é 0 quando todo alvo compilou, 1 caso contrário e 2 em erro de uso
- `cross` + `dist_multiarch.py` é o caminho suportado
- Sem `cross` o script tenta `cargo build --release --locked --target <triple>`, que exige linker e sysroot locais
- PROIBIDO: anexar builds debug unstripped como “release” sem documentar

### 29. Geração de SBOM (G-22)

```bash
# Preferred: CycloneDX JSON
cargo install cargo-cyclonedx
python3 scripts/generate_sbom.py
# → target/sbom/ssh-cli.cdx.json (or path argument)
```

- Alternativa sem cargo-cyclonedx: o script grava o inventário `cargo tree` num arquivo `.tree.txt` com o nome do caminho de saída e avisa que não é um SBOM CycloneDX
- Assinatura (mantenedor, offline):

```bash
# Example with a local cosign key or with gpg: choose one standard
# cosign sign-blob --key <path-to-cosign.key> --bundle target/sbom/ssh-cli.cdx.json.bundle target/sbom/ssh-cli.cdx.json
# gpg --detach-sign --armor target/sbom/ssh-cli.cdx.json
```

- Anexar SBOM + assinatura + binários multi-arch à página de release da tag `vX.Y.Z` somente após autorização explícita do mantenedor (mesma regra de `git push`)
- A publicação no crates.io permanece separada (`cargo publish`)
- SBOM é evidência de release, não arquivo do crate

### 30. Critérios de aceite G-22 (fechar inventário)
- [ ] `python3 scripts/dist_multiarch.py` produz pelo menos o artefato musl x86_64 + sha256 (ou skip documentado quando não há motor de contêiner)
- [ ] `python3 scripts/generate_sbom.py` produz CycloneDX JSON ou o inventário fallback documentado
- [ ] `python3 scripts/release_attest.py` imprime o checklist de atestado e sai 0, e `--require-cosign` sai 1 quando o `cosign` não está no PATH
- [ ] Release notes (quando autorizado) listam triples de binário + caminho do SBOM
- [ ] Sem auto-push / sem auto-publish a partir dos scripts


## Referência
- Registro de auditoria: inventário somente local do mantenedor (ignorado pelo git / excluído do cargo, não publicado)
- [../Cross.toml](../Cross.toml): imagens de alvo cross-rs
- [../scripts/dist_multiarch.py](../scripts/dist_multiarch.py): build multi-arch
- [../scripts/generate_sbom.py](../scripts/generate_sbom.py): SBOM / inventário
- [../deny.toml](../deny.toml): política de supply-chain
- [../scripts/check_all_gates.py](../scripts/check_all_gates.py): runner da bateria obrigatória de gates (os catorze gates, uma invocação)
- [../scripts/check_cross_targets.py](../scripts/check_cross_targets.py): gate de type-check de Windows e macOS
- [../scripts/check_advisory_freshness.py](../scripts/check_advisory_freshness.py): gate de frescor do banco de advisories
- [../scripts/verify_install_resolve.py](../scripts/verify_install_resolve.py): gate de re-resolve de install
- [../scripts/check_en_identifiers.py](../scripts/check_en_identifiers.py): gate residual de identificadores em inglês
- [../scripts/check_markdown_format.py](../scripts/check_markdown_format.py): gate de formatação do Markdown publicado
- [../tests/gaps_v039_integration.rs](../tests/gaps_v039_integration.rs): gates residuais LOG/JSON/CLI/DOC/DENY/CHG
- [../tests/gaps_v040_integration.rs](../tests/gaps_v040_integration.rs): gates residuais SCP/IO/DOC-004/REL-004
- [../tests/gaps_v041_integration.rs](../tests/gaps_v041_integration.rs): gates residuais EXP-001/TUN-002/CLI-005/006/IO-009/REL-006 (DOC-041)
- [../tests/gaps_v042_integration.rs](../tests/gaps_v042_integration.rs): gates residuais AUD-E2E (TUN-003, IO-010, ENV-001, SCP-024, …)
- [../tests/gaps_v051_integration.rs](../tests/gaps_v051_integration.rs): gates residuais 0.5.2 export/schema v3/secrets
- [../tests/gaps_v057_sftp.rs](../tests/gaps_v057_sftp.rs): gates residuais G-SFTP
- [../tests/gaps_v058_e2e_residual.rs](../tests/gaps_v058_e2e_residual.rs): gates residuais G-E2E (schema/doctor root, um único `vps-added`, `--use-agent`, ACME 64)
- [schemas/vps-show.schema.json](schemas/vps-show.schema.json): password `null` ou a string mascarada
- [schemas/scp-transfer.schema.json](schemas/scp-transfer.schema.json): JSON de sucesso SCP (só arquivos, exige `event`)
- [schemas/tunnel-listening.schema.json](schemas/tunnel-listening.schema.json): evento de bind do tunnel
- [schemas/vps-export.schema.json](schemas/vps-export.schema.json): só `ssh-cli vps export --json` (`event: "vps-export"`)
- [schemas/secrets-init.schema.json](schemas/secrets-init.schema.json): `ssh-cli secrets init --json`
- [schemas/secrets-reencrypt.schema.json](schemas/secrets-reencrypt.schema.json): `ssh-cli secrets reencrypt --json`
- [schemas/README.md](schemas/README.md): índice de schemas (linha de produto 0.5.6)


## Gate residual de docs G-E2E (v0.5.3)
- [ ] `RUST_LOG` ambiente ignorado documentado
- Só `-v`/`-vv`/`-vvv` controla verbosidade (G2/G14)
- [ ] Um único JSON `vps-added` com campo `secrets_key_auto_created` (família G8)
- [ ] Root `ssh-cli schema` / `ssh-cli doctor` documentados
- [ ] `ssh-cli vps add --use-agent` documentado
- [ ] ACME `invalidContact` → exit 64 permanente documentado
- [ ] Export redacted limpa os segredos (`password` é string vazia, os outros são `null`) documentado
- [ ] E2E XDG-first + SKIP offline documentado em TESTING
- [ ] E2E E17/E18 SFTP checksum/árvore documentado (G7)
- [ ] Integridade SFTP G1 + preferir 0.5.3+ documentado
- [ ] Árvore completa de comandos da CLI (incluindo `locale` + `tls mtls` / `tls acme` completos) documentada em HOW_TO_USE + AGENTS + COOKBOOK
- [ ] Descoberta `ssh-cli commands` documentada
- [ ] G6 `serial_test` / suíte determinística
- [ ] G11 baseline verde
- [ ] G16 identificadores SCP em inglês
- [ ] G18 `set_permissions` fail-closed no download SFTP
- [ ] G19 constante nomeada `SFTP_PERM_MASK`
- [ ] Docs de product line listam `locale` + árvore `tls` completa
- [ ] Registro de auditoria ignorado pelo git (`rg -n -F '/gaps.md' .gitignore` acha a regra)
- [ ] Sem testes circulares de texto FIXED (G13/G15)
- [ ] `cargo fmt --check` nos gates de publish (G10)
- [ ] Suíte `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs` nos gates de publish


## Matriz multi-OS local (G-E2E-18)
- Código de produto: `src/platform/{linux,macos,windows}.rs`, validado por scripts locais
- Multi-arch local: `scripts/dist_multiarch.py` quando toolchains cross estiverem instaladas
- Valide notas de path length / agent socket em macOS e Windows antes de taggear um release
- Todo gate é um script local rodado à mão, sem nenhum serviço de nuvem no caminho


## Notas de release
### 0.5.6 — release de edição e rename
- `ssh-cli vps rename <NAME> <NEW_NAME>` e `ssh-cli vps edit <NAME> --name <NEW>` mudam o nome do host nativamente, com as credenciais intactas
- O registro sela cada segredo com contexto não vinculado, então o dado associado não carrega nome de host e o segredo continua legível sob o nome novo
- Cada gravação re-sela com nonce novo, o que é normal e nunca toca o plaintext
- Isso substitui o desvio export/edit/import, que descartava auth de agente em silêncio porque o export não carregava campo de agente
- Oito flags de limpeza removem o campo em vez de sobrescrevê-lo
- `--dry-run` cobre os dois verbos
- O lock do config espera `--lock-wait-ms` limitado e depois sai 69 em vez de pendurar
- `ssh-cli commands --json` publica todo argumento

### 0.5.5 — release de segurança de designação de alvo
- Corrige GAP-SSH-EXEC-ARGC-001: `ssh-cli exec`/`ssh-cli sudo-exec`/`ssh-cli su-exec` decidiam o host alvo pela contagem de posicionais
- Assim `ssh-cli exec <HOST> --step <CMD>` executava o nome do host como binário remoto no host que `ssh-cli connect` selecionara por último
- BREAKING: um único posicional agora é erro de uso
- Alcance o marcador ativo com a nova `--use-active <COMMAND>`
- O envelope single-host ganhou `target_resolved` e `target_source`, então vincule consumidores novos a esses
- `host_resolved` e `host_source` são os aliases de leitura da 0.5.5
- O envelope também ganhou `active_fallback`, e o `ssh-cli doctor` passa a reportar `local.active_vps`

### 0.5.4 — release de segurança e agent-native
- Corrige DoS remoto pré-auth no banner SSH (A1)
- Impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3)
- Fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2)
- Adiciona flags de redução de payload aplicadas antes da serialização: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- BREAKING: falha parcial multi-host agora sai com exit 1 (era 65)
- BREAKING: `--bind` fora do loopback exige `--i-accept-network-exposure`
- Novo evento `tunnel_closed`
