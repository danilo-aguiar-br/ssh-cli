//! Definição de argumentos CLI via `clap` derive e dispatcher.
//!
//! 1. CRUD de VPS — `vps add|list|remove|edit|show|path|doctor|export|import`
//! 2. `connect` — grava arquivo irmão `active` (não campo TOML)
//! 3. Execução one-shot — `exec|sudo-exec|su-exec|scp|tunnel|health-check`
//! 4. `secrets` — master-key status/init/reencrypt (cifragem at-rest default)
//! 5. Completions
//!
//! ZERO `.env` em runtime. ZERO telemetria. Ciclo one-shot: nascer → executar → morrer.

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

/// Formato de saída suportado pela CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum FormatoSaida {
    /// Texto legível por humanos (padrão).
    #[default]
    Text,
    /// JSON estruturado.
    Json,
}

/// Argumentos globais do ssh-cli.
#[derive(Debug, Parser)]
#[command(
    name = "ssh-cli",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("SSH_CLI_COMMIT_HASH"), ")"),
    about = "CLI Rust one-shot multi-host XDG para LLMs operarem servidores via SSH.",
    long_about = "ssh-cli: binário leve one-shot (nascer→executar→morrer). Multi-host em storage XDG sem .env. \
Auth por senha ou chave. Sem telemetria."
)]
pub struct Argumentos {
    /// Força o idioma da CLI (ex.: `pt-BR`, `en-US`).
    #[arg(long, global = true, value_name = "LOCALE")]
    pub lang: Option<String>,

    /// Aumenta a verbosidade de logs em stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suprime output não-JSON (modo silencioso).
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Override do diretório de configuração (útil para testes).
    #[arg(long, global = true, value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// Desativa cores no output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Formato global de saída (text, json). Se omitido: JSON quando stdout não é TTY.
    #[arg(long, global = true, value_enum)]
    pub output_format: Option<FormatoSaida>,

    /// Desabilita sudo-exec/su-exec nesta invocação (alias --disableSudo).
    #[arg(long, global = true, alias = "disableSudo")]
    pub disable_sudo: bool,

    /// Substitui host key divergente no known_hosts TOFU.
    #[arg(long, global = true)]
    pub replace_host_key: bool,

    /// Subcomando a executar.
    #[command(subcommand)]
    pub comando: Comando,
}

/// Subcomandos de primeiro nível.
#[derive(Debug, Subcommand)]
pub enum Comando {
    /// Gerencia VPSs cadastradas.
    Vps {
        /// Ação específica do CRUD de VPS.
        #[command(subcommand)]
        acao: AcaoVps,
    },

    /// Define a VPS ativa (grava arquivo irmão `active` no diretório de config).
    Connect {
        /// Nome da VPS previamente adicionada via `vps add`.
        nome: String,
    },

    /// Executa um comando na VPS via SSH (stdout/stderr capturados).
    Exec {
        /// Nome da VPS.
        vps_nome: String,
        /// Comando shell a executar.
        comando: String,
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
        /// Lê senha SSH de stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Override de caminho de chave privada.
        #[arg(long)]
        key: Option<String>,
        /// Passphrase da chave (runtime).
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Lê passphrase da chave de stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Override de timeout em milissegundos.
        #[arg(long)]
        timeout: Option<u64>,
        /// Comentário shell anexado (comentário shell para auditoria).
        #[arg(long)]
        description: Option<String>,
    },

    /// Executa um comando com `sudo` (packing `sh -c` seguro).
    SudoExec {
        /// Nome da VPS.
        vps_nome: String,
        /// Comando shell.
        comando: String,
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
        /// Lê senha SSH de stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Override de senha sudo.
        #[arg(long, alias = "sudoPassword", alias = "sudo_password")]
        sudo_password: Option<String>,
        /// Lê senha sudo de stdin.
        #[arg(long)]
        sudo_password_stdin: bool,
        /// Override de chave.
        #[arg(long)]
        key: Option<String>,
        /// Passphrase da chave (runtime).
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Lê passphrase da chave de stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Override de timeout em milissegundos.
        #[arg(long)]
        timeout: Option<u64>,
        /// Comentário shell anexado.
        #[arg(long)]
        description: Option<String>,
    },

