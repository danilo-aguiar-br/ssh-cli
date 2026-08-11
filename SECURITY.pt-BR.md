# Política de Segurança

- Leia este documento em [English](SECURITY.md).

Linha de produto atual: **0.5.x**.

## Versões suportadas
- A tabela abaixo lista quais versões do ssh-cli recebem patches de segurança.
- Usuários em linhas sem suporte devem atualizar para a linha atual.

| Versão | Status | Patches de segurança |
| --- | --- | --- |
| 0.5.x | Suportada | Sim, **linha atual** — porém 0.5.0–0.5.4 carregam o GAP-SSH-EXEC-ARGC-001; atualize para **0.5.5+** |
| 0.4.x | Suportada | Críticas/altas quando viável; prefira **0.5.x** |
| 0.3.x | Limitada | Só críticas quando viável; no crates.io **0.3.9** o SCP era inoperante — atualize para **0.5.3+** para transferências |
| 0.2.x | Limitada | Só críticas quando viável |
| 0.1.x | Sem suporte | Sem patches |
| < 0.1 | Sem suporte | Sem patches |

## Correções de segurança na 0.5.5
- **GAP-SSH-EXEC-ARGC-001 — o host alvo era escolhido pela *contagem* de posicionais, não pelo conteúdo (CWE-441).** `exec`, `sudo-exec` e `su-exec` aceitavam um ou dois posicionais: dois significavam `<VPS> <COMANDO>`, um significava "comando contra o host que o `connect` marcou por último". `--step` é flag de append, não posicional, então `exec <HOST> --step <CMD>` colapsava no ramo de um posicional. O nome do host passava a ser executado como *binário* remoto (exit remoto **127**) e todo `--step` seguinte rodava no host marcado — um host que o operador nunca digitou. Como os passos seguintes tinham sucesso, o processo saía **0**, então o operador lia sucesso enquanto units systemd, pacotes e regras de firewall caíam em outro lugar. Isso é defeito de segurança e não de ergonomia: numa CLI de acesso remoto a dimensão crítica não é *o que* executa, é *onde*. Não exigia atacante — argv ambíguo de operador legítimo bastava, e é por isso que supera um bug que precisa de peer hostil.
- **Corrigido tornando a ambiguidade inexprimível, não adivinhando a intenção.** Um posicional solitário agora é erro de uso (exit **64**) em toda superfície remota, `health-check` inclusive. Designe o alvo explicitamente com `<VPS> <COMANDO>`, com um seletor (`--all`, `--hosts a,b`, `--tags prod`), ou com o marcador de propósito via a nova `--use-active`; optar por ele sem marcador presente é exit **66**. A falha agora ocorre no parse, antes de qualquer socket abrir, então o raio de dano do engano é uma mensagem de uso em vez de auditoria em dois hosts.
- **GAP-SSH-EXEC-ENVELOPE-002 — o envelope de host único não nomeava host, então o desvio acima era indetectável pelo stdout.** O caminho de frota já emitia um `name` por host enquanto o caminho single-host — justamente o que aceitava o fallback silencioso — não emitia nenhum, então a assimetria corria ao contrário do risco. Todo envelope agora carrega `target_resolved`, `target_source` (`argv` | `active_marker` | `selector`) e `active_fallback`, também no envelope de **erro** assim que o alvo resolve, onde ficam ausentes em vez de vazios. `host_resolved` e `host_source` acompanham como aliases de leitura da 0.5.5; vincule consumidores novos aos nomes canônicos. O `doctor` reporta `local.active_vps`, então o marcador passa a ser legível por máquina — antes da 0.5.5 nenhum comando o expunha e a única leitura era abrir o arquivo XDG.
- Atualize para a **0.5.5** por causa dos dois. Nenhum precisa de atacante remoto, e o primeiro reescrevia silenciosamente a máquina errada.

