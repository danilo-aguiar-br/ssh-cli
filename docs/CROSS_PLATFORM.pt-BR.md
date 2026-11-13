# Multiplataforma


> Fuja da cola SSH específica de cada sistema com um único binário Rust portátil.

- Leia este documento em [Inglês (en)](CROSS_PLATFORM.md)
- Linha de produto: 0.5.6


## A dor que você já conhece
- Invólucros de daemon em Node mudam conforme o gerenciador de pacotes e a versão do ambiente de execução do host
- Scripts SSH feitos só em shell vazam segredos no histórico e na lista de processos
- As convenções de caminho divergem entre as pastas de configuração do Linux, do macOS e do Windows
- Agentes precisam de uma superfície de comando que termina depois de cada execução, em qualquer sistema


## Matriz de suporte
### Alvos
- Seis dos oito alvos abaixo são nomeados por `Cross.toml`, `scripts/dist_multiarch.py` ou `scripts/check_cross_targets.py`, `x86_64-unknown-linux-gnu` é a compilação do host e `aarch64-apple-darwin` está listado só para dizer que nada o cobre
- Cada linha afirma só o que um script deste repositório cobre
- Reconfira as linhas de checagem de tipos com `python3 scripts/check_cross_targets.py`

| Alvo | Plataforma | Situação | Coberto por | Tamanho do binário de release |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | Linux x86_64 gnu | Suportado, alvo principal | Compilação do host, suíte completa de testes e `scripts/check_all_gates.py` | 9048976 bytes |
| `x86_64-unknown-linux-musl` | Linux x86_64 musl | Suportado (compilação cruzada) | Imagem do `Cross.toml` e `scripts/dist_multiarch.py`, mais `--features musl-allocator` quando necessário | Não medido |
| `aarch64-unknown-linux-musl` | Linux arm64 musl | Suportado (compilação cruzada) | Imagem do `Cross.toml` e `scripts/dist_multiarch.py` | Não medido |
| `aarch64-unknown-linux-gnu` | Linux arm64 gnu | Suportado (compilação cruzada) | Imagem do `Cross.toml` e `scripts/dist_multiarch.py` | Não medido |
| `x86_64-pc-windows-msvc` | Windows x86_64 | Suportado (checagem de tipos) | Gate `cross-targets` com `--no-default-features` | Não medido |
| `aarch64-pc-windows-msvc` | Windows arm64 | Suportado (checagem de tipos) | Gate `cross-targets` com `--no-default-features` | Não medido |
| `x86_64-apple-darwin` | macOS Intel | Suportado (checagem de tipos) | Gate `cross-targets` com `--no-default-features` | Não medido |
| `aarch64-apple-darwin` | macOS Apple Silicon | Não coberto por nenhum gate | Nada neste repositório compila nem checa este alvo | Não medido |

- Tamanho do binário: os 9048976 bytes foram medidos com `stat` no binário instalado de uma compilação intermediária da 0.5.6 do alvo principal, feita com o perfil `release` de tamanho mínimo, com sha256 `a4c2079731cbbd8dc4a0d212baa87e904d5d507ff458f6726de96599cd728536`
- A medição vale só para este host e esta compilação
- Nada nesta árvore compila os outros sete alvos, então o tamanho deles não foi medido e nenhum é afirmado
- Checagem de tipos significa que `cargo check` passa, e os binários de Windows e macOS nunca foram executados por esses gates
- A compilação com `--no-default-features` deixa de fora as features `ssh-real` e `tls`, então checa os tipos sem `russh`, `rustls` e `aws-lc-sys`, a crate que compila C e exigiria um compilador C cruzado
- Nenhuma execução nativa em Windows ou macOS é registrada por esses gates

### Ambientes

