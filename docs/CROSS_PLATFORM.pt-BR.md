# Multiplataforma

> Escape a cola SSH específica de SO com um binário Rust portátil.

- Leia este documento em [inglês](CROSS_PLATFORM.md).
- Linha de produto: **0.3.9**.


## A dor que você já conhece
- Wrappers daemon Node de longa duração diferem por gerenciador de pacotes e versão de runtime.
- Scripts SSH só-shell vazam segredos em histórico e listas de processo.
- Convenções de path divergem entre homes de config Linux, macOS e Windows.
- Agentes precisam de uma superfície de comando que morre após cada run em todo lugar.


## Matriz de suporte

| Plataforma | Status | Notas |
| --- | --- | --- |
| Linux gnu | Suportado | Alvo primário de desenvolvimento |
| Linux musl | Suportado | Use `--features musl-allocator` quando necessário |
| macOS | Suportado | Pode precisar remover quarentena do Gatekeeper |
| Windows | Suportado | Config usa project dirs da plataforma |
| Containers | Suportado | Monte ou defina `SSH_CLI_HOME` para persistência |


## Linux
- Prefira `cargo install ssh-cli --locked` em `~/.cargo/bin`.
- Espere config XDG em `~/.config/ssh-cli/` por padrão.
- Garanta mode 0600 após o primeiro save de `config.toml` e `secrets.key`.
- Cifragem at-rest default grava blobs no TOML; faça backup offline de `secrets.key`.


## macOS
- Mesmo caminho de install cargo que no Linux.
- Limpe a quarentena com `xattr -d com.apple.quarantine` quando o Gatekeeper bloquear o binário.
- Espere config sob application support/project dirs resolvidos por `directories` 6.
- Backend keyring para master-key é opcional via `SSH_CLI_USE_KEYRING=1` após `secrets init --keyring`.


## Windows
- Instale via Rustup e cargo no toolchain suportado (MSRV 1.85.0).
- Use completions PowerShell de `ssh-cli completions powershell`.
- Prefira arquivos de chave com paths explícitos em vez de atalhos de home Unix.
- Dirs de config vêm de `directories`; use `vps doctor --json` para ver o vencedor.


## Containers
- Copie o binário para imagens distroless ou de distro sem Node.
- Persista o dir de config ou defina `SSH_CLI_HOME` para memória multi-run (`config.toml`, `known_hosts`, `secrets.key`, `active`).
- Mantenha semântica one-shot; não embrulhe a CLI como sidecar de longa vida sem timeout de tunnel.
- Nunca embuta segredos vivos ou `secrets.key` em camadas de imagem.


## Suporte a shells
- Completions bash, zsh, fish e PowerShell são gerados sob demanda.
- Prefira arrays argv explícitos em runtimes de agentes a eval de string shell.
- Prefira flags stdin de segredo a embutir senhas no histórico do shell.


## Paths de arquivo e XDG
- Resolva o vencedor com `ssh-cli vps doctor --json` (inclui campos `secrets_*`).
- Sobrescreva só em testes via `--config-dir` ou `SSH_CLI_HOME`.
- Mantenha `known_hosts`, `active` e `secrets.key` como irmãos de `config.toml`.
- Escrita atômica + flock protege processos one-shot concorrentes no mesmo config.


## Performance por alvo
- Cold start Linux é a baseline sob 100 ms.
- Builds musl podem trocar características de alocador; habilite `musl-allocator` quando precisar.
- RTT de rede domina operações remotas em todo SO.


## Agentes validados por plataforma
- Hosts Linux são a superfície primária de validação de subprocessos de agentes.
- macOS e Windows seguem o mesmo contrato CLI e schemas JSON.
- Agentes em container devem preservar exit codes e separação stdout/stderr.
- Tracing padrão é error para o stderr do agente ficar sem prosa INFO salvo `RUST_LOG` ou `-v`.
- Parseie contratos de máquina só do stdout; trate tracing em stderr como log fora de contrato.
- Helpers de E2E SSH real ficam em `scripts/e2e_real_ssh.sh` (anti-leak; só local).
