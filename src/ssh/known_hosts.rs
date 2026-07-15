//! Persistência TOFU de fingerprints de host key em XDG.
//!
//! Formato linha a linha (0o600):
//! `host:porta <fingerprint_sha256>`

use crate::erros::{ErroSshCli, ResultadoSshCli};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Mapa host:porta → fingerprint.
#[derive(Debug, Default, Clone)]
pub struct KnownHosts {
    entradas: BTreeMap<String, String>,
    caminho: PathBuf,
}

impl KnownHosts {
    /// Chave canônica `host:porta`.
    #[must_use]
    pub fn chave(host: &str, porta: u16) -> String {
        format!("{host}:{porta}")
    }

    /// Carrega o arquivo (vazio se inexistente).
    pub fn carregar(caminho: PathBuf) -> ResultadoSshCli<Self> {
        let mut entradas = BTreeMap::new();
        if caminho.exists() {
            let texto = std::fs::read_to_string(&caminho)?;
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
        Ok(Self { entradas, caminho })
    }

    /// Caminho padrão `config_dir/known_hosts` a partir do path do `config.toml`.
    #[must_use]
    pub fn caminho_ao_lado_config(config_toml: &Path) -> PathBuf {
        config_toml
            .parent()
            .map(|p| p.join("known_hosts"))
            .unwrap_or_else(|| PathBuf::from("known_hosts"))
    }

    /// Consulta fingerprint gravado.
    #[must_use]
    pub fn obter(&self, host: &str, porta: u16) -> Option<&str> {
        self.entradas
            .get(&Self::chave(host, porta))
            .map(String::as_str)
    }

    /// Insere ou atualiza e persiste atomicamente.
    pub fn gravar(&mut self, host: &str, porta: u16, fingerprint: &str) -> ResultadoSshCli<()> {
        self.entradas
            .insert(Self::chave(host, porta), fingerprint.to_string());
        self.persistir()
    }

    fn persistir(&self) -> ResultadoSshCli<()> {
        if let Some(pai) = self.caminho.parent() {
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
            .caminho
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut tmp = tempfile::NamedTempFile::new_in(&pai).map_err(ErroSshCli::Io)?;
        use std::io::Write;
        tmp.write_all(corpo.as_bytes())?;
        tmp.as_file().sync_data()?;
        tmp.persist(&self.caminho)
            .map_err(|e| ErroSshCli::Io(e.error))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.caminho)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&self.caminho, perms)?;
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
    porta: u16,
    fingerprint: &str,
    substituir: bool,
) -> ResultadoSshCli<bool> {
    match kh.obter(host, porta) {
        None => {
            kh.gravar(host, porta, fingerprint)?;
            Ok(true)
        }
        Some(existente) if existente == fingerprint => Ok(true),
        Some(existente) if substituir => {
            tracing::warn!(
                host,
                porta,
                antigo = %existente,
                novo = %fingerprint,
                "substituindo host key (--replace-host-key)"
            );
            kh.gravar(host, porta, fingerprint)?;
            Ok(true)
        }
        Some(existente) => Err(ErroSshCli::HostKeyMudou {
            host: host.to_string(),
            porta,
            esperado: existente.to_string(),
            obtido: fingerprint.to_string(),
        }),
    }
}

#[cfg(test)]
mod testes {
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
        assert!(matches!(err, ErroSshCli::HostKeyMudou { .. }));
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
