// SPDX-License-Identifier: MIT OR Apache-2.0
//! CRUD e persistência de registros de VPS (XDG + TOML atômico + flock).
//!
//! ZERO arquivo `.env` em runtime. Schema v2 com auth password/chave.

pub mod model;

use crate::cli::{SecretsAction, VpsAction, OutputFormat};
use crate::erros::{SshCliError, SshCliResult};
use crate::output;
use crate::ssh::client::{SshClient, SshClientTrait, ConnectionConfig};
use crate::ssh::known_hosts::KnownHosts;
use crate::ssh::packing::{append_description, pack_su, pack_sudo};
use anyhow::Result;
use model::{effective_limit, parse_char_limit, VpsRecord};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Arquivo de configuração completo.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Versão do schema do arquivo.
    #[serde(default)]
    pub schema_version: u32,
    /// Mapa de VPSs por name.
    #[serde(default)]
    pub hosts: BTreeMap<String, VpsRecord>,
}

/// Resolve o path do arquivo de config a partir de um override opcional.
pub fn resolve_config_path(override_path: Option<PathBuf>) -> SshCliResult<PathBuf> {
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
        None => default_config_path(),
    }
}

/// Retorna o path do arquivo de config respeitando `SSH_CLI_HOME`.
pub fn default_config_path() -> SshCliResult<PathBuf> {
    if let Ok(home) = std::env::var("SSH_CLI_HOME") {
        if home.contains("..") {
            return Err(SshCliError::InvalidArgument(
                "SSH_CLI_HOME não pode conter '..'".to_string(),
            ));
        }
        return Ok(PathBuf::from(home).join("config.toml"));
    }

    let dirs = directories::ProjectDirs::from("", "", "ssh-cli").ok_or_else(|| {
        SshCliError::Generic("não foi possível resolver diretório de config".to_string())
    })?;
    Ok(dirs.config_dir().join("config.toml"))
}

/// Camada vencedora de configuração (doctor).
#[derive(Debug, Clone)]
pub struct CamadaConfig {
    /// Nome da camada.
    pub name: &'static str,
    /// Path resolvido.
    pub path: PathBuf,
}

/// Resolve e descreve a camada de config vencedora.
pub fn camada_vencedora(override_path: Option<PathBuf>) -> SshCliResult<CamadaConfig> {
    if override_path.is_some() {
        return Ok(CamadaConfig {
            name: "--config-dir",
            path: resolve_config_path(override_path)?,
        });
    }
    if std::env::var("SSH_CLI_HOME").is_ok() {
        return Ok(CamadaConfig {
            name: "SSH_CLI_HOME",
            path: default_config_path()?,
        });
    }
    Ok(CamadaConfig {
        name: "XDG ProjectDirs",
        path: default_config_path()?,
    })
}

/// Carrega o arquivo de configuração (retorna vazio se não existir).
pub fn carregar(path: &PathBuf) -> SshCliResult<ConfigFile> {
    if !path.exists() {
        return Ok(ConfigFile {
            schema_version: model::CURRENT_SCHEMA_VERSION,
            hosts: BTreeMap::new(),
        });
    }
    let conteudo = std::fs::read_to_string(path)?;
    let mut arquivo: ConfigFile = toml::from_str(&conteudo)?;
    for reg in arquivo.hosts.values_mut() {
        reg.normalize_schema();
    }
    if arquivo.schema_version < model::CURRENT_SCHEMA_VERSION {
        arquivo.schema_version = model::CURRENT_SCHEMA_VERSION;
    }
    Ok(arquivo)
}

