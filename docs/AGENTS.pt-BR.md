# Guia de agentes para ssh-cli

> Corte o desperdício de RAM de processos residentes e mantenha SSH multi-host sob controle do agente.

- Leia este documento em [inglês](AGENTS.md).
- Combine com [../INTEGRATIONS.pt-BR.md](../INTEGRATIONS.pt-BR.md) e [../skills/ssh-cli-pt/SKILL.md](../skills/ssh-cli-pt/SKILL.md).
- Linha de produto: **0.4.0** (inventário Fechado; russh 0.62.2).


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
- PROIBIDO: logar master-key, senhas de host ou segredos decifrados.


## Agentes e orquestradores compatíveis
- Claude Code com o pacote de skill embarcado
- Cursor com shell ou agent tools
- Windsurf shell tool
- Codex CLI shell tool
- OpenCode shell tool
- Aider, Continue, Gemini CLI, OpenHands, bash/zsh genérico


## Detalhes de integração do agente
### Contrato imperativo para autores
- OBRIGATÓRIO: invocar `ssh-cli` como subprocesso e aguardar o exit (one-shot).
- OBRIGATÓRIO: parsear JSON de stdout quando `--json` ou `--output-format json` (JSON auto se stdout não é TTY).
- OBRIGATÓRIO: tratar tracing em stderr como log fora de contrato; não parsear stderr como JSON.
- OBRIGATÓRIO: esperar tracing padrão no nível error; definir `RUST_LOG` ou `-v` só ao diagnosticar.
- OBRIGATÓRIO: cadastrar hosts com `vps add` antes de trabalho remoto repetido.
- OBRIGATÓRIO: fornecer senha ou chave; credencial vazia é rejeitada na gravação.
- OBRIGATÓRIO: tratar senha vazia em list/show JSON como `null` (hosts só-chave); não vazia mascara `***`.
- OBRIGATÓRIO: passar `--timeout-ms` em toda invocação de `tunnel`.
- OBRIGATÓRIO: pode passar `health-check --timeout <ms>` quando o timeout padrão do host for longo ou curto demais.
- OBRIGATÓRIO: preferir `--password-stdin` / `--key` a segredos em argv.
- OBRIGATÓRIO: instalar com `cargo install ssh-cli --locked` (ou path com pins).
- PROIBIDO: assumir conexão SSH longa entre runs de processo.
- PROIBIDO: reintroduzir packaging de daemon de longa duração neste repositório.
- PROIBIDO: habilitar ou emitir telemetria de produto.
- PROIBIDO: retry cego em exit 64, 65, 66 ou 77.
- PROIBIDO: imprimir ou armazenar material de master-key dos comandos `secrets`.


## Integrações de crate
- Consumidores publicados dependem do contrato da CLI, não de API de lib instável.
- Pine experimentos de lib em versão exata se linkar `ssh_cli` como lib.
- Prefira integração via binário no PATH para agentes.


## Contrato CRUD e JSON
### Operações legíveis por máquina
- Listar hosts: `ssh-cli vps list --json` retorna array de objetos mascarados.
- Mostrar host: `ssh-cli vps show <name> --json` retorna um objeto mascarado.
- Doctor: `ssh-cli vps doctor --json` retorna camada, paths, schema, contagem de hosts, `secrets_at_rest`, `secrets_key_source`, `secrets_key_file`, `secrets_plaintext_opt_out`, telemetry false.
- Secrets: `ssh-cli secrets status --json` retorna modo de cifragem sem material de chave.
- Família exec: `ssh-cli exec|sudo-exec|su-exec ... --json` retorna stdout, stderr, exit_code, flags de truncagem, duration_ms.
- Health: `ssh-cli health-check [--timeout <ms>] --json` retorna name, status, latency_ms.
- Campos de senha vazios serializam como JSON `null`; segredos não vazios mascaram como `***`.
- Valide payloads contra schemas em `docs/schemas/`.


## Roteamento de exit codes
- Exit 0 significa sucesso.
- Exit 1 significa falha genérica de runtime; inspecione stderr.
- Exit 64 significa erro de uso/argumento; corrija argv, não faça retry.
- Exit 65 significa erro de parse/dados; corrija o payload.
- Exit 66 significa VPS ou arquivo ausente; cadastre ou corrija o nome.
- Exit 73 significa falha de escrita de config; cheque permissões e disco.
- Exit 74 significa falha de IO/conexão SSH; retry de rede pode ajudar.
- Exit 77 significa falha de auth ou política de host-key; tente `--key` / `--password-stdin` / passphrase stdin; sem retry cego.
- Exit 130/143 significa término por sinal.


## Estratégia de retry
- Retry no máximo duas vezes no exit 74 com backoff.
- Nunca faça retry em 64, 65, 66, 77 sem mudar as entradas.
- Encurte ou divida comandos quando o exit indicar rejeição por max_command_chars.
- Confirme mudanças de host key com humano antes de `--replace-host-key`.