| Ambiente | Situação | Notas |
| --- | --- | --- |
| WSL1 / WSL2 | Suportado | Detectado por `WSL_DISTRO_NAME`, `WSL_INTEROP` ou `/proc/version`, então trate como Linux |
| Contêineres | Suportado | Monte a pasta de configuração ou passe `--config-dir`, e o `doctor` informa `local.runtime.is_container` |
| Termux (Android) | Melhor esforço | Detectado por `TERMUX_VERSION` ou `TERMUX_APK_RELEASE` como `local.runtime.is_termux`, e nenhum gate compila um alvo Android |
| WASM / WASI | Não distribuído | O `russh` precisa de sockets reais, então não é alvo do produto |
| Automação de navegador | Não se aplica | Sem descoberta de Chrome ou chromedriver, porque o produto é só SSH |


## Criptografia e transporte (G-TLS)
- A mesma pilha criptográfica em todo sistema suportado: SSH-2 pelo `russh` 0.65 sobre TCP simples com o backend `aws-lc-rs`
- O caminho opcional SSH-over-TLS, em que TLS é Transport Layer Security, cobre o TLS mútuo (mTLS) e o Automatic Certificate Management Environment (ACME), e usa `rustls` só com o provider `aws_lc_rs`
- O `Cargo.toml` pede `rustls` 0.23.25, e `ssh-cli tls provider` imprime o piso de segurança como `min_rustls` 0.23.18
- Compilar a partir do código-fonte exige um compilador C em todo sistema, porque o `aws-lc-sys` 0.47 compila os fontes C e assembly do AWS-LC
- No Windows x86 e x86_64 essa compilação também exige o NASM, a menos que `AWS_LC_SYS_PREBUILT_NASM=1` escolha os objetos pré-compilados que vêm dentro do `aws-lc-sys`
- A troca por um backend criptográfico em Rust puro foi tentada na 0.5.6 e revertida, porque o crates.io não tem backend em Rust puro para o `russh` 0.65 nem para o `instant-acme` 0.8
- Pilhas proibidas: OpenSSL, `native-tls`, `libssh2-sys` e `ring`
- Chaves de host: arquivo de confiança no primeiro uso (TOFU) na pasta de configuração da plataforma (`directories`, e XDG Base Directory ou XDG no Linux)
- A compressão do canal SSH é só `none` (sem zlib) em todas as plataformas
- Os gates de release desta política são scripts locais: `cargo deny` e `scripts/check_advisory_hits.py`


## Linux
- Prefira `cargo install ssh-cli --locked` em `~/.cargo/bin`
- Espere a configuração XDG em `~/.config/ssh-cli/` por padrão
- A interface de linha de comando (CLI) cria uma pasta de configuração nova com modo `0700` e grava `config.toml` e `secrets.key` com modo `0600`
- A cifragem em repouso padrão guarda os blocos cifrados no `config.toml`
- Mantenha uma cópia offline de `secrets.key`
- Com `--use-keyring` ou `secrets init --keyring` a chave primária vai para o Secret Service pelo zbus, que precisa de um barramento de sessão, e um cofre que não responde a uma escrita sai 69


## macOS
- O mesmo caminho de instalação pelo cargo do Linux
- Limpe a quarentena com `xattr -d com.apple.quarantine` quando o Gatekeeper bloquear o binário
- Espere a configuração nas pastas de suporte a aplicativos do macOS resolvidas pelo `directories` 6
- O cofre do sistema para a chave primária é o Keychain, escolhido por `--use-keyring` ou por `secrets init --keyring`, e os dois gravam na mesma entrada com escopo de perfil
- Só o `x86_64-apple-darwin` passa pela checagem de tipos de um gate, e o `aarch64-apple-darwin` não é coberto (veja a tabela de alvos)


