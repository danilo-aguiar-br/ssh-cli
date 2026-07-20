// SPDX-License-Identifier: MIT OR Apache-2.0
//! Testes property-based com proptest.

use proptest::prelude::*;
use ssh_cli::cli::CliArgs;
use ssh_cli::masking::mask;
use clap::Parser;

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
            result,
            s.as_str(),
            "mask must never return the original secret value"
        );
    }

    // ---------- G-23: CLI parse never panics on arbitrary argv tails ----------

    #[test]
    fn prop_cli_try_parse_never_panics(s in "\\PC{0,64}") {
        // Adversarial token after program name: must return Ok or Err, never panic.
        let _ = CliArgs::try_parse_from(["ssh-cli", s.as_str()]);
    }

    #[test]
    fn prop_cli_max_chars_parser_stable(n in 0usize..100_000) {
        let n_s = n.to_string();
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "add",
            "--name",
            "x",
            "--host",
            "h",
            "--user",
            "u",
            "--password",
            "p",
            "--max-command-chars",
            n_s.as_str(),
        ]);
        if let Ok(a) = args {
            if let ssh_cli::cli::Command::Vps {
                action: ssh_cli::cli::VpsAction::Add {
                    max_command_chars: Some(v),
                    ..
                },
            } = a.command
            {
                // 0 means unlimited; other values must round-trip.
                prop_assert_eq!(v, n);
            }
        }
    }
}


proptest! {
    /// G-O5: shell escape never panics.
    #[test]
    fn escape_shell_never_panics(s in "\\PC{0,200}") {
        let out = ssh_cli::ssh::packing::escape_shell_single_quotes(&s);
        assert!(out.as_bytes().first() == Some(&b'\''));
        assert!(out.as_bytes().last() == Some(&b'\''));
    }
}

// ---------- G-DOM-07: domain type roundtrips (chrono / uuid / url / decimal) ----------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn rfc3339_utc_reject_or_roundtrip(s in "\\PC{0,80}") {
        use ssh_cli::domain::Rfc3339Utc;
        if let Ok(t) = Rfc3339Utc::try_new(&s) {
            let wire = t.to_rfc3339();
            let back = Rfc3339Utc::try_new(&wire).expect("roundtrip");
            prop_assert_eq!(t, back);
            let j = serde_json::to_string(&t).unwrap();
            let from_json: Rfc3339Utc = serde_json::from_str(&j).unwrap();
            prop_assert_eq!(from_json, t);
        }
    }

    #[test]
    fn https_url_reject_or_roundtrip(s in "\\PC{0,120}") {
        use ssh_cli::domain::HttpsUrl;
        if let Ok(u) = HttpsUrl::try_new(&s) {
            prop_assert_eq!(u.as_url().scheme(), "https");
            let wire = u.as_str().to_owned();
            let back = HttpsUrl::try_new(&wire).expect("roundtrip");
            prop_assert_eq!(u, back);
        }
    }

    #[test]
    fn correlation_id_serde_roundtrip(_n in 0u8..32) {
        use ssh_cli::domain::CorrelationId;
        let id = CorrelationId::new();
        let j = serde_json::to_string(&id).unwrap();
        let back: CorrelationId = serde_json::from_str(&j).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn batch_run_id_serde_roundtrip(_n in 0u8..32) {
        use ssh_cli::domain::BatchRunId;
        let id = BatchRunId::new();
        let j = serde_json::to_string(&id).unwrap();
        let back: BatchRunId = serde_json::from_str(&j).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn money_brl_str_roundtrip(cents in 0i64..1_000_000) {
        use rust_decimal::Decimal;
        use ssh_cli::domain::{Brl, Money};
        let d = Decimal::new(cents, 2);
        let m = Money::<Brl>::try_new(d).unwrap();
        let j = serde_json::to_string(&m).unwrap();
        prop_assert!(!j.contains('.' ) || j.contains('"'), "must serialize as string: {j}");
        let back: Money<Brl> = serde_json::from_str(&j).unwrap();
        prop_assert_eq!(m.amount(), back.amount());
    }
}
