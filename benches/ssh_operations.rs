// SPDX-License-Identifier: MIT OR Apache-2.0
//! Benchmarks for ssh-cli operations.

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