## Windows
- Instale pelo Rustup e pelo cargo numa toolchain suportada (versão mínima suportada do Rust, ou MSRV, 1.89, que o `russh` 0.65 exige)
- O cofre do sistema para a chave primária é o Credential Manager, alcançado pelos mesmos `--use-keyring` e `secrets init --keyring`
- Na partida o binário define a code page 65001 (UTF-8) e habilita `ENABLE_VIRTUAL_TERMINAL_PROCESSING` para as cores ANSI no conhost e no PowerShell 5.1
- Use os completamentos do PowerShell gerados por `ssh-cli completions powershell`
- Um `~/` no início do caminho da chave se expande a partir de `HOME` e cai para `USERPROFILE`, mas caminhos explícitos são mais seguros
- As pastas de configuração e de projeto vêm do `directories`
- Use `ssh-cli vps doctor --json` para ver qual pasta de configuração venceu (`local.config_path` e `local.layer`)
- Cada componente de caminho local tem no máximo 255 bytes
- Um caminho total perto do legado `MAX_PATH` (260) é recusado, a menos que use o prefixo estendido `\\?\`
- Nomes novos no registro de VPS são ASCII estrito: letras, dígitos, ponto, hífen e sublinhado, começando por alfanumérico, com no máximo 255 bytes
- Os dispositivos reservados do Windows (`CON`, `NUL`, `COM1`, …) também são recusados, enquanto a leitura de um `config.toml` antigo continua tolerante
- O Windows não tem chmod, então o modo `0700` não existe ali e a pasta de configuração depende da lista de controle de acesso (ACL) do perfil
- O `doctor` omite `local.config_dir_permissions` no Windows e informa `local.config_dir_exposed` como `false`
- O modo `0700` e a checagem de `getuid` só passaram pela checagem de tipos para Windows e macOS e nunca foram executados lá
- Rode os scripts como `py -3 scripts/<name>.py`, porque os scripts `.sh` não existem mais


## Modos da pasta de configuração
- No Unix uma única função cria a pasta de configuração, e cada nível novo nasce com modo `0700`
- Uma pasta XDG padrão existente é apertada para `0700` só quando não é link simbólico e pertence ao usuário atual
- Qualquer outra pasta existente, como um `--config-dir` que você criou, nunca recebe chmod
- Essa pasta grava um aviso de log (visível com `-v`) e aparece como `local.config_dir_exposed` em `ssh-cli doctor --json`
- Leia com `ssh-cli doctor --json --select local.config_dir_exposed`, porque o `--select` também reduz o objeto único do `doctor`
- `local.config_dir_permissions` guarda o modo octal de quatro dígitos, como `0700`, e fica ausente fora do Unix
- `--config-dir something.toml` faz da pasta pai a pasta de configuração de todos os arquivos: `secrets.key`, `known_hosts`, `active`, `lang` e os arquivos TLS ficam ao lado do `config.toml`
- Quando essa pasta pai não existe, a CLI cria a pasta pai e nunca uma pasta chamada `something.toml`
- `vps export -o new/deep/e.json` cria as pastas que faltam com modo `0700` e nunca altera uma pasta existente
- `known_hosts.lock`, `config.toml.lock` e `lang` nascem com modo `0600`, sem janela antes de um chmod
- Chaves privadas ACME e mTLS são criadas com modo `0600`


## Texto, nomes e UTF-8
- O modo texto escapa caracteres de controle e bidirecionais como texto no estilo `\u{1b}` em todo sistema: mensagens, erros, `list`, `show`, `health-check`, `doctor` e saídas em lote
- `sftp ls` e `sftp stat` também escapam `\n` e `\t` em nomes remotos
- O JSON mantém os caracteres originais porque é para máquina, e a saída remota do `exec` segue crua por desenho
- Nomes novos de host aceitam só letras ASCII, dígitos, ponto, hífen e sublinhado em todo sistema
- Um nome antigo com espaço, acento ou caractere de controle ainda carrega, e `vps rename '<old name>' <new name>` o repara
- O `doctor` lista em `local.invalid_host_names` os nomes antigos que falham no conjunto estrito de caracteres
- `tls mtls import --name` recusa caracteres de controle, de formatação e bidirecionais com exit 64, enquanto `serviço-1` continua aceito
- Arquivos e configuração usam UTF-8, e o console do Windows muda para a code page 65001 na partida
- Desligue as cores com `--no-color` em todo sistema, porque ele vence qualquer detecção de terminal


## Contêineres
- Copie o binário para imagens distroless ou de distribuição sem Node
- Persista a pasta de configuração (ou passe `--config-dir`) para guardar os hosts entre execuções (`config.toml`, `known_hosts`, `secrets.key`, `active`)
- Mantenha a semântica de execução única
- Não empacote a CLI como processo auxiliar de longa duração sem timeout de tunnel
- Nunca embuta segredos reais ou `secrets.key` nas camadas da imagem
- Os marcadores de ambiente (`/.dockerenv`, `/run/.containerenv`, `KUBERNETES_SERVICE_HOST`, `container=`) aparecem como `local.runtime.is_container` em `vps doctor --json`


## Diagnóstico do ambiente
- `ssh-cli vps doctor --json` embute `local.runtime`: `os`, `arch`, `is_wsl`, `is_container`, `is_ci`, `is_termux`, `sandbox` (`flatpak` | `snap` | null)
- A instalação sob Flatpak ou Snap emite um aviso na partida, porque o sistema de arquivos e o cofre podem ser outros
- A detecção nunca chama o shell (`uname` e `systemd-detect-virt` não são usados)


## Processos externos (G-PROC)
- O código do produto nunca cria processos filhos locais: SSH, Secure Copy Protocol (SCP) e tunnels rodam dentro do processo pelo `russh`, então o host do agente não precisa de `ssh`, `scp` nem `ssh-keygen` do OpenSSH
- A elevação remota empacota `sudo`/`su` + `sh -c` no host alvo só pelo canal SSH, com aspas e com as senhas no stdin do canal
- Isso não é `Command` local
- Opcional na compilação: `git` em `build.rs` para o HEAD curto
- O hash resolve na ordem env, git, `.commit_hash`, `unknown`
- O arquivo guarda `<short hash> <version>` e só vale quando essa versão é a do pacote, então um arquivo velho dá `unknown`
- `Stdio` explícito null ou piped
- Sem shell
- Opcional nos testes: fixtures `ssh-keygen` para chaves OpenSSH reais
- Pule quando ausente
- Toolchain MSRV 1.89 ≥ 1.77.2 (CVE-2024-24576 BatBadBut)
- O produto não invoca `.bat` nem `.cmd`
- Job Objects e grupos de processos para árvores locais: não se aplica
- Comandos remotos recusam bytes NUL antes do empacotamento
- CR e LF em várias linhas são aceitos


## Suporte a shell
- Completamentos pelo `clap_complete`: Bash, Zsh, Fish, PowerShell, Elvish (`ssh-cli completions <shell>`)
- O Nushell não está no enum padrão `clap_complete::Shell`
- Gere por ferramenta externa se precisar
- Prefira listas argv explícitas nos ambientes de agente a avaliar texto de shell
- Prefira as flags de segredo por stdin a embutir senhas no histórico do shell


## Caminhos de arquivo e XDG
- Descubra o vencedor com `ssh-cli vps doctor --json` (inclui os campos `secrets_*`)
- Sobrescreva com `--config-dir` (o produto não lê `SSH_CLI_HOME`)
- Mantenha `known_hosts`, `active` e `secrets.key` como arquivos irmãos de `config.toml`
- Escritas atômicas e flock protegem processos de execução única concorrentes na mesma configuração
- Outro processo segurando `config.toml.lock` faz a CLI esperar `--lock-wait-ms` (padrão 5000) e depois sair 69
- Sem `.env` do produto para segredos, idioma ou pasta de configuração
- `locale show|set|clear` funciona em todas as plataformas (XDG e ProjectDirs pelo `directories`)
- O material de `tls mtls/*` e `tls acme/*` fica na pasta `tls/` da configuração da plataforma (XDG no Linux)


## Superfície completa da CLI (igual em todo sistema)
- O inventário de comandos é idêntico em todo sistema suportado
- Descubra em tempo de execução com `ssh-cli commands` e `ssh-cli schema`

| Superfície | Comandos |
| --- | --- |
| Registro de VPS | `ssh-cli vps add` `ssh-cli vps list` `ssh-cli vps remove` `ssh-cli vps rename` `ssh-cli vps edit` `ssh-cli vps show` `ssh-cli vps path` `ssh-cli vps doctor` `ssh-cli vps export` `ssh-cli vps import` |
| Sessão | `ssh-cli connect` |
| Exec remoto | `ssh-cli exec` `ssh-cli sudo-exec` `ssh-cli su-exec` |
| SCP | `ssh-cli scp upload` `ssh-cli scp download` (somente arquivos regulares) |
| SFTP | `ssh-cli sftp upload` `ssh-cli sftp download` `ssh-cli sftp ls` `ssh-cli sftp mkdir` `ssh-cli sftp rmdir` `ssh-cli sftp rm` `ssh-cli sftp stat` `ssh-cli sftp rename` |
| Rede | `ssh-cli tunnel` (local, `--reverse`, `--socks5`, `--remote-socket`) `ssh-cli health-check` |
| Segredos | `ssh-cli secrets status` `ssh-cli secrets init` `ssh-cli secrets reencrypt` |
| Descoberta | `ssh-cli completions` `ssh-cli commands` `ssh-cli schema` `ssh-cli doctor` (alias root de `ssh-cli vps doctor`) |
| Locale | `ssh-cli locale show` `ssh-cli locale set` `ssh-cli locale clear` |
| TLS | `ssh-cli tls provider` `ssh-cli tls paths` |
| TLS mTLS | `ssh-cli tls mtls list` `ssh-cli tls mtls import` `ssh-cli tls mtls show` `ssh-cli tls mtls remove` |
| TLS ACME | `ssh-cli tls acme account create` `ssh-cli tls acme account show` `ssh-cli tls acme issue` `ssh-cli tls acme complete` `ssh-cli tls acme status` `ssh-cli tls acme list` |

### Flags globais (todas as plataformas)
- Saída e logs: `--lang`, `-v`/`-vv`/`-vvv` (escopo na crate G2/G14, `RUST_LOG` do ambiente ignorado), `-q`, `--no-color`, `--output-format`, `--json`
- Redução do payload: `--select` (alias `--fields`), `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- Controle de execução: `--no-input`, `--dry-run`, `--lock-wait-ms`, `--timeout`, `--max-concurrency` (distribuição pela frota 1..=64), `--fail-fast`, `--scp-file-concurrency`
- Config e segredos: `--config-dir`, `--disable-sudo`, `--replace-host-key`, `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`
- O `--disable-sudo` desativa `sudo-exec` e `su-exec` só naquela invocação, e o alias `--disableSudo` é aceito antes do subcomando e depois de todo subcomando exceto `vps add` e `vps edit`
- Em `vps add` e `vps edit` a mesma flag grava a escolha no registro do host, venha ela antes ou depois do subcomando, e `vps edit --enable-sudo` a desfaz
- `vps add` e `vps edit` têm o seu próprio `--disable-sudo`, então o clap recusa o alias `--disableSudo` depois deles com exit 2
- Use a linha de produto 0.5.3 como piso para o SSH File Transfer Protocol (SFTP) em toda plataforma (integridade de upload G1)
- Verifique o destino com checksum
- Sem `.env` do produto: só XDG, ProjectDirs, `--config-dir` e flags da CLI

### Flags por folha (idênticas em toda plataforma)
- Esta tabela é a mesma lista que `ssh-cli commands --json` publica, então flag que existe aqui existe em todo lugar
- `vps add` e `vps edit` compartilham as flags de auth e de registro: `--host`, `--port`, `--user`, `--password`, `--password-stdin`, `--key`, `--key-passphrase`, `--key-passphrase-stdin`, `--sudo-password`, `--sudo-password-stdin`, `--su-password`, `--su-password-stdin`, `--tag`, `--timeout`
- `vps add` e `vps edit` também carregam os limites de comando e as flags TLS: `--max-chars`, `--max-command-chars`, `--max-output-chars`, `--tls`, `--tls-sni`, `--tls-client-cert`, `--tls-client-key`, `--use-agent`, `--agent-socket`
- Só `vps add` tem `--check`, que roda um `health-check` logo após registrar o host
- Só `vps edit` tem as flags de limpeza `--clear-tags`, `--clear-sudo-password`, `--clear-su-password`, `--no-key`, `--no-agent-socket`, `--no-tls`, `--no-tls-sni`, `--no-tls-client-cert`, `--no-tls-client-key` e `--enable-sudo`
- `vps list` recebe `--tag`, `vps import` recebe `--file` e `--allow-incomplete`, e `vps export` recebe `--output`, `--include-secrets` e `--i-understand-secrets-on-stdout`
- `exec`, `sudo-exec` e `su-exec` recebem `--all`, `--hosts`, `--tags`, `--use-active`, `--use-agent`, `--agent-socket`, `--step`, `--description` e `--timeout`
- `sudo-exec` acrescenta `--sudo-password` e `--sudo-password-stdin`, e `su-exec` acrescenta `--su-password` e `--su-password-stdin`
- `health-check` recebe `--all`, `--hosts`, `--use-active`, `--use-agent`, `--agent-socket` e `--timeout`
- `scp upload` e `scp download` recebem `--src` e `--dest`, mais `--all`, `--hosts` e o trio de auth
- `sftp upload` e `sftp download` recebem `--recursive`, `--src` e `--dest`, mais `--all` e `--hosts`
- `sftp ls`, `sftp mkdir`, `sftp rmdir`, `sftp rm`, `sftp stat` e `sftp rename` recebem só o trio de auth e `--timeout`
- `tunnel` recebe `--reverse`, `--socks5`, `--remote-socket`, `--bind`, `--i-accept-network-exposure` e `--timeout-ms`
- `secrets init` recebe `--keyring` e `--force`, e `--keyring` seleciona o mesmo store do global `--use-keyring`
- `vps doctor` e o `doctor` da raiz recebem `--hosts` e `--probe-ssh`
- `tls mtls import` recebe `--cert`, `--key` e `--name`
- `tls acme account create` recebe `--contact`, `--staging` e `--force`, e `tls acme issue` recebe `--domain`, `--staging` e `--print-challenge`
- As formas longas `--verbose` e `--quiet` são aceitas em todo lugar onde as curtas `-v` e `-q` são, e `--help` funciona em toda folha


## Portabilidade dos modos de tunnel
- Os quatro modos de `tunnel` existem em todo sistema suportado: encaminhamento local padrão, `--reverse`, `--socks5` e `--remote-socket`
- `--remote-socket <PATH>` nomeia um socket Unix no servidor, não nesta máquina, então o cliente pode rodar no Windows: localmente ele só fala TCP e pede ao servidor que alcance o socket
- O que decide o `--remote-socket` é o suporte do servidor à extensão `direct-streamlocal@openssh.com`, detectada no protocolo em vez de adivinhada pela plataforma local
- Cliente Windows contra servidor OpenSSH Linux funciona
- Cliente Linux contra servidor sem a extensão não
- `--remote-socket` valida somente o que não pode ser válido em host POSIX algum: caminho vazio, caminho relativo ou byte NUL embutido, cada um com exit 64
- Checar a existência local seria errado, porque o caminho pertence a um sistema de arquivos que esta máquina não vê
- `--socks5` abre uma escuta TCP local e fala RFC 1928 CONNECT sem autenticação, então não depende de plataforma alguma
- `--reverse` pede ao servidor que escute
- Se o servidor permite esse bind é governado por `GatewayPorts` e `AllowTcpForwarding` dele, que são configurações do servidor e não traço da plataforma do cliente
- O `--bind` local é lido como endereço IP pelo clap em todo sistema, então um erro de digitação falha com exit 2 antes de qualquer negociação SSH
- Qualquer bind roteável exige `--i-accept-network-exposure` em toda plataforma
- O bind remoto sob `--reverse` é comparado como texto, e não lido como IP (a RFC 4254 admite nomes e o texto vazio), então um erro de digitação ali dá exit 64 da guarda de exposição
- Os dois comportamentos são idênticos em toda plataforma, porque nenhuma das checagens consulta o sistema do host
- Note também que `--bind` é aceito e então ignorado sob `--reverse` em toda plataforma


## Portabilidade SCP / SFTP
- SCP é somente arquivos regulares em toda plataforma (sem transferência recursiva de diretório)
- Use `sftp upload|download --recursive` para árvores (sem seguir link simbólico)
- Downloads com falha ou em andamento usam um caminho irmão terminado em `.ssh-cli.partial` e depois renomeiam para o lugar (padrão atômico igual em toda plataforma)
- O upload envia em blocos de 32 KiB em todo sistema (evita carregar o arquivo inteiro na RAM)
- A preservação de mtime e modo segue o estilo OpenSSH com o `-p` remoto e a linha `T`
- No Unix as APIs locais de permissão aplicam os modos
- No Windows os bits de permissão podem não bater com o octal do Unix, então não assuma fidelidade completa à ACL POSIX
- Use a linha de produto 0.5.3 como piso para SFTP em toda plataforma (integridade de upload G1)
- Verifique o destino com checksum
- A matriz de SSH real E01–E21 (E10–E14 SCP, E17/E18 checksum e árvore SFTP, E19/E20 redução do payload, E21 rotação do keyring num profile vivo) em `scripts/e2e_real_ssh.py` é validada principalmente em hosts Linux
- `--local-sshd` é só Unix, e no Windows o script para com `FAIL E00: --local-sshd is Unix-only`
- Prefira um `sshd` local ou uma VPS descartável
- Nunca execute tempestades de falha de autenticação em hosts de produção (banimentos fail2ban)


## Desempenho por alvo
- Toda chamada inicia, responde e termina em todo sistema, então nenhum processo fica residente entre as chamadas
- Compilações musl podem mudar o comportamento do alocador de memória
- Habilite `musl-allocator` quando necessário
- O tempo de ida e volta da rede (RTT) domina as operações remotas em todo sistema


## Agentes validados por plataforma
- Hosts Linux são a superfície principal de validação para agentes que rodam a CLI como subprocesso
- macOS e Windows seguem o mesmo contrato da CLI e os mesmos JSON schemas
- Os aliases de descoberta da raiz funcionam em todo sistema: `ssh-cli doctor` (alias de `vps doctor`) e `ssh-cli schema` (catálogo embutido ou o corpo de um schema)
- Os contratos JSON (evento `scp-transfer`, `tunnel_listening`, flags de autenticação em tunnel e health-check) são idênticos em todo sistema
- Veja AGENTS.pt-BR.md e docs/schemas/
- O `--bind` do tunnel tem padrão `127.0.0.1` (loopback) em toda plataforma
- Sobrescreva só quando for expor a escuta de propósito
- Agentes em contêiner devem preservar os exit codes e a separação entre stdout e stderr
- O tracing padrão fica no nível error para manter o stderr do agente livre de texto INFO, a menos que `-v`/`-vv`/`-vvv` seja passado (escopo na crate, `RUST_LOG` do ambiente é ignorado)
- Leia os contratos de máquina só do stdout
- Trate o tracing em stderr como log fora do contrato
- Os envelopes de erro JSON usam stderr quando o modo JSON está ativo
- Os auxiliares de E2E com SSH real ficam em `scripts/e2e_real_ssh.py` (sem vazamento de segredo, só local, E01–E21, nunca tempestades de autenticação em produção)
- O carimbo de `ssh-cli --version` (versão do `Cargo` + hash git + `-dirty` opcional) é igual em todo sistema
- Sem `.env` do produto: só XDG, `--config-dir` e flags da CLI


## Matriz local de vários sistemas (G-E2E-18)
- Código do produto: módulos `src/platform/{linux,macos,windows}.rs`, e o comportamento de cada sistema é validado localmente
- Binários de várias arquiteturas via `scripts/dist_multiarch.py` (flags `--targets`, `--out-dir`, `--jobs`, `py -3` no Windows e `Cross.toml`) quando o `cross` ou um linker cruzado local com sysroot estiverem instalados
- Todo gate é um script local rodado à mão, e os mantenedores validam principalmente em Linux
- Confira as notas de tamanho de caminho e de socket do agente no macOS e no Windows antes de criar a tag de um release


## Notas de release
- Só as entradas da 0.5.6 descrevem o comportamento atual, as entradas mais antigas são história
### 0.5.6 — keyring e contrato
- O keyring compila um cofre real nos três sistemas: Keychain no macOS, Credential Manager no Windows e Secret Service por zbus nos demais alvos unix
- Um cofre indisponível sai 69 em toda plataforma em vez de responder sucesso, então uma máquina sem barramento de sessão informa a verdade
- A versão mínima de Rust é 1.89 nos três alvos, exigida pelo russh 0.65
- O binário release Linux x86_64 instalado mede 9048976 bytes com sha256 `a4c2079731cbbd8dc4a0d212baa87e904d5d507ff458f6726de96599cd728536`
- macOS e Windows seguem só com checagem de tipos, porque a bateria roda em Linux
### 0.5.6 — fechamento da auditoria
- A âncora TLS descarta a cadeia destituída `emSign ECC Root CA - G3`, e `scripts/check_advisory_hits.py` reprova a bateria quando um advisory alcança o lockfile
- Payload de objeto único aceita `--select` e `--truncate-content`, e um array nu reduzido relata a contagem em stderr
- Nenhuma mudança específica de plataforma: os mesmos três alvos compilam e passam os mesmos gates
- A migração para criptografia em Rust puro foi revertida, então o provider TLS continua `aws_lc_rs` e compilar do código-fonte ainda exige um compilador C
### 0.5.6 — edição e rename
- `vps rename <NAME> <NEW_NAME>` e `vps edit <NAME> --name <NEW>` mudam o nome do host nativamente, com as credenciais intactas
- O registro sela cada segredo sem dado associado, então ele continua legível sob o nome novo
- Cada gravação sela de novo com nonce novo, o que é normal e nunca toca o texto em claro
- Isso substitui o desvio export/edit/import, que descartava a autenticação por agente em silêncio porque o export não carregava campo de agente
- Oito flags de limpeza removem o campo em vez de sobrescrevê-lo
- `--dry-run` cobre os dois verbos
- O lock da configuração espera um `--lock-wait-ms` limitado e depois sai 69 em vez de travar
- `commands --json` publica todo argumento
- O modo texto escapa caracteres de controle, de formatação e bidirecionais em todo sistema, e o JSON os mantém crus
- Um nome novo de host aceita só letras ASCII, dígitos, ponto, hífen e sublinhado, e um `config.toml` antigo com outros nomes ainda carrega
- Uma pasta de configuração que a CLI cria nasce com modo `0700`, e uma existente é apertada só quando é a pasta XDG padrão
- `--config-dir something.toml` faz da pasta pai a pasta de configuração de todos os arquivos

### 0.5.5 — release de segurança de designação de alvo
- Corrige GAP-SSH-EXEC-ARGC-001: `exec`/`sudo-exec`/`su-exec` decidiam o host alvo pela contagem de posicionais
- Assim `exec <HOST> --step <CMD>` executava o nome do host como binário remoto no host que `connect` selecionara por último
- QUEBRA DE COMPATIBILIDADE: um único posicional agora é erro de uso
- Alcance o marcador ativo com a nova `--use-active <COMMAND>`
- O envelope de host único ganhou `target_resolved` e `target_source`, então ligue os consumidores novos a esses campos
- `host_resolved` e `host_source` são os aliases de leitura da 0.5.5
- O envelope também ganhou `active_fallback`, e o `doctor` passa a reportar `local.active_vps`

### 0.5.4 — release de segurança e de uso nativo por agentes
- Corrige uma negação de serviço (DoS) remota antes da autenticação no banner SSH (A1)
- Impede que bits setuid enviados pelo servidor caiam no arquivo baixado (A3)
- Fecha a janela de leitura pública em chaves privadas ACME/mTLS (A2)
- Adiciona flags de redução de payload aplicadas antes da serialização: `--select`, `--filter`, `--limit`, `--sort`, `--dedupe-by`, `--count-only`, `--truncate-content`, `--max-output-bytes`
- QUEBRA DE COMPATIBILIDADE: falha parcial em vários hosts agora sai com exit 1 (era 65)
- QUEBRA DE COMPATIBILIDADE: `--bind` fora do loopback exige `--i-accept-network-exposure`
- Novo evento `tunnel_closed`