## Correções de segurança na 0.5.4
- **A1 — abort disparável remotamente ao logar o banner pré-autenticação.** O banner SSH é entrada remota que chega *antes* da autenticação e é retida para log de diagnóstico. O teto de log `AUTH_BANNER_MAX_CHARS` (512) já existia (G-SSH-14); o defeito era o corte ser aplicado por **índice de byte**, que entra em pânico sempre que cai no meio de um caractere multibyte. O perfil de release usa `panic = "abort"`, então não há unwind: um par não autenticado conseguia matar o processo com um banner contendo um único caractere não-ASCII bem posicionado, derrubando um fan-out multi-host inteiro. O truncamento agora ocorre em fronteira de caractere. Note o limite residual da correção: o russh materializa o banner completo e o entrega ao callback, então o cliente ainda retém a string inteira em memória — o que A1 limita é o pânico, não a alocação. É a classe de maior severidade aqui porque não exige credencial alguma.
- **A2 — janela de leitura pública em chaves privadas ACME e mTLS.** O material de chave em XDG `tls/acme/` e `tls/mtls/` era escrito com `std::fs::write`, que cria o arquivo legível por qualquer usuário local e só depois restringe o modo — deixando uma janela que um co-inquilino podia ler. As escritas passam pelo helper compartilhado que *cria* em `0600`, então a janela não existe. Corrigir o modo depois apenas encurta a exposição; nunca a elimina.
- **A3 — bits de elevação enviados pelo servidor caindo no arquivo baixado.** O header `C` do SCP e os atributos do SFTP são produzidos pelo *servidor*, então o modo que carregam não é confiável. Mascarar com `0o7777` permitia que um par hostil colocasse setuid, setgid ou sticky no arquivo local. Os modos de entrada agora são mascarados com `SFTP_PERM_MASK_UNTRUSTED` (`0o0777`), então só triplas `rwx` simples atravessam a rede na entrada. A ponta remota não tem autoridade para decidir o que é privilegiado na sua máquina.
- **Bind fora do loopback agora exige reconhecimento.** `tunnel --bind` fora do loopback exige `--i-accept-network-exposure` (G-TUN-R13). Antes disso, `--bind 0.0.0.0` publicava em silêncio o serviço remoto encaminhado para toda a rede local. Sob `--reverse` a mesma flag protege o endereço de bind do **servidor**, porque é essa a ponta exposta naquele sentido.
- Atualize para **0.5.4** por causa das quatro. A1 e A3 são alcançáveis por um par remoto; A2 é alcançável por um co-inquilino local.

## Reportar uma vulnerabilidade
- Reporte problemas de segurança preferencialmente via GitHub Security Advisories no repositório público `ssh-cli` (canal privado preferido).
- Use o e-mail daniloaguiarbr@proton.me apenas como fallback quando o reporte privado no GitHub estiver indisponível.
- Nunca abra issue, pull request ou discussion pública no GitHub para relatos relacionados a segurança.
- Inclua reprodução mínima, versões afetadas e comportamento esperado versus atual.
- Inclua detalhes de ambiente como SO, arquitetura e versão do rustc.
- Inclua uma estimativa de severidade CVSS **v4.0** quando possível (CVSS 3.1 aceito como fallback) para acelerar a triagem.
- Omita ou mascare credenciais vivas de todo anexo e trecho de log.


## SLA de resposta
- A triagem de cada advisory começa em até 72 horas úteis após o envio.
- O reconhecimento inicial é enviado na mesma janela de 72 horas.
- Você recebe um identificador de caso e um maintainer responsável.
- Atualizações de progresso são compartilhadas no mínimo a cada 7 dias até resolução ou divulgação pública.


## SLA de correção por severidade CVSS
- Severidade crítica (CVSS 9.0 a 10.0) recebe patch em até 7 dias corridos após triagem validada.
- Severidade alta (CVSS 7.0 a 8.9) recebe patch em até 14 dias corridos após triagem validada.
- Severidade média (CVSS 4.0 a 6.9) recebe patch em até 30 dias corridos após triagem validada.
- Severidade baixa (CVSS 0.1 a 3.9) recebe patch em até 90 dias corridos após triagem validada.
- Correções publicadas incluem entrada no CHANGELOG e GitHub Security Advisory quando a linha ainda for suportada.


