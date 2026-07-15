//! CRUD e persistência de registros de VPS (XDG + TOML atômico + flock).
//!
//! ZERO arquivo `.env` em runtime. Schema v2 com auth senha/chave.

pub mod modelo;

use crate::cli::{AcaoSecrets, AcaoVps, FormatoSaida};
use crate::erros::{ErroSshCli, ResultadoSshCli};
use crate::output;
use crate::ssh::cliente::{ClienteSsh, ClienteSshTrait, ConfiguracaoConexao};
use crate::ssh::known_hosts::KnownHosts;
use crate::ssh::packing::{anexar_description, empacotar_su, empacotar_sudo};
use anyhow::Result;
use modelo::{limite_efetivo, parse_limite_chars, VpsRegistro};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Arquivo de configuração completo.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArquivoConfig {
    /// Versão do schema do arquivo.
    #[serde(default)]
    pub schema_version: u32,
    /// Mapa de VPSs por nome.
    #[serde(default)]
    pub hosts: BTreeMap<String, VpsRegistro>,
}

/// Resolve o caminho do arquivo de config a partir de um override opcional.
pub fn resolver_caminho_config(override_path: Option<PathBuf>) -> ResultadoSshCli<PathBuf> {
    match override_path {
        Some(p) => {
            if p.is_dir() {
                return Ok(p.join("config.toml"));
            }
            if p.extension().and_then(|e| e.to_str()) == Some("toml") {
                return Ok(p);
            }
            Ok(p.join("config.toml"))
        }
        None => caminho_config_padrao(),
    }
}

/// Retorna o caminho do arquivo de config respeitando `SSH_CLI_HOME`.
pub fn caminho_config_padrao() -> ResultadoSshCli<PathBuf> {
    if let Ok(home) = std::env::var("SSH_CLI_HOME") {
        if home.contains("..") {
            return Err(ErroSshCli::ArgumentoInvalido(
                "SSH_CLI_HOME não pode conter '..'".to_string(),
            ));
        }
        return Ok(PathBuf::from(home).join("config.toml"));
    }

    let dirs = directories::ProjectDirs::from("", "", "ssh-cli").ok_or_else(|| {
        ErroSshCli::Generico("não foi possível resolver diretório de config".to_string())
    })?;
    Ok(dirs.config_dir().join("config.toml"))
}

/// Camada vencedora de configuração (doctor).
#[derive(Debug, Clone)]
pub struct CamadaConfig {
    /// Nome da camada.
    pub nome: &'static str,
    /// Path resolvido.
    pub path: PathBuf,
}

/// Resolve e descreve a camada de config vencedora.
pub fn camada_vencedora(override_path: Option<PathBuf>) -> ResultadoSshCli<CamadaConfig> {
    if override_path.is_some() {
        return Ok(CamadaConfig {
            nome: "--config-dir",
            path: resolver_caminho_config(override_path)?,
        });
    }
    if std::env::var("SSH_CLI_HOME").is_ok() {
        return Ok(CamadaConfig {
            nome: "SSH_CLI_HOME",
            path: caminho_config_padrao()?,
        });
    }
    Ok(CamadaConfig {
        nome: "XDG ProjectDirs",
        path: caminho_config_padrao()?,
    })
}

/// Carrega o arquivo de configuração (retorna vazio se não existir).
pub fn carregar(caminho: &PathBuf) -> ResultadoSshCli<ArquivoConfig> {
    if !caminho.exists() {
        return Ok(ArquivoConfig {
            schema_version: modelo::SCHEMA_VERSION_ATUAL,
            hosts: BTreeMap::new(),
        });
    }
    let conteudo = std::fs::read_to_string(caminho)?;
    let mut arquivo: ArquivoConfig = toml::from_str(&conteudo)?;
    for reg in arquivo.hosts.values_mut() {
        reg.normalizar_schema();
    }
    if arquivo.schema_version < modelo::SCHEMA_VERSION_ATUAL {
        arquivo.schema_version = modelo::SCHEMA_VERSION_ATUAL;
    }
    Ok(arquivo)
}

