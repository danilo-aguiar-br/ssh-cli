// SPDX-License-Identifier: MIT OR Apache-2.0
//! Benchmarks for ssh-cli **local** cold/warm CPU paths.
//!
//! Scope (Rules Rust — no blind optimization / latência):
//! - These benches cover mask + path validation (pure CPU, no network).
//! - They are **not** a substitute for SSH/SCP flamegraphs under real RTT.
//! - Hot path of the product is I/O-bound (TCP/SSH); do not treat a 5% local
//!   win as production readiness without integration measurement.
//! - Do **not** report criterion means as product P99 latency — end-to-end
//!   latency is network RTT; local benches only guard cold CPU regressions.
//!
//! Run: `cargo bench --bench ssh_operations`
//! Compare size-min vs speed:
//!   `cargo build --release` vs `--profile release-fast` / `--profile release-lto`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ssh_cli::masking::mask;
use ssh_cli::paths::{normalize_nfc, validate_and_normalize, validate_name};

fn bench_masking(c: &mut Criterion) {
    c.bench_function("mask_short", |b| {
        b.iter(|| mask(black_box("short")))
    });
    c.bench_function("mask_long", |b| {
        b.iter(|| mask(black_box("very-long-secret-password-here-123456")))
    });
    c.bench_function("mask_unicode", |b| {
        b.iter(|| mask(black_box("ação você está configuração Itaú")))
    });
}

fn bench_paths(c: &mut Criterion) {
    c.bench_function("validate_name", |b| {
        b.iter(|| validate_name(black_box("my-production-server")))
    });
    c.bench_function("normalize_nfc_nfd", |b| {
        b.iter(|| normalize_nfc(black_box("cafe\u{0301}")))
    });
    c.bench_function("normalize_nfc_noop", |b| {
        b.iter(|| normalize_nfc(black_box("server")))
    });
    c.bench_function("validate_and_normalize", |b| {
        b.iter(|| validate_and_normalize(black_box("my-server")))
    });
}

criterion_group!(benches, bench_masking, bench_paths);
criterion_main!(benches);
