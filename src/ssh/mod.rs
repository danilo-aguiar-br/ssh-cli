//! Motor SSH via `russh` 0.60.x.
//!
//! - `cliente`: conexão one-shot, auth senha/chave, exec com timeout e abort
//! - `known_hosts`: TOFU de fingerprints em XDG
//! - `packing`: empacotamento seguro sudo/su (automação one-shot)

pub mod cliente;
pub mod known_hosts;
pub mod packing;

pub use cliente::{truncar_utf8, ClienteSsh, ConfiguracaoConexao, SaidaExecucao};
pub use packing::{
    anexar_description, empacotar_abort_pkill, empacotar_su, empacotar_sudo,
    escapar_shell_single_quotes, padrao_abort_remoto,
};