/// Escreve bytes em `path` de forma atômica (tempfile + fsync + rename + 0o600).
///
/// Usado por `salvar` e `export` (GAP-007 residual no export).
pub fn escrever_atomico(path: &Path, bytes: &[u8]) -> SshCliResult<()> {
    if let Some(pai) = path.parent() {
        std::fs::create_dir_all(pai)?;
    }
    let pai = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&pai)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_data()?;
    tmp.persist(path).map_err(|e| SshCliError::Io(e.error))?;
    apply_permissions_600(path)?;
    #[cfg(unix)]
    {
        if let Ok(dir) = std::fs::File::open(&pai) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

/// Salva o arquivo de configuração de forma atômica com flock e 0o600.
pub fn salvar(path: &Path, arquivo: &ConfigFile) -> SshCliResult<()> {
    if let Some(pai) = path.parent() {
        std::fs::create_dir_all(pai)?;
    }
    let texto = toml::to_string_pretty(arquivo)
        .map_err(|e| SshCliError::Generic(format!("falha serializando TOML: {e}")))?;

    // Lock em arquivo irmão para serializar mutações concorrentes (N one-shots).
    let lock_path = path.with_extension("toml.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)?;
    // GAP-SSH-PERM-001: lock com 0o600 (não 0644 do umask).
    apply_permissions_600(&lock_path)?;
    fs2::FileExt::lock_exclusive(&lock_file)?;

    escrever_atomico(path, texto.as_bytes())?;

    let _ = fs2::FileExt::unlock(&lock_file);
    Ok(())
}

/// Expande `~` no início do path (home do usuário).
fn expandir_tilde(path: &str) -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = home {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Valida que `key_path` aponta para um arquivo local existente (VAL-003)
/// e, com `ssh-real`, que o conteúdo é uma chave OpenSSH parseável (VAL-004).
///
/// Chaves cifradas sem passphrase no cadastro: se o parse indicar necessidade
/// de password, o arquivo é aceito (formato válido). Lixo de formato → 64.
fn validar_key_path_existe(key_path: &str) -> Result<(), SshCliError> {
    validar_key_path_existe_com_passphrase(key_path, None)
}

/// Como [`validar_key_path_existe`], com passphrase opcional do add/edit.
fn validar_key_path_existe_com_passphrase(
    key_path: &str,
    passphrase: Option<&str>,
) -> Result<(), SshCliError> {
    let p = expandir_tilde(key_path);
    if !p.is_file() {
        return Err(SshCliError::FileNotFound(format!(
            "chave privada não encontrada: {}",
            p.display()
        )));
    }
    #[cfg(feature = "ssh-real")]
    {
        match russh::keys::load_secret_key(&p, passphrase) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                // Key cifrada válida sem passphrase no write-path.
                if msg.contains("password")
                    || msg.contains("passphrase")
                    || msg.contains("encrypted")
                    || msg.contains("decrypt")
                {
                    return Ok(());
                }
                Err(SshCliError::InvalidArgument(format!(
                    "chave privada OpenSSH inválida em {}: {e}",
                    p.display()
                )))
            }
        }
    }
    #[cfg(not(feature = "ssh-real"))]
    {
        let _ = passphrase;
        Ok(())
    }
}

/// JSON efetivo a partir de flag local e formato global (IO-001/002).
#[must_use]
pub fn usar_json(json_local: bool, formato: OutputFormat) -> bool {
    json_local || formato == OutputFormat::Json
}

#[cfg(unix)]
fn apply_permissions_600(path: &Path) -> SshCliResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissoes = std::fs::metadata(path)?.permissions();
    permissoes.set_mode(0o600);
    std::fs::set_permissions(path, permissoes)?;
    Ok(())
}

#[cfg(not(unix))]
fn apply_permissions_600(_caminho: &Path) -> SshCliResult<()> {
    Ok(())
}

/// Lê uma linha de password de stdin (sem eco extra).
pub fn read_secret_stdin() -> SshCliResult<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf.trim_end_matches(['\r', '\n']).to_string())
}

