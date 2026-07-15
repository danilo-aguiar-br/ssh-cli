// SPDX-License-Identifier: MIT OR Apache-2.0
//! Empacotamento seguro de comandos sudo/su (one-shot multi-host para LLMs).
//!
//! Usa `sh -c` com escape de aspas para comandos compostos seguros.

use secrecy::{ExposeSecret, SecretString};

/// Escapa uma string para uso seguro dentro de single quotes no shell.
///
/// Estratégia: envolve em single quotes e escapa single quotes internas
/// com a sequência `'\''` (fecha quote, backslash-quote, abre quote).
#[must_use]
pub fn escape_shell_single_quotes(valor: &str) -> String {
    let mut resultado = String::with_capacity(valor.len() + 2);
    resultado.push('\'');
    for ch in valor.chars() {
        if ch == '\'' {
            resultado.push_str("'\\''");
        } else {
            resultado.push(ch);
        }
    }
    resultado.push('\'');
    resultado
}

/// Anexa `description` como comentário shell de forma segura .
#[must_use]
pub fn append_description(command: &str, description: Option<&str>) -> String {
    match description {
        Some(d) if !d.trim().is_empty() => {
            let limpo = d.replace(['\n', '\r'], " ");
            format!("{command} # {limpo}")
        }
        _ => command.to_string(),
    }
}

/// Resultado do packing: command remoto **sem** segredo na argv + bytes opcionais
/// a enviar no stdin do canal SSH (GAP-SSH-SEC-001).
#[derive(Debug, Clone)]
pub struct ComandoEmpacotado {
    /// Linha de command remota (sem password embutida).
    pub command: String,
    /// Dados a escrever no stdin do canal (ex.: password + `\n` para `sudo -S` / `su`).
    pub stdin: Option<Vec<u8>>,
}

/// Empacota command para `sudo` com `sh -c`.
///
/// - Com password: `sudo -S -p '' sh -c 'cmd'` e password no **stdin do canal** (não na argv).
/// - Sem password: `sudo -n sh -c 'cmd'`.
#[must_use]
pub fn pack_sudo(command: &str, sudo_password: Option<&SecretString>) -> ComandoEmpacotado {
    let cmd_esc = escape_shell_single_quotes(command);
    match sudo_password {
        Some(password) => {
            let mut stdin = password.expose_secret().as_bytes().to_vec();
            stdin.push(b'\n');
            ComandoEmpacotado {
                command: format!("sudo -S -p '' sh -c {cmd_esc}"),
                stdin: Some(stdin),
            }
        }
        None => ComandoEmpacotado {
            command: format!("sudo -n sh -c {cmd_esc}"),
            stdin: None,
        },
    }
}

/// Empacota command para `su - -c` one-shot; password vai no stdin do canal.
#[must_use]
pub fn pack_su(command: &str, su_password: &SecretString) -> ComandoEmpacotado {
    let cmd_esc = escape_shell_single_quotes(command);
    let mut stdin = su_password.expose_secret().as_bytes().to_vec();
    stdin.push(b'\n');
    ComandoEmpacotado {
        command: format!("su - -c {cmd_esc}"),
        stdin: Some(stdin),
    }
}

/// Sanitiza trecho de command para uso best-effort em `pkill -f`.
///
/// Aceita alfanuméricos e símbolos restritos; para no primeiro metacaractere
/// perigoso. Exige ao menos 3 chars. Nunca embute senhas (só o padrão do cmd).
#[must_use]
pub fn remote_abort_pattern(command: &str) -> Option<String> {
    let mut limpo = String::with_capacity(command.len().min(128));
    for ch in command.chars().take(128) {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ' ' | ':' | '=') {
            limpo.push(ch);
        } else {
            break;
        }
    }
    let t = limpo.trim();
    if t.len() < 3 {
        None
    } else {
        Some(t.to_string())
    }
}

/// Monta command de abort best-effort remoto (TERM depois KILL).
///
/// Não embute segredos; usa apenas o padrão sanitizado do command.
#[must_use]
pub fn pack_abort_pkill(padrao: &str) -> String {
    let esc = escape_shell_single_quotes(padrao);
    format!(
        "(pkill -TERM -f {esc} 2>/dev/null || true); sleep 0.2; (pkill -KILL -f {esc} 2>/dev/null || true)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_single_quote() {
        assert_eq!(escape_shell_single_quotes("ab'cd"), "'ab'\\''cd'");
        assert_eq!(escape_shell_single_quotes("abc"), "'abc'");
    }

    #[test]
    fn sudo_com_senha_usa_sh_c_sem_secret_na_argv() {
        let password = SecretString::from("s3cr3t".to_string());
        let pack = pack_sudo("echo hi | tee /tmp/x", Some(&password));
        assert!(pack.command.contains("sudo -S -p '' sh -c"));
        assert!(!pack.command.contains("s3cr3t"));
        assert!(!pack.command.contains("printf"));
        let stdin = pack.stdin.expect("stdin com senha");
        assert_eq!(stdin, b"s3cr3t\n");
    }

    #[test]
    fn sudo_sem_senha_usa_n() {
        let pack = pack_sudo("id", None);
        assert_eq!(pack.command, "sudo -n sh -c 'id'");
        assert!(pack.stdin.is_none());
    }

    #[test]
    fn su_pack_sem_secret_na_argv() {
        let password = SecretString::from("rootpw".to_string());
        let pack = pack_su("whoami", &password);
        assert!(pack.command.contains("su - -c"));
        assert!(!pack.command.contains("rootpw"));
        assert_eq!(pack.stdin.as_deref(), Some(b"rootpw\n".as_slice()));
    }

    #[test]
    fn description_anexa_comentario() {
        assert_eq!(
            append_description("ls", Some("lista arquivos")),
            "ls # lista arquivos"
        );
        assert_eq!(append_description("ls", None), "ls");
    }

    #[test]
    fn padrao_abort_sanitiza() {
        assert_eq!(
            remote_abort_pattern("sleep 999"),
            Some("sleep 999".to_string())
        );
        // GAP-SSH-TEST-003: metacaractere perigoso → rejeita (não tautologia).
        assert_eq!(remote_abort_pattern("$(rm -rf)"), None);
        assert!(remote_abort_pattern("ab").is_none());
    }
}
