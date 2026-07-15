// SPDX-License-Identifier: MIT OR Apache-2.0
//! Testes property-based com proptest.

use proptest::prelude::*;
use ssh_cli::masking::mask;

proptest! {
    // ---------- mask (GAP-SSH-SEC-002: sempre "***") ----------

    #[test]
    fn prop_mask_never_panics(s in "\\PC*") {
        let _ = mask(&s);
    }

    #[test]
    fn prop_mask_always_triple_asterisk(s in "\\PC*") {
        prop_assert_eq!(mask(&s), "***");
    }

    #[test]
    fn prop_mask_never_returns_input(s in "\\PC{1,100}") {
        let result = mask(&s);
        prop_assume!(s != "***");
        prop_assert_ne!(
            result.as_str(),
            s.as_str(),
            "mascarar NUNCA deve retornar o valor original"
        );
    }
}
