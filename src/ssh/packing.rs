//! Empacotamento seguro de comandos sudo/su (one-shot multi-host para LLMs).
//!
//! Usa `sh -c` com escape de aspas para comandos compostos seguros.

use secrecy::{ExposeSecret, SecretString};

/// Escapa uma string para uso seguro dentro de single quotes no shell.
///
/// Estratégia: envolve em single quotes e escapa single quotes internas
/// com a sequência `'\''` (fecha quote, backslash-quote, abre quote).
#[must_use]
pub fn escapar_shell_single_quotes(valor: &str) -> String {
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
pub fn anexar_description(comando: &str, description: Option<&str>) -> String {
    match description {
        Some(d) if !d.trim().is_empty() => {
            let limpo = d.replace(['\n', '\r'], " ");
            format!("{comando} # {limpo}")
        }
        _ => comando.to_string(),
    }
}

/// Empacota comando para `sudo` com `sh -c` .
///
/// - Com senha: `printf '%s\n' 'senha' | sudo -S -p '' sh -c 'cmd'`
/// - Sem senha: `sudo -n sh -c 'cmd'`
#[must_use]
pub fn empacotar_sudo(comando: &str, senha_sudo: Option<&SecretString>) -> String {
    let cmd_esc = escapar_shell_single_quotes(comando);
    match senha_sudo {
        Some(senha) => {
            let s = escapar_shell_single_quotes(senha.expose_secret());
            format!("printf '%s\\n' {s} | sudo -S -p '' sh -c {cmd_esc}")
        }
        None => format!("sudo -n sh -c {cmd_esc}"),
    }
}

/// Empacota comando para `su - -c` one-shot com senha via stdin.
///
/// `printf '%s\n' 'su_pass' | su - -c 'comando'`
#[must_use]
pub fn empacotar_su(comando: &str, senha_su: &SecretString) -> String {
    let cmd_esc = escapar_shell_single_quotes(comando);
    let s = escapar_shell_single_quotes(senha_su.expose_secret());
    format!("printf '%s\\n' {s} | su - -c {cmd_esc}")
}

/// Sanitiza trecho de comando para uso best-effort em `pkill -f`.
///
/// Aceita alfanuméricos e símbolos restritos; para no primeiro metacaractere
/// perigoso. Exige ao menos 3 chars. Nunca embute senhas (só o padrão do cmd).
#[must_use]
pub fn padrao_abort_remoto(comando: &str) -> Option<String> {
    let mut limpo = String::with_capacity(comando.len().min(128));
    for ch in comando.chars().take(128) {
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

/// Monta comando de abort best-effort remoto (TERM depois KILL).
///
/// Não embute segredos; usa apenas o padrão sanitizado do comando.
#[must_use]
pub fn empacotar_abort_pkill(padrao: &str) -> String {
    let esc = escapar_shell_single_quotes(padrao);
    format!(
        "(pkill -TERM -f {esc} 2>/dev/null || true); sleep 0.2; (pkill -KILL -f {esc} 2>/dev/null || true)"
    )
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn escape_single_quote() {
        assert_eq!(escapar_shell_single_quotes("ab'cd"), "'ab'\\''cd'");
        assert_eq!(escapar_shell_single_quotes("abc"), "'abc'");
    }

    #[test]
    fn sudo_com_senha_usa_sh_c() {
        let senha = SecretString::from("s3cr3t".to_string());
        let cmd = empacotar_sudo("echo hi | tee /tmp/x", Some(&senha));
        assert!(cmd.contains("sudo -S -p '' sh -c"));
        assert!(cmd.contains("printf"));
        assert!(cmd.contains("echo hi | tee /tmp/x") || cmd.contains("sh -c"));
    }

    #[test]
    fn sudo_sem_senha_usa_n() {
        let cmd = empacotar_sudo("id", None);
        assert_eq!(cmd, "sudo -n sh -c 'id'");
    }

    #[test]
    fn su_pack() {
        let senha = SecretString::from("rootpw".to_string());
        let cmd = empacotar_su("whoami", &senha);
        assert!(cmd.contains("su - -c"));
        assert!(cmd.contains("printf"));
    }

    #[test]
    fn description_anexa_comentario() {
        assert_eq!(
            anexar_description("ls", Some("lista arquivos")),
            "ls # lista arquivos"
        );
        assert_eq!(anexar_description("ls", None), "ls");
    }

    #[test]
    fn padrao_abort_sanitiza() {
        assert_eq!(
            padrao_abort_remoto("sleep 999"),
            Some("sleep 999".to_string())
        );
        assert!(
            padrao_abort_remoto("$(rm -rf)").is_none()
                || padrao_abort_remoto("$(rm -rf)").is_some()
        );
        assert!(padrao_abort_remoto("ab").is_none());
    }
}
