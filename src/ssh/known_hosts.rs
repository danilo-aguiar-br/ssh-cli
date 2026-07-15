// SPDX-License-Identifier: MIT OR Apache-2.0
//! Persistência TOFU de fingerprints de host key em XDG.
//!
//! Formato linha a linha (0o600):
//! `host:port <fingerprint_sha256>`

use crate::erros::{SshCliError, SshCliResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Mapa host:port → fingerprint.
#[derive(Debug, Default, Clone)]
pub struct KnownHosts {
    entradas: BTreeMap<String, String>,
    path: PathBuf,
}

impl KnownHosts {
    /// Chave canônica `host:port`.
    #[must_use]
    pub fn chave(host: &str, port: u16) -> String {
        format!("{host}:{port}")
    }

    /// Carrega o arquivo (vazio se inexistente).
    pub fn carregar(path: PathBuf) -> SshCliResult<Self> {
        let mut entradas = BTreeMap::new();
        if path.exists() {
            let texto = std::fs::read_to_string(&path)?;
            for linha in texto.lines() {
                let linha = linha.trim();
                if linha.is_empty() || linha.starts_with('#') {
                    continue;
                }
                let mut parts = linha.split_whitespace();
                if let (Some(k), Some(fp)) = (parts.next(), parts.next()) {
                    entradas.insert(k.to_string(), fp.to_string());
                }
            }
        }
        Ok(Self { entradas, path })
    }

    /// Caminho padrão `config_dir/known_hosts` a partir do path do `config.toml`.
    #[must_use]
    pub fn path_beside_config(config_toml: &Path) -> PathBuf {
        config_toml
            .parent()
            .map(|p| p.join("known_hosts"))
            .unwrap_or_else(|| PathBuf::from("known_hosts"))
    }

    /// Consulta fingerprint gravado.
    #[must_use]
    pub fn obter(&self, host: &str, port: u16) -> Option<&str> {
        self.entradas
            .get(&Self::chave(host, port))
            .map(String::as_str)
    }

    /// Insere ou atualiza e persiste atomicamente.
    pub fn gravar(&mut self, host: &str, port: u16, fingerprint: &str) -> SshCliResult<()> {
        self.entradas
            .insert(Self::chave(host, port), fingerprint.to_string());
        self.persistir()
    }

    fn persistir(&self) -> SshCliResult<()> {
        if let Some(pai) = self.path.parent() {
            std::fs::create_dir_all(pai)?;
        }
        let mut corpo = String::new();
        corpo.push_str("# ssh-cli known_hosts (TOFU)\n");
        for (k, v) in &self.entradas {
            corpo.push_str(k);
            corpo.push(' ');
            corpo.push_str(v);
            corpo.push('\n');
        }

        let pai = self
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut tmp = tempfile::NamedTempFile::new_in(&pai).map_err(SshCliError::Io)?;
        use std::io::Write;
        tmp.write_all(corpo.as_bytes())?;
        tmp.as_file().sync_data()?;
        tmp.persist(&self.path)
            .map_err(|e| SshCliError::Io(e.error))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&self.path, perms)?;
        }
        Ok(())
    }
}

/// Verifica fingerprint TOFU.
///
/// - Sem entrada: aceita e grava (TOFU).
/// - Com entrada igual: aceita.
/// - Com entrada diferente: recusa, a menos que `substituir` seja true.
pub fn verificar_tofu(
    kh: &mut KnownHosts,
    host: &str,
    port: u16,
    fingerprint: &str,
    substituir: bool,
) -> SshCliResult<bool> {
    match kh.obter(host, port) {
        None => {
            kh.gravar(host, port, fingerprint)?;
            Ok(true)
        }
        Some(existente) if existente == fingerprint => Ok(true),
        Some(existente) if substituir => {
            tracing::warn!(
                host,
                port,
                antigo = %existente,
                novo = %fingerprint,
                "substituindo host key (--replace-host-key)"
            );
            kh.gravar(host, port, fingerprint)?;
            Ok(true)
        }
        Some(existente) => Err(SshCliError::HostKeyChanged {
            host: host.to_string(),
            port,
            expected: existente.to_string(),
            obtained: fingerprint.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn tofu_grava_e_aceita_mesmo() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::carregar(path).unwrap();
        assert!(verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap());
        assert!(verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap());
    }

    #[test]
    fn tofu_recusa_mudanca() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::carregar(path).unwrap();
        verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap();
        let err = verificar_tofu(&mut kh, "h", 22, "fp2", false).unwrap_err();
        assert!(matches!(err, SshCliError::HostKeyChanged { .. }));
    }

    #[test]
    fn tofu_substitui_com_flag() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("known_hosts");
        let mut kh = KnownHosts::carregar(path).unwrap();
        verificar_tofu(&mut kh, "h", 22, "fp1", false).unwrap();
        assert!(verificar_tofu(&mut kh, "h", 22, "fp2", true).unwrap());
        assert_eq!(kh.obter("h", 22), Some("fp2"));
    }
}