/// Aplica overrides de runtime sobre um VpsRecord clonado.
///
/// Ordem dos parâmetros: password, sudo, su, timeout, key_path, key_passphrase.
pub(crate) fn aplicar_overrides(
    vps: &mut VpsRecord,
    password_override: Option<String>,
    sudo_password_override: Option<String>,
    su_password_override: Option<String>,
    timeout_override: Option<u64>,
    key_path_override: Option<String>,
    key_passphrase_override: Option<String>,
) {
    if let Some(pwd) = password_override {
        vps.password = SecretString::from(pwd);
    }
    if let Some(spwd) = sudo_password_override {
        vps.sudo_password = Some(SecretString::from(spwd));
    }
    if let Some(sp) = su_password_override {
        vps.su_password = Some(SecretString::from(sp));
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

fn validate_command_length(command: &str, max_command_chars: usize) -> SshCliResult<()> {
    let lim = effective_limit(max_command_chars);
    let len = command.chars().count();
    if len > lim {
        return Err(SshCliError::CommandTooLong {
            max: max_command_chars,
            len,
        });
    }
    if command.trim().is_empty() {
        return Err(SshCliError::InvalidArgument("comando vazio".to_string()));
    }
    Ok(())
}

/// Dispatcher dos subcomandos `vps`.
pub async fn run_vps_command(
    action: VpsAction,
    config_override: Option<PathBuf>,
    formato: OutputFormat,
) -> Result<()> {
    let path = resolve_config_path(config_override.clone())?;

    match action {
        VpsAction::Add {
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
            // GAP-SSH-VAL-001: validate na fronteira de escrita.
            let name = crate::paths::validate_and_normalize(&name)
                .map_err(|e| SshCliError::InvalidArgument(format!("nome de VPS inválido: {e}")))?;
            let mut arquivo = carregar(&path)?;
            if arquivo.hosts.contains_key(&name) {
                return Err(SshCliError::VpsDuplicate(name).into());
            }
            if password_stdin && (sudo_password_stdin || su_password_stdin) {
                return Err(SshCliError::InvalidArgument(
                    "apenas um --*-stdin por invocação one-shot; rode vps edit para sudo/su".into(),
                )
                .into());
            }
            let password = if password_stdin {
                SecretString::from(read_secret_stdin()?)
            } else {
                SecretString::from(password.unwrap_or_default())
            };
            let sudo_s = if sudo_password_stdin {
                Some(SecretString::from(read_secret_stdin()?))
            } else {
                sudo_password.map(SecretString::from)
            };
            let su_s = if su_password_stdin {
                Some(SecretString::from(read_secret_stdin()?))
            } else {
                su_password.map(SecretString::from)
            };
            if let Some(ref k) = key {
                validar_key_path_existe(k)?;
            }
            // max_chars legado → command se max_command não veio explicitamente
            let max_cmd = max_command_chars
                .as_deref()
                .or(max_chars.as_deref())
                .map(parse_char_limit)
                .unwrap_or(model::DEFAULT_MAX_COMMAND_CHARS);
            let max_out = max_output_chars
                .as_deref()
                .map(parse_char_limit)
                .unwrap_or(model::DEFAULT_MAX_OUTPUT_CHARS);
            let registro = VpsRecord::new(
                name.clone(),
                host,
                port,
                user,
                password,
                key,
                key_passphrase.map(SecretString::from),
                Some(timeout),
                Some(max_cmd),
                Some(max_out),
                sudo_s,
                su_s,
                disable_sudo,
            );
            // GAP-SSH-VAL-002 / VAL-003: domínio completo no write-path.
            registro.validate().map_err(SshCliError::InvalidArgument)?;
            arquivo.hosts.insert(name.clone(), registro);
            arquivo.schema_version = model::CURRENT_SCHEMA_VERSION;
            salvar(&path, &arquivo)?;
            crate::output::print_success(&crate::i18n::t(crate::i18n::Message::VpsAdded { name: name.clone() }));
            if check {
                run_health_check(
                    Some(&name),
                    config_override,
                    formato,
                    false,
                    None,
                    None,
                    None,
                    None,
                    false,
                )
                .await?;
            }
        }
        VpsAction::List { json } => {
            let arquivo = carregar(&path)?;
            let registros: Vec<_> = arquivo.hosts.values().cloned().collect();
            // GAP-SSH-IO-001: respeitar formato global.
            if usar_json(json, formato) {
                crate::output::print_list_json(&registros);
            } else {
                crate::output::print_list_text(&registros);
            }
        }
        VpsAction::Remove { name } => {
            let mut arquivo = carregar(&path)?;
            if arquivo.hosts.remove(&name).is_none() {
                return Err(SshCliError::VpsNotFound(name).into());
            }
            salvar(&path, &arquivo)?;
            // GAP-SSH-STATE-001: limpar active órfão.
            clear_active_if_name(&path, &name)?;
            crate::output::print_success(&crate::i18n::t(crate::i18n::Message::VpsRemoved { name: name.clone() }));
        }
        VpsAction::Edit {
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
        } => {
            let mut arquivo = carregar(&path)?;
            let registro = arquivo
                .hosts
                .get_mut(&name)
                .ok_or_else(|| SshCliError::VpsNotFound(name.clone()))?;
            if let Some(h) = host {
                registro.host = h;
            }
            if let Some(p) = port {
                registro.port = p;
            }
            if let Some(u) = user {
                registro.username = u;
            }
            if password_stdin {
                registro.password = SecretString::from(read_secret_stdin()?);
            } else if let Some(pw) = password {
                registro.password = SecretString::from(pw);
            }
            if let Some(k) = key {
                validar_key_path_existe(&k)?;
                registro.key_path = Some(k);
            }
            if let Some(kp) = key_passphrase {
                registro.key_passphrase = Some(SecretString::from(kp));
            }
            if let Some(t) = timeout {
                registro.timeout_ms = t;
            }
            if let Some(m) = max_command_chars.or(max_chars) {
                registro.max_command_chars = parse_char_limit(&m);
            }
            if let Some(m) = max_output_chars {
                registro.max_output_chars = parse_char_limit(&m);
            }
            if sudo_password_stdin {
                registro.sudo_password = Some(SecretString::from(read_secret_stdin()?));
            } else if let Some(sp) = sudo_password {
                registro.sudo_password = Some(SecretString::from(sp));
            }
            if su_password_stdin {
                registro.su_password = Some(SecretString::from(read_secret_stdin()?));
            } else if let Some(sp) = su_password {
                registro.su_password = Some(SecretString::from(sp));
            }
            if let Some(d) = disable_sudo {
                registro.disable_sudo = d;
            }
            registro.validate().map_err(SshCliError::InvalidArgument)?;
            salvar(&path, &arquivo)?;
            crate::output::print_success(&crate::i18n::t(crate::i18n::Message::VpsEdited { name: name.clone() }));
        }
        VpsAction::Show { name, json } => {
            let arquivo = carregar(&path)?;
            let registro = arquivo
                .hosts
                .get(&name)
                .ok_or_else(|| SshCliError::VpsNotFound(name.clone()))?;
            if usar_json(json, formato) {
                crate::output::print_details_json(registro);
            } else {
                crate::output::print_details_text(registro);
            }
        }
        VpsAction::Path => {
            crate::output::write_line(&path.display().to_string())?;
        }
        VpsAction::Doctor { json } => {
            run_doctor(config_override, usar_json(json, formato))?;
        }
        VpsAction::Export {
            include_secrets,
            output,
            json,
        } => {
            // GAP-SSH-UX-001: flag local --json ou --output-format json global.
            run_export(
                &path,
                include_secrets,
                output.as_deref(),
                usar_json(json, formato),
            )?;
        }
        VpsAction::Import {
            file,
            allow_incomplete,
        } => {
            run_import(&path, &file, allow_incomplete)?;
        }
    }
    Ok(())
}

/// Remove o arquivo `active` se o conteúdo for o name removido (STATE-001).
fn clear_active_if_name(caminho_config: &Path, name: &str) -> Result<()> {
    let active = caminho_config
        .parent()
        .map(|p| p.join("active"))
        .unwrap_or_else(|| PathBuf::from("active"));
    if !active.exists() {
        return Ok(());
    }
    let conteudo = std::fs::read_to_string(&active).unwrap_or_default();
    if conteudo.trim() == name {
        let _ = std::fs::remove_file(&active);
    }
    Ok(())
}

fn run_doctor(config_override: Option<PathBuf>, json: bool) -> Result<()> {
    let camada = camada_vencedora(config_override.clone())?;
    let path = camada.path.clone();
    let existe = path.exists();
    let arquivo = carregar(&path)?;
    let kh = KnownHosts::path_beside_config(&path);
    let active = path
        .parent()
        .map(|p| p.join("active"))
        .unwrap_or_else(|| PathBuf::from("active"));
    let perms = if existe {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            format!(
                "{:o}",
                std::fs::metadata(&path)?.permissions().mode() & 0o777
            )
        }
        #[cfg(not(unix))]
        {
            "n/a".to_string()
        }
    } else {
        "ausente".to_string()
    };

    let seg = crate::secrets::secrets_status()?;
    if json {
        let v = serde_json::json!({
            "layer": camada.name,
            "config_path": path.display().to_string(),
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
        // GAP-SSH-IO-005: println só em output.
        crate::output::print_json_value(&v)?;
    } else {
        let config_path_s = path.display().to_string();
        let kh_s = kh.display().to_string();
        let active_s = active.display().to_string();
        let key_file_s = seg.key_file_path.display().to_string();
        crate::output::print_doctor_text(
            camada.name,
            &config_path_s,
            existe,
            &perms,
            arquivo.schema_version,
            arquivo.hosts.len(),
            &kh_s,
            &active_s,
            if seg.cifragem_ativa {
                "encrypted"
            } else {
                "plaintext"
            },
            seg.fonte.as_str(),
            &key_file_s,
            seg.plaintext_opt_out,
        );
    }
    Ok(())
}

fn run_export(
    path: &PathBuf,
    include_secrets: bool,
    output: Option<&str>,
    json: bool,
) -> Result<()> {
    let arquivo = carregar(path)?;
    let mut export = ConfigFile {
        schema_version: arquivo.schema_version,
        hosts: BTreeMap::new(),
    };
    for (k, mut v) in arquivo.hosts {
        if !include_secrets {
            // EXP-001 parity: redacted limpa secrets (nunca sshcli-enc de empty).
            v.password = SecretString::from(String::new());
            v.sudo_password = None;
            v.su_password = None;
            v.key_passphrase = None;
        }
        export.hosts.insert(k, v);
    }

    let bytes = if json {
        // GAP-SSH-UX-001 / M-AUD-07: envelope agent-first com discriminador.
        let hosts_json = crate::output::export_hosts_to_json(&export.hosts, include_secrets);
        let v = serde_json::json!({
            "ok": true,
            "event": "vps-export",
            "schema_version": export.schema_version,
            "include_secrets": include_secrets,
            "hosts": hosts_json,
        });
        let texto = serde_json::to_string_pretty(&v)?;
        texto.into_bytes()
    } else {
        let texto = toml::to_string_pretty(&export)?;
        texto.into_bytes()
    };

    if let Some(path) = output {
        escrever_atomico(Path::new(path), &bytes)?;
        crate::output::print_success(&crate::i18n::t(crate::i18n::Message::ExportCompleted { path: path.to_string() }));
    } else {
        // TOML/JSON body to stdout (agent-first: single payload).
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        out.write_all(&bytes)?;
        if !bytes.ends_with(b"\n") {
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn run_import(path: &PathBuf, file: &Path, allow_incomplete: bool) -> Result<()> {
    let texto = std::fs::read_to_string(file)?;
    let importado: ConfigFile = toml::from_str(&texto)?;
    let mut atual = carregar(path)?;
    for (k, mut v) in importado.hosts {
        // VAL-001 no import.
        let name = crate::paths::validate_and_normalize(&k).map_err(|e| {
            SshCliError::InvalidArgument(format!("nome de VPS inválido no import '{k}': {e}"))
        })?;
        v.name = name.clone();
        if let Some(ref key) = v.key_path {
            if !key.trim().is_empty() {
                validar_key_path_existe(key)?;
            }
        }
        match v.validate() {
            Ok(()) => {
                atual.hosts.insert(name, v);
            }
            Err(msg) if allow_incomplete => {
                // GAP-SSH-IMP-001: esqueleto incompleto permitido.
                tracing::warn!(host = %name, %msg, "import incomplete permitido");
                atual.hosts.insert(name, v);
            }
            Err(msg) => {
                // Detectar export redacted.
                let redacted = !v.has_password() && !v.has_key();
                if redacted {
                    return Err(SshCliError::InvalidArgument(format!(
                        "host '{name}' parece export redacted (sem password/key). \
                         Use `vps export --include-secrets`, complete com `vps edit`, \
                         ou `vps import --allow-incomplete`. Detalhe: {msg}"
                    ))
                    .into());
                }
                return Err(SshCliError::InvalidArgument(format!(
                    "host '{name}' inválido no import: {msg}"
                ))
                .into());
            }
        }
    }
    atual.schema_version = model::CURRENT_SCHEMA_VERSION;
    salvar(path, &atual)?;
    crate::output::print_success(&crate::i18n::t(crate::i18n::Message::ImportCompleted));
    Ok(())
}

/// Dispatcher one-shot de `secrets status|init|reencrypt`.
pub async fn run_secrets_command(
    action: SecretsAction,
    config_override: Option<PathBuf>,
    formato: OutputFormat,
) -> Result<()> {
    // Garante alinhamento do secrets.key com --config-dir.
    crate::secrets::set_config_dir(config_override.clone());
    match action {
        SecretsAction::Status { json } => {
            let seg = crate::secrets::secrets_status()?;
            let usar_json = json || formato == OutputFormat::Json;
            if usar_json {
                let v = serde_json::json!({
                    "encryption_active": seg.cifragem_ativa,
                    "key_source": seg.fonte.as_str(),
                    "key_file": seg.key_file_path.display().to_string(),
                    "plaintext_opt_out": seg.plaintext_opt_out,
                    "at_rest": if seg.cifragem_ativa { "encrypted" } else { "plaintext" },
                });
                crate::output::print_json_value(&v)?;
            } else {
                crate::output::print_success(&format!(
                    "at-rest: {} | source: {} | key_file: {} | plaintext_opt_out: {}",
                    if seg.cifragem_ativa {
                        "encrypted"
                    } else {
                        "plaintext"
                    },
                    seg.fonte.as_str(),
                    seg.key_file_path.display(),
                    seg.plaintext_opt_out
                ));
            }
            Ok(())
        }
        SecretsAction::Init { keyring, force } => {
            let seg = crate::secrets::init_primary_key(keyring, force)?;
            crate::output::print_success(&format!(
                "primary-key pronta (source={}; key_file={})",
                seg.fonte.as_str(),
                seg.key_file_path.display()
            ));
            Ok(())
        }
        SecretsAction::Reencrypt => {
            let path = resolve_config_path(config_override)?;
            run_reencrypt(&path)?;
            Ok(())
        }
    }
}

/// Recarrega e regrava o config, re-cifando secrets com a chave atual.
fn run_reencrypt(path: &PathBuf) -> Result<()> {
    let (chave, fonte) = crate::secrets::ensure_key_for_write()?;
    if chave.is_none() {
        return Err(SshCliError::InvalidArgument(
            "sem primary-key; rode `ssh-cli secrets init` ou remova SSH_CLI_ALLOW_PLAINTEXT_SECRETS"
                .to_string(),
        )
        .into());
    }
    if let Some(mut k) = chave {
        use zeroize::Zeroize;
        k.zeroize();
    }
    let arquivo = carregar(path)?;
    salvar(path, &arquivo)?;
    crate::output::print_success(&format!(
        "reencrypt ok (source={}; hosts={})",
        fonte.as_str(),
        arquivo.hosts.len()
    ));
    Ok(())
}

/// Define a VPS ativa gravando seu name em `<config_dir>/active` (arquivo irmão).
pub async fn run_connect(name: &str, config_override: Option<PathBuf>) -> Result<()> {
    let path = resolve_config_path(config_override)?;
    let arquivo = carregar(&path)?;
    if !arquivo.hosts.contains_key(name) {
        return Err(SshCliError::VpsNotFound(name.to_string()).into());
    }

    let arquivo_ativo = path
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
    tmp.write_all(name.as_bytes())?;
    tmp.as_file().sync_data()?;
    tmp.persist(&arquivo_ativo)
        .map_err(|e| SshCliError::Io(e.error))?;
    crate::output::print_success(&crate::i18n::t(crate::i18n::Message::VpsActiveSelected { name: name.to_string() }));
    Ok(())
}

/// Busca um registro de VPS por name.
pub fn find_by_name(
    config_override: Option<PathBuf>,
    name: &str,
) -> SshCliResult<Option<VpsRecord>> {
    let path = resolve_config_path(config_override)?;
    let arquivo = carregar(&path)?;
    Ok(arquivo.hosts.get(name).cloned())
}

/// Lê o name da VPS ativa.
pub fn read_active_vps(config_override: Option<PathBuf>) -> SshCliResult<Option<String>> {
    let path = resolve_config_path(config_override)?;
    let arquivo_ativo = path
        .parent()
        .map(|p| p.join("active"))
        .unwrap_or_else(|| PathBuf::from("active"));
    if !arquivo_ativo.exists() {
        return Ok(None);
    }
    let name = std::fs::read_to_string(&arquivo_ativo)?;
    Ok(Some(name.trim().to_string()))
}

/// Constrói `ConnectionConfig` a partir de um `VpsRecord`.
pub fn build_connection_config(
    vps: &VpsRecord,
    config_toml: Option<&Path>,
    replace_host_key: bool,
) -> ConnectionConfig {
    let known_hosts_path = config_toml.map(KnownHosts::path_beside_config);
    ConnectionConfig {
        host: vps.host.clone(),
        port: vps.port,
        username: vps.username.clone(),
        password: vps.password.clone(),
        key_path: vps.key_path.clone(),
        key_passphrase: vps.key_passphrase.clone(),
        timeout_ms: vps.timeout_ms,
        known_hosts_path,
        replace_host_key,
    }
}

/// Opções comuns de execução remota.
#[derive(Debug, Default, Clone)]
pub struct ExecOptions {
    /// Override password.
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

/// Executa um command em uma VPS via SSH.
pub async fn run_exec(
    vps_name: &str,
    command: &str,
    config_override: Option<PathBuf>,
    formato: OutputFormat,
    json: bool,
    opts: ExecOptions,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    let path = resolve_config_path(config_override)?;
    let arquivo = carregar(&path)?;
    let vps_base = arquivo
        .hosts
        .get(vps_name)
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.to_string()))?;

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
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars)?;
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let cliente: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_exec_with_client(&vps, &cmd, cliente, formato, json).await
}

/// Versão testável de run_exec.
pub async fn run_exec_with_client(
    vps: &VpsRecord,
    command: &str,
    mut cliente: Box<dyn SshClientTrait>,
    formato: OutputFormat,
    json: bool,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    let max_out = effective_limit(vps.max_output_chars);
    let saida = cliente.run_command(command, max_out, None).await?;
    cliente.disconnect().await?;
    if formato == OutputFormat::Json || json {
        output::print_execution_output_json(&saida);
    } else {
        output::print_execution_output(&saida);
    }
    if let Some(code) = saida.exit_code {
        if code != 0 {
            return Err(SshCliError::CommandFailed {
                exit_code: code,
                stderr: saida.stderr.clone(),
            }
            .into());
        }
    }
    Ok(())
}

/// Executa um command com `sudo` (packing `sh -c`).
pub async fn run_sudo_exec(
    vps_name: &str,
    command: &str,
    config_override: Option<PathBuf>,
    formato: OutputFormat,
    json: bool,
    opts: ExecOptions,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    let path = resolve_config_path(config_override)?;
    let arquivo = carregar(&path)?;
    let vps_base = arquivo
        .hosts
        .get(vps_name)
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.to_string()))?;

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
        return Err(SshCliError::SudoDisabled.into());
    }
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars)?;
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let cliente: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    run_sudo_exec_with_client(&vps, &cmd, cliente, formato, json).await
}