## Política de divulgação
- Divulgação coordenada é o padrão para vulnerabilidades validadas.
- A divulgação pública é adiada até haver release corrigida ou até expirar a janela de Fix SLA com motivo documentado.
- Pesquisadores podem publicar após acordo mútuo ou após o fim da janela coordenada.


## Política de atualização de segurança
- Correções de segurança chegam primeiro na linha minor atual.
- Backports para linhas minor antigas só ocorrem quando impacto e esforço justificam o trabalho.
- Usuários devem atualizar com `cargo install ssh-cli --locked --force` após um release de segurança.


## Hall da fama
- Pesquisadores de segurança que pedirem crédito público são listados aqui após a divulgação coordenada.
- A lista começa vazia na linha de ownership atual sob `danilo-aguiar-br`.


## Modelo de ameaça (produto — G-SEC-10)
- **Fronteira de confiança:** argv da CLI, env, arquivos XDG (`config.toml`, `secrets.key`, `known_hosts`) e o servidor SSH remoto são entradas **não confiáveis**. Apenas a identidade local do processo e o keyring do SO (quando habilitado) são material de confiança mais alta.
- **Ativos:** senhas SSH / passphrases / secrets sudo-su (memória + at-rest), fingerprints de host key (TOFU), chave primária AEAD (`secrets.key` / keyring) a capacidade de executar shell remoto / SCP / port forwards locais e — como o GAP-SSH-EXEC-ARGC-001 provou — a **identidade do host alcançado**, já que o mesmo argv rodando na máquina errada é uma brecha mesmo com todas as credenciais intactas.
- **Adversários considerados:** (1) argv/env hostil de CLI/agente; (2) servidores SSH maliciosos ou MITM (mudança de host key); (3) usuários locais co-residentes lendo arquivos world-readable ou listagens de processo; (4) crates transitivas comprometidas (supply chain); (5) **argv ambíguo de operador ou agente legítimo** — sem atacante nenhum. O GAP-SSH-EXEC-ARGC-001 era dessa classe: uma linha de comando bem formada que o produto resolvia para um host que ninguém nomeou. Trate "qual máquina isto alcança" como pergunta controlada por atacante mesmo quando não existe atacante.
- **Fora de escopo:** isolamento multi-tenant SaaS, XSS/CSRF de browser, container escape, mitigações Spectre de kernel além do isolamento normal de processo, e superfície de daemon de longa duração (esta CLI é **one-shot**).
- **Controles:** validação tipada nas fronteiras CLI/import/path; `SecretString` + zeroize; ChaCha20-Poly1305 at-rest; TOFU com comparação constant-time de fingerprint; packing de shell com escape de aspas simples e secrets no stdin do canal; sem shell-out local para SSH; `deny.toml` + `cargo deny check` **local** (gates de produto não exigem GitHub Actions); `overflow-checks` em release; `unsafe` documentado só em bordas de console/sinal do SO.
- **Gatilho de revisão:** reler este modelo em qualquer mudança que adicione protocolos de rede, processos long-lived, novos stores de secret ou `unsafe` fora de wrappers FFI de SO.

