# Multiplataforma

> Fuja de cola SSH específica de SO com um binário Rust portátil.

- Leia este documento em [inglês](CROSS_PLATFORM.md).
- Linha de produto: 0.5.2.


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
| macOS | Suportado | arm64 + x86_64; pode precisar remover quarentena do Gatekeeper |
| Windows | Suportado | UTF-8 CP 65001 + VT no boot; config via ProjectDirs |
| WSL1 / WSL2 | Suportado | Detectado via `WSL_*` / `/proc/version`; trate como Linux |
| Containers | Suportado | Monte o dir de config ou passe `--config-dir`; doctor reporta `runtime.is_container` |
| Termux (Android) | Melhor esforço | Detectado via `TERMUX_*`; bionic quando o target existir |
| WASM / WASI | Não distribuído | `russh` precisa de sockets reais; fora do produto |
| Automação de browser | N/A | Sem descoberta Chrome/chromedriver (produto só SSH) |


## Crypto / transporte (G-TLS)
- Mesma stack crypto em todo SO suportado: **SSH-2** via `russh` + **aws-lc-rs** (não TLS/HTTPS, não OpenSSL, não `native-tls`, não `rustls` de produto).
- Host keys: arquivo TOFU no dir de config da plataforma (`directories` / XDG no Linux).
- Compressão de canal SSH é **somente `none`** (sem zlib) em todas as plataformas.
- Gates de release desta política são **locais** (`cargo deny`, testes residuais) — sem workflow cloud obrigatório.


## Linux
- Prefira `cargo install ssh-cli --locked` em `~/.cargo/bin`.
- Espere config XDG em `~/.config/ssh-cli/` por omissão.
- Garanta mode 0600 após o primeiro save de `config.toml` e `secrets.key`.
- Cifragem at-rest padrão guarda blobs em `config.toml`; mantenha backup offline de `secrets.key`.


## macOS
- Mesmo path de install cargo que no Linux.
- Limpe quarentena com `xattr -d com.apple.quarantine` quando o Gatekeeper bloquear o binário.
- Espere config sob application support/project dirs macOS resolvidos por `directories` 6.
- Backend de keyring para a primary-key é opcional via `--use-keyring` após `secrets init --keyring`.


## Windows
- Instale via Rustup e cargo em toolchain suportada (MSRV 1.85.0).
- No arranque o binário define code page **65001 (UTF-8)** e habilita
  **virtual terminal processing** para cores ANSI em conhost / PowerShell 5.1.
