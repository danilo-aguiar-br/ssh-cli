// SPDX-License-Identifier: MIT OR Apache-2.0
//! Policy gates for domain types 3-crates (G-DOM-01…10).
//!
//! Local-only gates (no GitHub Actions product dependency).

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_cargo_toml() -> String {
    fs::read_to_string(workspace_root().join("Cargo.toml")).expect("Cargo.toml")
}

fn walk_src_rs() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let src = workspace_root().join("src");
    fn rec(dir: &std::path::Path, out: &mut Vec<(PathBuf, String)>) {
        for e in fs::read_dir(dir).expect("read_dir") {
            let e = e.expect("entry");
            let p = e.path();
            if p.is_dir() {
                rec(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                let t = fs::read_to_string(&p).unwrap_or_default();
                out.push((p, t));
            }
        }
    }
    rec(&src, &mut out);
    out
}

#[test]
fn g_dom_01_four_crates_declared_with_features() {
    let toml = read_cargo_toml();
    assert!(
        toml.contains("chrono") && toml.contains("0.4.45") && toml.contains("serde"),
        "chrono 0.4.45 + serde required"
    );
    assert!(
        toml.contains("uuid")
            && toml.contains("\"v4\"")
            && toml.contains("\"v7\"")
            && toml.contains("serde"),
        "uuid with v4, v7, serde required"
    );
    assert!(
        !toml
            .lines()
            .any(|l| l.trim_start().starts_with("rust_decimal")),
        "rust_decimal was removed together with the Money module"
    );
    // url crate line (not just docs URL)
    assert!(
        toml.lines()
            .any(|l| l.trim_start().starts_with("url =") && l.contains("serde")),
        "url crate with serde feature required"
    );
}

#[test]
fn g_dom_02_no_local_now_in_src() {
    for (path, text) in walk_src_rs() {
        for (i, line) in text.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.starts_with("//!") || t.starts_with('*') {
                continue;
            }
            assert!(
                !t.contains("Local::now"),
                "{}:{} uses Local::now (forbidden; use Utc/Rfc3339Utc)",
                path.display(),
                i + 1
            );
        }
    }
}

#[test]
fn g_dom_03_vps_added_at_is_rfc3339_newtype() {
    let model = fs::read_to_string(workspace_root().join("src/vps/model.rs")).unwrap();
    assert!(
        model.contains("pub added_at: Rfc3339Utc"),
        "VpsRecord.added_at must be Rfc3339Utc, not String"
    );
    assert!(
        !model.contains("pub added_at: String"),
        "VpsRecord.added_at must not be raw String"
    );
}

#[test]
fn g_dom_04_acme_order_url_typed() {
    let acme = fs::read_to_string(workspace_root().join("src/tls/acme.rs")).unwrap();
    assert!(
        acme.contains("pub order_url: AcmeOrderUrl"),
        "PendingOrder.order_url must be AcmeOrderUrl"
    );
    assert!(
        acme.contains("pub created_at: Rfc3339Utc"),
        "PendingOrder.created_at must be Rfc3339Utc"
    );
}

#[test]
fn g_dom_05_batch_run_id_on_wire() {
    let wire = fs::read_to_string(workspace_root().join("src/json_wire/mod.rs")).unwrap()
        + &fs::read_to_string(workspace_root().join("src/json_wire/execution.rs")).unwrap()
        + &fs::read_to_string(workspace_root().join("src/json_wire/emit.rs")).unwrap();
    assert!(
        wire.contains("batch_run_id"),
        "batch envelopes need batch_run_id"
    );
    let batch = fs::read_to_string(workspace_root().join("src/output/batch.rs")).unwrap();
    assert!(
        batch.contains("BatchRunId"),
        "output/batch must generate BatchRunId"
    );
}

#[test]
fn g_dom_06_domain_is_split_modules() {
    let domain = workspace_root().join("src/domain");
    for name in [
        "mod.rs",
        "error.rs",
        "names.rs",
        "ports.rs",
        "limits.rs",
        "command.rs",
        "time.rs",
        "ids.rs",
        "http_url.rs",
    ] {
        assert!(
            domain.join(name).is_file(),
            "domain/{name} missing (SRP split)"
        );
    }
}

#[test]
fn g_dom_08_money_module_is_gone() {
    assert!(
        !workspace_root().join("src/domain/money.rs").exists(),
        "src/domain/money.rs was removed (no monetary surface in this product)"
    );
}

#[test]
fn g_dom_runtime_types_smoke() {
    use ssh_cli::domain::{AcmeOrderUrl, BatchRunId, CorrelationId, HttpsUrl, Rfc3339Utc};

    let t = Rfc3339Utc::try_new("2024-06-01T12:00:00Z").unwrap();
    assert!(t.to_rfc3339().starts_with("2024-06-01T12:00:00"));

    let u = HttpsUrl::try_new("https://example.com/acme").unwrap();
    assert_eq!(u.as_url().scheme(), "https");
    assert!(HttpsUrl::try_new("http://example.com").is_err());

    let o = AcmeOrderUrl::try_new("https://acme.example/order/1").unwrap();
    assert!(o.as_str().starts_with("https://"));

    let c = CorrelationId::new();
    assert!(CorrelationId::try_new(c.to_string_canonical()).is_ok());
    let b = BatchRunId::new();
    assert!(BatchRunId::try_new(b.to_string_canonical()).is_ok());
}
