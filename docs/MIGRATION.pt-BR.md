# Guia de migração

> **0.5.4** — release de segurança e agent-native. Corrige DoS remoto pré-auth no banner SSH (A1), impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3), fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2) e adiciona flags de redução de payload (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) aplicadas antes da serialização. BREAKING: falha parcial multi-host agora sai com exit **1** (era 65); `--bind` fora do loopback exige `--i-accept-network-exposure`. Novo evento `tunnel_closed`.


> Passe de ssh-cli 0.3.3 (ou posterior) para 0.5.3 sem perder o inventário multi-host.

- Leia este documento em [inglês](MIGRATION.md).


## O que muda

### Desde 0.5.4
- **BREAKING — falha parcial multi-host agora sai com exit `1` (era `65`).** Todo wrapper que ramificava em `65` para significar "alguns hosts falharam" precisa ser atualizado. `65` é `EX_DATAERR`, que afirma que a *entrada* estava malformada; um lote em que três de dez hosts ficaram inalcançáveis tinha entrada perfeitamente válida, então o código antigo mentia sobre a causa (G-ERR-R02). Verifique os resultados por host no envelope do lote, não apenas o exit do processo.
- **BREAKING — `tunnel --bind` fora do loopback agora exige `--i-accept-network-exposure`.** Todo script que passa `--bind 0.0.0.0` ou endereço de LAN falha até o reconhecimento ser acrescentado. Antes disso, a flag publicava em silêncio o serviço remoto encaminhado para toda a rede local (G-TUN-R13). Sob `--reverse` a mesma flag protege o endereço de bind do **servidor**, porque é essa a ponta exposta naquele sentido.
- **`tunnel` ganhou três modos.** O forward local padrão não mudou, então invocações existentes continuam funcionando.
  - `--reverse` (**G-TUN-R01**) pede ao servidor que escute e entregue conexões de volta à sua porta local — o caso do webhook de callback e do bastion invertido, que antes forçava o operador a sair desta CLI e usar o `ssh` do sistema. `REMOTE_PORT` pode ser `0`, deixando o servidor alocar e informar a porta.
  - `--socks5` (**G-TUN-R02**) serve um proxy SOCKS5 local (RFC 1928 no-auth + CONNECT) escolhendo o destino por conexão, então `REMOTE_HOST`/`REMOTE_PORT` são omitidos. Substitua N processos `tunnel` apontados a N hosts atrás de um bastion por uma invocação: o handshake é pago uma vez.
  - `--remote-socket <CAMINHO>` (**G-TUN-R03**) encaminha para socket Unix remoto via `direct-streamlocal@openssh.com`, alcançando sockets de Docker, PostgreSQL e systemd que nunca escutam em TCP. O caminho deve ser absoluto ou sai com exit **64**.
- **Novo evento `tunnel_closed`.** `tunnel --json` agora o emite no encerramento com `reason`, `forwards_served` e `capacity_waits`. Acrescente-o a qualquer parser que tratava `tunnel_listening` como o único evento de tunnel. É a única forma de distinguir deadline limpo de semáforo saturado, já que todos os finais saem com exit 0.
- **Oito flags de redução de payload agora são globais:** `--select` (apelido `--fields`), `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`. Elas agem *antes* da serialização, então trocar um post-filtro JSON externo por essas flags impede que o envelope gigante seja escrito. Nada muda quando elas são omitidas.
- **`--no-input` e `--dry-run` são globais.** `--dry-run` é honrada somente por `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init` e `secrets reencrypt`; em qualquer outro lugar é recusada com exit **64** em vez de aceita e ignorada.
- **Correções de segurança — atualize só por elas:** **A1** trunca o banner SSH pré-auth enviado pelo servidor por fronteira de **caractere**, e não por índice de byte — o teto de 512 caracteres no log já existia (G-SSH-14); o que estava quebrado é que cortar por índice de byte entra em pânico sempre que o corte cai no meio de um caractere multibyte, e o perfil de release usa `panic = "abort"`, então não há unwind: o processo morre e derruba um fan-out multi-host inteiro, **A2** cria as chaves privadas ACME/mTLS em `0600` em vez de restringir o modo depois de o arquivo já existir, **A3** mascara os modos enviados pelo servidor para triplas `rwx` simples, então setuid, setgid e sticky não pegam carona num download para o arquivo local. Ver `SECURITY.pt-BR.md` para o mecanismo de cada uma.
- `ssh-cli schema dry-run` e `ssh-cli schema tunnel-closed` agora emitem seus documentos com exit **0** (antes exit 64), então a descoberta de schema cobre o catálogo inteiro.

