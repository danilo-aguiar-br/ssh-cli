# Guia de agentes para ssh-cli

> **0.5.4** — release de segurança e agent-native. Corrige DoS remoto pré-auth no banner SSH (A1), impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3), fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2) e adiciona flags de redução de payload (`--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`) aplicadas antes da serialização. BREAKING: falha parcial multi-host agora sai com exit **1** (era 65); `--bind` fora do loopback exige `--i-accept-network-exposure`. Novo evento `tunnel_closed`.


> **G-E2E-16:** Prefira GraphRAG `list` / `read` pelo nome exato da memória a `hybrid-search` sob carga.
>
> **G-E2E-04 / G8 wire:** OBRIGATÓRIO um documento JSON por sucesso one-shot no data path.
> PROIBIDO: parsear dual-events NDJSON multi-linha como data path de sucesso.
> O campo `secrets_key_auto_created` (quando presente) vive no **mesmo** documento `vps-added` — nunca um segundo evento no stdout.
> `exec --json` de passo único emite exatamente **um** objeto (G8).
>
> **Descoberta:** `ssh-cli commands`, `ssh-cli schema`, `ssh-cli doctor` (alias root de `vps doctor`).
>
> Corte o desperdício de RAM de processos residentes e mantenha SSH multi-host sob controle do agente.

- Leia este documento em [inglês](AGENTS.md).
- Combine com [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md) e [../skills/ssh-cli-pt/SKILL.md](../skills/ssh-cli-pt/SKILL.md).
- Linha de produto: 0.5.4.


## Por quê
### Substitua processos SSH Node de longa duração por binário nascer-executar-morrer
- Sessões daemon persistentes queimam RAM com sockets ociosos.
- Um host por processo daemon multiplica processos para N servidores.
- Um binário Rust com storage XDG multi-host colapsa essa proliferação.
- Agentes ganham JSON determinístico e sysexits sem overhead de protocolo extra.


## Economia
### Meça o ganho operacional
- Cold start alvo abaixo de 100 ms em hosts Linux típicos.
- Memória do processo volta ao SO após cada comando.
- Sem taxa de runtime Node e sem gerenciador SSH permanente.
- Uma instalação serve Claude Code, Cursor, Windsurf, Codex e agentes shell.


## Soberania
### Mantenha credenciais e confiança de host locais
- Guarde hosts sob XDG sem proliferação de `.env`.
- Prefira chaves privadas e segredos via stdin a senhas coladas no chat.
- Cifragem at-rest por padrão (ChaCha20-Poly1305 + auto `secrets.key`); gerencie com `secrets status|init|reencrypt`.
- Force known_hosts TOFU para dificultar MITM silencioso.
- Desabilite elevação quando o workflow deve permanecer sem privilégio.
- PROIBIDO: logar primary-key, senhas de host ou segredos decifrados.


## Agentes e orquestradores compatíveis
- Claude Code com o pacote de skill embarcado
- Cursor com shell ou agent tools
- Windsurf shell tool
- Codex CLI shell tool
- OpenCode shell tool
- Aider, Continue, Gemini CLI, OpenHands, bash/zsh genérico


## Inventário de comandos (árvore completa)

Todas as 47 folhas estão escritas por extenso, não em notação de chaves: um agente que busca
`tls acme account create` precisa encontrar exatamente essa string. Descubra a mesma árvore em
runtime com `ssh-cli commands` (`event: "commands"`).

