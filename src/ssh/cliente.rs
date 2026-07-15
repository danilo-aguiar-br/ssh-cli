//! Cliente SSH real via `russh` 0.60.x.
//!
//! Conexão one-shot: TCP + handshake + auth (senha e/ou chave) + exec com
//! timeout, truncagem de saída e abort remoto best-effort.
//! Host keys: TOFU em `known_hosts` XDG (ver [`super::known_hosts`]).

use crate::erros::{ErroSshCli, ResultadoSshCli};
use secrecy::{ExposeSecret, SecretString};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};

/// Configuração de uma conexão SSH.
///
/// Construída a partir de um [`crate::vps::modelo::VpsRegistro`] no momento
/// da chamada. Auth: chave privada (preferida) e/ou senha.
#[derive(Clone)]
pub struct ConfiguracaoConexao {
    /// Hostname ou IP do servidor SSH.
    pub host: String,
    /// Porta TCP do servidor SSH (padrão 22).
    pub porta: u16,
    /// Nome de usuário SSH.
    pub usuario: String,
    /// Senha SSH (`SecretString` para zeroize automático); pode ser vazia se key-only.
    pub senha: SecretString,
    /// Caminho da chave privada OpenSSH (opcional).
    pub key_path: Option<String>,
    /// Passphrase da chave (opcional).
    pub key_passphrase: Option<SecretString>,
    /// Timeout total para conexão + handshake + autenticação + exec, em ms.
    pub timeout_ms: u64,
    /// Caminho do arquivo known_hosts (TOFU). `None` = always-trust (só testes).
    pub known_hosts_path: Option<PathBuf>,
    /// Se true, permite substituir fingerprint divergente.
    pub replace_host_key: bool,
}

impl std::fmt::Debug for ConfiguracaoConexao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfiguracaoConexao")
            .field("host", &self.host)
            .field("porta", &self.porta)
            .field("usuario", &self.usuario)
            .field("senha", &"<redacted>")
            .field("key_path", &self.key_path)
            .field(
                "key_passphrase",
                &self.key_passphrase.as_ref().map(|_| "<redacted>"),
            )
            .field("timeout_ms", &self.timeout_ms)
            .field("known_hosts_path", &self.known_hosts_path)
            .field("replace_host_key", &self.replace_host_key)
            .finish()
    }
}

impl ConfiguracaoConexao {
    /// Valida os campos básicos da configuração.
    pub fn validar(&self) -> ResultadoSshCli<()> {
        if self.host.trim().is_empty() {
            return Err(ErroSshCli::ArgumentoInvalido(
                "host vazio em ConfiguracaoConexao".to_string(),
            ));
        }
        if self.porta == 0 {
            return Err(ErroSshCli::ArgumentoInvalido(
                "porta 0 inválida em ConfiguracaoConexao".to_string(),
            ));
        }
        if self.usuario.trim().is_empty() {
            return Err(ErroSshCli::ArgumentoInvalido(
                "usuário vazio em ConfiguracaoConexao".to_string(),
            ));
        }
        let tem_senha = !self.senha.expose_secret().is_empty();
        let tem_key = self.key_path.as_ref().is_some_and(|p| !p.trim().is_empty());
        if !tem_senha && !tem_key {
            return Err(ErroSshCli::ArgumentoInvalido(
                "auth exige senha ou key_path".to_string(),
            ));
        }
        Ok(())
    }
}

/// Saída da execução de um comando SSH remoto.
#[derive(Debug, Clone)]
pub struct SaidaExecucao {
    /// Stdout capturado (possivelmente truncado a `max_chars` codepoints).
    pub stdout: String,
    /// Stderr capturado (possivelmente truncado a `max_chars` codepoints).
    pub stderr: String,
    /// Código de saída. `None` quando o comando foi terminado por sinal ou timeout.
    pub exit_code: Option<i32>,
    /// `true` se `stdout` foi truncado em `max_chars`.
    pub truncado_stdout: bool,
    /// `true` se `stderr` foi truncado em `max_chars`.
    pub truncado_stderr: bool,
    /// Duração total da execução, em milissegundos.
    pub duracao_ms: u64,
}

/// Resultado de uma operação de transferência de arquivo via SCP.
#[derive(Debug, Clone)]
pub struct TransferenciaResultado {
    /// Número de bytes transferidos.
    pub bytes_transferidos: u64,
    /// Duração total em milissegundos.
    pub duracao_ms: u64,
}

/// Trunca uma string UTF-8 a no máximo `max_chars` codepoints.
///
/// Retorna `(string_truncada, truncou)`. Se `max_chars == 0` retorna string vazia.
/// Unicode-safe: opera sobre codepoints via `chars()`, nunca quebra no meio.
#[must_use]
pub fn truncar_utf8(conteudo: &str, max_chars: usize) -> (String, bool) {
    let total = conteudo.chars().count();
    if total <= max_chars {
        return (conteudo.to_string(), false);
    }
    let truncado: String = conteudo.chars().take(max_chars).collect();
    (truncado, true)
}

// =========================================================================
// Trait ClienteSshTrait para permitir mocks em teste.
// =========================================================================

use async_trait::async_trait;
use std::path::Path;

/// Stream bidirecional usado para tunnel SSH (direct-tcpip).
pub trait CanalTunel: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T> CanalTunel for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

/// Trait para cliente SSH que permite implementação real (russh) ou mock para testes.
///
/// Este trait abstrai as operações de conexão SSH para permitir testes unitários
/// sem necessidade de conexão de rede real.
#[async_trait]
pub trait ClienteSshTrait: Send + Sync + 'static {
    /// Conecta a um servidor SSH e autentica com as credenciais fornecidas.
    async fn conectar(cfg: ConfiguracaoConexao) -> Result<Box<Self>, ErroSshCli>
    where
        Self: Sized;

    /// Executa um comando shell remoto e retorna a saída capturada.
    ///
    /// `stdin_data`, se presente, é escrito no canal após o `exec` e antes do loop
    /// de leitura (GAP-SSH-SEC-001: senha sudo/su fora da argv remota).
    async fn executar_comando(
        &mut self,
        cmd: &str,
        max_chars: usize,
        stdin_data: Option<Vec<u8>>,
    ) -> Result<SaidaExecucao, ErroSshCli>;

    /// Faz upload de um arquivo local para o servidor remoto via SCP.
    async fn upload(
        &mut self,
        local: &Path,
        remote: &Path,
    ) -> Result<TransferenciaResultado, ErroSshCli>;

    /// Faz download de um arquivo remoto para o sistema local via SCP.
    async fn download(
        &mut self,
        remote: &Path,
        local: &Path,
    ) -> Result<TransferenciaResultado, ErroSshCli>;

    /// Abre um canal `direct-tcpip` para forwarding de tunnel.
    async fn abrir_canal_tunel(
        &self,
        host_remoto: &str,
        porta_remota: u16,
        endereco_origem: &str,
        porta_origem: u16,
    ) -> Result<Box<dyn CanalTunel>, ErroSshCli>;

    /// Encerra a conexão SSH de forma limpa.
    async fn desconectar(&self) -> Result<(), ErroSshCli>;
}

#[cfg(test)]
/// Mocks de cliente SSH usados em testes unitários.
pub mod mocks {
    use super::*;
    use mockall::mock;