/// Versão testável de sudo-exec.
pub async fn run_sudo_exec_with_client(
    vps: &VpsRecord,
    command: &str,
    mut cliente: Box<dyn SshClientTrait>,
    formato: OutputFormat,
    json: bool,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    if vps.disable_sudo {
        return Err(SshCliError::SudoDisabled.into());
    }
    let pack = pack_sudo(command, vps.sudo_password.as_ref());
    let max_out = effective_limit(vps.max_output_chars);
    let saida = cliente
        .run_command(&pack.command, max_out, pack.stdin)
        .await?;
    cliente.disconnect().await?;
    if formato == OutputFormat::Json || json {
        output::print_execution_output_json(&saida);
    } else {
        output::print_execution_output(&saida);
    }
    if let Some(code) = saida.exit_code {
        if code != 0 {
            return Err(SshCliError::CommandFailed {
                exit_code: code,
                stderr: saida.stderr.clone(),
            }
            .into());
        }
    }
    Ok(())
}

/// Executa command via `su -` one-shot (consome `su_password`).
pub async fn run_su_exec(
    vps_name: &str,
    command: &str,
    config_override: Option<PathBuf>,
    formato: OutputFormat,
    json: bool,
    opts: ExecOptions,
) -> Result<()> {
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    let path = resolve_config_path(config_override)?;
    let arquivo = carregar(&path)?;
    let vps_base = arquivo
        .hosts
        .get(vps_name)
        .ok_or_else(|| SshCliError::VpsNotFound(vps_name.to_string()))?;

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
        return Err(SshCliError::SudoDisabled.into());
    }
    let su_password = vps.su_password.clone().ok_or(SshCliError::SuPasswordMissing)?;
    let cmd = append_description(command, opts.description.as_deref());
    validate_command_length(&cmd, vps.max_command_chars)?;
    let pack = pack_su(&cmd, &su_password);
    let cfg = build_connection_config(&vps, Some(&path), opts.replace_host_key);
    let mut cliente: Box<dyn SshClientTrait> =
        <SshClient as SshClientTrait>::connect(cfg).await?;
    let max_out = effective_limit(vps.max_output_chars);
    let saida = cliente
        .run_command(&pack.command, max_out, pack.stdin)
        .await?;
    cliente.disconnect().await?;
    if formato == OutputFormat::Json || json {
        output::print_execution_output_json(&saida);
    } else {
        output::print_execution_output(&saida);
    }
    if let Some(code) = saida.exit_code {
        if code != 0 {
            return Err(SshCliError::CommandFailed {
                exit_code: code,
                stderr: saida.stderr.clone(),
            }
            .into());
        }
    }
    Ok(())
}