## Política de transporte e crypto (G-TLS)
- **Caminho padrão:** transporte **SSH-2** via `russh` em TCP puro, crypto **aws-lc-rs** (`ssh-real`). Host auth **TOFU** (`known_hosts` sob XDG).
- **SSH-over-TLS opcional:** com `tls = true` no VPS (ou `--tls` em `vps add`/`edit`), o cliente faz handshake **rustls** (SNI + mTLS opcional) e só então o protocolo SSH. Piso: **rustls ≥ 0.23.18** (nunca a janela de CVE do Acceptor, 0.23.13–0.23.17).
- **Provider:** o binário chama `CryptoProvider::install_default` com **`aws_lc_rs` apenas** (uma vez, antes do runtime Tokio). Libraries usam `ClientConfig::builder` / default do processo. Dual provider **`ring` banido** no `deny.toml`.
- **Proibido:** `native-tls`, OpenSSL, `libssh2-sys`, uso de produto de `ring`. PEM via `rustls-pki-types` (sem `rustls-pemfile`).
- **mTLS:** PEMs de certificado e chave de cliente sob XDG `tls/mtls/<nome>/` (`ssh-cli tls mtls import`, `ssh-cli tls mtls list`, `ssh-cli tls mtls show`, `ssh-cli tls mtls remove`) ou caminhos absolutos no registro do VPS.
- **ACME:** Let's Encrypt via `instant-acme` (DNS-01, dois passos para agente: `tls acme issue --print-challenge` → publicar TXT → `tls acme complete`). Conta e certificados sob XDG `tls/acme/` (0o600).
- Compressão de canal SSH **desabilitada** (o cliente prefere apenas `none`) para reduzir side-channels de compressão quando segredos cruzam o canal.
- Gates locais (`cargo deny check`, `cargo tree`, testes residuais) — sem GitHub Actions obrigatório (G-TLS-11).


## Mapa STRIDE (componentes críticos — G-SECDEV-03)

| Componente | S | T | R | I | D | E | Controles principais |
| --- | --- | --- | --- | --- | --- | --- | --- |
| argv / env / stdin de secrets da CLI | spoof de args de agente | tamper de flags | negar qual host executou | vazamento via argv | DoS stdin grande | elev via sudo-exec | clap tipado + stdin → `SecretString` + teto de tamanho + designação explícita de alvo com `target_resolved` / `target_source` em todo envelope (0.5.5) |
| XDG `config.toml` / `secrets.key` | forjar host | reescrever secrets | — | leitura co-tenant | truncar/corromper | — | 0o600 + flock + AEAD at-rest |
| TOFU `known_hosts` | MITM de host | trocar fingerprint | — | oráculo de fingerprint | starve de lock | — | flock + comparação constant-time + `--replace-host-key` explícito |
| Sessão SSH (russh) | servidor falso | MITM / troca de key | — | sniff de canal | hang / flood | shell remoto | TOFU + timeouts + sem shell-out local |
| Packing remoto (`sh -c` sudo/su) | — | injetar metachar | — | secret em argv | — | elev não intencional | escape de aspas simples + senha no stdin do canal |
| Fan-out multi-host | — | reordenar jobs | — | vazamento cross-host em logs | exaustão de recurso | — | orçamento `Semaphore` + cancel + JSON redacted |
| Supply chain (deps) | typosquat | crate maliciosa | — | backdoor | DoS de build | — | `deny.toml` + `cargo deny` local + `--locked` + bans TLS no lockfile |

**Riscos residuais aceitos (explícitos):** (1) password em argv ainda é parseável (com warning; preferir stdin); (2) o host remoto após auth é totalmente confiado para o comando one-shot pedido pelo operador — premissa que só vale depois que o alvo é designado explicitamente, e é por isso que a 0.5.5 tornou a designação obrigatória; (3) co-tenant com mesmo UID pode ler memória do processo — isolamento do SO, não desta CLI; (4) ameaças de classe Spectre/side-channel de CPU fora de escopo para ferramenta userland one-shot.

## Logging e vazamento de segredos (G2 / G14)
- **Nunca** confie em `RUST_LOG` ambiente para debug de produto — o produto **ignora** essa variável (não é store de config).
- Use apenas as flags graduadas: `-v` (info) / `-vv` (debug) / `-vvv` (trace) via `ArgAction::Count`.
- O filtro é **sempre crate-scoped** (`warn,ssh_cli=*`) — nunca `debug`/`trace` global bare. Isso impede o russh (e outras deps) de despejar senhas ou material de canal no stderr.
- Mesmo `-vvv` **não** habilita dumps de canal cifrado do russh; só o crate `ssh_cli` (mais deps em nível warn) está na allowlist.
- Prefira **0.5.3+** para que agentes não habilitem acidentalmente debug global que vazava credenciais em linhas antigas.