| Superfície | Comandos |
| --- | --- |
| `vps` | `vps add` `vps list` `vps remove` `vps edit` `vps show` `vps path` `vps doctor` `vps export` `vps import` |
| Sessão | `connect` |
| Exec | `exec` `sudo-exec` `su-exec` |
| `scp` | `scp upload` `scp download` (somente arquivos regulares) |
| `sftp` | `sftp upload` `sftp download` `sftp ls` `sftp mkdir` `sftp rmdir` `sftp rm` `sftp stat` `sftp rename` |
| Rede | `tunnel` (quatro modos: local, `--reverse`, `--socks5`, `--remote-socket`) `health-check` |
| `secrets` | `secrets status` `secrets init` `secrets reencrypt` |
| Descoberta | `completions` `commands` `schema` `doctor` (alias root de `vps doctor`) |
| `locale` | `locale show` `locale set` `locale clear` |
| `tls` | `tls provider` `tls paths` |
| `tls mtls` | `tls mtls list` `tls mtls import` `tls mtls show` `tls mtls remove` |
| `tls acme` | `tls acme account create` `tls acme account show` `tls acme issue` `tls acme complete` `tls acme status` `tls acme list` |

### Flags globais relevantes
- `--lang`, `-v`/`-vv`/`-vvv` (G14 graduado; G2 com escopo `warn,ssh_cli=*`), `-q`, `--config-dir`, `--no-color`, `--output-format`, `--json`
- `--disable-sudo`, `--replace-host-key`
- `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- `--timeout`, `--max-concurrency`, `--fail-fast`, `--scp-file-concurrency`

### Redução de payload (0.5.4) — não gaste token em dado que você vai descartar
- Oito flags globais moldam a resposta **antes** da serialização, então o envelope gigante nunca é construído.
- `--select <CAMINHOS>` (apelido `--fields`) mantém somente esses caminhos pontilhados em cada registro.
- `--filter chave=valor` | `chave!=valor` | `chave~substring` mantém os registros que casam; repetível, combinada com AND.
- Predicado malformado é rejeitado no parse em vez de casar nada em silêncio, então um typo nunca é confundido com resultado vazio.
- `--limit N` emite no máximo N registros e é distinta dos limites de consulta de cada comando.
- `--sort <CAMINHO>` ordena de forma ascendente pelo caminho pontilhado, comparando números numericamente.
- `--dedupe-by <CAMINHO>` descarta registros posteriores que repetem o valor daquele caminho.
- `--count-only` substitui a coleção de registros por `{"count": N}`, contado depois de toda filtragem.
- `--truncate-content <CARACTERES>` encurta strings longas por **caracteres**, nunca por bytes, então o UTF-8 continua válido.
- `--max-output-bytes <BYTES>` limita o envelope descartando registros do fim, nunca fatiando o texto JSON.
- OBRIGATÓRIO: preferir essas flags a canalizar o stdout por ferramenta JSON externa. O pipe paga o custo total de token primeiro e encolhe depois; a flag nunca escreve o payload.

### Flags de recusa e de ensaio
- `--no-input` recusa ler o stdin e falha rápido em vez de bloquear esperando um humano ausente.
- OBRIGATÓRIO: passar `--no-input` em qualquer execução sem operador, porque prompt sem operador é travamento indefinido, não erro.
- `--dry-run` imprime o plano de uma operação destrutiva e sai sem executar.
- `--dry-run` é aceita somente por `vps remove`, `vps import`, `sftp rm`, `sftp rmdir`, `secrets init` e `secrets reencrypt`.
- Em qualquer outro lugar `--dry-run` é rejeitada com exit **64** em vez de aceita e ignorada, então um ensaio nunca é confundido com um no-op que já executou.


## Detalhes de integração do agente
### Contrato imperativo para autores
- OBRIGATÓRIO: invocar `ssh-cli` como subprocesso e aguardar o exit (one-shot).
- OBRIGATÓRIO: parsear JSON do stdout quando `--json` ou `--output-format json` estiver ativo (JSON auto quando stdout não é TTY).
- OBRIGATÓRIO: parsear **exatamente um** objeto JSON nos caminhos de sucesso (G8 / G-E2E-04) — nunca dual-events multi-linha.
- OBRIGATÓRIO: tratar tracing em stderr como log não-contratual; não parsear stderr como JSON de sucesso.
- OBRIGATÓRIO: quando o modo de erros JSON estiver ativo (`--json` / JSON efetivo em scp|sftp|tunnel|formato global), parsear envelopes de falha em **stderr** (`exit_code`, `message`, opcional `remote_exit_code`) via `docs/schemas/error-envelope.schema.json`.
- OBRIGATÓRIO: esperar tracing default em error; use `-v` / `-vv` / `-vvv` só ao depurar (info/debug/trace; sempre allowlist com escopo na crate — G2/G14).
- PROIBIDO: confiar em `RUST_LOG` ambiente — é ignorado; use só `-v`/`-vv`/`-vvv`.
- OBRIGATÓRIO: cadastrar hosts com `vps add` antes de trabalho remoto repetido (auth: senha **ou** chave **ou** `--use-agent` / `--agent-socket`).
- OBRIGATÓRIO: fornecer senha ou chave; credenciais vazias são rejeitadas na escrita.
- OBRIGATÓRIO: tratar senha vazia em list/show JSON como `null` (hosts só-chave); não vazia mascara `***`.
- OBRIGATÓRIO: comando remoto vazio falha com mensagem técnica `empty command` (sempre em inglês) e exit de uso 64.
- OBRIGATÓRIO: passar `--timeout-ms` em toda invocação de `tunnel`.
- OBRIGATÓRIO: tratar `scp` como **somente arquivos regulares** (sem diretórios, sem `-r`). Para árvores / FS remoto use `sftp` (`upload|download --recursive`, `ls`, `mkdir`, `rm`, `stat`, `rename`).
- OBRIGATÓRIO: preferir linha de produto **0.5.3+** para SFTP — G1 corrigiu truncamento de upload a zero bytes; verifique transferências com `sha256` no destino (G15), não só tamanho reportado pelo cliente.
- OBRIGATÓRIO: nunca depender do crates.io 0.3.9 para SCP; o wire estava quebrado — exija 0.5.3+.
- OBRIGATÓRIO: parsear sucesso SCP com `docs/schemas/scp-transfer.schema.json` (`ok`, `event` (`scp-transfer`), `direction`, `vps`, `local`, `remote`, `bytes`, `duration_ms`) no **stdout**.
- OBRIGATÓRIO: arquivo local/remoto ausente no SCP sai com exit 66 e mensagem `file not found: <path>` (path canônico/normalizado; sem prefixos `SCP:` empilhados).
- OBRIGATÓRIO: o corpo de `vps export` segue o formato resolvido, então um agente, cujo stdout nunca é TTY, recebe o envelope JSON `event: "vps-export"` sem passar `--json` e mesmo quando `-o` nomeia um arquivo `.toml`; corpo TOML exige `--output-format text`. JSON auto non-TTY **se aplica** ao export.
- OBRIGATÓRIO: em `vps export` redacted, secrets vazios são strings vazias, nunca ciphertext `sshcli-enc:` de vazio (EXP-001).
- OBRIGATÓRIO: `--include-secrets` exige `-o`/`--output` ou `--i-understand-secrets-on-stdout`.
- OBRIGATÓRIO: `vps import` aceita TOML (chaves EN + aliases PT legados na leitura) ou JSON `vps-export`; use `--allow-incomplete` para hosts redacted/skeleton.
- OBRIGATÓRIO: `added_at` / `adicionado_em` são opcionais no import (serde usa agora quando omitidos).
- OBRIGATÓRIO: wire schema v3 dual-read — serializa chaves EN, leitura ainda aceita aliases PT (`nome`/`porta`/`usuario`/`senha`/…).
- OBRIGATÓRIO: preferir flags de secrets `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` a env vars; preferir o termo primary-key; o keyring ainda pode aceitar o alias legado `secrets-master-key` na leitura.
- OBRIGATÓRIO: `secrets init --json` / `secrets reencrypt --json` emitem `secrets-init` / `secrets-reencrypt` (`docs/schemas/secrets-init.schema.json`, `docs/schemas/secrets-reencrypt.schema.json`); a 1ª gravação de segredo pode definir `secrets_key_auto_created: true` no mesmo JSON de sucesso (um documento). Catálogo: [docs/schemas/README.md](schemas/README.md).
- OBRIGATÓRIO: em `tunnel --json`, aguardar um objeto no stdout com `event: "tunnel_listening"` (`docs/schemas/tunnel-listening.schema.json`) antes de usar a porta local; o processo permanece vivo até timeout ou sinal; após `tunnel_listening`, o deadline pós-bind termina com exit 0 (TUN-002); timeout pré-bind permanece 74.
- OBRIGATÓRIO: tunnel `--bind` tem padrão `127.0.0.1` (loopback).
- PERMITIDO: `tunnel` / `health-check` podem usar `--password-stdin` / `--key` / `--key-passphrase` / `--key-passphrase-stdin` (paridade CLI-005/006 com exec/scp).
- PERMITIDO: passar `health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais.
- OBRIGATÓRIO: preferir fan-out multi-host para frota — `exec|sudo-exec|su-exec|scp|sftp|health-check --all` **ou** `--hosts a,b,c` roda sessões **concorrentes limitadas** (`Semaphore` + `JoinSet`), não um host por spawn de processo. JSON batch se aplica aos dois modos multi (mesmo se `--hosts` listar um nome).
- OBRIGATÓRIO: existe um terceiro seletor, só na família exec — `exec|sudo-exec|su-exec --tags t1,t2` endereça todo host que carregue qualquer uma dessas tags (`vps add --tag`). `--all`, `--hosts` e `--tags` são mutuamente exclusivos; o clap rejeita qualquer par. `scp`, `sftp` e `health-check` aceitam `--all` e `--hosts`, mas **não** `--tags`.
- OBRIGATÓRIO: parsear JSON multi-host via schemas batch: `health-check-batch` / `exec-batch` / `scp-batch` / `sftp-batch` (`docs/schemas/*-batch.schema.json`); o campo `max_concurrency` está no envelope.
- PERMITIDO: limitar fan-out com global `--max-concurrency N` (1..=64; auto = CPUs×4 vs RAM livre/2 / 16 MiB, clamp 1..=64). O mesmo gate limita forwards de tunnel.
- PROIBIDO: assumir multi-host sequencial por padrão quando `--all` estiver disponível — o wall-clock é dominado pelo RTT SSH; sessões concorrentes são o modus operandi do produto.
- OBRIGATÓRIO: cancelamento multi-arquivo SCP/SFTP preenche o resto cancelled para `results.len() == input.len()` (G5/G17).
- OBRIGATÓRIO: timeouts abaixo de 1000 ms e valores de senha em argv emitem warn em stderr — não parseie essas linhas como envelope de erro JSON.
- OBRIGATÓRIO: preferir `--password-stdin` / `--key` a secrets em argv.
- OBRIGATÓRIO: instalar com `cargo install ssh-cli --locked` (ou install por path com pins).
- PROIBIDO: assumir conexão SSH de longa duração entre runs de processo.
- PROIBIDO: reintroduzir empacotamento daemon de longa duração neste repositório.
- PROIBIDO: habilitar ou emitir telemetria de produto.
- PROIBIDO: retry cego em exit 64, 65, 66 ou 77.
- PROIBIDO: parsear dual-events NDJSON multi-linha no data path de sucesso — um documento JSON por sucesso one-shot; `secrets_key_auto_created` (quando definido) fica no mesmo objeto `vps-added`.
- PROIBIDO: tratar `RUST_LOG` ambiente como config de produto (ignorado; só `-v`/`-vv`/`-vvv`).
- PROIBIDO: imprimir ou armazenar material de primary-key dos comandos `secrets`.
- PROIBIDO: tratar árvores de diretório SCP ou `-r` recursivo como suportados.
- PROIBIDO: assumir que o host do agente roda binários cliente OpenSSH para trabalho de produto —
  `ssh-cli` é Rust puro (`russh`); sem spawn local de `ssh`/`scp`/`ssh-keygen` em runtime.