- Use completions PowerShell de `ssh-cli completions powershell`.
- Prefira arquivos de chave com paths explícitos em vez de atalhos de home Unix.
- Dirs de config/projeto vêm de `directories`; use `vps doctor --json` para ver o vencedor.
- Componentes de path local limitados a 255 bytes; path total perto do legado
  `MAX_PATH` (260) é rejeitado salvo prefixo estendido `\\?\`.
- Nomes no registry VPS rejeitam devices reservados do Windows (`CON`, `NUL`, `COM1`, …).


## Containers
- Copie o binário para imagens distroless ou distro sem Node.
- Persista o dir de config (ou passe `--config-dir`) para memória multi-run (`config.toml`, `known_hosts`, `secrets.key`, `active`).
- Mantenha semântica one-shot; não empacote a CLI como sidecar de longa duração sem timeout de tunnel.
- Nunca embuta segredos live ou `secrets.key` em layers de imagem.
- Marcadores de runtime (`/.dockerenv`, `/run/.containerenv`, `KUBERNETES_SERVICE_HOST`,
  `container=`) aparecem como `runtime.is_container` em `vps doctor --json`.


## Diagnóstico de runtime
- `ssh-cli vps doctor --json` embute o objeto `runtime`:
  `os`, `arch`, `is_wsl`, `is_container`, `is_ci`, `is_termux`, `sandbox`
  (`flatpak` | `snap` | null).
- Instalação sob Flatpak/Snap emite **warning** no boot (filesystem/keyring podem diferir).
- Detecção nunca faz shell-out (`uname`, `systemd-detect-virt` não são usados).


## Processos externos (G-PROC)
- **Código de produto em runtime nunca spawna filhos locais.** SSH, SCP e tunnels
  usam Rust puro (`russh`) — sem OpenSSH `ssh`/`scp`/`ssh-keygen` no host do agente.
- Elevação remota empacota `sudo`/`su` + `sh -c` **no host alvo** só via canal SSH
  (aspas; senhas no stdin do canal). Isso não é `Command` local.
- Build opcional: `git` em `build.rs` para HEAD curto (fallback env / `.commit_hash`
  / `unknown`). `Stdio` explícito null/piped; sem shell.
- Testes opcionais: fixtures `ssh-keygen` para chaves OpenSSH reais; skip se ausente.
- Toolchain MSRV **1.85.0** ≥ **1.77.2** (CVE-2024-24576 BatBadBut). Produto não
  invoca `.bat`/`.cmd`. Job Objects / process groups para árvores locais: **N/A**.
- Comandos remotos rejeitam bytes **NUL** antes do packing; CR/LF multi-linha ok.


## Suporte a shell
- Completions via `clap_complete`: **Bash, Zsh, Fish, PowerShell, Elvish**
  (`ssh-cli completions <shell>`).
- Nushell não está no enum padrão `clap_complete::Shell`; gere via tooling externo se precisar.
- Prefira arrays argv explícitos em runtimes de agente a eval de string shell.
- Prefira flags stdin de segredo a embutir senhas no histórico do shell.


## Paths de arquivo e XDG
- Resolva o vencedor com `ssh-cli vps doctor --json` (inclui campos `secrets_*`).
- Sobrescreva só em testes via `--config-dir` (o produto não lê `SSH_CLI_HOME`).
- Mantenha `known_hosts`, `active` e `secrets.key` como arquivos irmãos de `config.toml`.
- Escritas atômicas + flock protegem processos one-shot concorrentes no mesmo config.


## Portabilidade SCP
- SCP é somente arquivos regulares em toda plataforma (sem transferência recursiva de diretório; sem subsistema SFTP).
- Downloads com falha ou em andamento usam path irmão terminando em `.ssh-cli.partial`, depois rename no lugar (padrão atômico agnóstico de plataforma).
- Upload faz stream em blocos de 32 KiB em todo SO (evita carregar o arquivo inteiro na RAM).
- Preserve de mtime/mode segue estilo OpenSSH com remoto `-p` / linha `T`; em Unix APIs locais de permissão aplicam modes; no Windows bits de permissão podem não bater com octal Unix — não assuma fidelidade POSIX ACL completa.
- Matriz real-SSH E01–E16 (E10–E14 SCP) em `scripts/e2e_real_ssh.sh` é validada principalmente em hosts Linux; prefira `sshd` local / VPS throwaway. Nunca execute tempestades de falha de autenticação em hosts de produção (banimentos fail2ban).


## Performance por alvo
- Cold start Linux é a baseline sob alvo de 100 ms.
- Builds musl podem trocar características de allocator; habilite `musl-allocator` quando necessário.
- RTT de rede domina operações remotas em todo SO.


## Agentes validados por plataforma
- Hosts Linux são a superfície principal de validação para runs de subprocesso de agente.
- macOS e Windows seguem o mesmo contrato CLI e JSON schemas.
- Aliases root de descoberta funcionam em todo SO: `ssh-cli doctor` (alias de `vps doctor`) e `ssh-cli schema` (catálogo embarcado / um corpo de schema).
- Contratos JSON (`event` scp-transfer, `tunnel_listening`, flags de auth em tunnel/health) são idênticos em todo SO; veja AGENTS.pt-BR.md e docs/schemas/.
- Tunnel `--bind` tem padrão `127.0.0.1` (loopback) em toda plataforma; sobrescreva só ao expor o listener de propósito.
- Agentes em container devem preservar exit codes e separação stdout/stderr.
- Tracing padrão é nível error para manter stderr do agente livre de prosa INFO salvo `-v` (`RUST_LOG` ambiente é ignorado).
- Parseie contratos de máquina só do stdout; trate tracing em stderr como log fora de contrato; envelopes de erro JSON usam stderr quando o modo JSON está ativo.
- Helpers de E2E SSH real ficam em `scripts/e2e_real_ssh.sh` (anti-leak; só local; E01–E16; nunca tempestades de auth em produção / política fail2ban).
- O carimbo de `ssh-cli --version` (versão Cargo + hash git + `-dirty` opcional) é agnóstico de SO.


## Matriz multi-OS local (G-E2E-18)
- Código de produto: módulos `src/platform/{linux,macos,windows}.rs` — comportamento multi-OS é só local.
- Binários multi-arch locais via `scripts/dist_multiarch.sh` (e `Cross.toml`) quando toolchains cross / Docker estiverem instalados.
- **Sem CI cloud obrigatória de produto no GitHub Actions** — mantenedores validam principalmente em Linux; cheque notas de path length / agent socket em macOS e Windows antes de taggear um release.
- **Não** reintroduza `.github/workflows` para CI de produto (política: CLI one-shot, sem CI cloud de produto).