    /// Executa um comando com elevação `su -` one-shot.
    SuExec {
        /// Nome da VPS.
        vps_nome: String,
        /// Comando shell.
        comando: String,
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
        /// Override de senha su.
        #[arg(long, alias = "suPassword", alias = "su_password")]
        su_password: Option<String>,
        /// Lê senha su de stdin.
        #[arg(long)]
        su_password_stdin: bool,
        /// Override de chave.
        #[arg(long)]
        key: Option<String>,
        /// Passphrase da chave (runtime).
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Lê passphrase da chave de stdin.
        #[arg(long)]
        key_passphrase_stdin: bool,
        /// Override de timeout.
        #[arg(long)]
        timeout: Option<u64>,
        /// Comentário shell anexado.
        #[arg(long)]
        description: Option<String>,
    },

    /// Transferência de arquivos via SCP (upload/download).
    Scp {
        /// Ação específica do SCP.
        #[command(subcommand)]
        acao: AcaoScp,
    },

    /// Tunnel SSH com deadline obrigatório (one-shot limitado).
    Tunnel {
        /// Nome da VPS.
        vps_nome: String,
        /// Porta local.
        porta_local: u16,
        /// Host remoto.
        host_remoto: String,
        /// Porta remota.
        porta_remota: u16,
        /// Timeout obrigatório do tunnel em milissegundos.
        #[arg(long)]
        timeout_ms: u64,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
        /// Override de chave.
        #[arg(long)]
        key: Option<String>,
    },

    /// Verifica conectividade SSH com uma VPS.
    HealthCheck {
        /// Nome da VPS (usa ativa se omitido).
        vps_nome: Option<String>,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
    },

    /// Gerencia master-key e cifragem at-rest de secrets (one-shot).
    Secrets {
        /// Ação de secrets.
        #[command(subcommand)]
        acao: AcaoSecrets,
    },