- OBRIGATÓRIO: tratar strings de comando remoto como entrada hostil; bytes NUL são rejeitados
  com invalid-argument antes do exec no canal SSH (G-PROC-03).
- OBRIGATÓRIO: SETSTAT SFTP envia atime+mtime juntos (G3); set_metadata é fail-closed (G4); bits de permissão são mascarados **por direção** — no upload (saída) vale `SFTP_PERM_MASK` `0o7777`, que preserva setuid/setgid/sticky num arquivo que você já controla (G12/G19); no download (entrada) vale `SFTP_PERM_MASK_UNTRUSTED` `0o0777`, então bits de elevação enviados pelo servidor não caem no arquivo local (A3).
- OBRIGATÓRIO: falhas de `set_permissions` local no download SFTP são erros, não silenciosas (G18).
- OBRIGATÓRIO: o caminho wire do cliente SCP (`client_real_scp.rs`) usa identificadores em inglês e erros de canal em inglês (G16).
- NOTA: G6 (isolamento `serial_test` para estado global de signal/cancel) é preocupação de **harness de teste**, não de runtime do agente.
- PROIBIDO: assertar texto FIXED dentro do `gaps.md` local como teste de produto (G13/G15) — esse arquivo é inventário de auditoria local do mantenedor (gitignored / excluído do cargo), não um contrato publicado.


