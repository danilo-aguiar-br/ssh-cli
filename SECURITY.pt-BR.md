# Política de Segurança

- Leia este documento em [English](SECURITY.md).

Linha de produto atual: **0.5.x**.

## Versões suportadas
- A tabela abaixo lista quais versões do ssh-cli recebem patches de segurança.
- Usuários em linhas sem suporte devem atualizar para a linha atual.

| Versão | Status | Patches de segurança |
| --- | --- | --- |
| 0.5.x | Suportada | Sim, **linha atual** |
| 0.4.x | Suportada | Críticas/altas quando viável; prefira **0.5.x** |
| 0.3.x | Limitada | Só críticas quando viável; no crates.io **0.3.9** o SCP era inoperante — atualize para **0.5.1+** |
| 0.2.x | Limitada | Só críticas quando viável |
| 0.1.x | Sem suporte | Sem patches |
| < 0.1 | Sem suporte | Sem patches |

## Reportar uma vulnerabilidade
- Reporte problemas de segurança preferencialmente via GitHub Security Advisories no repositório público `ssh-cli` (canal privado preferido).
- Use o e-mail daniloaguiarbr@proton.me apenas como fallback quando o reporte privado no GitHub estiver indisponível.
- Nunca abra issue, pull request ou discussion pública no GitHub para relatos relacionados a segurança.
- Inclua reprodução mínima, versões afetadas e comportamento esperado versus atual.
- Inclua detalhes de ambiente como SO, arquitetura e versão do rustc.
- Inclua estimativa de severidade CVSS 3.1 quando possível para acelerar a triagem.
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


## Boas práticas para usuários
- Prefira autenticação por chave privada a senha quando o host permitir.
- Prefira `--password-stdin`, `--sudo-password-stdin` e `--su-password-stdin` a segredos em argv (password em argv emite warning em stderr em **0.5.1+**).
- Prefira flags stdin de senha em runs de agentes; evite embutir segredos vivos no histórico do shell.
- **Cifragem at-rest por padrão** (ChaCha20-Poly1305): na primeira gravação de segredo, cria `secrets.key` (0o600) ao lado do `config.toml`, salvo opt-out.
- Prefira flags CLI ao env para controle de secrets: `--allow-plaintext-secrets`, `--secrets-key-file`, `--use-keyring` (camadas env ainda funcionam quando as flags não estão setadas).
- Ordem da chave: flags CLI → `SSH_CLI_SECRETS_KEY` → `SSH_CLI_SECRETS_KEY_FILE` → keyring (`SSH_CLI_USE_KEYRING=1`) → XDG `secrets.key`.
- CLI: `ssh-cli secrets status|init|reencrypt` (nunca imprime a master-key); `--json` emite `secrets-init` / `secrets-reencrypt` sem material de chave.
- Opt-out só para testes: `--allow-plaintext-secrets` ou `SSH_CLI_ALLOW_PLAINTEXT_SECRETS=1`.
- **Nunca** logue master-key, senhas ou segredos decifrados.
- `--include-secrets` em pipe/non-TTY exige `-o`/`--output` ou `--i-understand-secrets-on-stdout` (guarda contra dump acidental de segredos no stdout).
- `vps export` redacted limpa segredos; secret vazio serializa como `""` e **nunca** como blob `sshcli-enc:…` (EXP-001 / 0.4.2).
- Mantenha `config.toml` com mode `0600` e restrinja locais de backup.
- Revise erros de mudança de host key TOFU antes de usar `--replace-host-key`.
- Nunca faça commit de inventários de host com segredos vivos.
- Nunca faça commit de sidecars MCP locais (ex.: `.setting.cyber/`), config Grok MCP (`~/.grok/config.toml`), XDG `config.toml` / `secrets.key` / `known_hosts`, ou arquivos de env E2E no repositório.
- E2E SSH real deve manter credenciais fora da árvore (`SSH_CLI_E2E_*` ou `$HOME/.grok/config.toml`); o script recusa config grok sob a raiz do repo.
- Senhas de demo na documentação pública são só placeholders (ex.: `demo-password-not-real`); nunca as reutilize em hosts reais.
- Desabilite elevação com `--disable-sudo` quando o workflow não deve escalar.
- Rode apenas comandos one-shot; nunca espere um daemon SSH de longa duração desta CLI.
- Instale com `--locked` para evitar drift de re-resolve crypto.
- Prefira a linha atual **0.5.1+** para o piso de supply-chain (russh 0.62.2) e para SCP com wire funcional (crates.io **0.3.9** SCP era inoperante).
- Honestidade histórica: **0.4.1** corrigiu export redacted de secret vazio (nunca `sshcli-enc:` de vazio) e exit 0 pós-bind do tunnel.
