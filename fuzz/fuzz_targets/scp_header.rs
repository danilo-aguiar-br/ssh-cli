#![no_main]
use libfuzzer_sys::fuzz_target;

// G-O5: exercise public packing + public API surface only (scp_wire is crate-private).
fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    let _ = ssh_cli::ssh::packing::escape_shell_single_quotes(&s);
    let _ = ssh_cli::ssh::packing::pack_sudo(&s, None);
});