/// Escreve bytes em `caminho` de forma atômica (tempfile + fsync + rename + 0o600).
///
/// Usado por `salvar` e `export` (GAP-007 residual no export).
pub fn escrever_atomico(caminho: &Path, bytes: &[u8]) -> ResultadoSshCli<()> {
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai)?;
    }
    let pai = caminho
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&pai)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_data()?;
    tmp.persist(caminho).map_err(|e| ErroSshCli::Io(e.error))?;
    aplicar_permissoes_600(caminho)?;
    #[cfg(unix)]
    {
        if let Ok(dir) = std::fs::File::open(&pai) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

/// Salva o arquivo de configuração de forma atômica com flock e 0o600.
pub fn salvar(caminho: &Path, arquivo: &ArquivoConfig) -> ResultadoSshCli<()> {
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai)?;
    }
    let texto = toml::to_string_pretty(arquivo)
        .map_err(|e| ErroSshCli::Generico(format!("falha serializando TOML: {e}")))?;

    // Lock em arquivo irmão para serializar mutações concorrentes (N one-shots).
    let lock_path = caminho.with_extension("toml.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)?;
    fs2::FileExt::lock_exclusive(&lock_file)?;

    escrever_atomico(caminho, texto.as_bytes())?;

    let _ = fs2::FileExt::unlock(&lock_file);
    Ok(())
}

#[cfg(unix)]
fn aplicar_permissoes_600(caminho: &Path) -> ResultadoSshCli<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissoes = std::fs::metadata(caminho)?.permissions();
    permissoes.set_mode(0o600);
    std::fs::set_permissions(caminho, permissoes)?;
    Ok(())
}

#[cfg(not(unix))]
fn aplicar_permissoes_600(_caminho: &Path) -> ResultadoSshCli<()> {
    Ok(())
}

/// Lê uma linha de senha de stdin (sem eco extra).
pub fn ler_segredo_stdin() -> ResultadoSshCli<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf.trim_end_matches(['\r', '\n']).to_string())
}

/// Aplica overrides de runtime sobre um VpsRegistro clonado.
fn aplicar_overrides(
    vps: &mut VpsRegistro,
    password_override: Option<String>,
    sudo_password_override: Option<String>,
    su_password_override: Option<String>,
    timeout_override: Option<u64>,
    key_path_override: Option<String>,
    key_passphrase_override: Option<String>,
) {
    if let Some(pwd) = password_override {
        vps.senha = SecretString::from(pwd);
    }
    if let Some(spwd) = sudo_password_override {
        vps.senha_sudo = Some(SecretString::from(spwd));
    }
    if let Some(sp) = su_password_override {
        vps.senha_su = Some(SecretString::from(sp));
    }
    if let Some(t) = timeout_override {
        vps.timeout_ms = t;
    }
    if let Some(k) = key_path_override {
        vps.key_path = Some(k);
    }
    if let Some(kp) = key_passphrase_override {
        vps.key_passphrase = Some(SecretString::from(kp));
    }
}

fn validar_comando_tamanho(comando: &str, max_command_chars: usize) -> ResultadoSshCli<()> {
    let lim = limite_efetivo(max_command_chars);
    let len = comando.chars().count();
    if len > lim {
        return Err(ErroSshCli::ComandoMuitoLongo {
            max: max_command_chars,
            len,
        });
    }
    if comando.trim().is_empty() {
        return Err(ErroSshCli::ArgumentoInvalido("comando vazio".to_string()));
    }
    Ok(())
}

