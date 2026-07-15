# Multiplataforma

> Fuja de cola SSH específica de SO com um binário Rust portátil.

- Leia este documento em [inglês](CROSS_PLATFORM.md).
- Linha de produto: **0.4.2**.


## A dor que você já conhece
- Wrappers daemon Node diferem por package manager e versão de runtime do host.
- Scripts SSH só-shell vazam segredos em histórico e listas de processos.
- Convenções de path divergem entre homes de config Linux, macOS e Windows.
- Agentes precisam de uma superfície de comando que morre após cada run em qualquer lugar.


## Matriz de suporte

| Plataforma | Status | Notas |
| --- | --- | --- |
| Linux gnu | Suportado | Alvo principal de desenvolvimento |
| Linux musl | Suportado | Use `--features musl-allocator` quando necessário |
| macOS | Suportado | Pode precisar remover quarentena do Gatekeeper |
| Windows | Suportado | Config usa project dirs da plataforma |
| Containers | Suportado | Monte ou defina `SSH_CLI_HOME` para persistência |


## Linux
- Prefira `cargo install ssh-cli --locked` em `~/.cargo/bin`.
- Espere config XDG em `~/.config/ssh-cli/` por omissão.
- Garanta mode 0600 após o primeiro save de `config.toml` e `secrets.key`.
- Cifragem at-rest padrão guarda blobs em `config.toml`; mantenha backup offline de `secrets.key`.


## macOS
- Mesmo path de install cargo que no Linux.
- Limpe quarentena com `xattr -d com.apple.quarantine` quando o Gatekeeper bloquear o binário.
- Espere config sob application support/project dirs macOS resolvidos por `directories` 6.
- Backend de keyring para master-key é opcional via `SSH_CLI_USE_KEYRING=1` após `secrets init --keyring`.


## Windows
- Instale via Rustup e cargo em toolchain suportada (MSRV 1.85.0).
- Use completions PowerShell de `ssh-cli completions powershell`.
- Prefira arquivos de chave com paths explícitos em vez de atalhos de home Unix.
- Dirs de config/projeto vêm de `directories`; use `vps doctor --json` para ver o vencedor.


## Containers
- Copie o binário para imagens distroless ou distro sem Node.
- Persista o dir de config ou defina `SSH_CLI_HOME` para memória multi-run (`config.toml`, `known_hosts`, `secrets.key`, `active`).
- Mantenha semântica one-shot; não empacote a CLI como sidecar de longa duração sem timeout de tunnel.
- Nunca embuta segredos live ou `secrets.key` em layers de imagem.


## Suporte a shell
- Completions bash, zsh, fish e PowerShell são gerados sob demanda.
- Prefira arrays argv explícitos em runtimes de agente a eval de string shell.
- Prefira flags stdin de segredo a embutir senhas no histórico do shell.


## Paths de arquivo e XDG
- Resolva o vencedor com `ssh-cli vps doctor --json` (inclui campos `secrets_*`).
- Sobrescreva só em testes via `--config-dir` ou `SSH_CLI_HOME`.
- Mantenha `known_hosts`, `active` e `secrets.key` como arquivos irmãos de `config.toml`.
- Escritas atômicas + flock protegem processos one-shot concorrentes no mesmo config.


## Portabilidade SCP
- SCP é **somente arquivos regulares** em toda plataforma (sem transferência recursiva de diretório; sem subsistema SFTP).
- Downloads com falha ou em andamento usam path irmão terminando em **`.ssh-cli.partial`**, depois rename no lugar (padrão atômico agnóstico de plataforma).
- Upload faz stream em blocos de 32 KiB em todo SO (evita carregar o arquivo inteiro na RAM).
- Preserve de mtime/mode segue estilo OpenSSH com remoto `-p` / linha `T`; em Unix APIs locais de permissão aplicam modes; no Windows bits de permissão podem não bater com octal Unix — não assuma fidelidade POSIX ACL completa.
- Matriz real-SSH E10–E14 em `scripts/e2e_real_ssh.sh` é validada principalmente em hosts Linux.


## Performance por alvo
- Cold start Linux é a baseline sob alvo de 100 ms.
- Builds musl podem trocar características de allocator; habilite `musl-allocator` quando necessário.
- RTT de rede domina operações remotas em todo SO.


## Agentes validados por plataforma
- Hosts Linux são a superfície principal de validação para runs de subprocesso de agente.
- macOS e Windows seguem o mesmo contrato CLI e JSON schemas.
- Agentes em container devem preservar exit codes e separação stdout/stderr.
- Tracing padrão é nível error para manter stderr do agente livre de prosa INFO salvo `RUST_LOG` ou `-v`.
- Parseie contratos de máquina só do stdout; trate tracing em stderr como log fora de contrato; envelopes de erro JSON usam stderr quando o modo JSON está ativo.
- Helpers de E2E SSH real ficam em `scripts/e2e_real_ssh.sh` (anti-leak; só local; E01–E14).
- Contratos JSON (`event` scp-transfer, `tunnel_listening`, flags de auth em tunnel/health) são idênticos em todo SO; veja AGENTS.pt-BR.md e docs/schemas/.
