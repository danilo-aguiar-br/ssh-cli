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

/// Resultado do packing: comando remoto **sem** segredo na argv + bytes opcionais
/// a enviar no stdin do canal SSH (GAP-SSH-SEC-001).
#[derive(Debug, Clone)]
pub struct ComandoEmpacotado {
    /// Linha de comando remota (sem senha embutida).
    pub comando: String,
    /// Dados a escrever no stdin do canal (ex.: senha + `\n` para `sudo -S` / `su`).
    pub stdin: Option<Vec<u8>>,
}

/// Empacota comando para `sudo` com `sh -c`.
///
/// - Com senha: `sudo -S -p '' sh -c 'cmd'` e senha no **stdin do canal** (não na argv).
/// - Sem senha: `sudo -n sh -c 'cmd'`.
#[must_use]
pub fn empacotar_sudo(comando: &str, senha_sudo: Option<&SecretString>) -> ComandoEmpacotado {
    let cmd_esc = escapar_shell_single_quotes(comando);
    match senha_sudo {
        Some(senha) => {
            let mut stdin = senha.expose_secret().as_bytes().to_vec();
            stdin.push(b'\n');
            ComandoEmpacotado {
                comando: format!("sudo -S -p '' sh -c {cmd_esc}"),
                stdin: Some(stdin),
            }
        }
        None => ComandoEmpacotado {
            comando: format!("sudo -n sh -c {cmd_esc}"),
            stdin: None,
        },
    }
}

/// Empacota comando para `su - -c` one-shot; senha vai no stdin do canal.
#[must_use]
pub fn empacotar_su(comando: &str, senha_su: &SecretString) -> ComandoEmpacotado {
    let cmd_esc = escapar_shell_single_quotes(comando);
    let mut stdin = senha_su.expose_secret().as_bytes().to_vec();
    stdin.push(b'\n');
    ComandoEmpacotado {
        comando: format!("su - -c {cmd_esc}"),
        stdin: Some(stdin),
    }
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
    fn sudo_com_senha_usa_sh_c_sem_secret_na_argv() {
        let senha = SecretString::from("s3cr3t".to_string());
        let pack = empacotar_sudo("echo hi | tee /tmp/x", Some(&senha));
        assert!(pack.comando.contains("sudo -S -p '' sh -c"));
        assert!(!pack.comando.contains("s3cr3t"));
        assert!(!pack.comando.contains("printf"));
        let stdin = pack.stdin.expect("stdin com senha");
        assert_eq!(stdin, b"s3cr3t\n");
    }

    #[test]
    fn sudo_sem_senha_usa_n() {
        let pack = empacotar_sudo("id", None);
        assert_eq!(pack.comando, "sudo -n sh -c 'id'");
        assert!(pack.stdin.is_none());
    }

    #[test]
    fn su_pack_sem_secret_na_argv() {
        let senha = SecretString::from("rootpw".to_string());
        let pack = empacotar_su("whoami", &senha);
        assert!(pack.comando.contains("su - -c"));
        assert!(!pack.comando.contains("rootpw"));
        assert_eq!(pack.stdin.as_deref(), Some(b"rootpw\n".as_slice()));
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
        // GAP-SSH-TEST-003: metacaractere perigoso → rejeita (não tautologia).
        assert_eq!(padrao_abort_remoto("$(rm -rf)"), None);
        assert!(padrao_abort_remoto("ab").is_none());
    }
}