/// Dispatcher dos subcomandos `vps`.
pub async fn executar_comando_vps(
    acao: AcaoVps,
    config_override: Option<PathBuf>,
    _formato: FormatoSaida,
) -> Result<()> {
    let caminho = resolver_caminho_config(config_override.clone())?;

    match acao {
        AcaoVps::Add {
            name,
            host,
            port,
            user,
            password,
            password_stdin,
            key,
            key_passphrase,
            timeout,
            max_command_chars,
            max_output_chars,
            max_chars,
            sudo_password,
            sudo_password_stdin,
            su_password,
            su_password_stdin,
            disable_sudo,
            check,
        } => {
            let name = crate::paths::normalizar_nfc(&name);
            let mut arquivo = carregar(&caminho)?;
            if arquivo.hosts.contains_key(&name) {
                return Err(ErroSshCli::VpsDuplicada(name).into());
            }
            if password_stdin && (sudo_password_stdin || su_password_stdin) {
                return Err(ErroSshCli::ArgumentoInvalido(
                    "apenas um --*-stdin por invocação one-shot; rode vps edit para sudo/su".into(),
                )
                .into());
            }
            let senha = if password_stdin {
                SecretString::from(ler_segredo_stdin()?)
            } else {
                SecretString::from(password.unwrap_or_default())
            };
            let sudo_s = if sudo_password_stdin {
                Some(SecretString::from(ler_segredo_stdin()?))
            } else {
                sudo_password.map(SecretString::from)
            };
            let su_s = if su_password_stdin {
                Some(SecretString::from(ler_segredo_stdin()?))
            } else {
                su_password.map(SecretString::from)
            };
            // max_chars legado → command se max_command não veio explicitamente
            let max_cmd = max_command_chars
                .as_deref()
                .or(max_chars.as_deref())
                .map(parse_limite_chars)
                .unwrap_or(modelo::MAX_COMMAND_CHARS_PADRAO);
            let max_out = max_output_chars
                .as_deref()
                .map(parse_limite_chars)
                .unwrap_or(modelo::MAX_OUTPUT_CHARS_PADRAO);
            let registro = VpsRegistro::novo(
                name.clone(),
                host,
                port,
                user,
                senha,
                key,
                key_passphrase.map(SecretString::from),
                Some(timeout),
                Some(max_cmd),
                Some(max_out),
                sudo_s,
                su_s,
                disable_sudo,
            );
            registro
                .validar_credenciais()
                .map_err(ErroSshCli::ArgumentoInvalido)?;
            arquivo.hosts.insert(name.clone(), registro);
            arquivo.schema_version = modelo::SCHEMA_VERSION_ATUAL;
            salvar(&caminho, &arquivo)?;
            crate::output::imprimir_sucesso(&format!("VPS '{name}' adicionada ao registro"));
            if check {
                executar_health_check(Some(&name), config_override, FormatoSaida::Text, None)
                    .await?;
            }
        }
        AcaoVps::List { json } => {
            let arquivo = carregar(&caminho)?;
            let registros: Vec<_> = arquivo.hosts.values().cloned().collect();
            if json {
                crate::output::imprimir_lista_json(&registros);
            } else {
                crate::output::imprimir_lista_texto(&registros);
            }
        }
        AcaoVps::Remove { nome } => {
            let mut arquivo = carregar(&caminho)?;
            if arquivo.hosts.remove(&nome).is_none() {
                return Err(ErroSshCli::VpsNaoEncontrada(nome).into());
            }
            salvar(&caminho, &arquivo)?;
            crate::output::imprimir_sucesso(&format!("VPS '{nome}' removida"));
        }
        AcaoVps::Edit {
            nome,
            host,
            port,
            user,
            password,
            password_stdin,
            key,
            key_passphrase,
            timeout,
            max_command_chars,
            max_output_chars,
            max_chars,
            sudo_password,
            sudo_password_stdin,
            su_password,
            su_password_stdin,
            disable_sudo,
        } => {
            let mut arquivo = carregar(&caminho)?;
            let registro = arquivo
                .hosts
                .get_mut(&nome)
                .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(nome.clone()))?;
            if let Some(h) = host {
                registro.host = h;
            }
            if let Some(p) = port {
                registro.porta = p;
            }
            if let Some(u) = user {
                registro.usuario = u;
            }
            if password_stdin {
                registro.senha = SecretString::from(ler_segredo_stdin()?);
            } else if let Some(pw) = password {
                registro.senha = SecretString::from(pw);
            }
            if let Some(k) = key {
                registro.key_path = Some(k);
            }
            if let Some(kp) = key_passphrase {
                registro.key_passphrase = Some(SecretString::from(kp));
            }
            if let Some(t) = timeout {
                registro.timeout_ms = t;
            }
            if let Some(m) = max_command_chars.or(max_chars) {
                registro.max_command_chars = parse_limite_chars(&m);
            }
            if let Some(m) = max_output_chars {
                registro.max_output_chars = parse_limite_chars(&m);
            }
            if sudo_password_stdin {
                registro.senha_sudo = Some(SecretString::from(ler_segredo_stdin()?));
            } else if let Some(sp) = sudo_password {
                registro.senha_sudo = Some(SecretString::from(sp));
            }
            if su_password_stdin {
                registro.senha_su = Some(SecretString::from(ler_segredo_stdin()?));
            } else if let Some(sp) = su_password {
                registro.senha_su = Some(SecretString::from(sp));
            }
            if let Some(d) = disable_sudo {
                registro.disable_sudo = d;
            }
            registro
                .validar_credenciais()
                .map_err(ErroSshCli::ArgumentoInvalido)?;
            salvar(&caminho, &arquivo)?;
            crate::output::imprimir_sucesso(&format!("VPS '{nome}' editada"));
        }
        AcaoVps::Show { nome, json } => {
            let arquivo = carregar(&caminho)?;
            let registro = arquivo
                .hosts
                .get(&nome)
                .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(nome.clone()))?;
            if json {
                crate::output::imprimir_detalhes_json(registro);
            } else {
                crate::output::imprimir_detalhes_texto(registro);
            }
        }
        AcaoVps::Path => {
            crate::output::escrever_linha(&caminho.display().to_string())?;
        }
        AcaoVps::Doctor { json } => {
            executar_doctor(config_override, json)?;
        }
        AcaoVps::Export {
            include_secrets,
            output,
        } => {
            executar_export(&caminho, include_secrets, output.as_deref())?;
        }
        AcaoVps::Import { file } => {
            executar_import(&caminho, &file)?;
        }
    }
    Ok(())
}