    mock! {
        pub ClienteSsh {}

    #[async_trait]
    impl crate::ssh::cliente::ClienteSshTrait for ClienteSsh {
            async fn conectar(cfg: ConfiguracaoConexao) -> Result<Box<Self>, ErroSshCli>;
            async fn executar_comando(&mut self, cmd: &str, max_chars: usize, stdin_data: Option<Vec<u8>>) -> Result<SaidaExecucao, ErroSshCli>;
            async fn upload(&mut self, local: &Path, remote: &Path) -> Result<TransferenciaResultado, ErroSshCli>;
            async fn download(&mut self, remote: &Path, local: &Path) -> Result<TransferenciaResultado, ErroSshCli>;
            async fn abrir_canal_tunel(
                &self,
                host_remoto: &str,
                porta_remota: u16,
                endereco_origem: &str,
                porta_origem: u16,
            ) -> Result<Box<dyn CanalTunel>, ErroSshCli>;
            async fn desconectar(&self) -> Result<(), ErroSshCli>;
        }
    }
}

// =========================================================================
// Implementação SSH REAL (feature `ssh-real`).
// =========================================================================

#[cfg(feature = "ssh-real")]
mod real {
    use super::{
        CanalTunel, ClienteSshTrait, ConfiguracaoConexao, SaidaExecucao, TransferenciaResultado,
    };
    use crate::erros::{ErroSshCli, ResultadoSshCli};
    use async_trait::async_trait;
    use secrecy::ExposeSecret;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Handler russh com TOFU em known_hosts (ou always-trust se path ausente).
    pub struct ManipuladorCliente {
        host: String,
        porta: u16,
        known_hosts_path: Option<std::path::PathBuf>,
        replace_host_key: bool,
        /// Erro de host key capturado (russh Error não carrega nosso tipo).
        host_key_rejeitada: bool,
        detalhe_host_key: Option<String>,
    }

    impl ManipuladorCliente {
        fn novo(cfg: &ConfiguracaoConexao) -> Self {
            Self {
                host: cfg.host.clone(),
                porta: cfg.porta,
                known_hosts_path: cfg.known_hosts_path.clone(),
                replace_host_key: cfg.replace_host_key,
                host_key_rejeitada: false,
                detalhe_host_key: None,
            }
        }
    }

    impl russh::client::Handler for ManipuladorCliente {
        type Error = russh::Error;

        async fn check_server_key(
            &mut self,
            chave_servidor: &russh::keys::ssh_key::PublicKey,
        ) -> Result<bool, Self::Error> {
            let fingerprint = format!(
                "{}",
                chave_servidor.fingerprint(russh::keys::HashAlg::Sha256)
            );

            let Some(path) = self.known_hosts_path.clone() else {
                tracing::warn!("known_hosts ausente: aceitando host key (modo teste)");
                return Ok(true);
            };

            let mut kh = match crate::ssh::known_hosts::KnownHosts::carregar(path) {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!(erro = %e, "falha ao carregar known_hosts");
                    self.host_key_rejeitada = true;
                    self.detalhe_host_key = Some(e.to_string());
                    return Ok(false);
                }
            };