### Desde 0.5.3
- **G1** upload SFTP não trunca mais o destino a zero bytes — prefira 0.5.3+ para todo SFTP; verifique com `sha256sum` no destino.
- **G2 / G14** verbosidade graduada (`-v` info / `-vv` debug / `-vvv` trace) e sempre com escopo na crate (`warn,ssh_cli=…`); debug global nu removido (sem vazamento de senha via russh). `RUST_LOG` ambiente continua ignorado.
- **G3** SETSTAT SFTP envia `atime`+`mtime` juntos (sem atime no epoch).
- **G4** Result de `set_metadata` SFTP é fail-closed (SETSTAT mutante não é best-effort).
- **G5 / G17** cancelamento multi-arquivo SCP/SFTP preenche o resto cancelled; `results.len() == input.len()`.
- **G6** testes de signal/cancel que tocam estado global (`CANCEL_FLAG`) usam isolamento `serial_test` para a suite ser determinística (não é runtime de agente).
- **G7** E2E real oficial cobre matriz de checksum SFTP + árvore recursiva (**E17/E18**); matriz completa **E01–E18**.
- **G8** `exec --json` de passo único emite exatamente um objeto JSON (sem dual-events no caminho de sucesso).
- **G9** download SCP propaga falha de `sync_data` antes do rename atômico.
- **G10** gate de release inclui `cargo fmt --check`.
- **G11** suite baseline fica verde sem loteria de re-execução; re-rodar até passar não é estratégia de gate.
- **G12 / G19** bits de permissão mascarados com `SFTP_PERM_MASK` nomeado (`0o7777`) no caminho de **saída** (upload). Desde a 0.5.4 o caminho de **entrada** (download) mascara com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), porque esse modo vem do servidor — veja A3 acima.
- **G13 / G15** sem testes circulares que assertam texto FIXED em `gaps.md` local; aceite exige prova de efeito no destino (checksum). `gaps.md` é local do mantenedor (gitignored / excluído do cargo) — não é contrato publicado.
- **G16** identificadores em inglês e erros de canal no caminho do cliente SCP (`client_real_scp.rs`).
- **G18** falhas de `set_permissions` local no download SFTP são sinalizadas.

### Desde 0.3.4 (paridade de automação SSH central)
- Grafo de crypto de install fixado para `cargo install --locked` funcionar (GAP-014).
- Auth aceita chaves privadas via `--key` / `key_path` (GAP-002).
- Semântica de `max_chars` dividida em `max_command_chars` e `max_output_chars` (GAP-004).
- `sudo-exec` empacota comandos com `sh -c` seguro (GAP-005).
- `su-exec` consome senha `su` armazenada (GAP-003).
- Escrita de config atômica com flock e mode 0600 (GAP-007).
- Host keys usam known_hosts TOFU (GAP-008).
- `tunnel` exige `--timeout-ms` (GAP-010).
- Schema version de registros novos era 2 na época (histórico; wire atual é schema v3).
- Licença dual MIT OR Apache-2.0.

### Desde 0.3.5
- `vps export` atômico, abort remoto mais forte (TERM+KILL).
- Caminho AEAD opcional maduro; doctor reporta `secrets_at_rest`.
- JSON automático quando stdout não é TTY.

### Desde 0.3.6
- Cifragem at-rest padrão de segredos em `config.toml` (ChaCha20-Poly1305).
- Auto-cria XDG `secrets.key` (0o600) na primeira gravação de segredo.
- CLI `secrets status|init|reencrypt` (nunca imprime a primary-key).
- Opt-out só para testes: `--allow-plaintext-secrets` (só CLI; sem store em env).
- Doctor: `secrets_key_file`, `secrets_plaintext_opt_out`.