fn executar_doctor(config_override: Option<PathBuf>, json: bool) -> Result<()> {
    let camada = camada_vencedora(config_override.clone())?;
    let caminho = camada.path.clone();
    let existe = caminho.exists();
    let arquivo = carregar(&caminho)?;
    let kh = KnownHosts::caminho_ao_lado_config(&caminho);
    let active = caminho
        .parent()
        .map(|p| p.join("active"))
        .unwrap_or_else(|| PathBuf::from("active"));
    let perms = if existe {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            format!(
                "{:o}",
                std::fs::metadata(&caminho)?.permissions().mode() & 0o777
            )
        }
        #[cfg(not(unix))]
        {
            "n/a".to_string()
        }
    } else {
        "ausente".to_string()
    };

    let seg = crate::secrets::status_segredos()?;
    if json {
        let v = serde_json::json!({
            "layer": camada.nome,
            "config_path": caminho.display().to_string(),
            "exists": existe,
            "permissions": perms,
            "schema_version": arquivo.schema_version,
            "hosts": arquivo.hosts.len(),
            "known_hosts": kh.display().to_string(),
            "active_file": active.display().to_string(),
            "secrets_at_rest": if seg.cifragem_ativa { "encrypted" } else { "plaintext" },
            "secrets_key_source": seg.fonte.as_str(),
            "secrets_key_file": seg.key_file_path.display().to_string(),
            "secrets_plaintext_opt_out": seg.plaintext_opt_out,
            "telemetry": false,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!("Camada vencedora: {}", camada.nome);
        println!("Config path:      {}", caminho.display());
        println!("Existe:           {existe}");
        println!("Permissões:       {perms}");
        println!("Schema:           {}", arquivo.schema_version);
        println!("Hosts:            {}", arquivo.hosts.len());
        println!("known_hosts:      {}", kh.display());
        println!("active file:      {}", active.display());
        println!(
            "Secrets at-rest:  {} (key source: {})",
            if seg.cifragem_ativa {
                "encrypted"
            } else {
                "plaintext"
            },
            seg.fonte.as_str()
        );
        println!("Secrets key file: {}", seg.key_file_path.display());
        println!(
            "Plaintext opt-out: {}",
            if seg.plaintext_opt_out { "yes" } else { "no" }
        );
        println!("Telemetria:       desabilitada");
    }
    Ok(())
}

fn executar_export(caminho: &PathBuf, include_secrets: bool, output: Option<&str>) -> Result<()> {
    let arquivo = carregar(caminho)?;
    let mut export = ArquivoConfig {
        schema_version: arquivo.schema_version,
        hosts: BTreeMap::new(),
    };
    for (k, mut v) in arquivo.hosts {
        if !include_secrets {
            v.senha = SecretString::from(String::new());
            v.senha_sudo = None;
            v.senha_su = None;
            v.key_passphrase = None;
        }
        export.hosts.insert(k, v);
    }
    let texto = toml::to_string_pretty(&export)?;
    if let Some(path) = output {
        escrever_atomico(Path::new(path), texto.as_bytes())?;
        crate::output::imprimir_sucesso(&format!("exportado para {path}"));
    } else {
        print!("{texto}");
    }
    Ok(())
}

fn executar_import(caminho: &PathBuf, file: &Path) -> Result<()> {
    let texto = std::fs::read_to_string(file)?;
    let importado: ArquivoConfig = toml::from_str(&texto)?;
    let mut atual = carregar(caminho)?;
    for (k, v) in importado.hosts {
        v.validar_credenciais()
            .map_err(ErroSshCli::ArgumentoInvalido)?;
        atual.hosts.insert(k, v);
    }
    atual.schema_version = modelo::SCHEMA_VERSION_ATUAL;
    salvar(caminho, &atual)?;
    crate::output::imprimir_sucesso("importação concluída");
    Ok(())
}

/// Dispatcher one-shot de `secrets status|init|reencrypt`.
pub async fn executar_comando_secrets(
    acao: AcaoSecrets,
    config_override: Option<PathBuf>,
    formato: FormatoSaida,
) -> Result<()> {
    // Garante alinhamento do secrets.key com --config-dir.
    crate::secrets::definir_diretorio_config(config_override.clone());
    match acao {
        AcaoSecrets::Status { json } => {
            let seg = crate::secrets::status_segredos()?;
            let usar_json = json || formato == FormatoSaida::Json;
            if usar_json {
                let v = serde_json::json!({
                    "encryption_active": seg.cifragem_ativa,
                    "key_source": seg.fonte.as_str(),
                    "key_file": seg.key_file_path.display().to_string(),
                    "plaintext_opt_out": seg.plaintext_opt_out,
                    "at_rest": if seg.cifragem_ativa { "encrypted" } else { "plaintext" },
                });
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!(
                    "at-rest: {} | source: {} | key_file: {} | plaintext_opt_out: {}",
                    if seg.cifragem_ativa {
                        "encrypted"
                    } else {
                        "plaintext"
                    },
                    seg.fonte.as_str(),
                    seg.key_file_path.display(),
                    seg.plaintext_opt_out
                );
            }
            Ok(())
        }
        AcaoSecrets::Init { keyring, force } => {
            let seg = crate::secrets::init_master_key(keyring, force)?;
            crate::output::imprimir_sucesso(&format!(
                "master-key pronta (source={}; key_file={})",
                seg.fonte.as_str(),
                seg.key_file_path.display()
            ));
            Ok(())
        }
        AcaoSecrets::Reencrypt => {
            let caminho = resolver_caminho_config(config_override)?;
            executar_reencrypt(&caminho)?;
            Ok(())
        }
    }
}

/// Recarrega e regrava o config, re-cifando secrets com a chave atual.
fn executar_reencrypt(caminho: &PathBuf) -> Result<()> {
    let (chave, fonte) = crate::secrets::garantir_chave_para_escrita()?;
    if chave.is_none() {
        return Err(ErroSshCli::ArgumentoInvalido(
            "sem master-key; rode `ssh-cli secrets init` ou remova SSH_CLI_ALLOW_PLAINTEXT_SECRETS"
                .to_string(),
        )
        .into());
    }
    if let Some(mut k) = chave {
        use zeroize::Zeroize;
        k.zeroize();
    }
    let arquivo = carregar(caminho)?;
    salvar(caminho, &arquivo)?;
    crate::output::imprimir_sucesso(&format!(
        "reencrypt ok (source={}; hosts={})",
        fonte.as_str(),
        arquivo.hosts.len()
    ));
    Ok(())
}

/// Define a VPS ativa gravando seu nome em `<config_dir>/active` (arquivo irmão).
pub async fn executar_connect(nome: &str, config_override: Option<PathBuf>) -> Result<()> {
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo = carregar(&caminho)?;
    if !arquivo.hosts.contains_key(nome) {
        return Err(ErroSshCli::VpsNaoEncontrada(nome.to_string()).into());
    }

    let arquivo_ativo = caminho
        .parent()
        .map(|p| p.join("active"))
        .unwrap_or_else(|| PathBuf::from("active"));
    if let Some(pai) = arquivo_ativo.parent() {
        std::fs::create_dir_all(pai)?;
    }
    // escrita atômica do active
    let pai = arquivo_ativo
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&pai)?;
    tmp.write_all(nome.as_bytes())?;
    tmp.as_file().sync_data()?;
    tmp.persist(&arquivo_ativo)
        .map_err(|e| ErroSshCli::Io(e.error))?;
    crate::output::imprimir_sucesso(&format!("VPS ativa definida: '{nome}'"));
    Ok(())
}