## Boas práticas para usuários
- Prefira autenticação por chave privada a senha quando o host permitir.
- Prefira `--password-stdin`, `--sudo-password-stdin` e `--su-password-stdin` a segredos em argv (password em argv emite warning em stderr em **0.5.2+**).
- Prefira flags stdin de senha em runs de agentes; evite embutir segredos vivos no histórico do shell.
- **Cifragem at-rest por padrão** (ChaCha20-Poly1305): na primeira gravação de segredo, cria `secrets.key` (0o600) ao lado do `config.toml`, salvo opt-out.
- Controle de secrets é só CLI/XDG: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring`, ou XDG `secrets.key`. Sem store de produto em `.env`.
- Ordem da chave: flags CLI → keyring (quando `--use-keyring`) → XDG `secrets.key`. `SSH_CLI_SECRETS_KEY` / `SSH_CLI_SECRETS_KEY_FILE` são **rejeitadas fail-closed** (não são store).
- CLI: `ssh-cli secrets status|init|reencrypt` (nunca imprime a master-key); `--json` emite `secrets-init` / `secrets-reencrypt` sem material de chave.
- Opt-out só para testes: `--allow-plaintext-secrets` (sem store em env).
- **Nunca** logue master-key, senhas ou segredos decifrados.
- `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout` (guarda contra dump acidental de segredos no stdout).
- `vps export` redacted limpa segredos; secret vazio serializa como `""` e **nunca** como blob `sshcli-enc:…` (EXP-001 / 0.4.2).
- Mantenha `config.toml` com mode `0600` e restrinja locais de backup.
- Revise erros de mudança de host key TOFU antes de usar `--replace-host-key`.
- Nunca faça commit de inventários de host com segredos vivos.
- Nunca faça commit de sidecars MCP locais (ex.: `.setting.cyber/`), config Grok MCP (`~/.grok/config.toml`), XDG `config.toml` / `secrets.key` / `known_hosts`, ou arquivos de env E2E no repositório.
- E2E SSH real (G-E2E-05) deve manter credenciais fora da árvore: prefira `--config-dir` / hosts pré-cadastrados, ou `$HOME/.grok/config.toml` via `--from-grok-config`; env harness-only `SSH_CLI_E2E_*` é aceito só pelo script (não é runtime de produto). Offline / sem lab → **SKIP** exit 0. O script recusa config grok sob a raiz do repo.
- Senhas de demo na documentação pública são só placeholders (ex.: `demo-password-not-real`); nunca as reutilize em hosts reais.
- Desabilite elevação com `--disable-sudo` quando o workflow não deve escalar.
- Mantenha `tunnel --bind` no loopback a menos que uma segunda máquina realmente precise alcançar o forward; qualquer bind roteável exige `--i-accept-network-exposure`, que é o reconhecimento dessa exposição e não uma formalidade.
- Trate `tunnel --remote-socket /var/run/docker.sock` como delegação de privilégio: encaminhar socket do Docker ou do systemd entrega à porta local toda a autoridade que aquele socket concede no host remoto.
- Prefira `tunnel --socks5` no loopback a um forward fixo roteável quando o agente precisar de vários destinos; um bind local reconhecido é melhor que N expostos.
- Rode apenas comandos one-shot; nunca espere um daemon SSH de longa duração desta CLI.
- Instale com `--locked` para evitar drift de re-resolve crypto.
- Prefira a linha atual **0.5.3+** para integridade SFTP (G1 fechou truncamento de upload a 0 bytes), verbosidade crate-scoped (G2/G14 — sem dump de senha do russh), piso de supply-chain (russh 0.62.5), wire SCP/SFTP funcional, classificação ACME permanente (`invalidContact` → exit **64**) e máscara de export redacted `***` (`FIXED_MASK`).
- Honestidade histórica: **0.4.1** corrigiu export redacted de secret vazio (nunca `sshcli-enc:` de vazio) e exit 0 pós-bind do tunnel.