### Desde 0.3.7
- Polimento de I/O para agentes: `--output-format` global em VPS CRUD, `health-check --json`, envelope de erro JSON, `--quiet` silencia sucesso humano.
- Tunnel `--timeout-ms` cobre connect SSH + loop.
- SCP valida arquivo local antes do connect; `vps remove` limpa `active` órfão.
- `su-exec --password-stdin`; conflitos clap para password/*_stdin.
- Exit remoto não-zero mapeia para exit de processo `1` com `remote_exit_code` no envelope JSON.
- Segredos longos sempre mascaram como `***` (sem vazamento de prefixo 12+4).
- Senha sudo/su no stdin do canal, não em argv remoto.

### Desde 0.3.8
- russh atualizado para 0.62.2 (piso de segurança ≥0.60.3).
- Banners humanos de tunnel fora do stdout do agente (JSON/non-TTY/quiet).
- Sem VPS ativa retorna sysexits 66 (`EX_NOINPUT`) via erro tipado.
- `cargo deny`: `yanked=deny`, ignore list vazia; `multiple-versions=warn` para duplicatas transitivas.
- String de versão reporta `-dirty` quando a working tree está suja.
- Suite residual completa `tests/gaps_v038_integration.rs`.

### Desde 0.4.1 (histórico)
- Patch AUD-POST: secrets vazios nunca viram blob `sshcli-enc` no export redacted (EXP-001); deadline do tunnel pós-bind sai 0 (TUN-002); paridade de flags auth em `tunnel`/`health-check` (CLI-005/006); JSON SCP com `event: "scp-transfer"` (IO-009). Só aditivo — sem breaking.
- Correção wire SCP (0.4.0): crates.io 0.3.9 SCP quebrado. Atualize para 0.4.0+ (prefira a linha de produto 0.5.4) antes de depender de `scp`.
- SCP é somente arquivos regulares (sem `-r`). Árvores usam `sftp --recursive`. Use `--timeout` para arquivos grandes (cobre connect + transfer). JSON de sucesso via `--json` / `--output-format json` (`docs/schemas/scp-transfer.schema.json`; SFTP: `sftp-transfer.schema.json`).
- Download SCP grava `{path}.ssh-cli.partial` e faz rename atômico; mode/times aplicados no partial antes do rename.
- Upload SCP faz stream em blocos de 32 KiB (sem `fs::read` do arquivo inteiro na RAM).
- Preserve mtime/mode bidirecional (remoto `scp -tp` / `-fp`; parse de `T` + mode `C`).
- Paridade de flags SCP com exec: `--timeout`, `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`, `--json`.
- Falhas de `scp --json` emitem envelope de erro JSON em stderr (`exit_code`, `message`) — paridade com tunnel (IO-007b).
- `tunnel --json` emite um objeto stdout `event: "tunnel_listening"` após o bind local (`docs/schemas/tunnel-listening.schema.json`); ainda exige `--timeout-ms`.
- Tracing default error (não info); `-v` ativa debug; `RUST_LOG` ambiente é ignorado — stderr JSON/tunnel limpo por omissão.
- Senha vazia ou ausente em VPS só-chave serializa como JSON `null` (não `"***"`); não vazia ainda mascara como `***`; texto humano em show usa "(não definida)" para vazio.
- `health-check` aceita override `--timeout <ms>` (alinhado ao exec).
- Docs de product line daquela era alinhados a 0.4.1; suites `tests/gaps_v039_integration.rs` + `tests/gaps_v040_integration.rs` + `tests/gaps_v041_integration.rs`; e2e oficial E01–E14 (E10–E14 cobrem SCP).

### Desde 0.4.2 (histórico, aditivo)
- Porta local efêmera de tunnel 0: após o bind, JSON/banner reportam a porta atribuída pelo SO (nunca 0 pós-bind) (TUN-003).
- Envelope formal de `vps export --json` (`event: "vps-export"`) amadurecido; secrets vazios permanecem `""` no export redacted.
- e2e oficial E15 (tunnel porta 0) + E16 (symlink); suite `tests/gaps_v042_integration.rs`.


## Migração passo a passo
### Atualize o binário

```bash
cargo install ssh-cli --locked --force
ssh-cli --version
```

### Valide inventário e modo de segredos

```bash
ssh-cli secrets status --json
ssh-cli vps doctor --json
ssh-cli vps list --json
```

### Se ainda houver segredos plaintext em disco
- No primeiro save com 0.3.6+, um `secrets.key` é auto-criado e novas gravações cifram.
- Para re-cifrar inventário plaintext existente:

```bash
ssh-cli secrets init   # se secrets.key ainda não existir
ssh-cli secrets reencrypt
```

- Faça backup offline de `config.toml` e `secrets.key`; perder a chave torna blobs cifrados ilegíveis.

### Adicione chaves a hosts só-chave

```bash
ssh-cli vps edit prod --key ~/.ssh/id_ed25519
```

### Revalide segredos de elevação (prefira stdin)

```bash
printf '%s' '...' | ssh-cli vps edit prod --sudo-password-stdin
ssh-cli sudo-exec prod "id"
ssh-cli su-exec prod "id"
```

### Atualize wrappers de agentes
- Passe `--timeout-ms` em tunnels.
- Em `tunnel --json`, aguarde `event == "tunnel_listening"` antes de usar a porta local.
- TUN-002: após `tunnel_listening`, o deadline one-shot pós-bind sai com exit 0 (não trate 74 como falha se o bind já foi sinalizado). Timeout pré-bind permanece 74.
- EXP-001: em `vps export` redacted, não espere nem parseie `sshcli-enc:` para secrets vazios — vazios serializam como `""`.
- IO-009: parseie sucesso SCP com `docs/schemas/scp-transfer.schema.json` incluindo `event: "scp-transfer"` obrigatório.
- CLI-005: `tunnel` aceita `--password-stdin`, `--key-passphrase` / `--key-passphrase-stdin` (além de `--key`).
- CLI-006: `health-check` aceita `--password-stdin`, `--key`, `--key-passphrase` / `--key-passphrase-stdin`.
- Em falha de `scp`/`tunnel` com `--json`, parseie o envelope de erro em stderr (não prosa humana).
- Trate SCP como somente arquivos regulares; não envie árvores de diretório.
- Prefira 0.5.3+ para SFTP; re-verifique uploads com checksum no destino após o upgrade (G1).
- Re-teste transferências após sair do 0.3.9 (SCP daquela release não era confiável).
- Se veio de 0.4.0: export redacted podia mostrar ciphertext falso de senha vazia; tunnel podia emitir `ok:true` e sair 74 — atualize wrappers e o binário para 0.5.3.
- Trate `--maxChars` como limite de entrada, não de saída.
- Prefira `--password-stdin` para segredos; senha em argv avisa em stderr (0.5.2+).
- Valores de timeout abaixo de 1000 ms avisam em stderr (unidade é milissegundos, não segundos).
- Comando remoto vazio falha com mensagem técnica `empty command` (qualquer locale).
- Trate erros de mismatch de host-key antes de forçar replace.
- Espere valores cifrados em `config.toml` com prefixo `sshcli-enc:v1:` (exceto export redacted de secret vazio).
- Espere tracing default error; use `-v`/`-vv`/`-vvv` ao diagnosticar (`RUST_LOG` ambiente é ignorado); não parseie stderr como JSON de sucesso.
- Parseie `exec --json` de host único como um objeto (G8).
- ACME `invalidContact` / validação permanente → exit **64** (não faça retry como 74) (G-E2E-01).
- Primeiro `vps add` com auto-key: **um** documento JSON `event: "vps-added"` com campo `secrets_key_auto_created` (G-E2E-04).
- Prefira root `ssh-cli schema` / `ssh-cli doctor` para descoberta de agente (G-E2E-02/03).
- Cadastre hosts só-agent com `vps add --use-agent` / `--agent-socket` (G-E2E-19).
- Export redacted: secrets não vazios → `***` (`FIXED_MASK`); secrets vazios permanecem `""` (G-E2E-10).
- Feature clap `env` removida — sem `#[arg(env=…)]` de config de produto (G-E2E-08).
- Stamp de versão anexa `-dirty` quando a working tree está suja mesmo com `.commit_hash` (G-E2E-06).
- Trate senha vazia em list/show JSON como `null` em hosts só-chave.
- Pode passar `health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais.
- Espere exit de processo `1` (com `remote_exit_code` no envelope JSON) quando o comando remoto falhar.
- Espere sem VPS ativa como exit 66; arquivo SCP ausente como exit 66 com `file not found: <path>`.
- Espere banners de tunnel só em caminhos humanos/TTY, não no stdout JSON do agente.
- Controle de secrets é só CLI/XDG (`--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, XDG `secrets.key`); stores env de secrets são rejeitados fail-closed.
- Assuma que JSON auto non-TTY se aplica a `vps export` — o corpo é JSON em qualquer stdout non-TTY, e TOML exige `--output-format text`.


## Mudanças de JSON Schema

- Histórico (era 0.3.4): registros novos gravavam `schema_version` 2 com o conjunto de campos daquela release.
- Atual (0.5.3): novas escritas usam schema v3 e chaves TOML em inglês; o load faz dual-read de aliases legados em português.
- Schemas de eventos de agente ficam em `docs/schemas/` (veja [schemas/README.md](schemas/README.md)).

### Após 0.3.4+
- `timeout_ms`
- `max_command_chars`
- `max_output_chars`
- `key_path`
- `key_passphrase` (mascarado)
- `disable_sudo`
- `schema_version` 2 (somente escritas históricas; wire atual é schema v3)

### Segredos at-rest (era 0.3.6; ainda atuais)
- Campos password/sudo/su/passphrase podem armazenar blobs `sshcli-enc:v1:…`.
- Prefira flags CLI: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`.
- Fontes de primary-key: CLI `--secrets-key-file` / `--use-keyring`, ou XDG `secrets.key`. `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` são **rejeitadas fail-closed** (não são store).

### Mascaramento (0.4.0)
- Senha vazia → JSON `null`; não vazia → string `***`.
- Texto humano em show ainda usa "(não definida)" para senha vazia.

### Eventos de transfer / tunnel (0.4.0 / 0.4.1+)
- JSON de sucesso SCP inclui `event: "scp-transfer"` obrigatório (IO-009).
- Tunnel continua emitindo `event: "tunnel_listening"` após bind.
- Sucesso SCP: `docs/schemas/scp-transfer.schema.json`
- Tunnel listening: `docs/schemas/tunnel-listening.schema.json`
- Falhas em modo JSON: `docs/schemas/error-envelope.schema.json` em stderr


## Notas de compatibilidade
- Hosts TOML existentes carregam e migram defaults de campos nos caminhos de leitura/gravação.
- Alias legado `--maxChars` mapeia para limite de entrada de comando.
- Timeout padrão é 60000 ms para automação de agentes.
- Comportamento always-trust de host key sumiu em builds de release.
- Cifragem padrão ligada; plaintext exige opt-out explícito só via CLI `--allow-plaintext-secrets` (stores env de secrets são rejeitados fail-closed).
- Tracing padrão é error; prosa INFO não é esperada no stderr do agente.
- SCP permanece file-only por design em 0.4.0+ (ainda verdade em 0.5.4; não é limitação temporária).
- Integridade SFTP exige 0.5.3+ (G1); não confie em upload SFTP pré-0.5.3 sem checksum externo.


## Rollback
- Reinstale versão anterior com pin exato se necessário.
- Mantenha export mascarado via `vps export` antes de experimentos grandes.
- Se voltar abaixo de 0.3.6, blobs cifrados exigem a primary-key correspondente ou re-export em plaintext ainda na 0.3.6+.
- Se voltar para 0.3.9, não espere wire SCP funcional (atualize de novo para 0.4.0+ para transferências).
- Se voltar abaixo de 0.5.3, não confie na integridade de upload SFTP sem checksum externo (G1).

## Formato wire 0.5.3 (schema v3) — atual

- O `schema_version` atual para novas escritas é 3 (não 2).
- Novas escritas usam chaves TOML em inglês: `name`, `port`, `username`, `password`, `added_at`, …
- O load ainda aceita chaves legadas em português (`nome`, `porta`, `usuario`, `senha`, `adicionado_em`) — dual-read serialize EN / aliases PT no load.
- `added_at` é opcional no import (padrão: agora quando ausente).
- O corpo de `vps export` segue o formato resolvido: o envelope de agente (`event: "vps-export"`) em qualquer stdout non-TTY, inclusive num arquivo `.toml`; `--output-format text` é o único caminho para corpo TOML. JSON auto non-TTY se aplica ao export como em todo o resto.
- `vps import` aceita TOML (EN + aliases PT) ou envelopes JSON `vps-export`; `--allow-incomplete` para hosts redacted/skeleton.
- `--include-secrets` exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`.
- Controle de secrets é só CLI/XDG: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, ou XDG `secrets.key` (stores env de secrets rejeitados fail-closed).
- Termo preferido para a chave at-rest é primary-key; entradas legadas de keyring rotuladas master-key ainda são legíveis.
- Export redacted: secrets não vazios → `***` (`FIXED_MASK`); secrets vazios permanecem `""` (G-E2E-10).
- `vps add --use-agent` / `--agent-socket` cadastra hosts só-agent (G-E2E-19).
- Feature clap `env` removida — sem `#[arg(env=…)]` de config de produto (G-E2E-08).
- Stamp de versão anexa `-dirty` quando a working tree está suja mesmo com `.commit_hash` (G-E2E-06).
- ACME `invalidContact` / validação permanente → exit **64** (não faça retry como 74) (G-E2E-01).
- Primeiro `vps add` com auto-key: **um** documento JSON `event: "vps-added"` com campo `secrets_key_auto_created` (G-E2E-04 / família G8).
- Root `ssh-cli schema` / `ssh-cli doctor` para descoberta de agente (G-E2E-02/03).
- Valores de timeout abaixo de 1000 ms emitem aviso em stderr (milissegundos, não segundos).
- Valores semelhantes a senha em argv avisam em stderr; prefira `--password-stdin` / `--*-stdin`.
- Comando remoto vazio falha com mensagem técnica em inglês `empty command` em qualquer locale.
- `secrets init --json` → `event: "secrets-init"`; `secrets reencrypt --json` → `event: "secrets-reencrypt"`; a 1ª gravação pode definir `secrets_key_auto_created: true` no mesmo JSON de sucesso (um documento).
- Eventos de sucesso CRUD em JSON efetivo: `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import`.
- Tunnel `--bind` tem padrão `127.0.0.1` (loopback).
- Exit 65 cobre `TomlDe` / dados ruins de import; exit 77 é auth/host-key/permissão; arquivo SCP ausente é exit 66 com `file not found: <path>`.
- Verbosidade graduada `-v`/`-vv`/`-vvv` (info/debug/trace), sempre com escopo na crate (G2/G14).
- Integridade de upload SFTP corrigida (G1); SETSTAT atime+mtime (G3); set_metadata fail-closed (G4); máscara de perms (G12); cardinalidade de cancel em batch (G5/G17).
- Suites: `tests/gaps_v042_integration.rs` + `tests/gaps_v051_integration.rs` + `tests/gaps_v056_ssh.rs` + `tests/gaps_v057_sftp.rs` + `tests/gaps_v058_e2e_residual.rs`; e2e oficial **E01–E18**.

Linha de produto: 0.5.4.

## Veja também
- [HOW_TO_USE.pt-BR.md](HOW_TO_USE.pt-BR.md) — superfície de comandos do usuário
- [AGENTS.pt-BR.md](AGENTS.pt-BR.md) — contratos de agente e roteamento de exit
- [COOKBOOK.pt-BR.md](COOKBOOK.pt-BR.md) — receitas copy-paste
- [schemas/README.md](schemas/README.md) — índice de schemas JSON