/// Busca um registro de VPS por nome.
pub fn buscar_por_nome(
    config_override: Option<PathBuf>,
    nome: &str,
) -> ResultadoSshCli<Option<VpsRegistro>> {
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo = carregar(&caminho)?;
    Ok(arquivo.hosts.get(nome).cloned())
}

/// Lê o nome da VPS ativa.
pub fn ler_vps_ativa(config_override: Option<PathBuf>) -> ResultadoSshCli<Option<String>> {
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo_ativo = caminho
        .parent()
        .map(|p| p.join("active"))
        .unwrap_or_else(|| PathBuf::from("active"));
    if !arquivo_ativo.exists() {
        return Ok(None);
    }
    let nome = std::fs::read_to_string(&arquivo_ativo)?;
    Ok(Some(nome.trim().to_string()))
}

/// Constrói `ConfiguracaoConexao` a partir de um `VpsRegistro`.
pub fn construir_configuracao(
    vps: &VpsRegistro,
    config_toml: Option<&Path>,
    replace_host_key: bool,
) -> ConfiguracaoConexao {
    let known_hosts_path = config_toml.map(KnownHosts::caminho_ao_lado_config);
    ConfiguracaoConexao {
        host: vps.host.clone(),
        porta: vps.porta,
        usuario: vps.usuario.clone(),
        senha: vps.senha.clone(),
        key_path: vps.key_path.clone(),
        key_passphrase: vps.key_passphrase.clone(),
        timeout_ms: vps.timeout_ms,
        known_hosts_path,
        replace_host_key,
    }
}

