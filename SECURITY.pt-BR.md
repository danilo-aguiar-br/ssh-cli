# Política de Segurança


[![Política de Segurança](https://img.shields.io/badge/security-policy-007AFF)](#reportar-uma-vulnerabilidade)

- Leia este documento em [Inglês (en)](SECURITY.md)


## Versões suportadas
- A linha de produto atual é a 0.5.x e a última release é a 0.5.6
- A tabela abaixo lista quais versões do ssh-cli recebem patches de segurança
- Usuários em linhas sem suporte devem atualizar para a linha atual

| Versão | Status | Patches de segurança |
| --- | --- | --- |
| 0.5.6 | Suportada | Sim, última release e linha atual |
| 0.5.x antes da 0.5.6 | Suportada | Sim, linha atual, porém 0.5.0 a 0.5.4 carregam o GAP-SSH-EXEC-ARGC-001 e nada antes da 0.5.6 tem as correções dela, então atualize para 0.5.6 |
| 0.4.x | Suportada | Críticas/altas quando viável, prefira 0.5.x |
| 0.3.x | Limitada | Só críticas quando viável, no crates.io 0.3.9 o SCP era inoperante, então atualize para 0.5.3+ para transferências |
| 0.2.x | Limitada | Só críticas quando viável |
| 0.1.x | Sem suporte | Sem patches |
| < 0.1 | Sem suporte | Sem patches |


## Vulnerabilidades conhecidas
- Toda versão do ssh-cli anterior à 0.5.6 resolve dependências que carregam os advisories abaixo, então atualize para a 0.5.6
- `russh` abaixo da 0.65.1 carrega RUSTSEC-2026-0492, RUSTSEC-2026-0495, GHSA-fw5c-8223-6h42 e GHSA-3hm4-p7h4-rvwp
- `rustix` 1.1.4 carrega RUSTSEC-2026-0486
- `webpki-roots` 1.0.9, `rustls-webpki` 0.103.15 e `webpki-root-certs` 1.0.9 carregam RUSTSEC-2026-0381, RUSTSEC-2026-0382 e RUSTSEC-2026-0383 (CVE-2026-111943)
- A causa, a versão corrigida e o limite residual de cada um estão em [Correções de segurança na 0.5.6](#correções-de-segurança-na-056)
- Reporte uma vulnerabilidade nova por [Reportar uma vulnerabilidade](#reportar-uma-vulnerabilidade)


## Correções de segurança na 0.5.6
### Advisories de dependência — russh, rustix, async-executor e webpki-roots
- O `russh` 0.62.5 agora é 0.65.1, o que fecha a RUSTSEC-2026-0492 (HIGH, callbacks do `client::Handler` disparados para um canal que nunca foi aberto) e a RUSTSEC-2026-0495 (o agent server nunca chamava `Agent::confirm`)
- O mesmo upgrade fecha a GHSA-fw5c-8223-6h42, em que um tamanho máximo de pacote zero travava toda escrita do canal, e a GHSA-3hm4-p7h4-rvwp, um estouro de pilha antes da autenticação disparado por certificado OpenSSH aninhado
- `Handler::check_server_key` agora recebe `&PublicKeyOrCertificate`, e o ramo de certificado é inalcançável porque `Preferred::DEFAULT` deixa `host_key_certificates` vazio, então a chave embutida passa pelo mesmo TOFU
- O `rustix` 1.1.4, marcado unsound pela RUSTSEC-2026-0486, agora é 1.1.6, e o `async-executor` 1.14.0 yanked agora é 1.14.1
- O `webpki-roots` 1.0.12 remove mais oito raízes destituídas: emSign Root CA - C1 e emSign ECC Root CA - C3 na 1.0.11, depois Buypass Class 2, Buypass Class 3, DigiCert Trusted Root G4, QuoVadis Root CA 1 G3, QuoVadis Root CA 3 G3 e Certigna Root CA na 1.0.12
- O gate `advisory-hits` lia `advisory.patched_versions` enquanto o `cargo audit` publica `versions.patched`, então dizia que não havia release corrigida para advisory que tinha, e agora lê o campo publicado

### Backend de keyring
- `ssh-cli secrets init --keyring` reportava sucesso e não persistia nada
- A causa era a crate, não a máquina: `keyring = "3"` resolvia 3.6.3, que não tem feature default e caía num store `mock` no próprio processo
- O mock guardava a chave na memória do processo que a escreveu, então a invocação seguinte respondia `encryption_active: false` e o agente ficava com um sucesso em que não podia confiar
- O backend agora é `keyring` 4.2.0 com a feature `v1`, que compila um store real nos três sistemas: Keychain no macOS, Credential Manager no Windows e Secret Service por zbus nos unix restantes
- O store é consultado por `Entry::store_status()` antes de toda escrita e de toda leitura, então um store indisponível sai 69 em vez de responder `Ok`
- Uma segunda causa foi medida: a flag do leaf `secrets init --keyring` nunca chegava ao estado de runtime, porque o dispatch repassava só o global `--use-keyring`
- Uma terceira causa foi medida: uma chave gravada por um `secrets init --keyring` anterior era invisível sem repetir a flag
- A entrada do keyring é escopada por um digest do diretório de config resolvido, porque o store do SO é da máquina enquanto `secrets.key` e `config.toml` são por perfil
- Sem esse escopo, uma execução isolada com `--config-dir` sobrescrevia a chave do perfil real, o que esta auditoria mediu acontecendo
- Uma quarta causa foi medida numa build intermediária da própria 0.5.6: `--use-keyring secrets init` respondia `key_source: xdg_file`, porque o subcomando recebia só a flag do leaf e a global nunca chegava ao caminho de gravação
- Uma quinta causa foi medida contra um profile vivo: `secrets init --keyring` num profile que já tinha `secrets.key` respondia `key_source: keyring` e cunhava uma segunda chave, deixando o arquivo responder primeiro em toda leitura seguinte
- O profile passava a ter duas chaves primárias diferentes, o `secrets status` respondia `xdg_file`, e todo segredo já cifrado falhava ao decriptar com a chave que o comando acabara de dizer que era a ativa
- A guarda agora pergunta pelo profile em vez do destino, então uma chave existente em qualquer dos dois armazenamentos recusa a escrita sem `--force`, e o `--force` remove a outra fonte para sobrar exatamente uma chave na rotação
- D2: `secrets init --force` não grava mais `secrets.key.bak`
- A cópia de segurança deixava material de chave em disco a cada rotação e violava a regra do projeto contra arquivo de backup
- O store é alcançado por uma thread de SO escopada, porque a fachada bloqueante do zbus estaciona a thread chamadora num reator `async-io` enquanto o processo roda um runtime Tokio multi-thread

### Limites residuais — backend de keyring
- O round-trip do keyring é provado no Linux com barramento de sessão, e macOS e Windows só chegam a type-check no gate `cross-targets`, então os stores Keychain e Credential Manager não são executados pela bateria desta release
- Uma máquina sem barramento de sessão sai 69 e os testes reportam SKIP, nunca PASS
- O digest do diretório de config é um identificador, não um segredo, então ele não resiste a um atacante local que já consegue ler os nomes de entrada do store

### Raízes TLS e gate de advisory — A1 e A4
- A1: a âncora TLS (Transport Layer Security) não confia mais numa autoridade destituída: `webpki-roots` 1.0.9, `rustls-webpki` 0.103.15 e `webpki-root-certs` 1.0.9 ainda carregavam a `emSign ECC Root CA - G3`, e as intermediárias eMudhra abaixo dela têm chave privada pública (Mozilla Bug 2083449, CVE-2026-111943, RUSTSEC-2026-0381/0382/0383)
- O cliente ACME (Automatic Certificate Management Environment) e toda conexão TLS de saída montam o `RootCertStore` a partir dessa lista, então o defeito alcançava os dois
- O mínimo está travado no manifesto, `webpki-roots = "1.0.10"`, e o lock resolve `rustls-webpki` 0.103.17 e `webpki-root-certs` 1.0.10
- A trava no manifesto existe de propósito: um update de lock sozinho podia voltar à versão comprometida em silêncio, e agora ele não compila
- A4: um gate novo compara o banco de advisory com a árvore resolvida a cada execução: `scripts/check_advisory_hits.py` roda `cargo audit --json` e falha em qualquer entrada de `vulnerabilities.list` ou em pacote yanked
- O gate `advisory-freshness` mais antigo media só a idade do clone, então um banco velho podia manter a bateria verde sobre uma CVE viva

### Limites residuais — raízes TLS e gate de advisory
- O gate `advisory-hits` recusa rodar quando o banco de advisory passa de sete dias, então uma máquina sem rede falha em vez de passar com dado velho
- A trava cobre as três crates do advisory medido: um advisory futuro contra outra crate continua dependendo da próxima atualização do banco

### Saída em texto, nomes de host e diretório de config
- Sequências de escape de terminal não chegam mais ao terminal do operador no modo texto: Um nome de host, uma mensagem de erro ou um campo guardado podia carregar ESC, BEL ou caracteres bidi, e o modo texto os imprimia crus
- `src/output/sanitize.rs` escapa caracteres de controle, de formatação e bidi como texto no estilo `\u{1b}` no funil único de texto: mensagens de sucesso, erro e aviso, `vps list`, `vps show`, `health-check`, `doctor` e as saídas em lote
- `sftp ls` e `sftp stat` também escapam `\n` e `\t` do nome remoto, então um servidor hostil não forja linhas extras numa listagem
- A saída JSON não muda, porque é para máquina e o `serde_json` já escapa ESC e BEL
- Nome de host novo tem charset estrito: `VpsName::try_new` aceita letras ASCII, dígitos, ponto, hífen e sublinhado, começando por letra ou dígito, de 1 a 255 bytes
- Ele guarda `vps add`, o destino de `vps rename` e `vps edit --name`, e `vps import`
- A leitura continua leniente, então um `config.toml` antigo com nome hostil ainda carrega, o `doctor` lista o nome escapado em `local.invalid_host_names` e o `vps rename` o repara
- Duas tabelas de host que normalizam (NFC) para o mesmo nome fazem o carregamento falhar com exit 65 e `config has two hosts named ...`, então um host nunca some em silêncio
- Nome que vira alvo (`exec`, `health-check`, `--hosts`, `connect`) é recusado com exit 64 quando carrega caractere de controle, de formatação ou bidi
- `tls mtls import --name` recusa os mesmos caracteres com exit 64 e não cria diretório, então um nome de identidade não vira nome de diretório hostil
- O diretório de config nasce privado: Todo nível de diretório que a CLI cria recebe modo `0700`, onde antes herdava `0755` do umask ao lado de um `config.toml` `0600`
- Um diretório existente só é apertado a `0700` quando é o diretório XDG padrão, não é symlink e pertence ao usuário atual
- Qualquer outro diretório existente nunca recebe chmod: o `doctor` informa `local.config_dir_exposed` e `local.config_dir_permissions`, e um aviso de log fica visível com `-v`
- `vps export -o` cria diretórios que faltam com `0700`, e `known_hosts.lock` e `lang` nascem `0600` sem janela antes de um chmod
- O RUSTSEC-2026-0235 é resolvido por remoção: O advisory vinha do `rkyv`, que só entrava no lock pelo `rust_decimal` opcional, então o módulo de dinheiro sem uso e essa dependência saíram
- A remoção é uma mudança BREAKING para quem usa a biblioteca e o changelog traz o caminho de migração
- O pacote publicado não leva mais os nove `scripts/*.sh`, o arquivo solto `json` da raiz nem arquivos `*.snap.new` pendentes, e o `exclude` do `Cargo.toml` os mantém fora
- O caminho do rename não tem etapa própria de recifragem, e a credencial nunca é redigitada: O `VpsRecord` sela os segredos sem dado associado, então o blob não fica preso ao nome do host
- O token de ciphertext muda, porque a gravação comum sela cada segredo de novo com nonce novo, e o que sobrevive é a credencial, legível sob o nome novo
- O desvio export, edit e import acabou: `vps export` redige segredo por projeto, então o ciclo obrigava o operador a redigitar uma credencial em argv ou em arquivo, que é a exposição que os caminhos de stdin existem para evitar
- Esse export também não carregava `use_agent` nem `agent_socket`, então host autenticado por agente voltava sem autenticação e falhava na conexão seguinte, não no import
- Limpar campo não pode deixar host inutilizável em silêncio: As oito flags de limpeza são recusadas com exit 64 quando o resultado ficaria sem método de auth primário, ou quando `--no-agent-socket` removeria o único endpoint numa plataforma sem padrão
- Registro malformado em repouso é pior que edição recusada, porque todo comando posterior herda o defeito
- A contenção é limitada, então um detentor de lock travado ou hostil não pendura a CLI: O lock do config espera `--lock-wait-ms` (padrão 5000, máximo 300000) com backoff full-jitter e depois sai 69 `unavailable`, que é retentável
- Antes disso, um processo vizinho segurando `config.toml.lock` mantinha todo escritor bloqueado indefinidamente, sem saída, sem exit e sem diagnóstico

### Limites residuais — saída em texto, nomes de host e diretório de config
- O stdout e o stderr de um `exec` remoto seguem crus por desenho, então um servidor hostil ainda pode escrever sequências de escape neles
- O JSON mantém crus todos os caracteres do conjunto escapado, por exemplo U+202E e ZWSP, porque JSON é para máquina
- O Windows não tem chmod, então o diretório de config depende da ACL do perfil
- As flags globais de modelagem reduziam só coleções, então não alcançavam o objeto do `doctor`: leia `local.config_dir_exposed` com `ssh-cli doctor --json | jaq .local.config_dir_exposed` (uma mudança posterior da 0.5.6 trouxe `--select` para objeto único, então este limite descreve as primeiras correções da 0.5.6)
- macOS e Windows só passam por type-check no gate `cross-targets`, então o modo `0700` não foi executado lá
- Um registro legado com caractere de controle em tag, host ou user ainda sai 65, porque só o NOME do host é tolerante no carregamento
- Os corpos de erro ficam em inglês por desenho, para o agente receber texto estável


## Correções de segurança na 0.5.5
- GAP-SSH-EXEC-ARGC-001: o host alvo era escolhido pela contagem de posicionais, não pelo conteúdo (CWE-441). `exec`, `sudo-exec` e `su-exec` aceitavam um ou dois posicionais: dois significavam `<VPS> <COMANDO>`, um significava "comando contra o host que o `connect` marcou por último"
- `--step` é flag de append, não posicional, então `exec <HOST> --step <CMD>` colapsava no ramo de um posicional
- O nome do host passava a ser executado como binário remoto (exit remoto 127) e todo `--step` seguinte rodava no host marcado: um host que o operador nunca digitou
- Como os passos seguintes tinham sucesso, o processo saía 0, então o operador lia sucesso enquanto units systemd, pacotes e regras de firewall caíam em outro lugar
- Isso é defeito de segurança e não de ergonomia: numa CLI de acesso remoto a dimensão crítica não é o que executa, é onde
- Não exigia atacante: argv ambíguo de operador legítimo bastava, e é por isso que supera um bug que precisa de peer hostil
- Corrigido tornando a ambiguidade inexprimível, não adivinhando a intenção: Um posicional solitário agora é erro de uso (exit 64) em toda superfície remota, `health-check` inclusive
- Designe o alvo explicitamente com `<VPS> <COMANDO>`, com um seletor (`--all`, `--hosts a,b`, `--tags prod`), ou com o marcador de propósito via a nova `--use-active`
- Optar por ele sem marcador presente é exit 66
- A falha agora ocorre no parse, antes de qualquer socket abrir, então o raio de dano do engano é uma mensagem de uso em vez de auditoria em dois hosts
- GAP-SSH-EXEC-ENVELOPE-002, o envelope de host único não nomeava host, então o desvio acima era indetectável pelo stdout: O caminho de frota já emitia um `name` por host enquanto o caminho single-host, justamente o que aceitava o fallback silencioso, não emitia nenhum, então a assimetria corria ao contrário do risco
- Todo envelope agora carrega `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) e `active_fallback`, também no envelope de erro assim que o alvo resolve, onde ficam ausentes em vez de vazios
- `host_resolved` e `host_source` acompanham como aliases de leitura da 0.5.5
- Vincule consumidores novos aos nomes canônicos
- O `doctor` reporta `local.active_vps`, então o marcador passa a ser legível por máquina: antes da 0.5.5 nenhum comando o expunha e a única leitura era abrir o arquivo XDG
- Atualize para a 0.5.5 por causa dos dois
- Nenhum precisa de atacante remoto, e o primeiro reescrevia silenciosamente a máquina errada


## Correções de segurança na 0.5.4
- A1, abort disparável remotamente ao logar o banner pré-autenticação: O banner SSH é entrada remota que chega antes da autenticação e é retida para log de diagnóstico
- O teto de log `AUTH_BANNER_MAX_CHARS` (512) já existia (G-SSH-14)
- O defeito era o corte ser aplicado por índice de byte, que entra em pânico sempre que cai no meio de um caractere multibyte
- O perfil de release usa `panic = "abort"`, então não há unwind: um par não autenticado conseguia matar o processo com um banner contendo um único caractere não-ASCII bem posicionado, derrubando um fan-out multi-host inteiro
- O truncamento agora ocorre em fronteira de caractere
- Note o limite residual da correção: o russh materializa o banner completo e o entrega ao callback, então o cliente ainda retém a string inteira em memória, o que A1 limita é o pânico, não a alocação
- É a classe de maior severidade aqui porque não exige credencial alguma
- A2, janela de leitura pública em chaves privadas ACME e mTLS (TLS mútuo): O material de chave em XDG `tls/acme/` e `tls/mtls/` era escrito com `std::fs::write`, que cria o arquivo legível por qualquer usuário local e só depois restringe o modo, deixando uma janela que um co-inquilino podia ler
- As escritas passam pelo helper compartilhado que cria em `0600`, então a janela não existe
- Corrigir o modo depois apenas encurta a exposição
- Nunca a elimina
- A3, bits de elevação enviados pelo servidor caindo no arquivo baixado: O header `C` do SCP e os atributos do SFTP (SSH File Transfer Protocol) são produzidos pelo servidor, então o modo que carregam não é confiável
- Mascarar com `0o7777` permitia que um par hostil colocasse setuid, setgid ou sticky no arquivo local
- Os modos de entrada agora são mascarados com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), então só triplas `rwx` simples atravessam a rede na entrada
- A ponta remota não tem autoridade para decidir o que é privilegiado na sua máquina
- Bind fora do loopback agora exige reconhecimento: `tunnel --bind` fora do loopback exige `--i-accept-network-exposure` (G-TUN-R13)
- Antes disso, `--bind 0.0.0.0` publicava em silêncio o serviço remoto encaminhado para toda a rede local
- Sob `--reverse` a mesma flag protege o endereço de bind do servidor, porque é essa a ponta exposta naquele sentido
- Atualize para 0.5.4 por causa das quatro
- A1 e A3 são alcançáveis por um par remoto
- A2 é alcançável por um co-inquilino local


## Reportar uma vulnerabilidade
- Reporte problemas de segurança preferencialmente via [GitHub Security Advisories](https://github.com/danilo-aguiar-br/ssh-cli/security/advisories/new) no repositório público `ssh-cli` (canal privado preferido)
- Use o e-mail daniloaguiarbr@proton.me apenas como fallback quando o reporte privado no GitHub estiver indisponível
- Nunca abra issue, pull request ou discussion pública no GitHub para relatos relacionados a segurança
- Inclua reprodução mínima, versões afetadas e comportamento esperado versus atual
- Inclua detalhes de ambiente como SO, arquitetura e versão do rustc
- Inclua uma estimativa de severidade CVSS v4.0 quando possível (CVSS 3.1 aceito como fallback) para acelerar a triagem
- Omita ou mascare credenciais vivas de todo anexo e trecho de log


## SLA de resposta
- A triagem de cada advisory começa em até 72 horas úteis após o envio
- O reconhecimento inicial é enviado na mesma janela de 72 horas úteis
- Você recebe um identificador de caso e um maintainer responsável
- Atualizações de progresso são compartilhadas no mínimo a cada 7 dias até resolução ou divulgação pública


## SLA de correção por severidade CVSS
- Severidade crítica (CVSS 9.0 a 10.0) recebe patch em até 7 dias corridos após triagem validada
- Severidade alta (CVSS 7.0 a 8.9) recebe patch em até 14 dias corridos após triagem validada
- Severidade média (CVSS 4.0 a 6.9) recebe patch em até 30 dias corridos após triagem validada
- Severidade baixa (CVSS 0.1 a 3.9) recebe patch em até 90 dias corridos após triagem validada
- Correções publicadas incluem entrada no CHANGELOG e GitHub Security Advisory quando a linha ainda for suportada


## Política de divulgação
- Divulgação coordenada é o padrão para vulnerabilidades validadas
- A divulgação pública é adiada até haver release corrigida ou até expirar a janela de Fix SLA com motivo documentado
- Pesquisadores podem publicar após acordo mútuo ou após o fim da janela coordenada


## Política de atualização de segurança
- Correções de segurança chegam primeiro na linha minor atual
- Backports para linhas minor antigas só ocorrem quando impacto e esforço justificam o trabalho
- Usuários devem atualizar com `cargo install ssh-cli --locked --force` após um release de segurança
- `python3 scripts/generate_sbom.py` grava um SBOM CycloneDX (software bill of materials, lista de componentes do software) em `target/sbom/ssh-cli.cdx.json` quando o `cargo-cyclonedx` está instalado
- Sem o `cargo-cyclonedx` o mesmo comando grava só um inventário do `cargo tree`, que não é um SBOM


## Hall da fama
- Pesquisadores de segurança que pedirem crédito público são listados aqui após a divulgação coordenada
- A lista começa vazia na linha de ownership atual sob `danilo-aguiar-br`


## Modelo de ameaça (produto: G-SEC-10)
- Fronteira de confiança: argv da CLI, env, arquivos XDG (`config.toml`, `secrets.key`, `known_hosts`) e o servidor SSH remoto são entradas não confiáveis
- Apenas a identidade local do processo e o keyring do SO (quando habilitado) são material de confiança mais alta
- Ativos: senhas SSH / passphrases / secrets sudo-su (memória + em repouso), fingerprints de host key (TOFU), chave primária AEAD (Authenticated Encryption with Associated Data) (`secrets.key` / keyring), a capacidade de executar shell remoto / SCP / port forwards locais e, como o GAP-SSH-EXEC-ARGC-001 provou, a identidade do host alcançado, já que o mesmo argv rodando na máquina errada é uma brecha mesmo com todas as credenciais intactas
- Adversários considerados: (1) argv/env hostil de CLI/agente
- (2) servidores SSH maliciosos ou MITM (mudança de host key)
- (3) usuários locais co-residentes lendo arquivos world-readable ou listagens de processo
- (4) crates transitivas comprometidas (supply chain)
- (5) argv ambíguo de operador ou agente legítimo: sem atacante nenhum
- O GAP-SSH-EXEC-ARGC-001 era dessa classe: uma linha de comando bem formada que o produto resolvia para um host que ninguém nomeou
- Trate "qual máquina isto alcança" como pergunta controlada por atacante mesmo quando não existe atacante
- Fora de escopo: isolamento multi-tenant SaaS, XSS/CSRF de browser, container escape, mitigações Spectre de kernel além do isolamento normal de processo, e superfície de daemon de longa duração (esta CLI é one-shot)
- Controles: validação tipada nas fronteiras CLI/import/path
- `SecretString` + zeroize
- ChaCha20-Poly1305 at-rest
- TOFU com comparação constant-time de fingerprint
- Packing de shell com escape de aspas simples e secrets no stdin do canal
- Sem shell-out local para SSH
- Saída de texto segura para o terminal: caracteres de controle, de formatação e bidi são escapados antes de qualquer texto chegar ao terminal (0.5.6)
- Charset estrito para nome de host e nome de identidade mTLS na criação (0.5.6)
- Diretório de config nasce `0700` (0.5.6)
- `deny.toml` + `cargo deny check` local
- `overflow-checks` em release
- `unsafe` documentado só em bordas de console/sinal do SO
- Gatilho de revisão: reler este modelo em qualquer mudança que adicione protocolos de rede, processos long-lived, novos stores de secret ou `unsafe` fora de wrappers FFI de SO


## Política de transporte e crypto (G-TLS)
- Caminho padrão: o transporte de rede do produto é SSH-2 via `russh` em TCP puro
- A crypto de fio usa o backend aws-lc-rs (`ssh-real`)
- A autenticação de host é TOFU (`known_hosts` sob XDG)
- SSH-over-TLS opcional: com `tls = true` no VPS (ou `--tls` em `vps add`/`edit`), o cliente faz handshake rustls (SNI + mTLS opcional) e só então o protocolo SSH
- Piso: rustls ≥ 0.23.18 (nunca a janela de CVE do Acceptor, 0.23.13–0.23.17), o valor `min_rustls` que `ssh-cli tls provider` imprime
- O manifesto pede mais que o piso: o `Cargo.toml` declara `rustls = { version = "0.23.25" }`
- Provider: o binário chama `CryptoProvider::install_default` com `aws_lc_rs` apenas (uma vez, antes do runtime Tokio)
- Libraries usam `ClientConfig::builder` / default do processo e nunca reinstalam
- Dual provider `ring` banido no `deny.toml`
- Proibido: `native-tls`, OpenSSL, `libssh2-sys`, uso de produto de `ring`
- PEM via `rustls-pki-types` (sem `rustls-pemfile`)
- mTLS: PEMs de certificado e chave de cliente sob XDG `tls/mtls/<nome>/` (`ssh-cli tls mtls import`, `ssh-cli tls mtls list`, `ssh-cli tls mtls show`, `ssh-cli tls mtls remove`) ou caminhos absolutos no registro do VPS
- ACME: Let's Encrypt via `instant-acme` (DNS-01, dois passos para agente: `tls acme issue --domain <DOMAIN> --print-challenge` → publicar TXT → `tls acme complete --domain <DOMAIN>`)
- Conta e certificados sob XDG `tls/acme/` (0o600)
- Prefira staging com `--staging` nos testes
- Compressão de canal SSH desabilitada (o cliente prefere apenas `none`) para reduzir side-channels de compressão quando segredos cruzam o canal
- Os gates de release e de supply-chain desta política são scripts locais (`cargo deny check`, `cargo tree`, testes residuais) (G-TLS-11)


## Mapa STRIDE (componentes críticos: G-SECDEV-03)

| Componente | S | T | R | I | D | E | Controles principais |
| --- | --- | --- | --- | --- | --- | --- | --- |
| argv / env / stdin de secrets da CLI | spoof de args de agente | tamper de flags | negar qual host executou | vazamento via argv | DoS stdin grande | elev via sudo-exec | clap tipado + stdin → `SecretString` + teto de tamanho + designação explícita de alvo com `target_resolved` / `target_source` em todo envelope (0.5.5) |
| XDG `config.toml` / `secrets.key` | forjar host | reescrever secrets | - | leitura co-tenant | truncar/corromper | - | arquivo 0o600 + diretório 0o700 (0.5.6) + flock + AEAD at-rest |
| TOFU `known_hosts` | MITM de host | trocar fingerprint | - | oráculo de fingerprint | starve de lock | - | flock + comparação constant-time + `--replace-host-key` explícito |
| Sessão SSH (russh) | servidor falso | MITM / troca de key | - | sniff de canal | hang / flood | shell remoto | TOFU + timeouts + sem shell-out local |
| Packing remoto (`sh -c` sudo/su) | - | injetar metachar | - | secret em argv | - | elev não intencional | escape de aspas simples + senha no stdin do canal |
| Fan-out multi-host | - | reordenar jobs | - | vazamento cross-host em logs | exaustão de recurso | - | orçamento `Semaphore` + cancel + JSON redigido |
| Saída de texto no terminal | nome de host hostil se passando por outro | ESC / bidi redesenha a tela | - | linhas escondidas ou forjadas | - | - | funil único de escape + charset estrito de nome (0.5.6) |
| Supply chain (deps) | typosquat | crate maliciosa | - | backdoor | DoS de build | - | `deny.toml` + `cargo deny` local + `--locked` + bans TLS no lockfile |

### Riscos residuais aceitos
- Password em argv ainda é parseável, com warning, então prefira stdin
- O host remoto após a autenticação é totalmente confiado para o comando one-shot pedido pelo operador, premissa que só vale depois que o alvo é designado explicitamente, e é por isso que a 0.5.5 tornou a designação obrigatória
- Co-tenant com mesmo UID pode ler memória do processo, e isso é isolamento do SO, não desta CLI
- Ameaças de classe Spectre e side-channel de CPU ficam fora de escopo para ferramenta userland one-shot


## Logging e vazamento de segredos (G2 / G14)
- Nunca: confie em `RUST_LOG` ambiente para debug de produto, o produto ignora essa variável (não é store de config)
- Use apenas as flags graduadas: `-v` (info) / `-vv` (debug) / `-vvv` (trace) via `ArgAction::Count`
- O filtro é sempre crate-scoped (`warn,ssh_cli=*`): nunca `debug`/`trace` global bare
- Isso impede o russh (e outras deps) de despejar senhas ou material de canal no stderr
- Mesmo `-vvv` não habilita dumps de canal cifrado do russh
- Só o crate `ssh_cli` (mais deps em nível warn) está na allowlist
- Prefira 0.5.3+ para que agentes não habilitem acidentalmente debug global que vazava credenciais em linhas antigas


## Boas práticas para usuários
- Prefira autenticação por chave privada a senha quando o host permitir
- Prefira `--password-stdin`, `--sudo-password-stdin` e `--su-password-stdin` a segredos em argv (password em argv emite warning em stderr em 0.5.2+)
- Prefira flags stdin de senha em runs de agentes
- Evite embutir segredos vivos no histórico do shell
- Cifragem at-rest por padrão (ChaCha20-Poly1305): na primeira gravação de segredo, cria `secrets.key` (0o600) ao lado do `config.toml`, salvo opt-out
- Controle de secrets é só CLI/XDG: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, ou XDG `secrets.key`
- Sem store de produto em `.env`
- Ordem da chave: `--secrets-key-file` → keyring (quando `--use-keyring`) → XDG `secrets.key` → keyring com a chave de um `secrets init --keyring` anterior
- `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` são rejeitadas fail-closed (não são store)
- CLI: `ssh-cli secrets status|init|reencrypt` (nunca imprime a master-key)
- `--json` emite `secrets-init` / `secrets-reencrypt` sem material de chave
- Opt-out só para testes: `--allow-plaintext-secrets` (sem store em env)
- Nunca: logue master-key, senhas ou segredos decifrados
- `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout` (guarda contra dump acidental de segredos no stdout)
- Mantenha `config.toml` com mode `0600` e restrinja locais de backup
- Revise erros de mudança de host key TOFU antes de usar `--replace-host-key`
- Nunca faça commit de inventários de host com segredos vivos
- Nunca faça commit de sidecars locais de ferramentas (ex.: `.setting.cyber/`), config do Grok (`~/.grok/config.toml`), XDG `config.toml` / `secrets.key` / `known_hosts`, ou os arquivos de senha entregues ao script E2E (`--password-file`, `--sudo-password-file`) no repositório
- E2E SSH real (G-E2E-05) deve manter credenciais fora da árvore: prefira `python3 scripts/e2e_real_ssh.py --local-sshd` (sshd sem privilégio no loopback, só Unix), ou as flags de lab `--host`, `--port`, `--user` e `--password-file` (mais `--sudo-password-file`, um caminho ou `-` para stdin), ou `$HOME/.grok/config.toml` via `--from-grok-config`
- O script aceita só flags e não lê variáveis de ambiente
- Offline / sem lab → SKIP exit 0. O script recusa config grok sob a raiz do repo
- Senhas de demo na documentação pública são só placeholders (ex.: `demo-password-not-real`)
- Nunca as reutilize em hosts reais
- Desabilite elevação com `--disable-sudo` quando o workflow não deve escalar
- Mantenha `tunnel --bind` no loopback a menos que uma segunda máquina realmente precise alcançar o forward
- Qualquer bind roteável exige `--i-accept-network-exposure`, que é o reconhecimento dessa exposição e não uma formalidade
- Trate `tunnel --remote-socket /var/run/docker.sock` como delegação de privilégio: encaminhar socket do Docker ou do systemd entrega à porta local toda a autoridade que aquele socket concede no host remoto
- Prefira `tunnel --socks5` no loopback a um forward fixo roteável quando o agente precisar de vários destinos
- Um bind local reconhecido é melhor que N expostos
- Rode apenas comandos one-shot
- Nunca espere um daemon SSH de longa duração desta CLI
- Instale com `--locked` para evitar drift de re-resolve crypto
- Leia a saída de máquina como JSON, porque o modo texto escapa caracteres de controle para o leitor humano
- Mantenha o diretório de config privado: A CLI nunca aplica chmod a um diretório existente que não seja o XDG padrão, então leia `local.config_dir_exposed` em `ssh-cli doctor --json` e rode `chmod 700` você mesmo quando for `true`
- Depois de atualizar para a 0.5.6, leia `local.invalid_host_names` em `ssh-cli doctor --json` e repare cada nome legado com `ssh-cli vps rename`
- Troque um host entre senha, chave e agente com `vps edit`, que limpa o segredo antigo, em vez do desvio export, edit e import
- Prefira a linha atual 0.5.3+ para integridade SFTP (G1 fechou truncamento de upload a 0 bytes), verbosidade crate-scoped (G2/G14: sem dump de senha do russh), piso de supply-chain (russh 0.65.1), wire SCP/SFTP funcional, classificação ACME permanente (`invalidContact` → exit 64) e a máscara de três asteriscos (`FIXED_MASK`) mostrada por `vps list` e `vps show`
- Honestidade histórica: 0.4.1 corrigiu export redigido de secret vazio (nunca `sshcli-enc:` de vazio) e exit 0 pós-bind do tunnel
- `vps export` redigido limpa segredos: `password` fica string vazia e os outros segredos ficam null
- Secret vazio serializa como `""` e nunca como blob `sshcli-enc:…` (EXP-001 / 0.4.2)