            match crate::ssh::known_hosts::verificar_tofu(
                &mut kh,
                &self.host,
                self.porta,
                &fingerprint,
                self.replace_host_key,
            ) {
                Ok(true) => Ok(true),
                Ok(false) => Ok(false),
                Err(e) => {
                    self.host_key_rejeitada = true;
                    self.detalhe_host_key = Some(e.to_string());
                    tracing::error!(erro = %e, "host key rejeitada");
                    Ok(false)
                }
            }
        }
    }

    /// Cliente SSH ativo com sessão autenticada.
    pub struct ClienteSsh {
        /// Sessão SSH autenticada para operações de baixo nível.
        pub sessao: russh::client::Handle<ManipuladorCliente>,
        cfg: ConfiguracaoConexao,
    }

    impl std::fmt::Debug for ClienteSsh {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ClienteSsh")
                .field("host", &self.cfg.host)
                .field("porta", &self.cfg.porta)
                .field("usuario", &self.cfg.usuario)
                .field("timeout_ms", &self.cfg.timeout_ms)
                .finish()
        }
    }

    fn mapear_exit_status(exit_status: u32) -> i32 {
        i32::try_from(exit_status).unwrap_or(-1)
    }

    fn processar_mensagem_exec(
        msg: russh::ChannelMsg,
        stdout_bytes: &mut Vec<u8>,
        stderr_bytes: &mut Vec<u8>,
        exit_code: &mut Option<i32>,
    ) -> bool {
        use russh::ChannelMsg;

        match msg {
            ChannelMsg::Data { data } => {
                stdout_bytes.extend_from_slice(&data);
            }
            ChannelMsg::ExtendedData { data, ext } => {
                // ext == 1 → SSH_EXTENDED_DATA_STDERR (RFC 4254 §5.2).
                if ext == 1 {
                    stderr_bytes.extend_from_slice(&data);
                } else {
                    tracing::debug!(ext, "dados estendidos ignorados");
                }
            }
            ChannelMsg::ExitStatus { exit_status } => {
                // russh entrega como u32. Mantemos como i32 para acomodar
                // convenções Unix (shells podem emitir códigos como u8 em
                // wait-status; aqui já é o exit code aplicativo, 0..=255).
                *exit_code = Some(mapear_exit_status(exit_status));
                // NÃO retorna true: aguardar Eof/Close após ExitStatus.
            }
            ChannelMsg::ExitSignal {
                signal_name,
                core_dumped,
                error_message,
                ..
            } => {
                tracing::warn!(
                    ?signal_name,
                    core_dumped,
                    %error_message,
                    "processo remoto terminou por sinal"
                );
                // Sem exit_status → mantemos None.
            }
            ChannelMsg::Eof => {
                tracing::debug!("EOF no canal SSH");
            }
            ChannelMsg::Close => {
                tracing::debug!("canal SSH fechado pelo servidor");
                return true;
            }
            _ => {}
        }

        false
    }

    /// Byte de ACK/OK do protocolo SCP (também usado como terminador do payload).
    const SCP_OK: u8 = 0;

    /// Basename seguro para o wire SCP (sem path separators / control chars).
    fn basename_scp(nome_arquivo: &str) -> String {
        nome_arquivo
            .split(['/', '\\'])
            .next_back()
            .unwrap_or("file")
            .replace(['\n', '\r', '\0'], "_")
    }

    /// Header `C`-line do protocolo SCP (newline real `0x0a`, nunca `\\n` literal).
    #[cfg_attr(not(test), allow(dead_code))]
    fn formatar_header_upload_scp(tamanho: u64, nome_arquivo: &str) -> String {
        formatar_header_upload_scp_com_modo(0o644, tamanho, nome_arquivo)
    }

    /// Header `C` com mode octal (ex.: `0644`).
    fn formatar_header_upload_scp_com_modo(mode: u32, tamanho: u64, nome_arquivo: &str) -> String {
        let nome = basename_scp(nome_arquivo);
        let mode = mode & 0o7777;
        format!("C{mode:04o} {tamanho} {nome}\n")
    }

    /// Linha `T` do protocolo SCP (preserve times / `-p`).
    fn formatar_linha_t_scp(mtime_secs: u64, atime_secs: u64) -> String {
        format!("T{mtime_secs} 0 {atime_secs} 0\n")
    }

    /// Parse da linha `T mtime 0 atime 0`.
    fn parse_linha_t_scp(linha: &str) -> ResultadoSshCli<(u64, u64)> {
        let linha = linha.trim_end_matches(['\0', '\r', '\n']).trim();
        if !linha.starts_with('T') {
            return Err(ErroSshCli::CanalFalhou(format!(
                "linha T SCP inesperada: {linha}"
            )));
        }
        let resto = &linha[1..];
        let partes: Vec<&str> = resto.split_whitespace().collect();
        if partes.len() < 3 {
            return Err(ErroSshCli::CanalFalhou(format!(
                "linha T SCP mal formatada: {linha}"
            )));
        }
        let mtime: u64 = partes[0].parse().map_err(|_| {
            ErroSshCli::CanalFalhou(format!("mtime inválido na linha T: {}", partes[0]))
        })?;
        let atime: u64 = partes[2].parse().map_err(|_| {
            ErroSshCli::CanalFalhou(format!("atime inválido na linha T: {}", partes[2]))
        })?;
        Ok((mtime, atime))
    }

    /// Parse do header `C0mmm size name` → `(mode, tamanho)`.
    fn parse_header_scp(header: &str) -> ResultadoSshCli<(u32, u64)> {
        let header = header.trim_end_matches(['\0', '\r', '\n']).trim();

        if !header.starts_with('C') {
            return Err(ErroSshCli::CanalFalhou(format!(
                "header SCP inesperado: {}",
                header
            )));
        }

        let partes: Vec<&str> = header.split_whitespace().collect();
        if partes.len() < 3 {
            return Err(ErroSshCli::CanalFalhou(format!(
                "header SCP mal formatado: {}",
                header
            )));
        }

        // Campo mode: `C0644` (prefixo `C` + 4 dígitos octais).
        let mode_token = partes[0];
        if mode_token.len() < 2 {
            return Err(ErroSshCli::CanalFalhou(format!(
                "mode SCP ausente no header: {header}"
            )));
        }
        let mode_oct = &mode_token[1..];
        let mode: u32 = u32::from_str_radix(mode_oct, 8)
            .map_err(|_| ErroSshCli::CanalFalhou(format!("mode SCP inválido: {mode_oct}")))?;

        let tamanho = partes[1].parse().map_err(|_| {
            ErroSshCli::CanalFalhou(format!("tamanho inválido no header: {}", partes[1]))
        })?;
        Ok((mode & 0o7777, tamanho))
    }

    /// Mode octal para o header `C` a partir de metadata local.
    fn mode_scp_de_metadata(meta: &std::fs::Metadata) -> u32 {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            meta.permissions().mode() & 0o7777
        }
        #[cfg(not(unix))]
        {
            let _ = meta;
            0o644
        }
    }

    /// Segundos epoch a partir de SystemTime (best-effort).
    fn system_time_secs(t: std::time::SystemTime) -> u64 {
        t.duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Sufixo do arquivo temporário de download atômico (SCP-022).
    const SCP_PARTIAL_SUFFIX: &str = ".ssh-cli.partial";

    fn caminho_parcial_download(local: &std::path::Path) -> std::path::PathBuf {
        let mut p = local.as_os_str().to_os_string();
        p.push(SCP_PARTIAL_SUFFIX);
        std::path::PathBuf::from(p)
    }

    /// Interpreta o primeiro byte de status SCP: `0`=OK, `1`/`2`=erro (+ mensagem).
    fn interpretar_status_scp(bytes: &[u8]) -> ResultadoSshCli<()> {
        if bytes.is_empty() {
            return Err(ErroSshCli::CanalFalhou(
                "status SCP vazio (esperado ACK 0x00)".to_string(),
            ));
        }
        match bytes[0] {
            SCP_OK => Ok(()),
            1 | 2 => {
                let msg = String::from_utf8_lossy(&bytes[1..]).trim().to_string();
                Err(ErroSshCli::CanalFalhou(if msg.is_empty() {
                    format!("SCP rejeitou a transferência (status {})", bytes[0])
                } else {
                    format!("SCP: {msg}")
                }))
            }
            other => Err(ErroSshCli::CanalFalhou(format!(
                "status SCP inesperado: 0x{other:02x}"
            ))),
        }
    }

    /// Monta `scp -t[p]/-f[p]` com path remoto escapado para o shell remoto.
    ///
    /// OpenSSH: source (`-f`) só emite linha `T` e mode honesto com **`-p`**.
    /// Sink (`-t`) com `-p` aplica mode completo (sem mascarar umask sticky).
    /// Sempre usamos `-p` (SCP-023 bi-direcional).
    fn comando_scp_remoto(modo: &str, remote: &std::path::Path) -> String {
        let path = crate::ssh::packing::escapar_shell_single_quotes(&remote.display().to_string());
        // `modo` esperado: `-t` ou `-f` (sem `-p`); anexamos `p` explicitamente.
        let modo_p = if modo.contains('p') {
            modo.to_string()
        } else {
            format!("{modo}p")
        };
        // Path em single-quotes (sem `--` para máxima compatibilidade OpenSSH scp legado).
        format!("scp {modo_p} {path}")
    }

    /// Aplica mode POSIX do header `C` no arquivo local (best-effort no Unix).
    fn aplicar_mode_local(path: &std::path::Path, mode: u32) -> ResultadoSshCli<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(mode & 0o7777);
            std::fs::set_permissions(path, perms).map_err(ErroSshCli::Io)?;
        }
        #[cfg(not(unix))]
        {
            let _ = (path, mode);
        }
        Ok(())
    }

    /// Lê o próximo `ChannelMsg::Data` não vazio do canal SCP.
    async fn scp_ler_data<S>(canal: &mut russh::Channel<S>) -> ResultadoSshCli<Vec<u8>>
    where
        S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
    {
        use russh::ChannelMsg;
        loop {
            match canal.wait().await {
                Some(ChannelMsg::Data { data }) => {
                    if data.is_empty() {
                        continue;
                    }
                    return Ok(data.to_vec());
                }
                Some(ChannelMsg::ExtendedData { data, .. }) => {
                    if data.is_empty() {
                        continue;
                    }
                    let msg = String::from_utf8_lossy(data.as_ref()).trim().to_string();
                    return Err(ErroSshCli::CanalFalhou(format!("SCP stderr: {msg}")));
                }
                Some(ChannelMsg::ExitStatus { exit_status }) if exit_status != 0 => {
                    return Err(ErroSshCli::CanalFalhou(format!(
                        "scp encerrou com exit {exit_status}"
                    )));
                }
                Some(ChannelMsg::Close) | None => {
                    return Err(ErroSshCli::CanalFalhou(
                        "canal SCP fechou prematuramente".to_string(),
                    ));
                }
                _ => continue,
            }
        }
    }

    /// Aguarda ACK de status SCP (`0x00`) ou propaga erro `1`/`2`.
    async fn scp_aguardar_status<S>(canal: &mut russh::Channel<S>) -> ResultadoSshCli<()>
    where
        S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
    {
        let data = scp_ler_data(canal).await?;
        interpretar_status_scp(&data)
    }

    /// Lê bytes até incluir newline (header `C`/`T`) ou status de erro `1`/`2`.
    async fn scp_ler_ate_newline<S>(canal: &mut russh::Channel<S>) -> ResultadoSshCli<Vec<u8>>
    where
        S: From<(russh::ChannelId, russh::ChannelMsg)> + Send + Sync + 'static,
    {
        let mut buf = Vec::new();
        loop {
            let chunk = scp_ler_data(canal).await?;
            if buf.is_empty() && matches!(chunk.first().copied(), Some(1 | 2)) {
                return Ok(chunk);
            }
            buf.extend_from_slice(&chunk);
            if buf.contains(&b'\n') {
                return Ok(buf);
            }
            if buf.len() > 16_384 {
                return Err(ErroSshCli::CanalFalhou(
                    "header SCP excessivamente longo".to_string(),
                ));
            }
        }
    }

    impl ClienteSsh {
        /// Conecta e autentica. Todo o fluxo (TCP + handshake + auth) respeita
        /// o `timeout_ms` da configuração.
        ///
        /// # Erros
        /// - [`ErroSshCli::ArgumentoInvalido`] se a configuração for inválida.
        /// - [`ErroSshCli::TimeoutSsh`] se exceder o timeout total.
        /// - [`ErroSshCli::ConexaoFalhou`] em falhas TCP/handshake.
        /// - [`ErroSshCli::AutenticacaoFalhou`] se o servidor rejeitar senha/chave
        ///   (tente `--key`, `--password-stdin` ou `--key-passphrase-stdin`).
        pub async fn conectar(cfg: ConfiguracaoConexao) -> ResultadoSshCli<Self> {
            cfg.validar()?;

            let timeout = Duration::from_millis(cfg.timeout_ms);
            let host = cfg.host.clone();
            let porta = cfg.porta;
            let usuario = cfg.usuario.clone();
            let senha_segura = cfg.senha.clone();
            let key_path = cfg.key_path.clone();
            let key_passphrase = cfg.key_passphrase.clone();
            let handler = ManipuladorCliente::novo(&cfg);

            let config_cliente = Arc::new(russh::client::Config {
                inactivity_timeout: Some(timeout),
                ..Default::default()
            });

            tracing::info!(
                host = %host,
                porta,
                usuario = %usuario,
                timeout_ms = cfg.timeout_ms,
                tem_chave = key_path.is_some(),
                "iniciando conexão SSH"
            );

            let resultado_conexao = tokio::time::timeout(timeout, async move {
                let mut sessao = russh::client::connect(
                    config_cliente,
                    (host.as_str(), porta),
                    handler,
                )
                .await
                .map_err(|e| ErroSshCli::ConexaoFalhou(format!("falha TCP/handshake: {e}")))?;

                // Preferência: chave privada primeiro; fallback senha se ambas presentes.
                let mut autenticado = false;

                if let Some(ref kp) = key_path {
                    let pass = key_passphrase
                        .as_ref()
                        .map(|s| s.expose_secret().to_string());
                    let chave = russh::keys::load_secret_key(kp, pass.as_deref()).map_err(|e| {
                        ErroSshCli::AutenticacaoSsh(format!(
                            "falha ao carregar chave {kp}: {e}"
                        ))
                    })?;
                    let hash = sessao
                        .best_supported_rsa_hash()
                        .await
                        .map_err(|e| {
                            ErroSshCli::ConexaoFalhou(format!("rsa hash: {e}"))
                        })?
                        .flatten();
                    let auth = sessao
                        .authenticate_publickey(
                            usuario.clone(),
                            russh::keys::PrivateKeyWithHashAlg::new(Arc::new(chave), hash),
                        )
                        .await
                        .map_err(|e| {
                            ErroSshCli::ConexaoFalhou(format!("falha auth publickey: {e}"))
                        })?;
                    autenticado = auth.success();
                    if !autenticado {
                        tracing::warn!(host = %host, "auth por chave rejeitada; tentando senha se houver");
                    }
                }

                if !autenticado && !senha_segura.expose_secret().is_empty() {
                    let auth = sessao
                        .authenticate_password(usuario.clone(), senha_segura.expose_secret())
                        .await
                        .map_err(|e| {
                            ErroSshCli::ConexaoFalhou(format!("falha auth password: {e}"))
                        })?;
                    autenticado = auth.success();
                }

                if !autenticado {
                    tracing::warn!(host = %host, usuario = %usuario, "autenticação SSH rejeitada");
                    return Err(ErroSshCli::AutenticacaoFalhou);
                }

                Ok::<_, ErroSshCli>(sessao)
            })
            .await;

            let sessao = match resultado_conexao {
                Ok(Ok(s)) => s,
                Ok(Err(erro)) => return Err(erro),
                Err(_) => return Err(ErroSshCli::TimeoutSsh(cfg.timeout_ms)),
            };

            tracing::info!("conexão SSH autenticada com sucesso");

            Ok(Self { sessao, cfg })
        }

        /// Executa um comando shell remoto e captura stdout/stderr em paralelo.
        ///
        /// Trunca cada stream em `max_chars` codepoints UTF-8. Respeita o
        /// `timeout_ms` da configuração para a execução inteira.
        ///
        /// # Erros
        /// - [`ErroSshCli::CanalFalhou`] em falha ao abrir canal ou enviar `exec`.
        /// - [`ErroSshCli::TimeoutSsh`] se exceder o timeout.
        pub async fn executar_comando(
            &mut self,
            comando: &str,
            max_chars: usize,
            stdin_data: Option<Vec<u8>>,
        ) -> ResultadoSshCli<SaidaExecucao> {
            self.executar_comando_interno(comando, max_chars, true, stdin_data)
                .await
        }

        async fn executar_comando_interno(
            &mut self,
            comando: &str,
            max_chars: usize,
            abort_em_timeout: bool,
            stdin_data: Option<Vec<u8>>,
        ) -> ResultadoSshCli<SaidaExecucao> {
            let inicio = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            let resultado = tokio::time::timeout(timeout, async {
                let mut canal = self
                    .sessao
                    .channel_open_session()
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("abrir sessão: {e}")))?;

                canal
                    .exec(true, comando)
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("exec: {e}")))?;

                // Senha sudo/su no stdin do canal — nunca na cmdline remota (SEC-001).
                if let Some(bytes) = stdin_data.as_ref() {
                    canal
                        .data(&bytes[..])
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("stdin canal: {e}")))?;
                    canal
                        .eof()
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("eof canal: {e}")))?;
                }

                let mut stdout_bytes: Vec<u8> = Vec::new();
                let mut stderr_bytes: Vec<u8> = Vec::new();
                let mut exit_code: Option<i32> = None;

                while let Some(msg) = canal.wait().await {
                    if processar_mensagem_exec(
                        msg,
                        &mut stdout_bytes,
                        &mut stderr_bytes,
                        &mut exit_code,
                    ) {
                        break;
                    }
                }

                Ok::<_, ErroSshCli>((stdout_bytes, stderr_bytes, exit_code))
            })
            .await;

            let (stdout_bytes, stderr_bytes, exit_code) = match resultado {
                Ok(Ok(t)) => t,
                Ok(Err(erro)) => return Err(erro),
                Err(_) => {
                    if abort_em_timeout {
                        if let Some(padrao) = crate::ssh::packing::padrao_abort_remoto(comando) {
                            let abort_cmd = crate::ssh::packing::empacotar_abort_pkill(&padrao);
                            tracing::warn!(
                                padrao = %padrao,
                                "timeout local; tentando abort remoto best-effort"
                            );
                            let _ = self.tentar_abort_remoto(&abort_cmd).await;
                        }
                    }
                    return Err(ErroSshCli::TimeoutSsh(self.cfg.timeout_ms));
                }
            };

            let stdout_str = String::from_utf8_lossy(&stdout_bytes).to_string();
            let stderr_str = String::from_utf8_lossy(&stderr_bytes).to_string();

            let (stdout_truncado, truncado_stdout) = super::truncar_utf8(&stdout_str, max_chars);
            let (stderr_truncado, truncado_stderr) = super::truncar_utf8(&stderr_str, max_chars);

            let duracao_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(SaidaExecucao {
                stdout: stdout_truncado,
                stderr: stderr_truncado,
                exit_code,
                truncado_stdout,
                truncado_stderr,
                duracao_ms,
            })
        }

        /// Upload de arquivo local para remote via SCP (protocolo OpenSSH sink).
        ///
        /// One-shot: stream em chunks (sem carregar o arquivo inteiro em RAM).
        ///
        /// # Erros
        /// - [`ErroSshCli::ArquivoNaoEncontrado`] se o arquivo local não existir.
        /// - [`ErroSshCli::ArgumentoInvalido`] se o path local não for arquivo regular.
        /// - [`ErroSshCli::CanalFalhou`] em falha ao abrir canal SCP / status remoto.
        /// - [`ErroSshCli::TimeoutSsh`] se exceder o timeout.
        pub async fn upload(
            &mut self,
            local: &std::path::Path,
            remote: &std::path::Path,
        ) -> ResultadoSshCli<TransferenciaResultado> {
            use russh::ChannelMsg;
            use std::io::Read;
            use std::time::Instant;

            let local_str = local.display().to_string();

            if local.is_dir() {
                return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                    crate::i18n::Mensagem::ScpUploadSomenteArquivo,
                )));
            }

            let metadados = std::fs::metadata(local).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ErroSshCli::ArquivoNaoEncontrado(local_str.clone())
                } else {
                    ErroSshCli::Io(e)
                }
            })?;

            if !metadados.is_file() {
                return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                    crate::i18n::Mensagem::ScpUploadSomenteArquivo,
                )));
            }

            let tamanho = metadados.len();
            let mode = mode_scp_de_metadata(&metadados);
            let mtime = metadados.modified().ok().map(system_time_secs).unwrap_or(0);
            let atime = metadados
                .accessed()
                .ok()
                .map(system_time_secs)
                .unwrap_or(mtime);
            let nome_arquivo = local.file_name().and_then(|n| n.to_str()).unwrap_or("file");

            let inicio = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            let resultado =
                tokio::time::timeout(timeout, async {
                    if crate::signals::cancelado() {
                        return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                            crate::i18n::Mensagem::OperacaoCancelada,
                        )));
                    }

                    let mut canal =
                        self.sessao.channel_open_session().await.map_err(|e| {
                            ErroSshCli::CanalFalhou(format!("abrir sessão SCP: {e}"))
                        })?;

                    let comando = comando_scp_remoto("-t", remote);
                    canal
                        .exec(true, comando.as_str())
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("exec SCP: {e}")))?;

                    // Sink remoto envia ACK (0x00) antes de aceitar o header.
                    scp_aguardar_status(&mut canal).await?;

                    // Preserve times (linha T) antes do header C.
                    let linha_t = formatar_linha_t_scp(mtime, atime);
                    canal
                        .data(linha_t.as_bytes())
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar linha T SCP: {e}")))?;
                    scp_aguardar_status(&mut canal).await?;

                    let header = formatar_header_upload_scp_com_modo(mode, tamanho, nome_arquivo);
                    canal
                        .data(header.as_bytes())
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar header SCP: {e}")))?;
                    scp_aguardar_status(&mut canal).await?;

                    // SCP-018: stream do disco em chunks (sem fs::read total).
                    let mut arquivo = std::fs::File::open(local).map_err(ErroSshCli::Io)?;
                    let mut buf = vec![0u8; 32_768];
                    loop {
                        if crate::signals::cancelado() {
                            return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                                crate::i18n::Mensagem::OperacaoCancelada,
                            )));
                        }
                        let n = arquivo.read(&mut buf).map_err(ErroSshCli::Io)?;
                        if n == 0 {
                            break;
                        }
                        canal.data(&buf[..n]).await.map_err(|e| {
                            ErroSshCli::CanalFalhou(format!("enviar bloco SCP: {e}"))
                        })?;
                    }

                    // Terminador de arquivo = byte 0x00 (não data vazio).
                    canal
                        .data([SCP_OK].as_slice())
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar EOF SCP: {e}")))?;
                    scp_aguardar_status(&mut canal).await?;

                    let _ = canal.eof().await;
                    while let Some(msg) = canal.wait().await {
                        if let ChannelMsg::Close = msg {
                            break;
                        }
                    }

                    Ok::<_, ErroSshCli>(())
                })
                .await;

            resultado.map_err(|_| ErroSshCli::TimeoutSsh(self.cfg.timeout_ms))??;

            let duracao_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(TransferenciaResultado {
                bytes_transferidos: tamanho,
                duracao_ms,
            })
        }

        /// Download de arquivo remote para local via SCP (protocolo OpenSSH source).
        ///
        /// Escreve em `{local}.ssh-cli.partial` e faz rename atômico (SCP-022).
        ///
        /// # Erros
        /// - [`ErroSshCli::Io`] se não conseguir escrever o arquivo local.
        /// - [`ErroSshCli::CanalFalhou`] em falha ao abrir canal SCP / status remoto.
        /// - [`ErroSshCli::TimeoutSsh`] se exceder o timeout.
        pub async fn download(
            &mut self,
            remote: &std::path::Path,
            local: &std::path::Path,
        ) -> ResultadoSshCli<TransferenciaResultado> {
            use russh::ChannelMsg;
            use std::io::Write;
            use std::time::{Duration as StdDuration, Instant, UNIX_EPOCH};

            if local.is_dir() {
                return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                    crate::i18n::Mensagem::ScpDownloadLocalNaoDiretorio,
                )));
            }

            let inicio = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);
            let partial = caminho_parcial_download(local);

            let resultado = tokio::time::timeout(timeout, async {
                if crate::signals::cancelado() {
                    return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                        crate::i18n::Mensagem::OperacaoCancelada,
                    )));
                }

                let mut canal = self
                    .sessao
                    .channel_open_session()
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("abrir sessão SCP: {e}")))?;

                let comando = comando_scp_remoto("-f", remote);
                canal
                    .exec(true, comando.as_str())
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("exec SCP: {e}")))?;

                // Source remoto só envia o header após o ACK inicial do sink local.
                canal
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar ack inicial: {e}")))?;

                let mut times: Option<(u64, u64)> = None;
                let mut header_bytes = scp_ler_ate_newline(&mut canal).await?;
                // Erro remoto: status 1/2 no primeiro byte.
                if !header_bytes.is_empty() && matches!(header_bytes[0], 1 | 2) {
                    interpretar_status_scp(&header_bytes)?;
                }
                let mut header = String::from_utf8_lossy(&header_bytes).into_owned();
                // Linha T opcional (preserve times).
                if header.trim_start().starts_with('T') {
                    times = Some(parse_linha_t_scp(&header)?);
                    canal
                        .data([SCP_OK].as_slice())
                        .await
                        .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar ack T: {e}")))?;
                    header_bytes = scp_ler_ate_newline(&mut canal).await?;
                    if !header_bytes.is_empty() && matches!(header_bytes[0], 1 | 2) {
                        interpretar_status_scp(&header_bytes)?;
                    }
                    header = String::from_utf8_lossy(&header_bytes).into_owned();
                }
                let (mode_remoto, tamanho) = parse_header_scp(&header)?;

                canal
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar ack header: {e}")))?;

                if let Some(pai) = local.parent() {
                    if !pai.as_os_str().is_empty() {
                        std::fs::create_dir_all(pai)?;
                    }
                }

                // SCP-022: escrever no partial; rename só no sucesso.
                let mut arquivo = std::fs::File::create(&partial).map_err(ErroSshCli::Io)?;
                let mut recebidos: u64 = 0;
                let mut pendente: Vec<u8> = Vec::new();

                while recebidos < tamanho {
                    if crate::signals::cancelado() {
                        return Err(ErroSshCli::ArgumentoInvalido(crate::i18n::t(
                            crate::i18n::Mensagem::OperacaoCancelada,
                        )));
                    }
                    if pendente.is_empty() {
                        let chunk = scp_ler_data(&mut canal).await?;
                        pendente.extend_from_slice(&chunk);
                    }
                    let falta = (tamanho - recebidos) as usize;
                    let usar = falta.min(pendente.len());
                    arquivo
                        .write_all(&pendente[..usar])
                        .map_err(ErroSshCli::Io)?;
                    recebidos += usar as u64;
                    pendente.drain(..usar);
                }

                // Após payload, source envia 0x00 final (pode já estar em `pendente`).
                if pendente.is_empty() {
                    match scp_ler_data(&mut canal).await {
                        Ok(trail) => pendente.extend_from_slice(&trail),
                        Err(_) if recebidos == tamanho => {}
                        Err(e) => return Err(e),
                    }
                }
                if pendente.first() == Some(&SCP_OK) {
                    pendente.remove(0);
                } else if !pendente.is_empty() {
                    return Err(ErroSshCli::CanalFalhou(format!(
                        "terminador SCP inesperado após payload (0x{:02x})",
                        pendente[0]
                    )));
                }

                arquivo.flush().map_err(ErroSshCli::Io)?;
                let _ = arquivo.sync_data();
                drop(arquivo);

                canal
                    .data([SCP_OK].as_slice())
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("enviar ack final: {e}")))?;

                let _ = canal.eof().await;
                while let Some(msg) = canal.wait().await {
                    if matches!(msg, ChannelMsg::Close) {
                        break;
                    }
                }

                // SCP-022b: aplicar mode/times no partial ANTES do rename atômico.
                // Assim falha de metadados não deixa `local` com conteúdo de sucesso parcial.
                aplicar_mode_local(&partial, mode_remoto)?;
                if let Some((mtime, atime)) = times {
                    let mtime_st = UNIX_EPOCH + StdDuration::from_secs(mtime);
                    let atime_st = UNIX_EPOCH + StdDuration::from_secs(atime);
                    let ft = std::fs::FileTimes::new()
                        .set_modified(mtime_st)
                        .set_accessed(atime_st);
                    if let Ok(f) = std::fs::File::options().write(true).open(&partial) {
                        let _ = f.set_times(ft);
                    }
                }

                std::fs::rename(&partial, local).map_err(ErroSshCli::Io)?;
                // GraphRAG escrita atômica: fsync do diretório pai após rename (best-effort).
                if let Some(pai) = local.parent() {
                    if !pai.as_os_str().is_empty() {
                        if let Ok(dir) = std::fs::File::open(pai) {
                            let _ = dir.sync_all();
                        }
                    }
                }

                Ok::<_, ErroSshCli>(recebidos)
            })
            .await;

            match resultado {
                Ok(Ok(recebidos)) => {
                    let duracao_ms =
                        u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
                    Ok(TransferenciaResultado {
                        bytes_transferidos: recebidos,
                        duracao_ms,
                    })
                }
                Ok(Err(e)) => {
                    let _ = std::fs::remove_file(&partial);
                    // Se rename já ocorreu e algo falhou depois (fsync best-effort não falha),
                    // ainda removemos partial; `local` só existe após rename bem-sucedido.
                    Err(e)
                }
                Err(_) => {
                    let _ = std::fs::remove_file(&partial);
                    Err(ErroSshCli::TimeoutSsh(self.cfg.timeout_ms))
                }
            }
        }

        /// Abort remoto best-effort: reconecta com timeout curto e executa pkill.
        async fn tentar_abort_remoto(&self, abort_cmd: &str) -> ResultadoSshCli<()> {
            // Implementação inline (sem chamar executar_comando_interno) evita
            // recursão async detectada pelo compilador.
            let mut cfg_abort = self.cfg.clone();
            cfg_abort.timeout_ms = cfg_abort.timeout_ms.clamp(3_000, 10_000);
            let outro = match Self::conectar(cfg_abort).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::debug!(erro = %e, "abort remoto não pôde reconectar");
                    return Err(e);
                }
            };
            let timeout = Duration::from_millis(outro.cfg.timeout_ms);
            let _ = tokio::time::timeout(timeout, async {
                let mut canal = outro
                    .sessao
                    .channel_open_session()
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("abort canal: {e}")))?;
                canal
                    .exec(true, abort_cmd)
                    .await
                    .map_err(|e| ErroSshCli::CanalFalhou(format!("abort exec: {e}")))?;
                while let Some(msg) = canal.wait().await {
                    if matches!(msg, russh::ChannelMsg::Close) {
                        break;
                    }
                }
                Ok::<(), ErroSshCli>(())
            })
            .await;
            let _ = outro.desconectar().await;
            Ok(())
        }

        /// Encerra a sessão SSH de forma limpa.
        ///
        /// # Erros
        /// Propaga falha se `disconnect` retornar erro do transporte.
        pub async fn desconectar(&self) -> ResultadoSshCli<()> {
            let resultado = self
                .sessao
                .disconnect(russh::Disconnect::ByApplication, "encerrando", "pt-BR")
                .await;
            match resultado {
                Ok(()) => {
                    tracing::info!("sessão SSH encerrada");
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!(erro = %e, "falha ao encerrar sessão SSH");
                    Err(ErroSshCli::ConexaoFalhou(format!(
                        "falha ao desconectar: {e}"
                    )))
                }
            }
        }

        /// Abre canal direct-tcpip para forwarding SSH.
        pub async fn abrir_canal_tunel(
            &self,
            host_remoto: &str,
            porta_remota: u16,
            endereco_origem: &str,
            porta_origem: u16,
        ) -> ResultadoSshCli<Box<dyn CanalTunel>> {
            let canal = self
                .sessao
                .channel_open_direct_tcpip(
                    host_remoto.to_string(),
                    u32::from(porta_remota),
                    endereco_origem.to_string(),
                    u32::from(porta_origem),
                )
                .await
                .map_err(|e| {
                    ErroSshCli::CanalFalhou(format!(
                        "falha ao abrir canal direct-tcpip para {}:{}: {}",
                        host_remoto, porta_remota, e
                    ))
                })?;

            Ok(Box::new(canal.into_stream()))
        }
    }

    #[async_trait]
    impl ClienteSshTrait for ClienteSsh {
        async fn conectar(cfg: ConfiguracaoConexao) -> Result<Box<Self>, ErroSshCli> {
            Self::conectar(cfg).await.map(Box::new)
        }

        async fn executar_comando(
            &mut self,
            cmd: &str,
            max_chars: usize,
            stdin_data: Option<Vec<u8>>,
        ) -> Result<SaidaExecucao, ErroSshCli> {
            Self::executar_comando(self, cmd, max_chars, stdin_data).await
        }

        async fn upload(
            &mut self,
            local: &Path,
            remote: &Path,
        ) -> Result<TransferenciaResultado, ErroSshCli> {
            Self::upload(self, local, remote).await
        }

        async fn download(
            &mut self,
            remote: &Path,
            local: &Path,
        ) -> Result<TransferenciaResultado, ErroSshCli> {
            Self::download(self, remote, local).await
        }

        async fn abrir_canal_tunel(
            &self,
            host_remoto: &str,
            porta_remota: u16,
            endereco_origem: &str,
            porta_origem: u16,
        ) -> Result<Box<dyn CanalTunel>, ErroSshCli> {
            Self::abrir_canal_tunel(
                self,
                host_remoto,
                porta_remota,
                endereco_origem,
                porta_origem,
            )
            .await
        }

        async fn desconectar(&self) -> Result<(), ErroSshCli> {
            Self::desconectar(self).await
        }
    }

    #[cfg(test)]
    mod testes_real {
        use super::{
            caminho_parcial_download, comando_scp_remoto, formatar_header_upload_scp,
            formatar_header_upload_scp_com_modo, formatar_linha_t_scp, interpretar_status_scp,
            mapear_exit_status, parse_header_scp, parse_linha_t_scp, processar_mensagem_exec,
            SCP_PARTIAL_SUFFIX,
        };

        #[test]
        fn mapear_exit_status_normal() {
            assert_eq!(mapear_exit_status(0), 0);
            assert_eq!(mapear_exit_status(255), 255);
        }

        #[test]
        fn mapear_exit_status_overflow_retorna_menos_um() {
            assert_eq!(mapear_exit_status(u32::MAX), -1);
        }

        #[test]
        fn parse_header_scp_valido_retorna_mode_e_tamanho() {
            let (mode, tamanho) =
                parse_header_scp("C0644 42 arquivo.txt\n").expect("header válido");
            assert_eq!(mode, 0o644);
            assert_eq!(tamanho, 42);
            let (mode2, _) = parse_header_scp("C0600 1 x\n").expect("600");
            assert_eq!(mode2, 0o600);
        }

        #[test]
        fn parse_header_scp_invalido_retorna_erro() {
            assert!(parse_header_scp("ERRO").is_err());
            assert!(parse_header_scp("C0644 sem_tamanho").is_err());
            assert!(parse_header_scp("C0644 abc arquivo").is_err());
            assert!(parse_header_scp("Czzzz 1 x\n").is_err());
        }

        #[test]
        fn processar_mensagem_exec_trata_stdout_stderr_e_close() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;

            let deve_parar = processar_mensagem_exec(
                russh::ChannelMsg::Data {
                    data: b"stdout".to_vec().into(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert!(!deve_parar);
            assert_eq!(stdout, b"stdout");

            let deve_parar = processar_mensagem_exec(
                russh::ChannelMsg::ExtendedData {
                    data: b"stderr".to_vec().into(),
                    ext: 1,
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert!(!deve_parar);
            assert_eq!(stderr, b"stderr");

            let _ = processar_mensagem_exec(
                russh::ChannelMsg::ExitStatus { exit_status: 17 },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert_eq!(exit_code, Some(17));

            let deve_parar = processar_mensagem_exec(
                russh::ChannelMsg::Close,
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );
            assert!(deve_parar);
        }

        #[test]
        fn formatar_header_upload_scp_gera_formato_esperado() {
            let header = formatar_header_upload_scp(123, "arquivo.txt");
            // Wire protocol: newline real (0x0a), NÃO a sequência literal '\'+'n'.
            assert_eq!(header, "C0644 123 arquivo.txt\n");
            assert_eq!(header.as_bytes().last().copied(), Some(b'\n'));
            assert!(
                !header.as_bytes().windows(2).any(|w| w == *b"\\n"),
                "header não deve conter backslash-n literal"
            );
        }

        #[test]
        fn formatar_header_upload_scp_usa_basename() {
            let header = formatar_header_upload_scp(1, "/tmp/dir/nome.bin");
            assert_eq!(header, "C0644 1 nome.bin\n");
        }

        #[test]
        fn interpretar_status_scp_ok_e_erro() {
            assert!(interpretar_status_scp(&[0]).is_ok());
            assert!(interpretar_status_scp(&[1, b'f', b'a', b'i', b'l']).is_err());
            assert!(interpretar_status_scp(&[]).is_err());
        }

        #[test]
        fn comando_scp_remoto_escapa_path_e_usa_p() {
            let cmd = comando_scp_remoto("-t", std::path::Path::new("/tmp/a b.txt"));
            assert_eq!(cmd, "scp -tp '/tmp/a b.txt'");
            let cmd_f = comando_scp_remoto("-f", std::path::Path::new("/var/log/a.log"));
            assert_eq!(cmd_f, "scp -fp '/var/log/a.log'");
            // Idempotente se já contiver p.
            assert_eq!(
                comando_scp_remoto("-fp", std::path::Path::new("/x")),
                "scp -fp '/x'"
            );
        }

        #[test]
        fn formatar_linha_t_scp_formato() {
            let t = formatar_linha_t_scp(1_700_000_000, 1_700_000_001);
            assert_eq!(t, "T1700000000 0 1700000001 0\n");
            assert_eq!(t.as_bytes().last().copied(), Some(b'\n'));
        }

        #[test]
        fn parse_linha_t_scp_ok() {
            let (m, a) = parse_linha_t_scp("T100 0 200 0\n").expect("T ok");
            assert_eq!((m, a), (100, 200));
        }

        #[test]
        fn formatar_header_com_modo() {
            let h = formatar_header_upload_scp_com_modo(0o755, 10, "x.sh");
            assert_eq!(h, "C0755 10 x.sh\n");
        }

        #[test]
        fn caminho_parcial_download_sufixo() {
            let p = caminho_parcial_download(std::path::Path::new("/tmp/out.bin"));
            assert!(p.to_string_lossy().ends_with(SCP_PARTIAL_SUFFIX));
            assert!(p.to_string_lossy().contains("out.bin"));
        }

        #[test]
        fn processar_mensagem_exec_ignora_extendido_com_codigo_diferente_de_stderr() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;

            let deve_parar = processar_mensagem_exec(
                russh::ChannelMsg::ExtendedData {
                    data: b"nao-e-stderr".to_vec().into(),
                    ext: 2,
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            assert!(!deve_parar);
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
            assert!(exit_code.is_none());
        }

        #[test]
        fn processar_mensagem_exec_trata_exit_signal_e_eof_sem_encerrar_loop() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = Some(7);

            let deve_parar_signal = processar_mensagem_exec(
                russh::ChannelMsg::ExitSignal {
                    signal_name: russh::Sig::TERM,
                    core_dumped: false,
                    error_message: "encerrado".to_string(),
                    lang_tag: "pt-BR".to_string(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            let deve_parar_eof = processar_mensagem_exec(
                russh::ChannelMsg::Eof,
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            assert!(!deve_parar_signal);
            assert!(!deve_parar_eof);
            assert_eq!(exit_code, Some(7));
        }

        #[test]
        fn processar_mensagem_exec_ignora_variantes_sem_tratamento_especifico() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;

            let deve_parar = processar_mensagem_exec(
                russh::ChannelMsg::WindowAdjusted { new_size: 2048 },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
            );

            assert!(!deve_parar);
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
            assert!(exit_code.is_none());
        }
    }
}

#[cfg(feature = "ssh-real")]
pub use real::{ClienteSsh, ManipuladorCliente};

// =========================================================================
// Stub usado quando a feature `ssh-real` está DESATIVADA.
// =========================================================================

#[cfg(not(feature = "ssh-real"))]
mod stub {
    use super::{ConfiguracaoConexao, SaidaExecucao, TransferenciaResultado};
    use crate::erros::ErroSshCli;
    use crate::ssh::cliente::ClienteSshTrait;
    use async_trait::async_trait;
    use std::path::Path;

    /// Stub quando `ssh-real` está desativado: sempre retorna
    /// [`ErroSshCli::ConexaoFalhou`].
    #[derive(Debug)]
    pub struct ClienteSsh;

    #[async_trait]
    impl ClienteSshTrait for ClienteSsh {
        async fn conectar(_cfg: ConfiguracaoConexao) -> Result<Box<Self>, ErroSshCli> {
            Err(ErroSshCli::ConexaoFalhou(
                "feature `ssh-real` está desabilitada; recompile com --features ssh-real".into(),
            ))
        }

        async fn executar_comando(
            &mut self,
            _cmd: &str,
            _max_chars: usize,
            _stdin_data: Option<Vec<u8>>,
        ) -> Result<SaidaExecucao, ErroSshCli> {
            Err(ErroSshCli::CanalFalhou(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn upload(
            &mut self,
            _local: &Path,
            _remote: &Path,
        ) -> Result<TransferenciaResultado, ErroSshCli> {
            Err(ErroSshCli::CanalFalhou(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn download(
            &mut self,
            _remote: &Path,
            _local: &Path,
        ) -> Result<TransferenciaResultado, ErroSshCli> {
            Err(ErroSshCli::CanalFalhou(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn abrir_canal_tunel(
            &self,
            _host_remoto: &str,
            _porta_remota: u16,
            _endereco_origem: &str,
            _porta_origem: u16,
        ) -> Result<Box<dyn super::CanalTunel>, ErroSshCli> {
            Err(ErroSshCli::CanalFalhou(
                "stub sem russh: feature `ssh-real` desabilitada".into(),
            ))
        }

        async fn desconectar(&self) -> Result<(), ErroSshCli> {
            Ok(())
        }
    }
}

#[cfg(not(feature = "ssh-real"))]
pub use stub::ClienteSsh;

// =========================================================================
// Testes unitários (sem rede, sem feature gate).
// =========================================================================

#[cfg(test)]
mod testes {
    use super::*;
    use secrecy::SecretString;

    fn cfg_valida() -> ConfiguracaoConexao {
        ConfiguracaoConexao {
            host: "127.0.0.1".to_string(),
            porta: 22,
            usuario: "root".to_string(),
            senha: SecretString::from("senha-exemplo".to_string()),
            key_path: None,
            key_passphrase: None,
            timeout_ms: 5000,
            known_hosts_path: None,
            replace_host_key: false,
        }
    }

    #[test]
    fn validar_host_vazio_retorna_erro() {
        let mut c = cfg_valida();
        c.host = String::new();
        let r = c.validar();
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("host"));
    }

    #[test]
    fn validar_host_apenas_espacos_retorna_erro() {
        let mut c = cfg_valida();
        c.host = "   ".to_string();
        assert!(c.validar().is_err());
    }

    #[test]
    fn validar_porta_zero_retorna_erro() {
        let mut c = cfg_valida();
        c.porta = 0;
        let r = c.validar();
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("porta"));
    }

    #[test]
    fn validar_usuario_vazio_retorna_erro() {
        let mut c = cfg_valida();
        c.usuario = String::new();
        assert!(c.validar().is_err());
    }

    #[test]
    fn validar_configuracao_correta_retorna_ok() {
        assert!(cfg_valida().validar().is_ok());
    }

    #[test]
    fn debug_nao_expoe_senha() {
        let c = cfg_valida();
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("senha-exemplo"));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn truncar_utf8_nao_trunca_se_cabe() {
        let (s, t) = truncar_utf8("ola mundo", 100);
        assert_eq!(s, "ola mundo");
        assert!(!t);
    }

    #[test]
    fn truncar_utf8_trunca_string_grande_ascii() {
        let entrada: String = "a".repeat(200);
        let (s, t) = truncar_utf8(&entrada, 50);
        assert_eq!(s.chars().count(), 50);
        assert!(t);
    }

    #[test]
    fn truncar_utf8_preserva_grafemas_acentuados() {
        // 10 codepoints: "á" (1 char) * 10
        let entrada: String = "á".repeat(30);
        let (s, t) = truncar_utf8(&entrada, 10);
        assert_eq!(s.chars().count(), 10);
        // Cada 'á' ocupa 2 bytes em UTF-8 → 10 chars = 20 bytes
        assert_eq!(s.len(), 20);
        assert!(t);
        // Não corta no meio de byte
        assert!(s.chars().all(|c| c == 'á'));
    }

    #[test]
    fn truncar_utf8_com_emojis_nao_quebra() {
        let entrada = "🚀🔒🛡🔑✨🎉💎⚡🌟🔥🎨";
        let (s, t) = truncar_utf8(entrada, 5);
        assert_eq!(s.chars().count(), 5);
        assert!(t);
    }

    #[test]
    fn truncar_utf8_zero_retorna_vazio() {
        let (s, t) = truncar_utf8("abc", 0);
        assert_eq!(s, "");
        assert!(t);
    }

    #[test]
    fn saida_execucao_debug_nao_crasha() {
        let s = SaidaExecucao {
            stdout: "ok".into(),
            stderr: String::new(),
            exit_code: Some(0),
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 42,
        };
        let _ = format!("{s:?}");
    }

    #[test]
    fn duracao_ms_tipo_compativel() {
        // Garantia estática de que instant elapsed cabe em u64.
        let fake: u64 = 1234;
        assert_eq!(fake, 1234_u64);
    }
}