/// Health-check SSH.
/// Health-check one-shot com paridade de auth (GAP-SSH-CLI-006) e TOFU (M1).
#[allow(clippy::too_many_arguments)]
pub async fn run_health_check(
    vps_name: Option<&str>,
    config_override: Option<PathBuf>,
    formato: OutputFormat,
    json_local: bool,
    password_override: Option<String>,
    timeout_override: Option<u64>,
    key_override: Option<String>,
    key_passphrase_override: Option<String>,
    replace_host_key: bool,
) -> Result<()> {
    // M2: --json local ou formato global → envelope de erro JSON em falha.
    if json_local || formato == OutputFormat::Json {
        crate::output::set_json_errors(true);
    }
    if crate::signals::cancelado() || crate::signals::terminado() {
        return Err(anyhow::anyhow!(crate::i18n::t(
            crate::i18n::Message::OperationCancelled
        )));
    }
    let nome_resolvido: String = match vps_name {
        Some(n) => n.to_string(),
        None => {
            // GAP-SSH-EXIT-002: tipado → exit 66 (não anyhow string → exit 1).
            let ativa = read_active_vps(config_override.clone())?;
            ativa.ok_or(SshCliError::NoActiveVps)?
        }
    };
    let path = resolve_config_path(config_override)?;
    let arquivo = carregar(&path)?;
    let vps_base = arquivo
        .hosts
        .get(&nome_resolvido)
        .ok_or_else(|| SshCliError::VpsNotFound(nome_resolvido.clone()))?;

    let mut vps = vps_base.clone();
    // GAP-SSH-CLI-004: --timeout; GAP-SSH-CLI-006: key + passphrase.
    // Ordem: password, sudo, su, timeout, key_path, key_passphrase.
    aplicar_overrides(
        &mut vps,
        password_override,
        None,
        None,
        timeout_override,
        key_override,
        key_passphrase_override,
    );
    // M1: honra --replace-host-key global (paridade exec/scp/tunnel).
    let cfg = build_connection_config(&vps, Some(&path), replace_host_key);
    let inicio = std::time::Instant::now();
    let cliente: Box<dyn SshClientTrait> = <SshClient as SshClientTrait>::connect(cfg).await?;
    let latency_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    cliente.disconnect().await?;

    if usar_json(json_local, formato) {
        output::print_health_check_json(&nome_resolvido, latency_ms);
    } else {
        output::print_health_check(&nome_resolvido, latency_ms);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use tempfile::TempDir;

    fn reg_min() -> VpsRecord {
        VpsRecord::new(
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
        let arq = ConfigFile {
            schema_version: model::CURRENT_SCHEMA_VERSION,
            hosts: BTreeMap::new(),
        };
        let texto = toml::to_string(&arq).unwrap();
        assert!(texto.contains("schema_version = 2"));
    }

    #[test]
    fn parse_limite_none() {
        assert_eq!(parse_char_limit("none"), 0);
        assert_eq!(parse_char_limit("0"), 0);
        assert_eq!(parse_char_limit("1000"), 1000);
    }

    #[test]
    fn construir_configuracao_copia_campos() {
        let registro = reg_min();
        let cfg = build_connection_config(&registro, None, false);
        assert_eq!(cfg.host, "host.example.com");
        assert_eq!(cfg.port, 2222);
        assert_eq!(cfg.username, "admin");
        assert_eq!(cfg.timeout_ms, 60_000);
        assert!(cfg.known_hosts_path.is_none());
    }

    #[test]
    #[serial_test::serial]
    fn salvar_atomico_roundtrip() {
        let tmp = TempDir::new().unwrap();
        crate::secrets::set_config_dir(Some(tmp.path().to_path_buf()));
        // SAFETY:

        // 1. Contract: temporary mutation of process environment for a serial test/setup path.

        // 2. Invariant: no concurrent threads in this process mutate the same env keys.

        // 3. Caller guarantees serial_test::serial (or single-threaded test) around this block.

        // 4. See std::env::set_var / remove_var safety notes for multi-threaded processes.

        unsafe {
            std::env::set_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS", "1");
        }
        let path = tmp.path().join("config.toml");
        let mut arq = ConfigFile {
            schema_version: 2,
            hosts: BTreeMap::new(),
        };
        arq.hosts.insert("a".into(), reg_min());
        salvar(&path, &arq).unwrap();
        let lido = carregar(&path).unwrap();
        assert_eq!(lido.hosts.len(), 1);
        assert_eq!(lido.hosts["a"].password.expose_secret(), "pass");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
            let lock = path.with_extension("toml.lock");
            if lock.exists() {
                let lm = std::fs::metadata(&lock).unwrap().permissions().mode() & 0o777;
                assert_eq!(lm, 0o600);
            }
        }
        // SAFETY:

        // 1. Contract: temporary mutation of process environment for a serial test/setup path.

        // 2. Invariant: no concurrent threads in this process mutate the same env keys.

        // 3. Caller guarantees serial_test::serial (or single-threaded test) around this block.

        // 4. See std::env::set_var / remove_var safety notes for multi-threaded processes.

        unsafe {
            std::env::remove_var("SSH_CLI_ALLOW_PLAINTEXT_SECRETS");
        }
        crate::secrets::set_config_dir(None);
    }

    #[test]
    fn resolver_caminho_config_com_override_diretorio() {
        let resultado = resolve_config_path(Some(PathBuf::from("/tmp/test-dir")));
        assert_eq!(
            resultado.unwrap(),
            PathBuf::from("/tmp/test-dir/config.toml")
        );
    }

    #[test]
    fn validar_comando_longo() {
        let err = validate_command_length(&"x".repeat(20), 10).unwrap_err();
        assert!(matches!(err, SshCliError::CommandTooLong { .. }));
    }

    #[test]
    fn empacotar_sudo_integrado() {
        let pack = pack_sudo("ls -la", None);
        assert_eq!(pack.command, "sudo -n sh -c 'ls -la'");
        assert!(pack.stdin.is_none());
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
        assert_eq!(v.password.expose_secret(), "nova");
        assert_eq!(v.timeout_ms, 1000);
        assert_eq!(v.key_path.as_deref(), Some("/k"));
    }

    #[tokio::test]
    async fn executar_sudo_exec_with_client_ok() {
        use crate::ssh::client::mocks::MockSshClient;
        use crate::ssh::client::ExecutionOutput;
        let mut mock = MockSshClient::new();
        mock.expect_run_command().returning(|c, _, stdin| {
            assert!(c.contains("sudo -n sh -c"));
            assert!(stdin.is_none());
            Ok(ExecutionOutput {
                stdout: "ok".into(),
                stderr: String::new(),
                exit_code: Some(0),
                truncated_stdout: false,
                truncated_stderr: false,
                duration_ms: 1,
            })
        });
        mock.expect_disconnect().returning(|| Ok(()));

        let vps = reg_min();
        run_sudo_exec_with_client(&vps, "id", Box::new(mock), OutputFormat::Text, false)
            .await
            .unwrap();
    }
}