    /// Gera completions de shell.
    Completions {
        /// Shell alvo.
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Ações do subcomando `vps`.
#[derive(Debug, Subcommand)]
pub enum AcaoVps {
    /// Adiciona uma nova VPS ao registro.
    Add {
        /// Nome único da VPS.
        #[arg(long)]
        name: String,
        /// Hostname ou IP.
        #[arg(long)]
        host: String,
        /// Porta SSH.
        #[arg(long, default_value_t = 22)]
        port: u16,
        /// Usuário SSH.
        #[arg(long)]
        user: String,
        /// Senha SSH.
        #[arg(long)]
        password: Option<String>,
        /// Lê senha de stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Caminho da chave privada OpenSSH.
        #[arg(long)]
        key: Option<String>,
        /// Passphrase da chave.
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Timeout em milissegundos (default 60000).
        #[arg(long, default_value_t = 60_000)]
        timeout: u64,
        /// Limite de caracteres do comando (entrada). Alias legado: maxChars.
        #[arg(long)]
        max_command_chars: Option<String>,
        /// Limite de caracteres de saída.
        #[arg(long)]
        max_output_chars: Option<String>,
        /// Alias legado: mapeia para max_command_chars .
        #[arg(long, alias = "maxChars")]
        max_chars: Option<String>,
        /// Senha para `sudo`.
        #[arg(long, alias = "sudoPassword", alias = "sudo_password")]
        sudo_password: Option<String>,
        /// Lê senha sudo de stdin.
        #[arg(long)]
        sudo_password_stdin: bool,
        /// Senha para `su -`.
        #[arg(long, alias = "suPassword", alias = "su_password")]
        su_password: Option<String>,
        /// Lê senha su de stdin.
        #[arg(long)]
        su_password_stdin: bool,
        /// Desabilita sudo/su neste host.
        #[arg(long, default_value_t = false)]
        disable_sudo: bool,
        /// Roda health-check após add.
        #[arg(long)]
        check: bool,
    },

    /// Lista todas as VPSs (senhas mascaradas).
    List {
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
    },

    /// Remove uma VPS do registro.
    Remove {
        /// Nome da VPS a remover.
        nome: String,
    },

    /// Edita campos de uma VPS existente.
    Edit {
        /// Nome da VPS a editar.
        nome: String,
        /// Novo hostname/IP.
        #[arg(long)]
        host: Option<String>,
        /// Nova porta SSH.
        #[arg(long)]
        port: Option<u16>,
        /// Novo usuário.
        #[arg(long)]
        user: Option<String>,
        /// Nova senha.
        #[arg(long)]
        password: Option<String>,
        /// Lê senha de stdin.
        #[arg(long)]
        password_stdin: bool,
        /// Nova chave.
        #[arg(long)]
        key: Option<String>,
        /// Nova passphrase.
        #[arg(long)]
        key_passphrase: Option<String>,
        /// Novo timeout.
        #[arg(long)]
        timeout: Option<u64>,
        /// Novo max command chars.
        #[arg(long)]
        max_command_chars: Option<String>,
        /// Novo max output chars.
        #[arg(long)]
        max_output_chars: Option<String>,
        /// Alias legado maxChars → command.
        #[arg(long, alias = "maxChars")]
        max_chars: Option<String>,
        /// Nova senha sudo.
        #[arg(long, alias = "sudoPassword", alias = "sudo_password")]
        sudo_password: Option<String>,
        /// Lê senha sudo de stdin.
        #[arg(long)]
        sudo_password_stdin: bool,
        /// Nova senha su.
        #[arg(long, alias = "suPassword", alias = "su_password")]
        su_password: Option<String>,
        /// Lê senha su de stdin.
        #[arg(long)]
        su_password_stdin: bool,
        /// Define disable_sudo.
        #[arg(long)]
        disable_sudo: Option<bool>,
    },

    /// Exibe detalhes de uma VPS (senhas mascaradas).
    Show {
        /// Nome da VPS.
        nome: String,
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
    },

    /// Exibe o caminho do arquivo de configuração.
    Path,

    /// Diagnóstico de camadas XDG / path / schema.
    Doctor {
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
    },

    /// Exporta hosts (senhas redacted por default).
    Export {
        /// Inclui segredos no export.
        #[arg(long)]
        include_secrets: bool,
        /// Arquivo de saída (stdout se omitido).
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Importa hosts de um TOML.
    Import {
        /// Arquivo de origem.
        #[arg(long)]
        file: PathBuf,
    },
}

/// Ações do subcomando `scp`.
#[derive(Debug, Subcommand)]
pub enum AcaoScp {
    /// Upload de arquivo local para remote.
    Upload {
        /// Nome da VPS.
        vps_nome: String,
        /// Caminho local.
        local: PathBuf,
        /// Caminho remote.
        remote: PathBuf,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
    },

    /// Download de arquivo remote para local.
    Download {
        /// Nome da VPS.
        vps_nome: String,
        /// Caminho remote.
        remote: PathBuf,
        /// Caminho local.
        local: PathBuf,
        /// Override de senha SSH.
        #[arg(long)]
        password: Option<String>,
    },
}

/// Ações do subcomando `secrets` (master-key / AEAD).
#[derive(Debug, Subcommand)]
pub enum AcaoSecrets {
    /// Mostra status da cifragem (sem material sensível).
    Status {
        /// Saída em JSON.
        #[arg(long)]
        json: bool,
    },
    /// Gera e grava master-key (`secrets.key` ou keyring). Nunca imprime a chave.
    Init {
        /// Grava no OS keyring em vez de `secrets.key`.
        #[arg(long)]
        keyring: bool,
        /// Sobrescreve chave existente.
        #[arg(long)]
        force: bool,
    },
    /// Regrava `config.toml` re-cifando secrets com a chave atual.
    Reencrypt,
}

/// Faz parsing dos argumentos da CLI.
#[must_use]
pub fn parse_args() -> Argumentos {
    Argumentos::parse()
}

/// Inicializa `tracing-subscriber`.
pub fn inicializar_logs(args: &Argumentos) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else if args.verbose {
        EnvFilter::new("debug")
    } else if args.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("info")
    };

    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_ansi(false)
        .try_init();
}

/// Gera completions de shell para stdout.
pub fn gerar_completions(shell: Shell) {
    use clap::CommandFactory;
    let mut cmd = Argumentos::command();
    clap_complete::generate(shell, &mut cmd, "ssh-cli", &mut std::io::stdout());
}

fn ler_stdin_se(flag: bool, valor: Option<String>) -> Result<Option<String>> {
    if flag {
        Ok(Some(crate::vps::ler_segredo_stdin()?))
    } else {
        Ok(valor)
    }
}

/// Resolve formato de saída: explícito > JSON se não-TTY > Text.
#[must_use]
pub fn resolver_formato(explicit: Option<FormatoSaida>) -> FormatoSaida {
    explicit.unwrap_or_else(|| {
        if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            FormatoSaida::Json
        } else {
            FormatoSaida::Text
        }
    })
}