## Integrações de crate
- Consumidores publicados dependem do contrato da CLI, não de API de lib instável.
- Fixe experimentos de lib em versão exata se linkar `ssh_cli` como lib.
- Prefira integração via binário no PATH para agentes.


## Contrato CRUD e JSON
### Operações legíveis por máquina
- Listar hosts: `ssh-cli vps list --json` devolve array de objetos de host mascarados.
- Mostrar host: `ssh-cli vps show <name> --json` devolve um objeto de host mascarado.
- Descoberta: `ssh-cli commands`, `ssh-cli schema [NAME]`, `ssh-cli doctor` (alias de `vps doctor`).
- Doctor: `ssh-cli vps doctor --json` (ou `ssh-cli doctor --json`) devolve layer, paths, schema, contagem de hosts, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out` (booleano JSON), telemetry false.
- Secrets: `ssh-cli secrets status --json` devolve modo de cifragem sem material de chave; `secrets init --json` → `event: "secrets-init"`; `secrets reencrypt --json` → `event: "secrets-reencrypt"`.
- Eventos de sucesso CRUD quando JSON está efetivo (`--json` / `--output-format json` / JSON auto non-TTY): `vps-added`, `vps-edited`, `vps-removed`, `vps-connected`, `vps-import` (com campo opcional `secrets_key_auto_created` quando uma chave é auto-criada — ainda um documento).
- Família exec (host único): `ssh-cli exec|sudo-exec|su-exec <vps> <cmd> --json` devolve stdout, stderr, exit_code, flags de truncamento, duration_ms — **um objeto** (G8).
- Família exec (frota): `ssh-cli exec|sudo-exec|su-exec --all '<cmd>' --json` ou `--hosts a,b '<cmd>'` → `event: "exec-batch"` (`exec-batch.schema.json`); falha parcial por host não aborta irmãos.
- Tunnel: **somente host único** (um bind + uma sessão por one-shot). Tunnels multi-host = N invocações com portas/`--bind` distintos. Forwards ainda limitados por `--max-concurrency`.
- Doctor: `ssh-cli vps doctor [--json]` emite raiz única `event: vps-doctor` (`local` + `ssh_probe: null`). Adicione `--probe-ssh` para fan-out de health multi-host embutido em `ssh_probe` (opcional `--hosts a,b`). Nunca duas raízes JSON.
- SCP multi-arquivo (host único): `ssh-cli scp upload <VPS> f1 f2 … <REMOTE_DIR>` / `download <VPS> r1 r2 … <LOCAL_DIR>` usa **uma sessão SSH** e transfers seriais (auth uma vez; G-PAR-47).
- SCP multi-host × multi-arquivo: `ssh-cli scp upload --all f1 f2 … <REMOTE_DIR>` ou `--hosts a,b` — **sessões** limitadas por host; arquivos seriais em cada sessão (G-PAR-48). Download de frota multi-arquivo grava sob `<LOCAL_DIR>/<host>/`.
- Health (único): `ssh-cli health-check [<vps>] [--timeout <ms>] [--password-stdin|--key|--key-passphrase[-stdin]] --json` devolve name, status, latency_ms.
- Health (frota): `ssh-cli health-check --all --json` ou `--hosts a,b --json` → `event: "health-check-batch"` (`health-check-batch.schema.json`).
- SCP (único): `ssh-cli scp upload|download <vps> <local> <remote> --json` devolve sucesso de transfer no stdout (`scp-transfer.schema.json` com `event: "scp-transfer"` obrigatório); falhas usam envelope de erro em stderr; arquivo ausente → exit 66 `file not found: <path>` (path canônico/normalizado).
- SCP (frota / batch multi-arquivo): `event: "scp-batch"` (`scp-batch.schema.json`); download de frota de um arquivo grava `local.<vps>`; multi-arquivo usa subdirs por host; resultado multi-host×multi-arquivo pode ter `name` `host:path`.
- Fatos operacionais SCP: exija 0.5.3+; upload faz stream de 32 KiB; download grava `{path}.ssh-cli.partial` e renomeia; falha de `sync_data` é propagada antes do rename (G9); preservação de mtime/mode é best-effort e reportada em `mtime_preserved`, e o fsync do diretório pai em `durable` (G-SCP-R01/R02).
- SFTP: `ssh-cli sftp upload|download|ls|mkdir|rmdir|rm|stat|rename` com schemas `sftp-transfer` / `sftp-list` / `sftp-fs-op` / `sftp-batch`; prefira 0.5.3+ (integridade G1).
- Locale: `ssh-cli locale show|set|clear`; one-shot `--lang`.
- TLS: `ssh-cli tls provider|paths`; `tls mtls list|import|show|remove`; `tls acme account create|show`; `tls acme issue|complete|status|list`.
- Tunnel: `ssh-cli tunnel <vps> <local_port> [remote_host] [remote_port] --timeout-ms <ms> [--bind 127.0.0.1] [--password-stdin|--key|--key-passphrase[-stdin]] --json` emite `tunnel_listening` no stdout após o bind; `--bind` padrão `127.0.0.1`; deadline pós-bind sai com exit 0; timeout pré-bind permanece 74.
- Modos de tunnel (0.5.4): `--socks5` serve proxy `CONNECT` sem autenticação (RFC 1928), `--remote-socket <PATH>` encaminha para socket Unix remoto, `--reverse` pede ao servidor que escute e entregue de volta em `<local_port>`. Os três são mutuamente exclusivos. `remote_host`/`remote_port` são omitidos sob `--socks5` e `--remote-socket` (passá-los é exit 64) e significam o bind do **servidor** sob `--reverse`, onde `remote_port 0` deixa o servidor alocar e informar a porta em `local_port`.
- Os dois eventos de tunnel carregam `mode` (`local` / `socks5` / `streamlocal` / `reverse`); leia-o antes de interpretar os campos vizinhos, já que `local_port` é a porta do servidor sob `--reverse` e não existe destino único sob `--socks5`.
- `--i-accept-network-exposure` guarda o `--bind` local nos três modos locais e o bind **remoto** sob `--reverse`, que é o lado exposto nessa direção. O bind local é parseado como IP pelo clap (typo → exit 2); o bind remoto é comparado como texto, já que a RFC 4254 admite nomes e a string vazia (typo → exit 64 do guard).
- PROIBIDO: passar `--bind` junto de `--reverse` esperando que surta efeito — a flag é aceita pelo clap e então descartada, em silêncio, porque a entrega reversa é forçada para loopback. Enderece o listener do lado servidor pelo posicional `<remote_host>`.
- Export: o corpo de `ssh-cli vps export` segue o formato resolvido, então é o envelope JSON `event: "vps-export"` em qualquer stdout non-TTY e `--output-format text` é o único caminho para TOML; secrets vazios serializam como `""` (nunca `sshcli-enc:`). `--include-secrets` precisa de `-o` ou `--i-understand-secrets-on-stdout`.
- Import: `ssh-cli vps import --file <path> [--allow-incomplete]` aceita TOML (serialização EN / aliases PT na leitura) ou JSON `vps-export`; `added_at` / `adicionado_em` opcionais (default agora).
- Campos de senha vazios serializam como JSON `null`; secrets não vazios mascaram como `***` (`FIXED_MASK`). `vps export` redacted: não vazio → `***`; vazio → `""`.
- Valide payloads contra schemas em `docs/schemas/`; índice: [docs/schemas/README.md](schemas/README.md).


## Roteamento de exit codes
- Exit 0 significa sucesso.
- Exit 1 significa falha genérica de runtime; inspecione stderr.
- Exit 64 significa erro de uso ou argumento (incluindo comando vazio) **ou** validação ACME permanente (`invalidContact` / 4xx); corrija argv/contato, não faça retry.
- Exit 65 (`TomlDe` / JSON / schema) significa erro de parse/dados; corrija o payload de entrada.
- Exit 66 significa VPS ou arquivo ausente (`file not found: <path>` no SCP); cadastre ou corrija o nome/path.
- Exit 73 significa falha de escrita de config; verifique permissões e disco.
- Exit 74 significa falha de IO/conexão SSH; retry de rede pode ajudar.
- Exit 77 significa falha de auth ou política de host-key; tente `--key` / `--password-stdin` / passphrase stdin; não faça retry cego.
- Exit 130/143 significa término por sinal.


## Estratégia de retry
- Prefira campos do envelope de erro JSON `retryable` + `error_class` a heurísticas só de exit (`docs/schemas/error-envelope.schema.json`).
- Ramifique por `error_code`, NUNCA por `message`: o B2 tornou a linha humana de erro localizável, e `--lang pt-BR` agora a renderiza em português no modo **texto**.
- O `message` do envelope JSON permanece em **inglês estável** por contrato, então o parse do agente nunca depende do locale do host.
- O ponto de junção é `emit_resolved_ssh_error`: o ramo `--json` mantém o `Display` inglês de `SshCliError`, o ramo humano consulta `i18n::localized_error_text`.
- Código de erro sem tradução cai no `Display` inglês — a localização é fail-open e nunca apaga a linha de erro.
- Retry no máximo duas vezes em `retryable: true` / exit 74 com backoff **exponential full-jitter** (base 200ms, cap 5s; veja `ssh_cli::retry::RetryConfig::agent_default`).
- Nunca faça retry em `retryable: false` ou exits 64, 65, 66, 77, 1 (comando remoto), 130/143/141 sem mudar entradas.
- Validação ACME permanente (`invalidContact` / 4xx) é exit **64**, não exit 74 — **não** trate como IO de rede retryable.
- O binário **não** faz auto-retry de `exec`/`scp`/`sftp` não-idempotentes in-process (one-shot least privilege); o agente reinvoca o processo.
- Encurte ou divida comandos quando o exit indicar rejeição de max_command_chars.
- Confirme mudanças de host key com um humano antes de `--replace-host-key`.