/// Opções comuns de execução remota.
#[derive(Debug, Default, Clone)]
pub struct OpcoesExec {
    /// Override senha.
    pub password: Option<String>,
    /// Override sudo.
    pub sudo_password: Option<String>,
    /// Override su.
    pub su_password: Option<String>,
    /// Override timeout.
    pub timeout: Option<u64>,
    /// Override key path.
    pub key: Option<String>,
    /// Override key passphrase.
    pub key_passphrase: Option<String>,
    /// Optional shell description comment.
    pub description: Option<String>,
    /// replace host key.
    pub replace_host_key: bool,
    /// disable sudo global.
    pub disable_sudo: bool,
}

/// Executa um comando em uma VPS via SSH.
pub async fn executar_exec(
    vps_nome: &str,
    comando: &str,
    config_override: Option<PathBuf>,
    formato: FormatoSaida,
    json: bool,
    opts: OpcoesExec,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Mensagem::OperacaoCancelada
        )));
    }
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo = carregar(&caminho)?;
    let vps_base = arquivo
        .hosts
        .get(vps_nome)
        .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(vps_nome.to_string()))?;

    let mut vps = vps_base.clone();
    aplicar_overrides(
        &mut vps,
        opts.password,
        opts.sudo_password,
        opts.su_password,
        opts.timeout,
        opts.key,
        opts.key_passphrase,
    );
    let cmd = anexar_description(comando, opts.description.as_deref());
    validar_comando_tamanho(&cmd, vps.max_command_chars)?;
    let cfg = construir_configuracao(&vps, Some(&caminho), opts.replace_host_key);
    let cliente: Box<dyn ClienteSshTrait> = <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
    executar_exec_with_client(&vps, &cmd, cliente, formato, json).await
}

/// Versão testável de executar_exec.
pub async fn executar_exec_with_client(
    vps: &VpsRegistro,
    comando: &str,
    mut cliente: Box<dyn ClienteSshTrait>,
    formato: FormatoSaida,
    json: bool,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Mensagem::OperacaoCancelada
        )));
    }
    let max_out = limite_efetivo(vps.max_output_chars);
    let saida = cliente.executar_comando(comando, max_out).await?;
    cliente.desconectar().await?;
    if formato == FormatoSaida::Json || json {
        output::imprimir_saida_execucao_json(&saida);
    } else {
        output::imprimir_saida_execucao(&saida);
    }
    if let Some(code) = saida.exit_code {
        if code != 0 {
            return Err(ErroSshCli::ComandoFalhou {
                exit_code: code,
                stderr: saida.stderr.clone(),
            }
            .into());
        }
    }
    Ok(())
}

/// Executa um comando com `sudo` (packing `sh -c`).
pub async fn executar_sudo_exec(
    vps_nome: &str,
    comando: &str,
    config_override: Option<PathBuf>,
    formato: FormatoSaida,
    json: bool,
    opts: OpcoesExec,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Mensagem::OperacaoCancelada
        )));
    }
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo = carregar(&caminho)?;
    let vps_base = arquivo
        .hosts
        .get(vps_nome)
        .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(vps_nome.to_string()))?;

    let mut vps = vps_base.clone();
    aplicar_overrides(
        &mut vps,
        opts.password.clone(),
        opts.sudo_password.clone(),
        opts.su_password.clone(),
        opts.timeout,
        opts.key.clone(),
        opts.key_passphrase.clone(),
    );
    if opts.disable_sudo || vps.disable_sudo {
        return Err(ErroSshCli::SudoDesabilitado.into());
    }
    let cmd = anexar_description(comando, opts.description.as_deref());
    validar_comando_tamanho(&cmd, vps.max_command_chars)?;
    let cfg = construir_configuracao(&vps, Some(&caminho), opts.replace_host_key);
    let cliente: Box<dyn ClienteSshTrait> = <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
    executar_sudo_exec_with_client(&vps, &cmd, cliente, formato, json).await
}

