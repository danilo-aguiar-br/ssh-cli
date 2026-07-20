#![no_main]
//! G-SERDE-12: adversarial JSON/TOML import envelope deserialize.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    // Cap like product path (avoid OOM in fuzz).
    if s.len() > 64 * 1024 {
        return;
    }
    let _ = ssh_cli::vps::parse_import_payload(s);
});
