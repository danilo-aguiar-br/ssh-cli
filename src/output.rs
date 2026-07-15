//! Único módulo autorizado a emitir output em stdout para CRUD de VPS.
//!
//! Este módulo centraliza TODA formatação de CRUD: texto e JSON.
//!
//! Logs (tracing) vão para stderr, gerenciados por `tracing-subscriber`.

use crate::mascaramento::mascarar;
use crate::ssh::SaidaExecucao;
use crate::vps::modelo::VpsRegistro;
use secrecy::ExposeSecret;
use serde_json::json;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Flag global de `--quiet` (suprime mensagens humanas em stdout).
static QUIET: AtomicBool = AtomicBool::new(false);

/// Quando true, erros em `main` usam envelope JSON em stderr (IO-003).
static JSON_ERROS: AtomicBool = AtomicBool::new(false);

/// Define se a CLI está em modo quiet (GAP-SSH-IO-004).
pub fn definir_quiet(quiet: bool) {
    QUIET.store(quiet, Ordering::SeqCst);
}

/// Define se erros devem sair como envelope JSON em stderr.
pub fn definir_json_erros(json: bool) {
    JSON_ERROS.store(json, Ordering::SeqCst);
}

/// Retorna se quiet está ativo.
#[must_use]
pub fn esta_quiet() -> bool {
    QUIET.load(Ordering::SeqCst)
}

/// Retorna se erros devem ser envelope JSON.
#[must_use]
pub fn quer_json_erros() -> bool {
    JSON_ERROS.load(Ordering::SeqCst)
}

/// Escreve uma linha em stdout garantindo LF puro (nunca CRLF).
///
/// Em quiet, ainda escreve (dados/paths são API). Preferir `imprimir_sucesso`
/// para mensagens humanas.
///
/// # Erros
/// Retorna erro se o I/O em stdout falhar.
pub fn escrever_linha(conteudo: &str) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(conteudo.as_bytes())?;
    handle.write_all(b"\n")?;
    handle.flush()?;
    Ok(())
}

/// Imprime mensagem de sucesso em texto para humanos (silenciada com `--quiet`).
pub fn imprimir_sucesso(mensagem: &str) {
    if esta_quiet() {
        return;
    }
    println!("{mensagem}");
}

/// Banner humano (tunnel etc.): só Text+TTY+!quiet+!JSON erros (GAP-SSH-IO-006).
///
/// Em pipes/agentes, progresso vai para `tracing` (stderr), nunca para stdout.
pub fn imprimir_banner_humano(mensagem: &str) {
    if esta_quiet() || quer_json_erros() {
        return;
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        return;
    }
    if std::env::var_os("SSH_CLI_FORCE_TEXT").is_none() {
        // Sem FORCE_TEXT, non-TTY já retornou; TTY humano ok.
    }
    println!("{mensagem}");
}

/// Imprime mensagem de erro em stderr (para humanos).
pub fn imprimir_erro(mensagem: &str) {
    eprintln!("{mensagem}");
}