/// Versão testável de sudo-exec.
pub async fn executar_sudo_exec_with_client(
    vps: &VpsRegistro,
    comando: &str,
    mut cliente: Box<dyn ClienteSshTrait>,
    formato: FormatoSaida,
    json: bool,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Mensagem::OperacaoCancelada
        )));
    }
    if vps.disable_sudo {
        return Err(ErroSshCli::SudoDesabilitado.into());
    }
    let sudo_cmd = empacotar_sudo(comando, vps.senha_sudo.as_ref());
    let max_out = limite_efetivo(vps.max_output_chars);
    let saida = cliente.executar_comando(&sudo_cmd, max_out).await?;
    cliente.desconectar().await?;
    if formato == FormatoSaida::Json || json {
        output::imprimir_saida_execucao_json(&saida);
    } else {
        output::imprimir_saida_execucao(&saida);
    }
    if let Some(code) = saida.exit_code {
        if code != 0 {
            return Err(ErroSshCli::ComandoFalhou {
                exit_code: code,
                stderr: saida.stderr.clone(),
            }
            .into());
        }
    }
    Ok(())
}

/// Executa comando via `su -` one-shot (consome `senha_su`).
pub async fn executar_su_exec(
    vps_nome: &str,
    comando: &str,
    config_override: Option<PathBuf>,
    formato: FormatoSaida,
    json: bool,
    opts: OpcoesExec,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Mensagem::OperacaoCancelada
        )));
    }
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo = carregar(&caminho)?;
    let vps_base = arquivo
        .hosts
        .get(vps_nome)
        .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(vps_nome.to_string()))?;

    let mut vps = vps_base.clone();
    aplicar_overrides(
        &mut vps,
        opts.password,
        opts.sudo_password,
        opts.su_password,
        opts.timeout,
        opts.key,
        opts.key_passphrase,
    );
    if opts.disable_sudo || vps.disable_sudo {
        return Err(ErroSshCli::SudoDesabilitado.into());
    }
    let senha_su = vps.senha_su.clone().ok_or(ErroSshCli::SenhaSuAusente)?;
    let cmd = anexar_description(comando, opts.description.as_deref());
    validar_comando_tamanho(&cmd, vps.max_command_chars)?;
    let su_cmd = empacotar_su(&cmd, &senha_su);
    let cfg = construir_configuracao(&vps, Some(&caminho), opts.replace_host_key);
    let mut cliente: Box<dyn ClienteSshTrait> =
        <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
    let max_out = limite_efetivo(vps.max_output_chars);
    let saida = cliente.executar_comando(&su_cmd, max_out).await?;
    cliente.desconectar().await?;
    if formato == FormatoSaida::Json || json {
        output::imprimir_saida_execucao_json(&saida);
    } else {
        output::imprimir_saida_execucao(&saida);
    }
    if let Some(code) = saida.exit_code {
        if code != 0 {
            return Err(ErroSshCli::ComandoFalhou {
                exit_code: code,
                stderr: saida.stderr.clone(),
            }
            .into());
        }
    }
    Ok(())
}

