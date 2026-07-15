//! Testes property-based com proptest.

use proptest::prelude::*;
use ssh_cli::mascaramento::mascarar;

proptest! {
    // ---------- mascarar (GAP-SSH-SEC-002: sempre "***") ----------

    #[test]
    fn prop_mascarar_never_panics(s in "\\PC*") {
        let _ = mascarar(&s);
    }

    #[test]
    fn prop_mascarar_sempre_triplo_asterisco(s in "\\PC*") {
        prop_assert_eq!(mascarar(&s), "***");
    }

    #[test]
    fn prop_mascarar_nunca_retorna_input(s in "\\PC{1,100}") {
        let resultado = mascarar(&s);
        prop_assume!(s != "***");
        prop_assert_ne!(
            resultado.as_str(),
            s.as_str(),
            "mascarar NUNCA deve retornar o valor original"
        );
    }
}