/// Emite envelope JSON de erro em stderr (GAP-SSH-IO-003).
pub fn imprimir_erro_envelope(
    exit_code: i32,
    message: &str,
    remote_exit_code: Option<i32>,
) -> io::Result<()> {
    let mut v = json!({
        "exit_code": exit_code,
        "message": message,
    });
    if let Some(r) = remote_exit_code {
        v["remote_exit_code"] = json!(r);
    }
    let s = serde_json::to_string(&v).unwrap_or_else(|_| {
        format!(r#"{{"exit_code":{exit_code},"message":"erro de serialização"}}"#)
    });
    let mut err = io::stderr().lock();
    err.write_all(s.as_bytes())?;
    err.write_all(b"\n")?;
    err.flush()?;
    Ok(())
}

/// Imprime valor JSON pretty em stdout (dados de API; respeita quiet=false sempre).
pub fn imprimir_json_value(v: &serde_json::Value) -> io::Result<()> {
    let s = serde_json::to_string_pretty(v).map_err(io::Error::other)?;
    escrever_linha(&s)
}

/// Imprime relatório doctor em texto (GAP-SSH-IO-005).
#[allow(clippy::too_many_arguments)]
pub fn imprimir_doctor_texto(
    camada: &str,
    config_path: &str,
    existe: bool,
    perms: &str,
    schema_version: u32,
    hosts: usize,
    known_hosts: &str,
    active_file: &str,
    secrets_at_rest: &str,
    secrets_key_source: &str,
    secrets_key_file: &str,
    plaintext_opt_out: bool,
) {
    if esta_quiet() {
        return;
    }
    println!("Camada vencedora: {camada}");
    println!("Config path:      {config_path}");
    println!("Existe:           {existe}");
    println!("Permissões:       {perms}");
    println!("Schema:           {schema_version}");
    println!("Hosts:            {hosts}");
    println!("known_hosts:      {known_hosts}");
    println!("active file:      {active_file}");
    println!("Secrets at-rest:  {secrets_at_rest} (key source: {secrets_key_source})");
    println!("Secrets key file: {secrets_key_file}");
    println!(
        "Plaintext opt-out: {}",
        if plaintext_opt_out { "yes" } else { "no" }
    );
    println!("Telemetria:       desabilitada");
}

/// Imprime lista de VPS em formato texto (mascarado).
pub fn imprimir_lista_texto(registros: &[VpsRegistro]) {
    if esta_quiet() {
        return;
    }
    if registros.is_empty() {
        println!(
            "{}",
            crate::i18n::t(crate::i18n::Mensagem::VpsRegistroVazio)
        );
        return;
    }

    println!(
        "{:<20} {:<30} {:<6} {:<15} {:<20}",
        "NOME", "HOST", "PORTA", "USUÁRIO", "SENHA"
    );
    for r in registros {
        println!(
            "{:<20} {:<30} {:<6} {:<15} {:<20}",
            r.nome,
            r.host,
            r.porta,
            r.usuario,
            mascarar(r.senha.expose_secret())
        );
    }
}

/// Imprime lista de VPS em formato JSON (mascarado).
pub fn imprimir_lista_json(registros: &[VpsRegistro]) {
    let lista: Vec<_> = registros.iter().map(registro_para_json_mascarado).collect();
    match serde_json::to_string_pretty(&lista) {
        Ok(s) => println!("{s}"),
        Err(erro) => eprintln!("erro ao serializar JSON: {erro}"),
    }
}

/// Imprime detalhes de UMA VPS em texto (mascarado).
pub fn imprimir_detalhes_texto(r: &VpsRegistro) {
    if esta_quiet() {
        return;
    }
    println!("Nome:           {}", r.nome);
    println!("Host:           {}", r.host);
    println!("Porta:          {}", r.porta);
    println!("Usuário:        {}", r.usuario);
    // GAP-SSH-JSON-001: senha vazia (key-only) não finge valor mascarado.
    println!(
        "Senha:          {}",
        if r.senha.expose_secret().is_empty() {
            "(não definida)".to_string()
        } else {
            mascarar(r.senha.expose_secret())
        }
    );
    println!(
        "Key path:       {}",
        r.key_path.as_deref().unwrap_or("(não definida)")
    );
    println!(
        "Senha sudo:     {}",
        r.senha_sudo
            .as_ref()
            .map_or_else(|| "(não definida)".into(), |s| mascarar(s.expose_secret()))
    );
    println!(
        "Senha su:       {}",
        r.senha_su
            .as_ref()
            .map_or_else(|| "(não definida)".into(), |s| mascarar(s.expose_secret()))
    );
    println!("Timeout (ms):   {}", r.timeout_ms);
    println!("Max cmd chars:  {}", r.max_command_chars);
    println!("Max out chars:  {}", r.max_output_chars);
    println!("Disable sudo:   {}", r.disable_sudo);
    println!("Schema version: {}", r.schema_version);
    println!("Adicionado em:  {}", r.adicionado_em);
}

/// Imprime detalhes de UMA VPS em JSON (mascarado).
pub fn imprimir_detalhes_json(r: &VpsRegistro) {
    let v = registro_para_json_mascarado(r);
    match serde_json::to_string_pretty(&v) {
        Ok(s) => println!("{s}"),
        Err(erro) => eprintln!("erro ao serializar JSON: {erro}"),
    }
}

fn registro_para_json_mascarado(r: &VpsRegistro) -> serde_json::Value {
    // GAP-SSH-JSON-001: password ausente/vazio → null (como sudo/su); presente → "***".
    let password = if r.senha.expose_secret().is_empty() {
        json!(null)
    } else {
        json!(mascarar(r.senha.expose_secret()))
    };
    json!({
        "name": r.nome,
        "host": r.host,
        "port": r.porta,
        "user": r.usuario,
        "password": password,
        "key_path": r.key_path,
        "key_passphrase": r.key_passphrase.as_ref().map(|s| mascarar(s.expose_secret())),
        "sudo_password": r.senha_sudo.as_ref().map(|s| mascarar(s.expose_secret())),
        "su_password": r.senha_su.as_ref().map(|s| mascarar(s.expose_secret())),
        "timeout_ms": r.timeout_ms,
        "max_command_chars": r.max_command_chars,
        "max_output_chars": r.max_output_chars,
        "disable_sudo": r.disable_sudo,
        "schema_version": r.schema_version,
        "added_at": r.adicionado_em,
    })
}

/// Imprime stdout/stderr de execução de comando SSH.
///
/// Formato:
/// ```text
/// --- stdout ---
/// <stdout>
/// --- stderr ---
/// <stderr>
/// --- exit code: <code> (<duracao_ms>ms) ---
/// ```
pub fn imprimir_saida_execucao(saida: &SaidaExecucao) {
    println!("--- stdout ---");
    if saida.stdout.is_empty() {
        println!("(vazio)");
    } else {
        println!("{}", saida.stdout);
    }
    println!("--- stderr ---");
    if saida.stderr.is_empty() {
        println!("(vazio)");
    } else {
        println!("{}", saida.stderr);
    }
    let code_str = saida
        .exit_code
        .map(|c| c.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    println!("--- exit code: {} ({}ms) ---", code_str, saida.duracao_ms);
    if saida.truncado_stdout {
        println!("(stdout foi truncado)");
    }
    if saida.truncado_stderr {
        println!("(stderr foi truncado)");
    }
}

/// Imprime stdout/stderr de execução de comando SSH em formato JSON.
pub fn imprimir_saida_execucao_json(saida: &SaidaExecucao) {
    let v = json!({
        "stdout": saida.stdout,
        "stderr": saida.stderr,
        "exit_code": saida.exit_code,
        "truncated_stdout": saida.truncado_stdout,
        "truncated_stderr": saida.truncado_stderr,
        "duration_ms": saida.duracao_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("erro ao serializar JSON: {e}"),
    }
}

/// Imprime resultado de health-check em formato texto.
pub fn imprimir_health_check(nome: &str, latencia_ms: u64) {
    if esta_quiet() {
        return;
    }
    println!(
        "{}",
        crate::i18n::t(crate::i18n::Mensagem::HealthCheckOk {
            nome: nome.to_string(),
        })
    );
    println!("  latência: {latencia_ms}ms");
}

/// Imprime resultado de health-check em formato JSON.
pub fn imprimir_health_check_json(nome: &str, latencia_ms: u64) {
    let v = json!({
        "name": nome,
        "status": "ok",
        "latency_ms": latencia_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("erro ao serializar JSON: {e}"),
    }
}

/// Imprime resultado de transferência SCP em JSON (GAP-SSH-IO-007 / SCP-021).
pub fn imprimir_transferencia_json(
    direction: &str,
    vps: &str,
    local: &str,
    remote: &str,
    bytes: u64,
    duration_ms: u64,
) {
    let v = json!({
        "ok": true,
        "direction": direction,
        "vps": vps,
        "local": local,
        "remote": remote,
        "bytes": bytes,
        "duration_ms": duration_ms,
    });
    match serde_json::to_string_pretty(&v) {
        Ok(s) => {
            let _ = escrever_linha(&s);
        }
        Err(e) => eprintln!("erro ao serializar JSON: {e}"),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::ssh::SaidaExecucao;
    use crate::vps::modelo::VpsRegistro;
    use secrecy::SecretString;

    fn registro_teste() -> VpsRegistro {
        VpsRegistro::novo(
            "vps-teste".into(),
            "1.2.3.4".into(),
            22,
            "root".into(),
            SecretString::from("senha-super-secreta".to_string()),
            None,
            None,
            Some(5000),
            Some(1000),
            Some(1000),
            Some(SecretString::from("sudo-password-longa-aqui".to_string())),
            None,
            false,
        )
    }

    #[test]
    fn registro_para_json_mascarado_contem_campos_obrigatorios() {
        let r = registro_teste();
        let json = registro_para_json_mascarado(&r);
        assert_eq!(json["name"], "vps-teste");
        assert_eq!(json["host"], "1.2.3.4");
        assert_eq!(json["port"], 22);
        assert_eq!(json["user"], "root");
        assert_eq!(json["password"].as_str().unwrap(), "***");
        assert_eq!(json["sudo_password"].as_str().unwrap(), "***");
        assert!(json["su_password"].is_null());
        assert_eq!(json["timeout_ms"], 5000);
        assert_eq!(json["max_command_chars"], 1000);
        assert_eq!(json["max_output_chars"], 1000);
        assert_eq!(json["schema_version"], 2);
    }

    #[test]
    fn registro_para_json_mascarado_senha_sudo_nula_quando_nao_definida() {
        let mut r = registro_teste();
        r.senha_sudo = None;
        let json = registro_para_json_mascarado(&r);
        assert!(json["sudo_password"].is_null());
    }

    #[test]
    fn registro_para_json_mascarado_su_password_presente() {
        let mut r = registro_teste();
        r.senha_su = Some(SecretString::from("senha-su-muito-longa-aqui".to_string()));
        let json = registro_para_json_mascarado(&r);
        assert_eq!(json["su_password"].as_str().unwrap(), "***");
    }

    #[test]
    fn registro_para_json_mascarado_password_null_quando_vazio() {
        let mut r = registro_teste();
        r.senha = SecretString::from(String::new());
        let json = registro_para_json_mascarado(&r);
        assert!(json["password"].is_null());
    }

    #[test]
    fn escribir_linha_ok() {
        let resultado = escrever_linha("teste de escrita");
        assert!(resultado.is_ok());
    }

    #[test]
    fn escribir_linha_com_caracteres_especiais() {
        let resultado = escrever_linha("linha com \t tab e \"aspas\"");
        assert!(resultado.is_ok());
    }

    #[test]
    fn salida_execucao_completa_formatada() {
        let saida = SaidaExecucao {
            stdout: "output do comando".to_string(),
            stderr: "erro do comando".to_string(),
            exit_code: Some(0),
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 150,
        };
        let resultado = escrever_linha(&format!(
            "stdout: {}, stderr: {}, exit: {:?}",
            saida.stdout, saida.stderr, saida.exit_code
        ));
        assert!(resultado.is_ok());
    }

    #[test]
    fn salida_execucao_sem_exit_code() {
        let saida = SaidaExecucao {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: None,
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 0,
        };
        let code_str = saida
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        assert_eq!(code_str, "N/A");
    }

    #[test]
    fn vps_registro_debug_nao_expoe_senha() {
        let r = registro_teste();
        let json = registro_para_json_mascarado(&r);
        let json_str = serde_json::to_string(&json).unwrap();
        assert!(!json_str.contains("senha-super-secreta"));
        assert!(!json_str.contains("sudo-password-longa-aqui"));
    }

    #[test]
    fn salida_execucao_truncada_mostra_aviso() {
        let saida = SaidaExecucao {
            stdout: "output".to_string(),
            stderr: "erro".to_string(),
            exit_code: Some(1),
            truncado_stdout: true,
            truncado_stderr: true,
            duracao_ms: 100,
        };
        assert!(saida.truncado_stdout);
        assert!(saida.truncado_stderr);
    }

    #[test]
    fn salida_execucao_com_exit_code_numerico() {
        let saida = SaidaExecucao {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: Some(127),
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 0,
        };
        let code_str = saida
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        assert_eq!(code_str, "127");
    }

    #[test]
    fn escribir_linha_string_vazia() {
        let resultado = escrever_linha("");
        assert!(resultado.is_ok());
    }

    #[test]
    fn escribir_linha_com_unicode_brasileiro() {
        let resultado = escrever_linha("ação você está Itaú");
        assert!(resultado.is_ok());
    }

    #[test]
    fn escribir_linha_com_emojis() {
        let resultado = escrever_linha("texto com 🚀 e 🔐");
        assert!(resultado.is_ok());
    }

    #[test]
    fn escribir_linha_com_newlines() {
        let resultado = escrever_linha("linha1\nlinha2\nlinha3");
        assert!(resultado.is_ok());
    }

    #[test]
    fn escribir_linha_longo_texto() {
        let texto_longo = "a".repeat(10000);
        let resultado = escrever_linha(&texto_longo);
        assert!(resultado.is_ok());
    }

    #[test]
    fn registro_para_json_mascarado_com_senha_curta_mascara_com_asteriscos() {
        let mut r = registro_teste();
        r.senha = SecretString::from("curta".to_string());
        let json = registro_para_json_mascarado(&r);
        let senha_str = json["password"].as_str().unwrap();
        assert_eq!(senha_str, "***");
    }

    #[test]
    fn registro_para_json_mascarado_com_sudo_e_su_definidos() {
        let mut r = registro_teste();
        r.senha_sudo = Some(SecretString::from("sudo-pass-longa-aqui".to_string()));
        r.senha_su = Some(SecretString::from("su-pass-longa-aqui".to_string()));
        let json = registro_para_json_mascarado(&r);
        assert!(!json["sudo_password"].is_null());
        assert!(!json["su_password"].is_null());
        assert_eq!(json["sudo_password"].as_str().unwrap(), "***");
        assert_eq!(json["su_password"].as_str().unwrap(), "***");
    }

    #[test]
    fn saida_execucao_formatacao_completa() {
        let saida = SaidaExecucao {
            stdout: "comando executado".to_string(),
            stderr: "aviso harmless".to_string(),
            exit_code: Some(0),
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 1000,
        };
        assert_eq!(saida.stdout, "comando executado");
        assert_eq!(saida.stderr, "aviso harmless");
        assert_eq!(saida.exit_code, Some(0));
        assert_eq!(saida.duracao_ms, 1000);
        assert!(!saida.truncado_stdout);
        assert!(!saida.truncado_stderr);
    }

    #[test]
    fn saida_execucao_sem_stderr() {
        let saida = SaidaExecucao {
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 50,
        };
        assert!(saida.stderr.is_empty());
    }

    #[test]
    fn saida_execucao_com_sinal_em_vez_de_exit_code() {
        let saida = SaidaExecucao {
            stdout: String::new(),
            stderr: "signal received".to_string(),
            exit_code: None,
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 5000,
        };
        assert!(saida.exit_code.is_none());
    }

    #[test]
    fn saida_execucao_json_contem_campos_obrigatorios() {
        let saida = SaidaExecucao {
            stdout: "output".to_string(),
            stderr: "erro".to_string(),
            exit_code: Some(0),
            truncado_stdout: false,
            truncado_stderr: false,
            duracao_ms: 100,
        };
        imprimir_saida_execucao_json(&saida);
    }
}