/// Health-check SSH.
pub async fn executar_health_check(
    vps_nome: Option<&str>,
    config_override: Option<PathBuf>,
    formato: FormatoSaida,
    password_override: Option<String>,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Mensagem::OperacaoCancelada
        )));
    }
    let nome_resolvido: String = match vps_nome {
        Some(n) => n.to_string(),
        None => {
            let ativa = ler_vps_ativa(config_override.clone())?;
            ativa.ok_or_else(|| {
                anyhow::anyhow!(crate::i18n::t(crate::i18n::Mensagem::HealthCheckSemVps))
            })?
        }
    };
    let caminho = resolver_caminho_config(config_override)?;
    let arquivo = carregar(&caminho)?;
    let vps_base = arquivo
        .hosts
        .get(&nome_resolvido)
        .ok_or_else(|| ErroSshCli::VpsNaoEncontrada(nome_resolvido.clone()))?;

    let mut vps = vps_base.clone();
    aplicar_overrides(&mut vps, password_override, None, None, None, None, None);
    let cfg = construir_configuracao(&vps, Some(&caminho), false);
    let inicio = std::time::Instant::now();
    let cliente: Box<dyn ClienteSshTrait> = <ClienteSsh as ClienteSshTrait>::conectar(cfg).await?;
    let latencia_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    cliente.desconectar().await?;

    if formato == FormatoSaida::Json {
        output::imprimir_health_check_json(&nome_resolvido, latencia_ms);
    } else {
        output::imprimir_health_check(&nome_resolvido, latencia_ms);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use secrecy::ExposeSecret;
    use tempfile::TempDir;

    fn reg_min() -> VpsRegistro {
        VpsRegistro::novo(
            "srv".into(),
            "host.example.com".into(),
            2222,
            "admin".into(),
            SecretString::from("pass".to_string()),
            None,
            None,
            Some(60_000),
            Some(1_000),
            Some(50_000),
            None,
            None,
            false,
        )
    }

    #[test]
    fn arquivo_vazio_serializa_com_schema() {
        let arq = ArquivoConfig {
            schema_version: modelo::SCHEMA_VERSION_ATUAL,
            hosts: BTreeMap::new(),
        };
        let texto = toml::to_string(&arq).unwrap();
        assert!(texto.contains("schema_version = 2"));
    }

    #[test]
    fn parse_limite_none() {
        assert_eq!(parse_limite_chars("none"), 0);
        assert_eq!(parse_limite_chars("0"), 0);
        assert_eq!(parse_limite_chars("1000"), 1000);
    }

    #[test]
    fn construir_configuracao_copia_campos() {
        let registro = reg_min();
        let cfg = construir_configuracao(&registro, None, false);
        assert_eq!(cfg.host, "host.example.com");
        assert_eq!(cfg.porta, 2222);
        assert_eq!(cfg.usuario, "admin");
        assert_eq!(cfg.timeout_ms, 60_000);
        assert!(cfg.known_hosts_path.is_none());
    }

    #[test]
    fn salvar_atomico_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut arq = ArquivoConfig {
            schema_version: 2,
            hosts: BTreeMap::new(),
        };
        arq.hosts.insert("a".into(), reg_min());
        salvar(&path, &arq).unwrap();
        let lido = carregar(&path).unwrap();
        assert_eq!(lido.hosts.len(), 1);
        assert_eq!(lido.hosts["a"].senha.expose_secret(), "pass");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn resolver_caminho_config_com_override_diretorio() {
        let resultado = resolver_caminho_config(Some(PathBuf::from("/tmp/test-dir")));
        assert_eq!(
            resultado.unwrap(),
            PathBuf::from("/tmp/test-dir/config.toml")
        );
    }

    #[test]
    fn validar_comando_longo() {
        let err = validar_comando_tamanho(&"x".repeat(20), 10).unwrap_err();
        assert!(matches!(err, ErroSshCli::ComandoMuitoLongo { .. }));
    }

    #[test]
    fn empacotar_sudo_integrado() {
        let cmd = empacotar_sudo("ls -la", None);
        assert_eq!(cmd, "sudo -n sh -c 'ls -la'");
    }

    #[test]
    fn aplicar_overrides_senha() {
        let mut v = reg_min();
        aplicar_overrides(
            &mut v,
            Some("nova".into()),
            Some("sudo".into()),
            None,
            Some(1000),
            Some("/k".into()),
            None,
        );
        assert_eq!(v.senha.expose_secret(), "nova");
        assert_eq!(v.timeout_ms, 1000);
        assert_eq!(v.key_path.as_deref(), Some("/k"));
    }

    #[tokio::test]
    async fn executar_sudo_exec_with_client_ok() {
        use crate::ssh::cliente::mocks::MockClienteSsh;
        use crate::ssh::cliente::SaidaExecucao;
        use mockall::predicate::*;

        let mut mock = MockClienteSsh::new();
        mock.expect_executar_comando()
            .with(function(|c: &str| c.contains("sudo -n sh -c")), always())
            .returning(|_, _| {
                Ok(SaidaExecucao {
                    stdout: "ok".into(),
                    stderr: String::new(),
                    exit_code: Some(0),
                    truncado_stdout: false,
                    truncado_stderr: false,
                    duracao_ms: 1,
                })
            });
        mock.expect_desconectar().returning(|| Ok(()));

        let vps = reg_min();
        executar_sudo_exec_with_client(&vps, "id", Box::new(mock), FormatoSaida::Text, false)
            .await
            .unwrap();
    }
}