/// Executa o subcomando solicitado.
pub async fn executar(args: Argumentos) -> Result<()> {
    let config_override = args.config_dir.clone();
    // Alinha `secrets.key` com `--config-dir` / testes isolados.
    crate::secrets::definir_diretorio_config(config_override.clone());
    let formato = resolver_formato(args.output_format);
    let disable_sudo = args.disable_sudo;
    let replace_host_key = args.replace_host_key;

    match args.comando {
        Comando::Vps { acao } => {
            crate::vps::executar_comando_vps(acao, config_override, formato).await
        }
        Comando::Connect { nome } => crate::vps::executar_connect(&nome, config_override).await,
        Comando::Exec {
            vps_nome,
            comando,
            json,
            password,
            password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
            description,
        } => {
            let password = ler_stdin_se(password_stdin, password)?;
            let key_passphrase = ler_stdin_se(key_passphrase_stdin, key_passphrase)?;
            let opts = crate::vps::OpcoesExec {
                password,
                key,
                key_passphrase,
                timeout,
                description,
                replace_host_key,
                disable_sudo,
                ..Default::default()
            };
            crate::vps::executar_exec(&vps_nome, &comando, config_override, formato, json, opts)
                .await
        }
        Comando::SudoExec {
            vps_nome,
            comando,
            json,
            password,
            password_stdin,
            sudo_password,
            sudo_password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
            description,
        } => {
            let password = ler_stdin_se(password_stdin, password)?;
            let sudo_password = ler_stdin_se(sudo_password_stdin, sudo_password)?;
            let key_passphrase = ler_stdin_se(key_passphrase_stdin, key_passphrase)?;
            let opts = crate::vps::OpcoesExec {
                password,
                sudo_password,
                key,
                key_passphrase,
                timeout,
                description,
                replace_host_key,
                disable_sudo,
                ..Default::default()
            };
            crate::vps::executar_sudo_exec(
                &vps_nome,
                &comando,
                config_override,
                formato,
                json,
                opts,
            )
            .await
        }
        Comando::SuExec {
            vps_nome,
            comando,
            json,
            password,
            su_password,
            su_password_stdin,
            key,
            key_passphrase,
            key_passphrase_stdin,
            timeout,
            description,
        } => {
            let su_password = ler_stdin_se(su_password_stdin, su_password)?;
            let key_passphrase = ler_stdin_se(key_passphrase_stdin, key_passphrase)?;
            let opts = crate::vps::OpcoesExec {
                password,
                su_password,
                key,
                key_passphrase,
                timeout,
                description,
                replace_host_key,
                disable_sudo,
                ..Default::default()
            };
            crate::vps::executar_su_exec(&vps_nome, &comando, config_override, formato, json, opts)
                .await
        }
        Comando::Scp { acao } => {
            let pwd = match &acao {
                AcaoScp::Upload { password, .. } | AcaoScp::Download { password, .. } => {
                    password.clone()
                }
            };
            crate::scp::executar_scp(acao, config_override, pwd).await
        }
        Comando::Tunnel {
            vps_nome,
            porta_local,
            host_remoto,
            porta_remota,
            timeout_ms,
            password,
            key,
        } => {
            crate::tunnel::executar_tunnel(
                &vps_nome,
                porta_local,
                &host_remoto,
                porta_remota,
                config_override,
                password,
                key,
                timeout_ms,
                replace_host_key,
            )
            .await
        }
        Comando::HealthCheck { vps_nome, password } => {
            crate::vps::executar_health_check(
                vps_nome.as_deref(),
                config_override,
                formato,
                password,
            )
            .await
        }
        Comando::Secrets { acao } => {
            crate::vps::executar_comando_secrets(acao, config_override, formato).await
        }
        Comando::Completions { shell } => {
            gerar_completions(shell);
            Ok(())
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use clap::Parser;

    #[test]
    fn parser_entende_tunnel_com_timeout() {
        let args = Argumentos::try_parse_from([
            "ssh-cli",
            "tunnel",
            "vps-a",
            "8080",
            "127.0.0.1",
            "5432",
            "--timeout-ms",
            "5000",
        ])
        .expect("tunnel");
        match args.comando {
            Comando::Tunnel {
                timeout_ms,
                porta_local,
                ..
            } => {
                assert_eq!(timeout_ms, 5000);
                assert_eq!(porta_local, 8080);
            }
            _ => panic!("esperado tunnel"),
        }
    }

    #[test]
    fn parser_vps_add_key() {
        let args = Argumentos::try_parse_from([
            "ssh-cli",
            "vps",
            "add",
            "--name",
            "x",
            "--host",
            "h",
            "--user",
            "u",
            "--key",
            "/tmp/id_ed25519",
        ])
        .expect("add key");
        match args.comando {
            Comando::Vps {
                acao: AcaoVps::Add { key, password, .. },
            } => {
                assert_eq!(key.as_deref(), Some("/tmp/id_ed25519"));
                assert!(password.is_none());
            }
            _ => panic!("esperado add"),
        }
    }

    #[test]
    fn parser_sudo_exec_description() {
        let args = Argumentos::try_parse_from([
            "ssh-cli",
            "sudo-exec",
            "v",
            "id",
            "--description",
            "who am i",
        ])
        .unwrap();
        match args.comando {
            Comando::SudoExec { description, .. } => {
                assert_eq!(description.as_deref(), Some("who am i"));
            }
            _ => panic!("sudo-exec"),
        }
    }

    #[test]
    fn parser_su_exec() {
        let args = Argumentos::try_parse_from(["ssh-cli", "su-exec", "v", "whoami"]).unwrap();
        assert!(matches!(args.comando, Comando::SuExec { .. }));
    }

    #[test]
    fn parser_disable_sudo_global() {
        let args =
            Argumentos::try_parse_from(["ssh-cli", "--disable-sudo", "vps", "path"]).unwrap();
        assert!(args.disable_sudo);
    }

    #[test]
    fn parser_doctor() {
        let args = Argumentos::try_parse_from(["ssh-cli", "vps", "doctor", "--json"]).unwrap();
        match args.comando {
            Comando::Vps {
                acao: AcaoVps::Doctor { json },
            } => assert!(json),
            _ => panic!("doctor"),
        }
    }
}
